use crate::models::app::{AppInstallSource, InstalledApp};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub enum UninstallResult {
    Success { message: String },
    NeedsForceCleanup { reason: String },
    Error { message: String },
}

pub struct UninstallerRunner;

impl UninstallerRunner {
    /// Ejecuta el desinstalador oficial de manera inteligente según el tipo de aplicación
    pub fn execute(app: &InstalledApp) -> UninstallResult {
        // 1. Caso UWP / Windows Store
        if app.source == AppInstallSource::WindowsStoreUwp {
            return Self::execute_uwp_uninstall(app);
        }

        // 2. Caso Gestor Winget
        if app.source == AppInstallSource::Winget {
            return Self::execute_winget_uninstall(app);
        }

        // 3. Caso Windows Installer (MSI)
        if let Some(guid) = &app.windows_installer_guid {
            return Self::execute_msi_uninstall(guid);
        }

        let raw_cmd = app
            .quiet_uninstall_string
            .as_ref()
            .or(app.uninstall_string.as_ref());

        let cmd_str = match raw_cmd {
            Some(s) if !s.trim().is_empty() => s.trim(),
            _ => {
                return UninstallResult::NeedsForceCleanup {
                    reason: "No existe comando oficial de desinstalación registrado. Se procederá a eliminación forzada.".into(),
                };
            }
        };

        // Si el string de desinstalación contiene MsiExec
        if cmd_str.to_lowercase().contains("msiexec") {
            if let Some(guid) = Self::extract_guid_from_string(cmd_str) {
                return Self::execute_msi_uninstall(&guid);
            }
        }

        // 4. Caso Win32 estándar (Inno Setup, NSIS, InstallShield, etc.)
        Self::execute_win32_uninstaller(cmd_str, app)
    }

