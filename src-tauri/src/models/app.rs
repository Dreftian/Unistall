use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppArchitecture {
    X64,
    X86,
    Arm64,
    Universal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppInstallSource {
    Win32Registry,
    WindowsStoreUwp,
    Winget,
    Scoop,
    Chocolatey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RegistryHiveSource {
    Hklm64,
    Hklm32,
    Hkcu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApp {
    pub id: String,
    pub display_name: String,
    pub display_version: Option<String>,
    pub publisher: Option<String>,
    pub install_date: Option<String>,
    pub install_location: Option<PathBuf>,
    pub uninstall_string: Option<String>,
    pub quiet_uninstall_string: Option<String>,
    pub display_icon: Option<String>,
    pub icon_base64: Option<String>,
    pub estimated_size_kb: Option<u64>,
    pub real_size_bytes: Option<u64>,
    pub architecture: AppArchitecture,
    pub source: AppInstallSource,
    pub registry_hive: Option<RegistryHiveSource>,
    pub registry_key_path: Option<String>,
    pub is_system_component: bool,
    pub is_startup_enabled: bool,
    pub windows_installer_guid: Option<String>,

    // Inteligencia y seguridad del sistema
    pub description: String,
    pub category: String,
    pub is_critical_system: bool,
    pub system_risk_warning: Option<String>,
}

impl InstalledApp {
    pub fn generate_id(hive: Option<RegistryHiveSource>, subkey_name: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        if let Some(h) = hive {
            format!("{:?}", h).hash(&mut hasher);
        }
        subkey_name.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}
