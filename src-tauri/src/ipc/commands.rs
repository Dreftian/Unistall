use crate::cleaner::leftover_hunter::LeftoverHunter;
use crate::cleaner::orphan_scanner::{
    DeepCleanResult, OrphanScanner, OrphanedItem, ResidualCleanProgressUpdate, ResidualFileDetail,
};
use crate::cleaner::uninstaller_runner::{UninstallResult, UninstallerRunner};
use crate::elevation::PrivilegeManager;
use crate::models::app::InstalledApp;
use crate::models::leftover::{LeftoverItem, LeftoverType, StartupEntry};
use crate::scanner::MasterScanner;
use crate::startup::run_keys::StartupManager;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use tauri::command;
use tauri::ipc::Channel;

#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
pub enum ScanProgressEvent {
    Batch { apps: Vec<InstalledApp> },
    Progress { percentage: u8, stage: String, count: usize },
    Status { message: String, scanned_count: usize },
    Finished { total_count: usize },
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallOptions {
    pub run_official_uninstaller: bool,
    pub remove_installation_directory: bool,
    pub remove_app_data: bool,
    pub remove_local_low_and_hidden: bool,
    pub remove_registry_keys: bool,
    pub remove_shortcuts: bool,
    pub remove_temp_and_junk: bool,
}

#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
pub enum UninstallProgressUpdate {
    Step { percentage: u8, task_name: String, current_item: String },
    Finished { total_items_deleted: usize, total_bytes_freed: u64, summary: Vec<String> },
    Error { message: String },
}

#[command]
pub async fn start_app_scan_stream(on_event: Channel<ScanProgressEvent>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _ = on_event.send(ScanProgressEvent::Progress {
            percentage: 5,
            stage: "Inicializando motor de escaneo nativo...".into(),
            count: 0,
        });

        // 1. Escaneo de registro Win32 HKLM 64-bit y WoW64
        let _ = on_event.send(ScanProgressEvent::Progress {
            percentage: 20,
            stage: "Analizando HKLM (64-bit) y WoW6432Node (32-bit)...".into(),
            count: 0,
        });

        // 2. Escaneo de Registro HKCU
        let _ = on_event.send(ScanProgressEvent::Progress {
            percentage: 45,
            stage: "Consultando aplicaciones por usuario (HKCU)...".into(),
            count: 0,
        });

        // 3. UWP / Store
        let _ = on_event.send(ScanProgressEvent::Progress {
            percentage: 65,
            stage: "Leyendo repositorio Windows.ApplicationModel (MSIX/Store)...".into(),
            count: 0,
        });

        // 4. Gestores CLI y Extracción de iconos
        let _ = on_event.send(ScanProgressEvent::Progress {
            percentage: 80,
            stage: "Extrayendo iconos nativos y evaluando seguridad del sistema...".into(),
            count: 0,
        });

        let apps = MasterScanner::scan_everything();
        let total = apps.len();

        let _ = on_event.send(ScanProgressEvent::Progress {
            percentage: 95,
            stage: format!("Indexando {} aplicaciones detectadas...", total),
            count: total,
        });

        // Enviar en chunks de 30 elementos para actualización fluida de la vista a 120 FPS
        const CHUNK_SIZE: usize = 30;
        for chunk in apps.chunks(CHUNK_SIZE) {
            let _ = on_event.send(ScanProgressEvent::Batch {
                apps: chunk.to_vec(),
            });
        }

        let _ = on_event.send(ScanProgressEvent::Progress {
            percentage: 100,
            stage: "Escaneo completado con éxito.".into(),
            count: total,
        });

        let _ = on_event.send(ScanProgressEvent::Finished { total_count: total });
    });

    Ok(())
}

#[command]
pub async fn get_all_apps_instant() -> Result<Vec<InstalledApp>, String> {
    Ok(MasterScanner::scan_everything())
}