    /// Desinstalación de paquetes UWP con PowerShell
    fn execute_uwp_uninstall(app: &InstalledApp) -> UninstallResult {
        // El id es estilo uwp_Microsoft.SkypeApp_...
        let pkg_name = app.id.strip_prefix("uwp_").unwrap_or(&app.id);

        let ps_cmd = format!(
            "Get-AppxPackage -Name '*{0}*' | Remove-AppxPackage -ErrorAction SilentlyContinue",
            pkg_name
        );

        match Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &ps_cmd])
            .status()
        {
            Ok(status) if status.success() => UninstallResult::Success {
                message: "Paquete UWP desinstalado correctamente vía AppxPackageManager".into(),
            },
            Ok(_) => UninstallResult::NeedsForceCleanup {
                reason: "PowerShell finalizó con código no cero, verificando limpieza residual...".into(),
            },
            Err(e) => UninstallResult::Error {
                message: format!("Error al invocar PowerShell: {}", e),
            },
        }
    }

    /// Desinstalación de paquetes Winget
    fn execute_winget_uninstall(app: &InstalledApp) -> UninstallResult {
        let winget_id = app.id.strip_prefix("winget_").unwrap_or(&app.id);
        match Command::new("winget")
            .args(["uninstall", "--id", winget_id, "--silent", "--accept-source-agreements"])
            .status()
        {
            Ok(status) if status.success() => UninstallResult::Success {
                message: "Paquete desinstalado mediante Winget CLI".into(),
            },
            _ => UninstallResult::NeedsForceCleanup {
                reason: "Winget falló o paquete requiere desinstalación manual".into(),
            },
        }
    }

    /// Desinstalación oficial de MSI mediante msiexec.exe /X {GUID}
    fn execute_msi_uninstall(guid: &str) -> UninstallResult {
        // En Windows, MsiExec /I es instalar; para desinstalar SIEMPRE debe ser /X
        let clean_guid = if !guid.starts_with('{') {
            format!("{{{}}}", guid)
        } else {
            guid.to_string()
        };

        // Ejecutar msiexec /x {GUID} /qb /norestart para mostrar barra nativa sin reiniciar
        match Command::new("msiexec.exe")
            .args(["/x", &clean_guid, "/qb", "/norestart"])
            .status()
        {
            Ok(status) => {
                let code = status.code().unwrap_or(-1);
                // 0 = éxito, 1605 = ya desinstalado, 3010 = reinicio pendiente
                if code == 0 || code == 1605 || code == 3010 {
                    UninstallResult::Success {
                        message: format!("MSI desinstalado con código {}", code),
                    }
                } else {
                    UninstallResult::NeedsForceCleanup {
                        reason: format!("MsiExec finalizó con código de salida: {}", code),
                    }
                }
            }
            Err(e) => UninstallResult::Error {
                message: format!("No se pudo ejecutar MsiExec: {}", e),
            },
        }
    }

    /// Ejecuta desinstaladores ejecutables (.exe) de Win32
    fn execute_win32_uninstaller(cmd_str: &str, app: &InstalledApp) -> UninstallResult {
        let (exe_path, mut args) = Self::parse_command_line(cmd_str);

        let path = Path::new(&exe_path);
        if !path.exists() {
            return UninstallResult::NeedsForceCleanup {
                reason: format!("El archivo ejecutable del desinstalador no existe: {}", exe_path),
            };
        }

        // Optimización para desinstaladores NSIS: si no tiene _?= agregamos el flag para que sea sincrónico
        let exe_lower = exe_path.to_lowercase();
        if (exe_lower.contains("uninst") || exe_lower.contains("setup")) && !args.iter().any(|a| a.starts_with("_?=")) {
            if let Some(loc) = &app.install_location {
                args.push(format!("_?={}", loc.to_string_lossy()));
            }
        }

        let mut cmd = Command::new(&exe_path);
        cmd.args(&args);

        // Si la app tiene directorio de instalación, establecerlo como directorio de trabajo (cwd)
        if let Some(loc) = &app.install_location {
            if loc.exists() {
                cmd.current_dir(loc);
            }
        }

        match cmd.status() {
            Ok(status) => {
                // Dar un breve tiempo para que cualquier proceso hijo temporal del desinstalador termine de escribir
                std::thread::sleep(std::time::Duration::from_millis(500));

                if status.success() {
                    UninstallResult::Success {
                        message: "Desinstalador oficial finalizado con éxito".into(),
                    }
                } else {
                    UninstallResult::NeedsForceCleanup {
                        reason: format!("Desinstalador oficial finalizó con código {:?}", status.code()),
                    }
                }
            }
            Err(e) => {
                // Si falla la invocación directa, intentar ejecutar a través del shell cmd con comillas seguras
                let fallback = Command::new("cmd")
                    .args(["/s", "/c", &format!("\"{}\"", cmd_str)])
                    .status();

                match fallback {
                    Ok(s) if s.success() => UninstallResult::Success {
                        message: "Desinstalador ejecutado vía shell".into(),
                    },
                    _ => UninstallResult::NeedsForceCleanup {
                        reason: format!("Error al ejecutar comando ({:?}). Procediendo a erradicación forzada.", e),
                    },
                }
            }
        }
    }

    /// Extrae un GUID {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX} de un string
    fn extract_guid_from_string(s: &str) -> Option<String> {
        let open_idx = s.find('{')?;
        let close_idx = s.find('}')?;
        if close_idx > open_idx && (close_idx - open_idx) == 37 {
            Some(s[open_idx..=close_idx].to_string())
        } else {
            None
        }
    }

    /// Separa una línea de comando de Windows en (Ruta Ejecutable, Argumentos) respetando comillas
    pub fn parse_command_line(cmd: &str) -> (String, Vec<String>) {
        let trimmed = cmd.trim();
        if trimmed.starts_with('"') {
            // Caso: "C:\Program Files\App\unins.exe" /arg1 /arg2
            if let Some(end_quote) = trimmed[1..].find('"') {
                let exe = &trimmed[1..=end_quote];
                let remainder = trimmed[end_quote + 2..].trim();
                let args = if remainder.is_empty() {
                    Vec::new()
                } else {
                    remainder.split_whitespace().map(|s| s.to_string()).collect()
                };
                return (exe.to_string(), args);
            }
        }

        // Si no empieza con comillas, buscar si termina en .exe
        let lower = trimmed.to_lowercase();
        if let Some(exe_idx) = lower.find(".exe") {
            let possible_exe = &trimmed[..exe_idx + 4];
            let remainder = trimmed[exe_idx + 4..].trim();
            let args = if remainder.is_empty() {
                Vec::new()
            } else {
                remainder.split_whitespace().map(|s| s.to_string()).collect()
            };
            return (possible_exe.trim_matches('"').to_string(), args);
        }

        // Fallback: primer espacio
        let mut parts = trimmed.split_whitespace();
        let exe = parts.next().unwrap_or_default().trim_matches('"').to_string();
        let args = parts.map(|s| s.to_string()).collect();
        (exe, args)
    }
}
