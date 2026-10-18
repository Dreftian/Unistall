use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::{
    AdjustTokenPrivileges, GetTokenInformation, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES,
    SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_ELEVATION, TOKEN_PRIVILEGES, TOKEN_QUERY,
    TokenElevation,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

pub struct PrivilegeManager;

impl PrivilegeManager {
    /// Comprueba si el proceso actual tiene permisos de Administrador / Elevado
    pub fn is_elevated() -> bool {
        unsafe {
            let mut token: HANDLE = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION::default();
            let mut returned_len = 0u32;
            let status = GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut returned_len,
            );

            if status.is_ok() {
                elevation.TokenIsElevated != 0
            } else {
                false
            }
        }
    }

    /// Habilita privilegios del sistema en el token (SeDebugPrivilege, etc.) para desbloquear y matar procesos
    pub fn enable_process_privileges() {
        let privileges = [
            "SeDebugPrivilege",
            "SeTakeOwnershipPrivilege",
            "SeBackupPrivilege",
            "SeRestorePrivilege",
        ];

        for priv_name in privileges {
            let _ = Self::enable_single_privilege(priv_name);
        }
    }

    fn enable_single_privilege(priv_name: &str) -> bool {
        unsafe {
            let mut token: HANDLE = HANDLE::default();
            if OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                &mut token,
            )
            .is_err()
            {
                return false;
            }

            let priv_hstr = HSTRING::from(priv_name);
            let mut luid = windows::Win32::Foundation::LUID::default();

            if LookupPrivilegeValueW(PCWSTR::null(), PCWSTR(priv_hstr.as_ptr()), &mut luid).is_err() {
                return false;
            }

            let tp = TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                Privileges: [LUID_AND_ATTRIBUTES {
                    Luid: luid,
                    Attributes: SE_PRIVILEGE_ENABLED,
                }],
            };

            AdjustTokenPrivileges(token, false, Some(&tp), 0, None, None).is_ok()
        }
    }

    /// Lanza una instancia elevada del proceso actual mediante ShellExecuteW "runas"
    pub fn elevate_current_process() -> Result<(), String> {
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_hstr = HSTRING::from(exe_path.to_string_lossy().as_ref());
        let verb_hstr = HSTRING::from("runas");

        unsafe {
            let instance = ShellExecuteW(
                None,
                PCWSTR(verb_hstr.as_ptr()),
                PCWSTR(exe_hstr.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );

            if instance.0 as isize > 32 {
                Ok(())
            } else {
                Err("El usuario canceló la elevación UAC".into())
            }
        }
    }
}