#[command]
pub async fn run_uninstall(uninstall_string: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (exe, args) = UninstallerRunner::parse_command_line(&uninstall_string);
        let mut cmd = Command::new(&exe);
        cmd.args(&args);
        let _ = cmd.spawn().map_err(|e| e.to_string())?;
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Ejecuta la desinstalación profunda verificando ejecución real y erradicación total
#[command]
pub async fn stream_full_uninstall(
    app: InstalledApp,
    options: UninstallOptions,
    on_progress: Channel<UninstallProgressUpdate>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        // Asegurar privilegios de sistema elevados
        PrivilegeManager::enable_process_privileges();

        // Paso 1: Desinstalador oficial (0% - 30%)
        if options.run_official_uninstaller {
            let _ = on_progress.send(UninstallProgressUpdate::Step {
                percentage: 10,
                task_name: "Invocando desinstalador oficial del software...".into(),
                current_item: app.display_name.clone(),
            });

            let uninst_res = UninstallerRunner::execute(&app);
            match uninst_res {
                UninstallResult::Success { message } => {
                    let _ = on_progress.send(UninstallProgressUpdate::Step {
                        percentage: 30,
                        task_name: format!("Desinstalador finalizado: {}", message),
                        current_item: "Verificando estado post-desinstalación...".into(),
                    });
                }
                UninstallResult::NeedsForceCleanup { reason } => {
                    let _ = on_progress.send(UninstallProgressUpdate::Step {
                        percentage: 30,
                        task_name: format!("Aviso: {}. Procediendo a erradicación...", reason),
                        current_item: "Iniciando motor de limpieza profunda".into(),
                    });
                }
                UninstallResult::Error { message } => {
                    let _ = on_progress.send(UninstallProgressUpdate::Step {
                        percentage: 30,
                        task_name: format!("Error en desinstalador oficial: {}. Continuando con limpieza forzada...", message),
                        current_item: "Erradicación forzada activada".into(),
                    });
                }
            }
        }

        let _ = on_progress.send(UninstallProgressUpdate::Step {
            percentage: 35,
            task_name: "Rastreando archivos de instalación, carpetas ocultas y ramas de registro...".into(),
            current_item: app.display_name.clone(),
        });

        std::thread::sleep(std::time::Duration::from_millis(300));

        // Paso 2: Cacería exhaustiva de restos en sistema de archivos y registro
        let mut leftovers = LeftoverHunter::hunt(&app);
        let total_candidates = leftovers.len();

        let _ = on_progress.send(UninstallProgressUpdate::Step {
            percentage: 45,
            task_name: format!("Se detectaron {} elementos residuales asociados. Iniciando erradicación...", total_candidates),
            current_item: format!("{} elementos", total_candidates),
        });

        let mut deleted_count = 0;
        let mut bytes_freed = 0u64;
        let mut summary = Vec::new();

        // Paso 3: Erradicación progresiva (45% a 90%)
        for (i, item) in leftovers.iter_mut().enumerate() {
            // Filtrar según opciones del usuario
            let should_delete = match item.item_type {
                LeftoverType::RegistryKey | LeftoverType::RegistryValue => options.remove_registry_keys,
                LeftoverType::File => {
                    if item.path_or_key.ends_with(".lnk") {
                        options.remove_shortcuts
                    } else {
                        options.remove_temp_and_junk || options.remove_app_data
                    }
                }
                LeftoverType::Directory => {
                    let lower = item.path_or_key.to_lowercase();
                    if lower.contains("locallow") {
                        options.remove_local_low_and_hidden
                    } else if item.id.starts_with("fs_install_") {
                        options.remove_installation_directory
                    } else {
                        options.remove_app_data
                    }
                }
                _ => true,
            };

            if should_delete {
                let current_pct = 45 + (((i + 1) * 45) / total_candidates.max(1)) as u8;
                let _ = on_progress.send(UninstallProgressUpdate::Step {
                    percentage: current_pct.min(90),
                    task_name: format!("Erradicando: {}", item.rationale),
                    current_item: item.path_or_key.clone(),
                });

                if LeftoverHunter::delete_leftover(item).is_ok() {
                    deleted_count += 1;
                    if let Some(sz) = item.size_bytes {
                        bytes_freed += sz;
                    }
                    summary.push(format!("{}: {}", if item.item_type == LeftoverType::RegistryKey { "Clave" } else { "Archivo" }, item.path_or_key));
                }
            }
        }

        // Paso 4: Comprobación estricta post-desinstalación (90% - 100%)
        let _ = on_progress.send(UninstallProgressUpdate::Step {
            percentage: 95,
            task_name: "Verificando erradicación total en el Registro de Windows...".into(),
            current_item: "Garantía de limpieza total".into(),
        });

        // Garantizar que la entrada de desinstalación de Windows desapareció
        if options.remove_registry_keys {
            if let Some(uninst_key) = &app.registry_key_path {
                let _ = LeftoverHunter::delete_registry_key_recursive(uninst_key);
            }
        }

        // Garantizar que el directorio raíz de instalación fue eliminado
        if options.remove_installation_directory {
            if let Some(loc) = &app.install_location {
                if loc.exists() {
                    let _ = LeftoverHunter::force_remove_directory_recursive(loc);
                }
            }
            if let Some(deduced) = LeftoverHunter::deduce_install_directory(&app) {
                if deduced.exists() {
                    let _ = LeftoverHunter::force_remove_directory_recursive(&deduced);
                }
            }
        }

        let _ = on_progress.send(UninstallProgressUpdate::Step {
            percentage: 100,
            task_name: "¡Desinstalación y erradicación completada exitosamente al 100%!".into(),
            current_item: "Finalizado".into(),
        });

        let _ = on_progress.send(UninstallProgressUpdate::Finished {
            total_items_deleted: deleted_count,
            total_bytes_freed: bytes_freed,
            summary,
        });
    });

    Ok(())
}

