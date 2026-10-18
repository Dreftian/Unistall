use crate::models::leftover::StartupEntry;
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegEnumValueW, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER,
    HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE, REG_SZ,
};

const RUN_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

pub struct StartupManager;

impl StartupManager {
    pub fn get_all_startup_entries() -> Vec<StartupEntry> {
        let mut entries = Vec::new();
        entries.append(&mut Self::scan_hive(HKEY_CURRENT_USER, "HKCU\\Run"));
        entries.append(&mut Self::scan_hive(HKEY_LOCAL_MACHINE, "HKLM\\Run"));
        entries
    }

    fn scan_hive(root: HKEY, location_label: &str) -> Vec<StartupEntry> {
        let mut list = Vec::new();
        let path_hstr = HSTRING::from(RUN_KEY_PATH);
        let mut hkey = HKEY::default();

        unsafe {
            let status = RegOpenKeyExW(root, PCWSTR(path_hstr.as_ptr()), 0, KEY_READ, &mut hkey);
            if status != ERROR_SUCCESS {
                return list;
            }

            let mut index = 0u32;
            let mut name_buf = [0u16; 260];
            let mut data_buf = [0u8; 1024];

            loop {
                let mut name_len = name_buf.len() as u32;
                let mut val_type = 0u32;
                let mut data_len = data_buf.len() as u32;

                let status = RegEnumValueW(
                    hkey,
                    index,
                    PWSTR(name_buf.as_mut_ptr()),
                    &mut name_len,
                    None,
                    Some(&mut val_type),
                    Some(data_buf.as_mut_ptr()),
                    Some(&mut data_len),
                );

                if status == ERROR_NO_MORE_ITEMS {
                    break;
                } else if status == ERROR_SUCCESS && val_type == REG_SZ.0 {
                    let name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
                    let data_u16_slice = std::slice::from_raw_parts(
                        data_buf.as_ptr() as *const u16,
                        (data_len as usize / 2).saturating_sub(1),
                    );
                    let command = String::from_utf16_lossy(data_u16_slice);

                    list.push(StartupEntry {
                        id: format!("{}_{}", location_label, name),
                        name,
                        command,
                        location: location_label.to_string(),
                        enabled: true,
                        publisher: None,
                    });

                    index += 1;
                } else {
                    index += 1;
                }
            }

            let _ = RegCloseKey(hkey);
        }

        list
    }

    pub fn disable_startup_entry(entry_id: &str) -> Result<(), String> {
        let parts: Vec<&str> = entry_id.splitn(2, '_').collect();
        if parts.len() != 2 {
            return Err("ID de entrada no válido".into());
        }

        let (loc, name) = (parts[0], parts[1]);
        let root = if loc.starts_with("HKCU") {
            HKEY_CURRENT_USER
        } else {
            HKEY_LOCAL_MACHINE
        };

        let path_hstr = HSTRING::from(RUN_KEY_PATH);
        let name_hstr = HSTRING::from(name);
        let mut hkey = HKEY::default();

        unsafe {
            let status = RegOpenKeyExW(root, PCWSTR(path_hstr.as_ptr()), 0, KEY_WRITE, &mut hkey);
            if status != ERROR_SUCCESS {
                return Err("No se pudo abrir la clave Run con permisos de escritura".into());
            }

            let del_status = RegDeleteValueW(hkey, PCWSTR(name_hstr.as_ptr()));
            let _ = RegCloseKey(hkey);

            if del_status == ERROR_SUCCESS {
                Ok(())
            } else {
                Err(format!("Error eliminando valor: {:?}", del_status))
            }
        }
    }
}
