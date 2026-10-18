use crate::models::app::{AppArchitecture, AppInstallSource, InstalledApp};
use std::fs;
use std::path::PathBuf;

pub struct CliScanner;

impl CliScanner {
    pub fn scan_all() -> Vec<InstalledApp> {
        let mut apps = Vec::new();
        apps.append(&mut Self::scan_scoop());
        apps.append(&mut Self::scan_chocolatey());
        apps
    }

    /// Escanea Scoop leyendo directamente el directorio %USERPROFILE%\scoop\apps
    fn scan_scoop() -> Vec<InstalledApp> {
        let mut list = Vec::new();
        let user_profile = match std::env::var("USERPROFILE") {
            Ok(p) => PathBuf::from(p),
            Err(_) => return list,
        };

        let scoop_apps_dir = user_profile.join("scoop").join("apps");
        if !scoop_apps_dir.exists() || !scoop_apps_dir.is_dir() {
            return list;
        }

        if let Ok(entries) = fs::read_dir(&scoop_apps_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let app_name = entry.file_name().to_string_lossy().to_string();
                    if app_name.eq_ignore_ascii_case("scoop") {
                        continue;
                    }

                    // Determinar versión buscando la subcarpeta o enlace simbólico 'current'
                    let current_link = path.join("current");
                    let mut version = None;
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub in sub_entries.flatten() {
                            let sub_name = sub.file_name().to_string_lossy().to_string();
                            if sub_name != "current" && sub.path().is_dir() {
                                version = Some(sub_name);
                                break;
                            }
                        }
                    }

                    let uninstall_cmd = format!("scoop uninstall {}", app_name);

                    let info = crate::models::intelligence::AppIntelligence::evaluate(&app_name, Some("Scoop"));

                    list.push(InstalledApp {
                        id: format!("scoop_{}", app_name),
                        display_name: format!("{} (Scoop)", app_name),
                        display_version: version,
                        publisher: Some("Scoop Package Manager".into()),
                        install_date: None,
                        install_location: Some(if current_link.exists() { current_link } else { path }),
                        uninstall_string: Some(uninstall_cmd.clone()),
                        quiet_uninstall_string: Some(uninstall_cmd),
                        display_icon: None,
                        icon_base64: None,
                        estimated_size_kb: None,
                        real_size_bytes: None,
                        architecture: AppArchitecture::X64,
                        source: AppInstallSource::Scoop,
                        registry_hive: None,
                        registry_key_path: None,
                        is_system_component: false,
                        is_startup_enabled: false,
                        windows_installer_guid: None,
                        description: info.description,
                        category: info.category,
                        is_critical_system: info.is_critical,
                        system_risk_warning: info.risk_warning,
                    });
                }
            }
        }

        list
    }

    /// Escanea Chocolatey inspeccionando %ProgramData%\chocolatey\lib
    fn scan_chocolatey() -> Vec<InstalledApp> {
        let mut list = Vec::new();
        let program_data = match std::env::var("ProgramData") {
            Ok(p) => PathBuf::from(p),
            Err(_) => return list,
        };

        let choco_lib = program_data.join("chocolatey").join("lib");
        if !choco_lib.exists() || !choco_lib.is_dir() {
            return list;
        }

        if let Ok(entries) = fs::read_dir(&choco_lib) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let app_name = entry.file_name().to_string_lossy().to_string();
                    if app_name.eq_ignore_ascii_case("chocolatey") {
                        continue;
                    }

                    let uninstall_cmd = format!("choco uninstall {} -y", app_name);
                    let info = crate::models::intelligence::AppIntelligence::evaluate(&app_name, Some("Chocolatey"));

                    list.push(InstalledApp {
                        id: format!("choco_{}", app_name),
                        display_name: format!("{} (Chocolatey)", app_name),
                        display_version: None,
                        publisher: Some("Chocolatey".into()),
                        install_date: None,
                        install_location: Some(path),
                        uninstall_string: Some(uninstall_cmd.clone()),
                        quiet_uninstall_string: Some(uninstall_cmd),
                        display_icon: None,
                        icon_base64: None,
                        estimated_size_kb: None,
                        real_size_bytes: None,
                        architecture: AppArchitecture::X64,
                        source: AppInstallSource::Chocolatey,
                        registry_hive: None,
                        registry_key_path: None,
                        is_system_component: false,
                        is_startup_enabled: false,
                        windows_installer_guid: None,
                        description: info.description,
                        category: info.category,
                        is_critical_system: info.is_critical,
                        system_risk_warning: info.risk_warning,
                    });
                }
            }
        }

        list
    }
}
