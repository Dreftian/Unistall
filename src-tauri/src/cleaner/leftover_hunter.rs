use crate::cleaner::restart_manager::FileLockerResolver;
use crate::cleaner::uninstaller_runner::UninstallerRunner;
use crate::models::app::InstalledApp;
use crate::models::leftover::{LeftoverItem, LeftoverType};
use std::fs;
use std::path::{Path, PathBuf};
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
use windows::Win32::Storage::FileSystem::{SetFileAttributesW, FILE_ATTRIBUTE_NORMAL};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteKeyExW, RegDeleteKeyW, RegDeleteTreeW, RegEnumKeyExW, RegOpenKeyExW,
    HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY,
    KEY_WOW64_64KEY, KEY_WRITE, REG_SAM_FLAGS,
};

pub struct LeftoverHunter;

impl LeftoverHunter {
    /// Rastrear restos exhaustivos en el sistema de archivos (incluyendo ocultos) y en el registro
    pub fn hunt(app: &InstalledApp) -> Vec<LeftoverItem> {
        let mut results = Vec::new();
        let app_name = app.display_name.trim();

        if app_name.is_empty() || Self::is_dangerous_system_word(app_name) {
            return results;
        }

        // 1. Escaneo en todas las ubicaciones de archivos y carpetas (visibles y ocultas)
        results.append(&mut Self::scan_filesystem_locations(app));

        // 2. Escaneo de accesos directos huérfanos (.lnk)
        results.append(&mut Self::scan_shortcuts(app));

        // 3. Escaneo en todas las ramas del Registro de Windows
        results.append(&mut Self::scan_registry_locations(app));

        // Deduplicar resultados por path_or_key
        results.sort_by(|a, b| a.path_or_key.cmp(&b.path_or_key));
        results.dedup_by(|a, b| a.path_or_key.eq_ignore_ascii_case(&b.path_or_key));

        results
    }

    /// Deduce el directorio raíz de instalación si no vino especificado en el registro
    pub fn deduce_install_directory(app: &InstalledApp) -> Option<PathBuf> {
        // 1. Desde display_icon (ej: "C:\Program Files\App\app.exe,0")
        if let Some(icon) = &app.display_icon {
            let clean = icon.split(',').next().unwrap_or("").trim().trim_matches('"');
            let p = Path::new(clean);
            if p.is_file() {
                if let Some(parent) = p.parent() {
                    let name = parent.file_name().unwrap_or_default().to_string_lossy();
                    if parent.exists() && !Self::is_dangerous_system_word(&name) {
                        return Some(parent.to_path_buf());
                    }
                }
            }
        }

        // 2. Desde uninstall_string (ej: "\"C:\Program Files\App\unins000.exe\" /silent")
        if let Some(uninst) = &app.uninstall_string {
            let (exe, _) = UninstallerRunner::parse_command_line(uninst);
            let p = Path::new(&exe);
            if p.exists() || p.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("exe")) {
                if let Some(parent) = p.parent() {
                    let name = parent.file_name().unwrap_or_default().to_string_lossy();
                    if parent.exists() && !Self::is_dangerous_system_word(&name) {
                        return Some(parent.to_path_buf());
                    }
                }
            }
        }

