pub mod cleaner;
pub mod elevation;
pub mod models;
pub mod scanner;
pub mod startup;
pub mod ipc;

use ipc::commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Activar privilegios de sistema para gestión de archivos bloqueados y servicios
    elevation::PrivilegeManager::enable_process_privileges();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_app_scan_stream,
            get_all_apps_instant,
            run_uninstall,
            execute_full_deep_uninstall,
            stream_full_uninstall,
            hunt_leftovers,
            clean_leftovers_batch,
            scan_orphaned_residuals,
            clean_orphaned_residuals,
            stream_clean_orphaned_residuals,
            get_residual_file_details,
            is_system_elevated,
            get_startup_entries,
            toggle_startup_entry,
            open_in_explorer,
            open_in_regedit,
            request_uac_elevation,
            reboot_system_now,
            window_minimize,
            window_maximize_toggle,
            window_close
        ])
        .run(tauri::generate_context!())
        .expect("Error al ejecutar el núcleo de Unistall");
}
