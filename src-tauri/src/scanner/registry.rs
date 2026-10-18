use crate::models::app::{
    AppArchitecture, AppInstallSource, InstalledApp, RegistryHiveSource,
};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER,
    HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY, REG_DWORD,
    REG_EXPAND_SZ, REG_SAM_FLAGS, REG_SZ, REG_VALUE_TYPE,
};

const UNINSTALL_ROOT: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall";

#[derive(Clone, Copy)]
enum RootHive {
    LocalMachine,
    CurrentUser,
}

impl RootHive {
    fn to_hkey(self) -> HKEY {
        match self {
            RootHive::LocalMachine => HKEY_LOCAL_MACHINE,
            RootHive::CurrentUser => HKEY_CURRENT_USER,
        }
    }
}

#[derive(Clone, Copy)]
struct RegistryTarget {
    root: RootHive,
    sam_flags: REG_SAM_FLAGS,
    hive_type: RegistryHiveSource,
    arch: AppArchitecture,
    human_path: &'static str,
}

pub struct RegistryScanner;

impl RegistryScanner {
    pub fn scan_all_targets() -> Vec<InstalledApp> {
        let targets = vec![
            // 1. Win32 Nativo x64 (HKLM 64-bit view)
            RegistryTarget {
                root: RootHive::LocalMachine,
                sam_flags: KEY_READ | KEY_WOW64_64KEY,
                hive_type: RegistryHiveSource::Hklm64,
                arch: AppArchitecture::X64,
                human_path: "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            },
            // 2. Win32 Wow6432Node (HKLM 32-bit view)
            RegistryTarget {
                root: RootHive::LocalMachine,
                sam_flags: KEY_READ | KEY_WOW64_32KEY,
                hive_type: RegistryHiveSource::Hklm32,
                arch: AppArchitecture::X86,
                human_path: "HKLM\\Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            },
            // 3. Win32 Per-User (HKCU)
            RegistryTarget {
                root: RootHive::CurrentUser,
                sam_flags: KEY_READ,
                hive_type: RegistryHiveSource::Hkcu,
                arch: AppArchitecture::X64,
                human_path: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            },
        ];

        targets
            .into_par_iter()
            .flat_map(|target| Self::scan_single_target(target))
            .collect()
    }

    fn scan_single_target(target: RegistryTarget) -> Vec<InstalledApp> {
        let subkeys = match Self::enumerate_subkeys(target.root.to_hkey(), target.sam_flags) {
            Ok(keys) => keys,
            Err(_) => return Vec::new(),
        };

        subkeys
            .into_par_iter()
            .filter_map(move |subkey_name| Self::parse_app_entry(target, &subkey_name))
            .collect()
    }

    fn enumerate_subkeys(root: HKEY, sam: REG_SAM_FLAGS) -> Result<Vec<String>, WIN32_ERROR> {
        let mut hkey = HKEY::default();
        let path_hstring = HSTRING::from(UNINSTALL_ROOT);

        unsafe {
            let status = RegOpenKeyExW(root, PCWSTR(path_hstring.as_ptr()), 0, sam, &mut hkey);
            if status != ERROR_SUCCESS {
                return Err(status);
            }
        }

        let mut subkeys = Vec::new();
        let mut index = 0u32;
        let mut name_buffer = [0u16; 256];

        loop {
            let mut name_len = name_buffer.len() as u32;
            let status = unsafe {
                RegEnumKeyExW(
                    hkey,
                    index,
                    PWSTR(name_buffer.as_mut_ptr()),
                    &mut name_len,
                    None,
                    PWSTR::null(),
                    None,
                    None,
                )
            };

            if status == ERROR_NO_MORE_ITEMS {
                break;
            } else if status == ERROR_SUCCESS {
                let name = String::from_utf16_lossy(&name_buffer[..name_len as usize]);
                subkeys.push(name);
                index += 1;
            } else {
                index += 1;
            }
        }

        unsafe {
            let _ = RegCloseKey(hkey);
        }

        Ok(subkeys)
    }

    fn parse_app_entry(target: RegistryTarget, subkey_name: &str) -> Option<InstalledApp> {
        let full_subkey_path = format!("{}\\{}", UNINSTALL_ROOT, subkey_name);
        let path_hstring = HSTRING::from(full_subkey_path.as_str());
        let mut app_hkey = HKEY::default();

        unsafe {
            let status = RegOpenKeyExW(
                target.root.to_hkey(),
                PCWSTR(path_hstring.as_ptr()),
                0,
                target.sam_flags,
                &mut app_hkey,
            );
            if status != ERROR_SUCCESS {
                return None;
            }
        }

        let display_name = Self::query_string_value(app_hkey, "DisplayName");
        let display_name = match display_name {
            Some(name) if !name.trim().is_empty() => name.trim().to_string(),
            _ => {
                unsafe { let _ = RegCloseKey(app_hkey); }
                return None;
            }
        };

        let is_system_component = Self::query_dword_value(app_hkey, "SystemComponent")
            .map(|val| val == 1)
            .unwrap_or(false);

        let parent_key_name = Self::query_string_value(app_hkey, "ParentKeyName");
        if parent_key_name.is_some() {
            unsafe { let _ = RegCloseKey(app_hkey); }
            return None;
        }

        let display_version = Self::query_string_value(app_hkey, "DisplayVersion");
        let publisher = Self::query_string_value(app_hkey, "Publisher");
        let install_date = Self::query_string_value(app_hkey, "InstallDate");
        let uninstall_string = Self::query_string_value(app_hkey, "UninstallString");
        let quiet_uninstall_string = Self::query_string_value(app_hkey, "QuietUninstallString");
        let display_icon = Self::query_string_value(app_hkey, "DisplayIcon");
        let install_location = Self::query_string_value(app_hkey, "InstallLocation")
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from);

        let estimated_size_kb = Self::query_dword_value(app_hkey, "EstimatedSize").map(|v| v as u64);
        let windows_installer = Self::query_dword_value(app_hkey, "WindowsInstaller").map(|v| v == 1);

        let guid = if windows_installer == Some(true) {
            Some(subkey_name.to_string())
        } else {
            None
        };

        unsafe {
            let _ = RegCloseKey(app_hkey);
        }

        let id = InstalledApp::generate_id(Some(target.hive_type), subkey_name);

        // Deducir ubicación real si no vino en el registro (ej. Steam, GOG, juegos y portables)
        let effective_install_location = install_location.clone().or_else(|| {
            if let Some(icon) = &display_icon {
                let clean = icon.split(',').next().unwrap_or("").trim().trim_matches('"');
                let p = Path::new(clean);
                if p.exists() {
                    if let Some(parent) = p.parent() {
                        let name = parent.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        if parent.exists() && name != "windows" && name != "system32" && name != "program files" && name != "program files (x86)" {
                            return Some(parent.to_path_buf());
                        }
                    }
                }
            }
            if let Some(uninst) = &uninstall_string {
                let clean = uninst.trim().trim_matches('"');
                let first_token = clean.split(".exe").next().map(|s| format!("{}.exe", s)).unwrap_or_default();
                let p = Path::new(first_token.trim().trim_matches('"'));
                if p.exists() {
                    if let Some(parent) = p.parent() {
                        let name = parent.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        if parent.exists() && name != "windows" && name != "system32" && name != "program files" && name != "program files (x86)" {
                            return Some(parent.to_path_buf());
                        }
                    }
                }
            }
            None
        });

        // Fecha de instalación: si el registro no la tiene, deducir de la fecha de creación de la carpeta raíz
        let final_install_date = Self::normalize_install_date(install_date).or_else(|| {
            if let Some(loc) = &effective_install_location {
                if let Ok(meta) = loc.metadata() {
                    if let Ok(time) = meta.created().or_else(|_| meta.modified()) {
                        let dt: ChronoMockDate = time.into();
                        return Some(dt.to_string());
                    }
                }
            }
            None
        });

        // Tamaño real en disco: si el registro no tiene EstimatedSize o es 0, recorrer la carpeta raíz
        let mut real_size_bytes = None;
        let mut final_estimated_size_kb = estimated_size_kb;

        if let Some(loc) = &effective_install_location {
            if let Some(calculated_bytes) = Self::calculate_dir_size_safe(loc) {
                real_size_bytes = Some(calculated_bytes);
                if final_estimated_size_kb.is_none() || final_estimated_size_kb == Some(0) {
                    final_estimated_size_kb = Some(calculated_bytes / 1024);
                }
            }
        }

        if real_size_bytes.is_none() {
            if let Some(kb) = final_estimated_size_kb {
                if kb > 0 {
                    real_size_bytes = Some(kb * 1024);
                }
            }
        }

        let icon_base64 = crate::scanner::icons::IconExtractor::get_app_icon_base64(
            display_icon.as_deref(),
            effective_install_location.as_deref(),
        );

        let info = crate::models::intelligence::AppIntelligence::evaluate(&display_name, publisher.as_deref());

        Some(InstalledApp {
            id,
            display_name,
            display_version,
            publisher,
            install_date: final_install_date,
            install_location: effective_install_location,
            uninstall_string,
            quiet_uninstall_string,
            display_icon,
            icon_base64,
            estimated_size_kb: final_estimated_size_kb,
            real_size_bytes,
            architecture: target.arch,
            source: AppInstallSource::Win32Registry,
            registry_hive: Some(target.hive_type),
            registry_key_path: Some(format!("{}\\{}", target.human_path, subkey_name)),
            is_system_component,
            is_startup_enabled: false,
            windows_installer_guid: guid,
            description: info.description,
            category: info.category,
            is_critical_system: info.is_critical,
            system_risk_warning: info.risk_warning,
        })
    }

    fn query_string_value(hkey: HKEY, value_name: &str) -> Option<String> {
        let value_hstring = HSTRING::from(value_name);
        let mut value_type: REG_VALUE_TYPE = REG_VALUE_TYPE::default();
        let mut byte_len: u32 = 0;

        unsafe {
            let status = RegQueryValueExW(
                hkey,
                PCWSTR(value_hstring.as_ptr()),
                None,
                Some(&mut value_type),
                None,
                Some(&mut byte_len),
            );

            if status != ERROR_SUCCESS || byte_len == 0 {
                return None;
            }

            if value_type != REG_SZ && value_type != REG_EXPAND_SZ {
                return None;
            }

            let u16_count = (byte_len as usize / 2) + 1;
            let mut buffer: Vec<u16> = vec![0u16; u16_count];

            let status = RegQueryValueExW(
                hkey,
                PCWSTR(value_hstring.as_ptr()),
                None,
                Some(&mut value_type),
                Some(buffer.as_mut_ptr() as *mut u8),
                Some(&mut byte_len),
            );

            if status != ERROR_SUCCESS {
                return None;
            }

            let actual_len = (byte_len as usize / 2).saturating_sub(1);
            let cleaned_slice = &buffer[..actual_len];
            let raw_str = String::from_utf16_lossy(cleaned_slice);
            Some(raw_str.trim_matches('\0').to_string())
        }
    }

    fn query_dword_value(hkey: HKEY, value_name: &str) -> Option<u32> {
        let value_hstring = HSTRING::from(value_name);
        let mut value_type = REG_VALUE_TYPE::default();
        let mut dword_val = 0u32;
        let mut byte_len = std::mem::size_of::<u32>() as u32;

        unsafe {
            let status = RegQueryValueExW(
                hkey,
                PCWSTR(value_hstring.as_ptr()),
                None,
                Some(&mut value_type),
                Some(&mut dword_val as *mut u32 as *mut u8),
                Some(&mut byte_len),
            );

            if status == ERROR_SUCCESS && value_type == REG_DWORD {
                Some(dword_val)
            } else {
                None
            }
        }
    }

    fn normalize_install_date(raw_date: Option<String>) -> Option<String> {
        let s = raw_date?;
        if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
            Some(format!("{}-{}-{}", &s[0..4], &s[4..6], &s[6..8]))
        } else {
            Some(s)
        }
    }

