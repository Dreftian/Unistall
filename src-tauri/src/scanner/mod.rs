pub mod cli_managers;
pub mod icons;
pub mod registry;
pub mod uwp;

use crate::models::app::InstalledApp;
use rayon::prelude::*;

pub struct MasterScanner;

impl MasterScanner {
    /// Ejecuta el escáner multi-fuente en paralelo: Win32, UWP, CLI
    pub fn scan_everything() -> Vec<InstalledApp> {
        let mut apps: Vec<InstalledApp> = Vec::new();

        // 1. Escaneo nativo de Registro (HKLM64, HKLM32, HKCU)
        let mut registry_apps = registry::RegistryScanner::scan_all_targets();
        apps.append(&mut registry_apps);

        // 2. Escaneo de paquetes UWP/MSIX de Windows Store
        let mut uwp_apps = uwp::UwpScanner::scan_packages();
        apps.append(&mut uwp_apps);

        // 3. Escaneo de gestores CLI (Winget, Scoop, Chocolatey)
        let mut cli_apps = cli_managers::CliScanner::scan_all();
        apps.append(&mut cli_apps);

        // Deduplicación por nombre / ruta
        apps.par_sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));
        apps.dedup_by(|a, b| {
            a.display_name.eq_ignore_ascii_case(&b.display_name)
                && a.display_version == b.display_version
        });

        apps
    }
}