        None
    }

    /// Filtro de seguridad estricto para evitar borrar componentes compartidos de Windows
    pub fn is_dangerous_system_word(name: &str) -> bool {
        let blacklist = [
            "windows", "microsoft", "system", "system32", "syswow64", "program files",
            "users", "appdata", "local", "locallow", "roaming", "common files", "temp", "desktop",
            "intel", "amd", "nvidia", "realtek", "driver", "directx", "downloads", "documents",
            "start menu", "programs", "startup", "default", "all users", "public",
        ];
        let lower = name.trim().to_lowercase();
        blacklist.iter().any(|&item| lower == item)
    }

    /// Escanea AppData (Roaming, Local, LocalLow), ProgramData, ProgramFiles, UserProfile, Temp
    fn scan_filesystem_locations(app: &InstalledApp) -> Vec<LeftoverItem> {
        let mut items = Vec::new();
        let mut targets: Vec<(PathBuf, &'static str)> = Vec::new();

        if let Ok(p) = std::env::var("APPDATA") {
            targets.push((PathBuf::from(p), "Roaming AppData"));
        }
        if let Ok(p) = std::env::var("LOCALAPPDATA") {
            let local_path = PathBuf::from(&p);
            targets.push((local_path.clone(), "Local AppData"));
            // LocalLow (usado intensivamente por navegadores, apps Chromium y motores)
            if let Some(parent) = local_path.parent() {
                let locallow = parent.join("LocalLow");
                if locallow.exists() {
                    targets.push((locallow, "LocalLow AppData (Archivos Ocultos/Caché)"));
                }
            }
        }
        if let Ok(p) = std::env::var("ProgramData") {
            targets.push((PathBuf::from(p), "ProgramData (Datos globales)"));
        }
        if let Ok(p) = std::env::var("ProgramFiles") {
            targets.push((PathBuf::from(p), "Program Files (x64)"));
        }
        if let Ok(p) = std::env::var("ProgramFiles(x86)") {
            targets.push((PathBuf::from(p), "Program Files (x86)"));
        }
        if let Ok(p) = std::env::var("TEMP") {
            targets.push((PathBuf::from(p), "Archivos Temporales"));
        }
        if let Ok(p) = std::env::var("USERPROFILE") {
            targets.push((PathBuf::from(p), "Perfil de Usuario (Carpetas de Configuración)"));
        }

        let app_name_clean = Self::sanitize_token(&app.display_name);
        let publisher_clean = app.publisher.as_deref().map(Self::sanitize_token).unwrap_or_default();

        for (base_path, label) in targets {
            if !base_path.exists() || !base_path.is_dir() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(&base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    let dir_name_clean = Self::sanitize_token(&dir_name);

                    if Self::is_dangerous_system_word(&dir_name_clean) {
                        continue;
                    }

                    // 1. Coincidencia exacta con el nombre de la app (ej: Discord, Steam, Spotify)
                    if dir_name_clean == app_name_clean
                        || dir_name_clean == format!("{}.app", app_name_clean)
                        || dir_name_clean == format!(".{}", app_name_clean) // Carpetas ocultas estilo .vscode
                    {
                        items.push(LeftoverItem {
                            id: format!("fs_{}", path.to_string_lossy()),
                            item_type: if path.is_dir() { LeftoverType::Directory } else { LeftoverType::File },
                            path_or_key: path.to_string_lossy().to_string(),
                            confidence_score: 95,
                            size_bytes: Self::calculate_size(&path),
                            rationale: format!("Directorio/archivo asociado en {}", label),
                            is_selected: true,
                        });
                    }
                    // 2. Coincidencia con Publisher (ej: AppData\Google\Chrome o AppData\Valve\Steam)
                    else if !publisher_clean.is_empty() && dir_name_clean == publisher_clean {
                        let sub_app = path.join(&app.display_name);
                        if sub_app.exists() {
                            items.push(LeftoverItem {
                                id: format!("fs_{}", sub_app.to_string_lossy()),
                                item_type: LeftoverType::Directory,
                                path_or_key: sub_app.to_string_lossy().to_string(),
                                confidence_score: 98,
                                size_bytes: Self::calculate_size(&sub_app),
                                rationale: format!("Carpeta del distribuidor en {}", label),
                                is_selected: true,
                            });
                        }
                    }
                }
            }
        }

        // Comprobar la carpeta original de instalación (o deducida)
        let effective_loc = app.install_location.clone().or_else(|| Self::deduce_install_directory(app));
        if let Some(loc) = effective_loc {
            if loc.exists() {
                let loc_name = loc.file_name().unwrap_or_default().to_string_lossy();
                if !Self::is_dangerous_system_word(&loc_name) {
                    items.push(LeftoverItem {
                        id: format!("fs_install_{}", loc.to_string_lossy()),
                        item_type: if loc.is_dir() { LeftoverType::Directory } else { LeftoverType::File },
                        path_or_key: loc.to_string_lossy().to_string(),
                        confidence_score: 100,
                        size_bytes: Self::calculate_size(&loc),
                        rationale: "Directorio raíz de instalación del software".into(),
                        is_selected: true,
                    });
                }
            }
        }

        items
    }

    /// Rastrear accesos directos huérfanos (.lnk) en Escritorio y Menú Inicio
    fn scan_shortcuts(app: &InstalledApp) -> Vec<LeftoverItem> {
        let mut items = Vec::new();
        let mut shortcut_dirs = Vec::new();

        if let Ok(u) = std::env::var("USERPROFILE") {
            shortcut_dirs.push(PathBuf::from(u).join("Desktop"));
        }
        shortcut_dirs.push(PathBuf::from(r"C:\Users\Public\Desktop"));

        if let Ok(a) = std::env::var("APPDATA") {
            shortcut_dirs.push(PathBuf::from(a).join(r"Microsoft\Windows\Start Menu\Programs"));
        }
        if let Ok(p) = std::env::var("ProgramData") {
            shortcut_dirs.push(PathBuf::from(p).join(r"Microsoft\Windows\Start Menu\Programs"));
        }

        let app_name_lower = app.display_name.to_lowercase();

        for dir in shortcut_dirs {
            if !dir.exists() {
                continue;
            }

            for entry in walkdir::WalkDir::new(&dir).max_depth(3).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case("lnk") {
                        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy().to_lowercase();
                        if file_stem == app_name_lower || file_stem.starts_with(&app_name_lower) {
                            items.push(LeftoverItem {
                                id: format!("lnk_{}", path.to_string_lossy()),
                                item_type: LeftoverType::File,
                                path_or_key: path.to_string_lossy().to_string(),
                                confidence_score: 95,
                                size_bytes: entry.metadata().ok().map(|m| m.len()),
                                rationale: "Acceso directo (.lnk) residual de la aplicación".into(),
                                is_selected: true,
                            });
                        }
                    }
                }
            }
        }

        items
    }

    /// Rastrear claves huérfanas en HKCU y HKLM Software, WOW6432Node y App Paths
    fn scan_registry_locations(app: &InstalledApp) -> Vec<LeftoverItem> {
        let mut items = Vec::new();
        let app_name = &app.display_name;

        let hives = [
            (HKEY_CURRENT_USER, "HKCU", "Software"),
            (HKEY_LOCAL_MACHINE, "HKLM", "Software"),
            (HKEY_LOCAL_MACHINE, "HKLM", "Software\\WOW6432Node"),
        ];

        for (root, prefix, parent_branch) in hives {
            let key_path = format!("{}\\{}", parent_branch, app_name);
            if Self::registry_key_exists(root, &key_path) {
                items.push(LeftoverItem {
                    id: format!("reg_{}_{}", prefix, key_path),
                    item_type: LeftoverType::RegistryKey,
                    path_or_key: format!("{}\\{}", prefix, key_path),
                    confidence_score: 95,
                    size_bytes: None,
                    rationale: format!("Rama de configuración de software en {}\\{}", prefix, parent_branch),
                    is_selected: true,
                });
            }

            // Si hay publisher conocido, comprobar Software\<Publisher>\<AppName>
            if let Some(pub_name) = &app.publisher {
                if !pub_name.is_empty() && !Self::is_dangerous_system_word(pub_name) {
                    let pub_key_path = format!("{}\\{}\\{}", parent_branch, pub_name, app_name);
                    if Self::registry_key_exists(root, &pub_key_path) {
                        items.push(LeftoverItem {
                            id: format!("reg_pub_{}_{}", prefix, pub_key_path),
                            item_type: LeftoverType::RegistryKey,
                            path_or_key: format!("{}\\{}", prefix, pub_key_path),
                            confidence_score: 98,
                            size_bytes: None,
                            rationale: "Rama específica de editor/aplicación en registro".into(),
                            is_selected: true,
                        });
                    }
                }
            }
        }

        // Si tiene la subclave de desinstalación registrada (HKLM/HKCU ... Uninstall\...)
        if let Some(uninst_path) = &app.registry_key_path {
            items.push(LeftoverItem {
                id: format!("reg_uninst_{}", uninst_path),
                item_type: LeftoverType::RegistryKey,
                path_or_key: uninst_path.clone(),
                confidence_score: 100,
                size_bytes: None,
                rationale: "Entrada principal de desinstalación en el Registro de Windows".into(),
                is_selected: true,
            });
        }

        items
    }

    pub fn registry_key_exists(root: HKEY, subkey: &str) -> bool {
        let path_hstr = HSTRING::from(subkey);
        let mut hkey = HKEY::default();
        unsafe {
            let status = RegOpenKeyExW(root, PCWSTR(path_hstr.as_ptr()), 0, KEY_READ, &mut hkey);
            if status == ERROR_SUCCESS {
                let _ = RegCloseKey(hkey);
                true
            } else {
                false
            }
        }
    }

    fn sanitize_token(s: &str) -> String {
        s.trim().to_lowercase().replace(['"', '\'', '(', ')', '[', ']'], "")
    }

    pub fn calculate_size(path: &Path) -> Option<u64> {
        if path.is_file() {
            return fs::metadata(path).ok().map(|m| m.len());
        }
        let mut total = 0u64;
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                }
            }
        }
        Some(total)
    }

    /// Elimina de forma forzada un resto del sistema de archivos o del registro
    pub fn delete_leftover(item: &LeftoverItem) -> Result<(), String> {
        match item.item_type {
            LeftoverType::Directory => {
                let p = Path::new(&item.path_or_key);
                if p.exists() {
                    Self::force_remove_directory_recursive(p)?;
                }
                Ok(())
            }
            LeftoverType::File => {
                let p = Path::new(&item.path_or_key);
                if p.exists() {
                    Self::force_remove_single_file(p)?;
                }
                Ok(())
            }
            LeftoverType::RegistryKey => {
                Self::delete_registry_key_recursive(&item.path_or_key)
            }
            _ => Ok(()),
        }
    }

    /// Elimina un archivo individual, limpiando atributos de solo lectura si es necesario
    pub fn force_remove_single_file(path: &Path) -> Result<(), String> {
        // Desactivar flags de solo lectura / oculto en Win32
        let path_hstr = HSTRING::from(path.to_string_lossy().as_ref());
        unsafe {
            let _ = SetFileAttributesW(PCWSTR(path_hstr.as_ptr()), FILE_ATTRIBUTE_NORMAL);
        }

        if fs::remove_file(path).is_err() {
            // Si está bloqueado, intentar desbloquear con Restart Manager
            let _ = FileLockerResolver::unlock_and_terminate_holding_processes(path);
            if fs::remove_file(path).is_err() {
                // Si aún sigue ocupado, programar borrado en el siguiente arranque con MoveFileExW
                FileLockerResolver::schedule_delete_on_reboot(path)?;
            }
        }
        Ok(())
    }

    /// Elimina recursivamente un directorio entero, tratando cada archivo oculto o de solo lectura
    pub fn force_remove_directory_recursive(dir_path: &Path) -> Result<(), String> {
        for entry in walkdir::WalkDir::new(dir_path)
            .contents_first(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            let p_hstr = HSTRING::from(p.to_string_lossy().as_ref());
            unsafe {
                let _ = SetFileAttributesW(PCWSTR(p_hstr.as_ptr()), FILE_ATTRIBUTE_NORMAL);
            }

            if p.is_file() {
                let _ = Self::force_remove_single_file(p);
            } else if p.is_dir() && p != dir_path {
                let _ = fs::remove_dir(p);
            }
        }

        // Eliminar el directorio raíz contenedor
        if dir_path.exists() {
            if fs::remove_dir_all(dir_path).is_err() {
                let _ = fs::remove_dir(dir_path);
            }
        }
        Ok(())
    }

    /// Elimina una clave del Registro de Windows y TODOS sus subárboles de manera recursiva
    pub fn delete_registry_key_recursive(full_key_str: &str) -> Result<(), String> {
        let parts: Vec<&str> = full_key_str.splitn(2, '\\').collect();
        if parts.len() != 2 {
            return Err("Ruta de clave inválida".into());
        }

        let (root, is_hklm) = match parts[0] {
            "HKCU" => (HKEY_CURRENT_USER, false),
            "HKLM" => (HKEY_LOCAL_MACHINE, true),
            _ => return Err("Hive no soportado".into()),
        };

        let subkey = parts[1];

        // Para HKLM, intentar tanto en la vista nativa de 64 bits como en WOW6432Node
        let mut success = false;

        if is_hklm {
            // Intentar con vista de 64 bits
            if Self::delete_registry_tree_internal(root, subkey, KEY_WOW64_64KEY) {
                success = true;
            }
            // Intentar con vista de 32 bits
            if Self::delete_registry_tree_internal(root, subkey, KEY_WOW64_32KEY) {
                success = true;
            }
        } else {
            if Self::delete_registry_tree_internal(root, subkey, REG_SAM_FLAGS(0)) {
                success = true;
            }
        }

        if success {
            Ok(())
        } else {
            // Si ya no existe, considerarlo un éxito
            if !Self::registry_key_exists(root, subkey) {
                Ok(())
            } else {
                Err(format!("No se pudo eliminar la rama del registro: {}", full_key_str))
            }
        }
    }

    fn delete_registry_tree_internal(root: HKEY, subkey: &str, sam_extra: REG_SAM_FLAGS) -> bool {
        let subkey_hstr = HSTRING::from(subkey);

        // 1. Intentar RegDeleteTreeW directamente
        unsafe {
            let tree_res = RegDeleteTreeW(root, PCWSTR(subkey_hstr.as_ptr()));
            if tree_res == ERROR_SUCCESS {
                // Ahora eliminar la clave raíz de la rama
                let del_status = RegDeleteKeyExW(root, PCWSTR(subkey_hstr.as_ptr()), sam_extra.0, 0);
                if del_status == ERROR_SUCCESS {
                    return true;
                }
            }
        }

        // 2. Eliminación recursiva manual de subclaves si RegDeleteTreeW falló
        Self::delete_subkeys_recursively(root, subkey, sam_extra);

        // 3. Eliminar la clave después de vaciar sus hijos
        unsafe {
            let del_status = RegDeleteKeyExW(root, PCWSTR(subkey_hstr.as_ptr()), sam_extra.0, 0);
            if del_status == ERROR_SUCCESS {
                return true;
            }
            let fallback_status = RegDeleteKeyW(root, PCWSTR(subkey_hstr.as_ptr()));
            fallback_status == ERROR_SUCCESS
        }
    }

    fn delete_subkeys_recursively(root: HKEY, subkey_path: &str, sam: REG_SAM_FLAGS) {
        let subkey_hstr = HSTRING::from(subkey_path);
        let mut hkey = HKEY::default();

        unsafe {
            let open_status = RegOpenKeyExW(
                root,
                PCWSTR(subkey_hstr.as_ptr()),
                0,
                KEY_READ | KEY_WRITE | sam,
                &mut hkey,
            );

            if open_status != ERROR_SUCCESS {
                return;
            }

            let mut child_keys = Vec::new();
            let mut index = 0u32;
            let mut buf = [0u16; 512];

            loop {
                let mut len = buf.len() as u32;
                let st = RegEnumKeyExW(
                    hkey,
                    index,
                    PWSTR(buf.as_mut_ptr()),
                    &mut len,
                    None,
                    PWSTR::null(),
                    None,
                    None,
                );

                if st == ERROR_NO_MORE_ITEMS {
                    break;
                } else if st == ERROR_SUCCESS {
                    let name = String::from_utf16_lossy(&buf[..len as usize]);
                    child_keys.push(name);
                    index += 1;
                } else {
                    break;
                }
            }

            let _ = RegCloseKey(hkey);

            for child in child_keys {
                let child_full = format!("{}\\{}", subkey_path, child);
                Self::delete_subkeys_recursively(root, &child_full, sam);
                let child_hstr = HSTRING::from(child_full.as_str());
                let _ = RegDeleteKeyExW(root, PCWSTR(child_hstr.as_ptr()), sam.0, 0);
            }
        }
    }
}