    fn calculate_dir_size_safe(dir: &Path) -> Option<u64> {
        if !dir.exists() || !dir.is_dir() {
            return None;
        }
        let lower = dir.to_string_lossy().to_lowercase();
        if lower == "c:\\" || lower.starts_with("c:\\windows") || lower == "c:\\program files" || lower == "c:\\program files (x86)" {
            return None;
        }

        let mut total = 0u64;
        let mut count = 0usize;
        for entry in walkdir::WalkDir::new(dir)
            .max_depth(8)
            .into_iter()
            .filter_entry(|e| !crate::cleaner::orphan_scanner::OrphanScanner::is_reparse_point_or_symlink(e.path()))
            .filter_map(|e| e.ok())
        {
            if let Ok(meta) = entry.path().symlink_metadata() {
                if meta.is_file() {
                    total += meta.len();
                    count += 1;
                    if count >= 25000 {
                        break;
                    }
                }
            }
        }

        if total > 0 {
            Some(total)
        } else {
            None
        }
    }
}

// Estructura para formatear fecha de archivo sin depender de crates externos
struct ChronoMockDate(std::time::SystemTime);

impl From<std::time::SystemTime> for ChronoMockDate {
    fn from(t: std::time::SystemTime) -> Self {
        ChronoMockDate(t)
    }
}

impl ToString for ChronoMockDate {
    fn to_string(&self) -> String {
        if let Ok(duration) = self.0.duration_since(std::time::UNIX_EPOCH) {
            let secs = duration.as_secs();
            let days = secs / 86400;
            let year = 1970 + (days / 365);
            let month = ((days % 365) / 30) + 1;
            let day = ((days % 365) % 30) + 1;
            format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31))
        } else {
            "Reciente".into()
        }
    }
}