#[command]
pub async fn execute_full_deep_uninstall(app: InstalledApp, wait_for_official: bool) -> Result<DeepCleanResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        PrivilegeManager::enable_process_privileges();

        if wait_for_official {
            let _ = UninstallerRunner::execute(&app);
        }

        std::thread::sleep(std::time::Duration::from_millis(400));

        let leftovers = LeftoverHunter::hunt(&app);
        let mut deleted_count = 0;
        let mut bytes_freed = 0u64;
        let mut details = Vec::new();

        for item in &leftovers {
            if LeftoverHunter::delete_leftover(item).is_ok() {
                deleted_count += 1;
                if let Some(sz) = item.size_bytes {
                    bytes_freed += sz;
                }
                details.push(format!("{}: {}", if item.item_type == LeftoverType::RegistryKey { "Clave" } else { "Archivo" }, item.path_or_key));
            }
        }

        if let Some(uninst_key) = &app.registry_key_path {
            let _ = LeftoverHunter::delete_registry_key_recursive(uninst_key);
        }

        Ok(DeepCleanResult {
            total_items_deleted: deleted_count,
            total_bytes_freed: bytes_freed,
            details,
            reboot_required: false,
            reboot_reason: None,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn hunt_leftovers(app: InstalledApp) -> Result<Vec<LeftoverItem>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        Ok(LeftoverHunter::hunt(&app))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn clean_leftovers_batch(items: Vec<LeftoverItem>) -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || {
        PrivilegeManager::enable_process_privileges();
        let mut count = 0;
        for item in items {
            if item.is_selected {
                if LeftoverHunter::delete_leftover(&item).is_ok() {
                    count += 1;
                }
            }
        }
        Ok(count)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Escanea todo el sistema en busca de archivos residuales y carpetas huérfanas de apps desinstaladas en el pasado
#[command]
pub async fn scan_orphaned_residuals() -> Result<Vec<OrphanedItem>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        Ok(OrphanScanner::scan_orphaned_residuals())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Elimina carpetas y archivos residuales de apps antiguas emitiendo progreso en tiempo real
#[command]
pub async fn stream_clean_orphaned_residuals(
    items: Vec<OrphanedItem>,
    on_progress: Channel<ResidualCleanProgressUpdate>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        PrivilegeManager::enable_process_privileges();
        let _ = OrphanScanner::stream_delete_orphaned_items(&items, on_progress);
    });
    Ok(())
}

/// Obtiene los archivos específicos dentro de una carpeta residual
#[command]
pub async fn get_residual_file_details(path: String) -> Result<Vec<ResidualFileDetail>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let p = Path::new(&path);
        Ok(OrphanScanner::get_folder_file_details(p))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Elimina carpetas y archivos residuales de apps antiguas seleccionadas (compatibilidad síncrona)
#[command]
pub async fn clean_orphaned_residuals(items: Vec<OrphanedItem>) -> Result<DeepCleanResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        PrivilegeManager::enable_process_privileges();
        OrphanScanner::delete_orphaned_items(&items)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Comprueba si la aplicación está ejecutándose como Administrador / Elevado
#[command]
pub async fn is_system_elevated() -> Result<bool, String> {
    Ok(PrivilegeManager::is_elevated())
}

#[command]
pub async fn get_startup_entries() -> Result<Vec<StartupEntry>, String> {
    Ok(StartupManager::get_all_startup_entries())
}

#[command]
pub async fn toggle_startup_entry(id: String) -> Result<(), String> {
    StartupManager::disable_startup_entry(&id)
}

#[command]
pub async fn open_in_explorer(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    let target = if p.is_file() {
        format!("/select,\"{}\"", path)
    } else {
        format!("\"{}\"", path)
    };

    let _ = Command::new("explorer")
        .arg(&target)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub async fn open_in_regedit() -> Result<(), String> {
    let _ = Command::new("regedit")
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub async fn request_uac_elevation() -> Result<(), String> {
    PrivilegeManager::elevate_current_process()
}

#[command]
pub async fn reboot_system_now() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| {
        let _ = Command::new("shutdown")
            .args(["/r", "/t", "5", "/c", "Unistall: Reiniciando para completar la limpieza de archivos residuales bloqueados."])
            .spawn();
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Controles nativos para ventana Frameless (Sin bordes nativos de Windows)
#[command]
pub async fn window_minimize(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[command]
pub async fn window_maximize_toggle(window: tauri::Window) -> Result<(), String> {
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

#[command]
pub async fn window_close(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}
