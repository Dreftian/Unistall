use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LeftoverType {
    Directory,
    File,
    RegistryKey,
    RegistryValue,
    ComClass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeftoverItem {
    pub id: String,
    pub item_type: LeftoverType,
    pub path_or_key: String,
    pub confidence_score: u8, // 0 - 100
    pub size_bytes: Option<u64>,
    pub rationale: String,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupEntry {
    pub id: String,
    pub name: String,
    pub command: String,
    pub location: String, // "HKCU\Run", "HKLM\Run", "StartupFolder"
    pub enabled: bool,
    pub publisher: Option<String>,
}
