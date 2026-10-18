use std::path::Path;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_DELAY_UNTIL_REBOOT};
use windows::Win32::System::RestartManager::{
    RmEndSession, RmRegisterResources, RmShutdown, RmStartSession,
};

pub struct FileLockerResolver;

impl FileLockerResolver {
    /// Programa la eliminación del archivo bloqueado en el próximo reinicio usando MoveFileExW
    pub fn schedule_delete_on_reboot(file_path: &Path) -> Result<(), String> {
        let path_str = file_path.to_string_lossy().to_string();
        let path_hstr = HSTRING::from(path_str.as_str());

        unsafe {
            let success = MoveFileExW(
                PCWSTR(path_hstr.as_ptr()),
                PCWSTR::null(),
                MOVEFILE_DELAY_UNTIL_REBOOT,
            );

            if success.is_ok() {
                Ok(())
            } else {
                Err("No se pudo programar la eliminación con MoveFileExW".into())
            }
        }
    }

    /// Intenta desbloquear archivos usando Windows Restart Manager API
    pub fn unlock_and_terminate_holding_processes(file_path: &Path) -> Result<(), String> {
        let mut session_handle = 0u32;
        let mut session_key = [0u16; 64];

        unsafe {
            let status = RmStartSession(&mut session_handle, 0, windows::core::PWSTR(session_key.as_mut_ptr()));
            if status != ERROR_SUCCESS {
                return Err(format!("RmStartSession falló: {:?}", status));
            }

            let path_str = file_path.to_string_lossy().to_string();
            let path_hstr = HSTRING::from(path_str.as_str());
            let files = [PCWSTR(path_hstr.as_ptr())];

            let reg_status = RmRegisterResources(
                session_handle,
                Some(&files),
                None,
                None,
            );

            if reg_status == ERROR_SUCCESS {
                // 1 = RmForceShutdown
                let _ = RmShutdown(session_handle, 1, None);
            }

            let _ = RmEndSession(session_handle);
        }

        Ok(())
    }
}
