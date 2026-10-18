use crate::models::app::{AppArchitecture, AppInstallSource, InstalledApp};
use rayon::prelude::*;
use std::path::PathBuf;
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_READ, REG_SZ, REG_VALUE_TYPE,
};

const UWP_REPOSITORY_ROOT: &str =
    "Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\Repository\\Packages";

pub struct UwpScanner;

impl UwpScanner {
    /// Escanea las aplicaciones UWP / MSIX registradas en el perfil de usuario actual
    pub fn scan_packages() -> Vec<InstalledApp> {
        let mut hkey = HKEY::default();
        let path_hstring = HSTRING::from(UWP_REPOSITORY_ROOT);

        unsafe {
            let status = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path_hstring.as_ptr()),
                0,
                KEY_READ,
                &mut hkey,
            );
            if status != ERROR_SUCCESS {
                return Vec::new();
            }
        }

        let mut package_names = Vec::new();
        let mut index = 0u32;
        let mut buffer = [0u16; 512];

        loop {
            let mut len = buffer.len() as u32;
            let status = unsafe {
                RegEnumKeyExW(
                    hkey,
                    index,
                    PWSTR(buffer.as_mut_ptr()),
                    &mut len,
                    None,
                    PWSTR::null(),
                    None,
                    None,
                )
            };

            if status == ERROR_NO_MORE_ITEMS {
                break;
            } else if status == ERROR_SUCCESS {
                let name = String::from_utf16_lossy(&buffer[..len as usize]);
                package_names.push(name);
                index += 1;
            } else {
                index += 1;
            }
        }

        unsafe {
            let _ = RegCloseKey(hkey);
        }

        package_names
            .into_par_iter()
            .filter_map(|pkg_full_name| Self::parse_uwp_package(&pkg_full_name))
            .collect()
    }

    fn parse_uwp_package(pkg_full_name: &str) -> Option<InstalledApp> {
        // Filtrar paquetes internos de Windows como Microsoft.NET.Native, VCLibs, o host de UI
        if pkg_full_name.contains("VCLibs")
            || pkg_full_name.contains(".NET.Native")
            || pkg_full_name.contains("DirectX")
            || pkg_full_name.contains("UI.Xaml")
        {
            return None;
        }

        let subkey_path = format!("{}\\{}", UWP_REPOSITORY_ROOT, pkg_full_name);
        let path_hstring = HSTRING::from(subkey_path.as_str());
        let mut app_hkey = HKEY::default();

        unsafe {
            let status = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path_hstring.as_ptr()),
                0,
                KEY_READ,
                &mut app_hkey,
            );
            if status != ERROR_SUCCESS {
                return None;
            }
        }

        let display_name_raw = Self::query_string(app_hkey, "DisplayName");
        let publisher = Self::query_string(app_hkey, "PublisherDisplayName");
        let package_root = Self::query_string(app_hkey, "PackageRootFolder");

        unsafe {
            let _ = RegCloseKey(app_hkey);
        }

        // Si el DisplayName está referenciado por string de recursos ms-resource:, usamos el nombre del paquete
        let display_name = match display_name_raw {
            Some(name) if !name.starts_with("ms-resource:") && !name.trim().is_empty() => {
                name.trim().to_string()
            }
            _ => {
                let parts: Vec<&str> = pkg_full_name.split('_').collect();
                if !parts.is_empty() {
                    parts[0].to_string()
                } else {
                    return None;
                }
            }
        };

        let id = format!("uwp_{}", pkg_full_name);
        let uninstall_cmd = format!("powershell -Command \"Remove-AppxPackage -Package {}\"", pkg_full_name);

        let info = crate::models::intelligence::AppIntelligence::evaluate(&display_name, publisher.as_deref());

        Some(InstalledApp {
            id,
            display_name,
            display_version: None,
            publisher,
            install_date: None,
            install_location: package_root.map(PathBuf::from),
            uninstall_string: Some(uninstall_cmd.clone()),
            quiet_uninstall_string: Some(uninstall_cmd),
            display_icon: None,
            icon_base64: None,
            estimated_size_kb: None,
            real_size_bytes: None,
            architecture: AppArchitecture::Universal,
            source: AppInstallSource::WindowsStoreUwp,
            registry_hive: None,
            registry_key_path: Some(format!("HKCU\\{}", subkey_path)),
            is_system_component: false,
            is_startup_enabled: false,
            windows_installer_guid: None,
            description: info.description,
            category: info.category,
            is_critical_system: info.is_critical,
            system_risk_warning: info.risk_warning,
        })
    }

    fn query_string(hkey: HKEY, value_name: &str) -> Option<String> {
        let val_hstr = HSTRING::from(value_name);
        let mut val_type = REG_VALUE_TYPE::default();
        let mut byte_len = 0u32;

        unsafe {
            let status = RegQueryValueExW(
                hkey,
                PCWSTR(val_hstr.as_ptr()),
                None,
                Some(&mut val_type),
                None,
                Some(&mut byte_len),
            );
            if status != ERROR_SUCCESS || byte_len == 0 || val_type != REG_SZ {
                return None;
            }

            let u16_count = (byte_len as usize / 2) + 1;
            let mut buf = vec![0u16; u16_count];

            let status = RegQueryValueExW(
                hkey,
                PCWSTR(val_hstr.as_ptr()),
                None,
                Some(&mut val_type),
                Some(buf.as_mut_ptr() as *mut u8),
                Some(&mut byte_len),
            );
            if status != ERROR_SUCCESS {
                return None;
            }

            let actual_len = (byte_len as usize / 2).saturating_sub(1);
            let raw = String::from_utf16_lossy(&buf[..actual_len]);
            Some(raw.trim_matches('\0').to_string())
        }
    }
}
