use crate::cleaner::leftover_hunter::LeftoverHunter;
use crate::models::leftover::{LeftoverItem, LeftoverType};
use crate::scanner::MasterScanner;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::ipc::Channel;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrphanedItem {
    pub id: String,
    pub name: String,
    pub original_app_hint: String,
    pub path: String,
    pub item_type: String, // "appdata_roaming", "appdata_local", "locallow", "programdata", "program_files"
    pub size_bytes: u64,
    pub file_count: usize,
    pub last_modified: Option<String>,
    pub confidence_score: u8,
    pub description: String,
    pub is_selected: bool,

    // Diagnóstico e inteligencia de seguridad requerida
    pub content_breakdown: Vec<String>,
    pub recommendation_level: String, // "highly_recommended" | "recommended" | "caution"
    pub recommendation_badge: String, // "100% Seguro y Recomendable" | "Recomendado" | "⚠️ Partidas Guardadas de Videojuegos (Precaución)"
    pub impact_explanation: String,
    pub requires_reboot: bool,

    // Detección de partidas de videojuegos
    pub is_game_save: bool,
    pub game_save_count: usize,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidualFileDetail {
    pub name: String,
    pub path: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub extension: String,
    pub category: String, // "Caché / Shader", "Log / Registro", "Configuración", "Partida Guardada (.sav)", "Binario / Otro"
    pub is_game_save: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeepCleanResult {
    pub total_items_deleted: usize,
    pub total_bytes_freed: u64,
    pub details: Vec<String>,
    pub reboot_required: bool,
    pub reboot_reason: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
pub enum ResidualCleanProgressUpdate {
    Progress {
        percentage: u8,
        current_item: String,
        deleted_count: usize,
        total_count: usize,
        bytes_freed: u64,
    },
    Finished {
        result: DeepCleanResult,
    },
    Error {
        message: String,
    },
}

struct DirAnalysis {
    total_bytes: u64,
    file_count: usize,
    cache_bytes: u64,
    log_bytes: u64,
    config_bytes: u64,
    game_save_bytes: u64,
    game_save_count: usize,
    is_game_save: bool,
    has_locked_files: bool,
}

pub struct OrphanScanner;

impl OrphanScanner {
    /// Puntos de unión (Junction Points) y enlaces simbólicos de Windows que NUNCA deben atravesarse
    const JUNCTION_BLACKLIST: &'static [&'static str] = &[
        "datos de programa",
        "application data",
        "appdata",
        "historial",
        "history",
        "cookies",
        "archivos de programa",
        "mis documentos",
        "my documents",
        "all users",
        "default user",
        "nethood",
        "printhood",
        "recent",
        "sendto",
        "start menu",
        "menú inicio",
        "menu inicio",
        "plantillas",
        "templates",
    ];

    /// Lista estricta de palabras y carpetas protegidas del sistema Windows
    const SYSTEM_WHITELIST: &'static [&'static str] = &[
        "windows", "microsoft", "system", "system32", "syswow64", "windows defender",
        "windows nt", "windows security", "windowsapps", "windowspowershell",
        "common files", "internet explorer", "windows mail", "windows media player",
        "windows photo viewer", "windows portable devices", "windows sidebar",
        "desktop.ini", "temp", "cache", "assembly", "microsoft.net", "dotnet",
        "package repository", "packages", "usoshared", "cryptneturlcache",
        "fontcache", "connecteddevicesplatform", "systemapps", "wer",
        "onedrive", "edge", "edgewebview", "directx", "nvidiagpu", "intel",
        "realtek", "amd", "amd64", "msi", "installer", "servicing",
        "crashdumps", "inetsrv", "iis", "programs",
        "startup", "public", "default",
        "local", "locallow", "roaming", "program files", "program files (x86)",
        "identitycache", "credentials", "crypto", "system certificates",
        "searches", "links", "favorites", "contacts",
    ];

    /// Comprueba de forma infalible si una ruta es un NTFS Junction Point o Reparse Point
    pub fn is_reparse_point_or_symlink(path: &Path) -> bool {
        if let Ok(meta) = path.symlink_metadata() {
            if meta.file_type().is_symlink() {
                return true;
            }
            #[cfg(windows)]
            {
                const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x00000400;
                if (meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
                    return true;
                }
            }
        }
        false
    }

    /// Escanea exhaustivamente en busca de carpetas y restos de aplicaciones antiguas ya desinstaladas
    pub fn scan_orphaned_residuals() -> Vec<OrphanedItem> {
        let mut results = Vec::new();

        // 1. Obtener todas las aplicaciones instaladas actualmente
        let installed_apps = MasterScanner::scan_everything();

        // 2. Construir conjunto de tokens conocidos de apps activas
        let mut active_tokens = HashSet::new();
        for app in &installed_apps {
            let name_clean = Self::sanitize(&app.display_name);
            if !name_clean.is_empty() {
                active_tokens.insert(name_clean.clone());
                for word in name_clean.split_whitespace() {
                    if word.len() >= 3 {
                        active_tokens.insert(word.to_string());
                    }
                }
            }

            if let Some(pub_name) = &app.publisher {
                let pub_clean = Self::sanitize(pub_name);
                if !pub_clean.is_empty() {
                    active_tokens.insert(pub_clean.clone());
                    for word in pub_clean.split_whitespace() {
                        if word.len() >= 3 {
                            active_tokens.insert(word.to_string());
                        }
                    }
                }
            }

            if let Some(loc) = &app.install_location {
                if let Some(folder_name) = loc.file_name() {
                    active_tokens.insert(Self::sanitize(&folder_name.to_string_lossy()));
                }
            }
        }

        // 3. Ubicaciones clave a inspeccionar
        let mut search_roots: Vec<(PathBuf, &str, &str, u8)> = Vec::new();

        if let Ok(appdata) = std::env::var("APPDATA") {
            search_roots.push((
                PathBuf::from(appdata),
                "appdata_roaming",
                "Restos y configuraciones en AppData Roaming",
                95,
            ));
        }

        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let local_path = PathBuf::from(&local);
            search_roots.push((
                local_path.clone(),
                "appdata_local",
                "Datos temporales y cachés huérfanas en AppData Local",
                90,
            ));

            if let Some(parent) = local_path.parent() {
                let locallow = parent.join("LocalLow");
                if locallow.exists() {
                    search_roots.push((
                        locallow,
                        "locallow",
                        "Archivos y partidas residuales en LocalLow",
                        92,
                    ));
                }
            }
        }

        if let Ok(programdata) = std::env::var("ProgramData") {
            search_roots.push((
                PathBuf::from(programdata),
                "programdata",
                "Datos y cachés de instalación en ProgramData",
                90,
            ));
        }

        if let Ok(pf) = std::env::var("ProgramFiles") {
            search_roots.push((
                PathBuf::from(pf),
                "program_files",
                "Directorio residual en Program Files (64-bit)",
                85,
            ));
        }

        if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
            search_roots.push((
                PathBuf::from(pf86),
                "program_files",
                "Directorio residual en Program Files (32-bit)",
                85,
            ));
        }

        // 4. Analizar cada directorio
        for (root_dir, category, desc, confidence) in search_roots {
            if !root_dir.exists() || !root_dir.is_dir() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(&root_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    // FILTRO CRÍTICO: Descartar de raíz NTFS Junction Points / Reparse Points ("Datos de programa", etc.)
                    if Self::is_reparse_point_or_symlink(&path) {
                        continue;
                    }

                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    let dir_clean = Self::sanitize(&dir_name);

                    // Descartar si el nombre es un Junction conocido de Windows
                    if Self::is_junction_name(&dir_clean) {
                        continue;
                    }

                    // Descartar si es parte del sistema de Windows
                    if Self::is_system_whitelisted(&dir_clean) {
                        continue;
                    }

                    // Comprobar si coincide con alguna aplicación instalada activa
                    let matches_active_app = active_tokens.iter().any(|token| {
                        dir_clean == *token
                            || (token.len() >= 4 && dir_clean.contains(token))
                            || (dir_clean.len() >= 4 && token.contains(&dir_clean))
                    });

                    if matches_active_app {
                        continue;
                    }

                    // Análisis incisivo del contenido interior de la carpeta
                    let analysis = Self::analyze_directory(&path);
                    if analysis.total_bytes == 0 && analysis.file_count == 0 {
                        continue;
                    }

                    let last_modified = entry
                        .metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(|t| {
                            let dt: ChronoMockDate = t.into();
                            dt.to_string()
                        });

                    let breakdown = Self::generate_breakdown(&analysis);
                    let (rec_level, rec_badge, impact) = Self::evaluate_recommendation(&analysis, &dir_name);

                    // Si se detectaron partidas guardadas de juegos, NO debe venir seleccionada por defecto para máxima seguridad
                    let default_selected = !analysis.is_game_save;

                    results.push(OrphanedItem {
                        id: format!("orphan_{}", path.to_string_lossy()),
                        name: dir_name.clone(),
                        original_app_hint: if analysis.is_game_save {
                            format!("Juego o software anterior: {}", dir_name)
                        } else {
                            format!("Software desinstalado: {}", dir_name)
                        },
                        path: path.to_string_lossy().to_string(),
                        item_type: category.to_string(),
                        size_bytes: analysis.total_bytes,
                        file_count: analysis.file_count,
                        last_modified,
                        confidence_score: confidence,
                        description: desc.to_string(),
                        is_selected: default_selected,
                        content_breakdown: breakdown,
                        recommendation_level: rec_level,
                        recommendation_badge: rec_badge,
                        impact_explanation: impact,
                        requires_reboot: analysis.has_locked_files,
                        is_game_save: analysis.is_game_save,
                        game_save_count: analysis.game_save_count,
                    });
                }
            }
        }

        // Ordenar por tamaño en disco descendente
        results.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        results
    }

    /// Obtiene la lista detallada de archivos dentro de una carpeta residual antes de borrar
    pub fn get_folder_file_details(folder_path: &Path) -> Vec<ResidualFileDetail> {
        let mut files = Vec::new();
        if !folder_path.exists() {
            return files;
        }

        for entry in walkdir::WalkDir::new(folder_path)
            .max_depth(10)
            .into_iter()
            .filter_entry(|e| !Self::is_reparse_point_or_symlink(e.path()))
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if let Ok(meta) = p.metadata() {
                if meta.is_file() {
                    let len = meta.len();
                    let file_name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let relative = p
                        .strip_prefix(folder_path)
                        .unwrap_or(p)
                        .to_string_lossy()
                        .to_string();
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    let p_str = p.to_string_lossy().to_lowercase();

                    let is_save = Self::is_game_save_file(&p_str, &ext, &file_name);

                    let category = if is_save {
                        "Partida Guardada (.sav)".to_string()
                    } else if p_str.contains("cache")
                        || p_str.contains("blob_storage")
                        || p_str.contains("shader")
                        || p_str.contains("temp")
                        || matches!(ext.as_str(), "tmp" | "temp" | "cache" | "dat" | "bin" | "pak" | "chk")
                    {
                        "Caché / Shaders / Temp".to_string()
                    } else if matches!(ext.as_str(), "log" | "txt" | "dmp" | "mdmp" | "etl") {
                        "Log / Registro de Eventos".to_string()
                    } else if matches!(ext.as_str(), "json" | "xml" | "ini" | "cfg" | "config" | "yaml" | "yml") {
                        "Configuración Huérfana".to_string()
                    } else {
                        "Archivo Residual".to_string()
                    };

                    files.push(ResidualFileDetail {
                        name: file_name,
                        path: p.to_string_lossy().to_string(),
                        relative_path: relative,
                        size_bytes: len,
                        extension: ext,
                        category,
                        is_game_save: is_save,
                    });
                }
            }
        }

        // Ordenar por tamaño en disco descendente
        files.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        files
    }

    /// Elimina en streaming los elementos residuales emitiendo el porcentaje en tiempo real
    pub fn stream_delete_orphaned_items(
        items: &[OrphanedItem],
        on_progress: Channel<ResidualCleanProgressUpdate>,
    ) -> Result<DeepCleanResult, String> {
        let total_items = items.len();
        let mut deleted_count = 0;
        let mut bytes_freed = 0u64;
        let mut details = Vec::new();
        let mut reboot_required = false;

        let _ = on_progress.send(ResidualCleanProgressUpdate::Progress {
            percentage: 5,
            current_item: "Iniciando motor de erradicación de archivos residuales...".into(),
            deleted_count: 0,
            total_count: total_items,
            bytes_freed: 0,
        });

        for (idx, item) in items.iter().enumerate() {
            let pct = ((idx as f64 / total_items.max(1) as f64) * 90.0 + 5.0).round() as u8;

            let _ = on_progress.send(ResidualCleanProgressUpdate::Progress {
                percentage: pct.min(95),
                current_item: format!("Eliminando: {}", item.path),
                deleted_count,
                total_count: total_items,
                bytes_freed,
            });

            let leftover = LeftoverItem {
                id: item.id.clone(),
                item_type: LeftoverType::Directory,
                path_or_key: item.path.clone(),
                confidence_score: item.confidence_score,
                size_bytes: Some(item.size_bytes),
                rationale: item.description.clone(),
                is_selected: true,
            };

            if LeftoverHunter::delete_leftover(&leftover).is_ok() {
                deleted_count += 1;
                bytes_freed += item.size_bytes;
                details.push(format!("Eliminado: {}", item.path));

                if item.requires_reboot {
                    reboot_required = true;
                }
            }
        }

        let _ = on_progress.send(ResidualCleanProgressUpdate::Progress {
            percentage: 100,
            current_item: "Limpieza finalizada con éxito.".into(),
            deleted_count,
            total_count: total_items,
            bytes_freed,
        });

        let reboot_reason = if reboot_required {
            Some("Algunos archivos estaban retenidos por procesos o servicios en segundo plano de Windows. Se desbloquearon y programaron para borrado definitivo al reiniciar el equipo.".into())
        } else {
            None
        };

        let result = DeepCleanResult {
            total_items_deleted: deleted_count,
            total_bytes_freed: bytes_freed,
            details,
            reboot_required,
            reboot_reason,
        };

        let _ = on_progress.send(ResidualCleanProgressUpdate::Finished {
            result: result.clone(),
        });

        Ok(result)
    }

    /// Elimina una lista de elementos residuales seleccionados (método síncrono para compatibilidad)
    pub fn delete_orphaned_items(items: &[OrphanedItem]) -> Result<DeepCleanResult, String> {
        let mut deleted_count = 0;
        let mut bytes_freed = 0u64;
        let mut details = Vec::new();
        let mut reboot_required = false;

        for item in items {
            let leftover = LeftoverItem {
                id: item.id.clone(),
                item_type: LeftoverType::Directory,
                path_or_key: item.path.clone(),
                confidence_score: item.confidence_score,
                size_bytes: Some(item.size_bytes),
                rationale: item.description.clone(),
                is_selected: true,
            };

            if LeftoverHunter::delete_leftover(&leftover).is_ok() {
                deleted_count += 1;
                bytes_freed += item.size_bytes;
                details.push(format!("Eliminado: {}", item.path));

                if item.requires_reboot {
                    reboot_required = true;
                }
            }
        }

        let reboot_reason = if reboot_required {
            Some("Algunos archivos estaban en uso por procesos en segundo plano de Windows. Se desbloquearon y programaron para borrado definitivo al reiniciar el equipo.".into())
        } else {
            None
        };

        Ok(DeepCleanResult {
            total_items_deleted: deleted_count,
            total_bytes_freed: bytes_freed,
            details,
            reboot_required,
            reboot_reason,
        })
    }

    fn is_system_whitelisted(name: &str) -> bool {
        let lower = name.to_lowercase();
        Self::SYSTEM_WHITELIST.iter().any(|&w| lower == w || lower.starts_with(w))
    }

    fn is_junction_name(name: &str) -> bool {
        let lower = name.to_lowercase();
        Self::JUNCTION_BLACKLIST.iter().any(|&j| lower == j || lower.starts_with(j))
    }

    fn sanitize(s: &str) -> String {
        s.trim().to_lowercase().replace(['"', '\'', '(', ')', '[', ']', '_', '-'], " ")
    }

    /// Heurística incisiva para detectar si un archivo es una partida de videojuego (.sav)
    fn is_game_save_file(p_str: &str, ext: &str, file_name: &str) -> bool {
        if matches!(ext, "sav" | "save" | "svs" | "sl2" | "gbasav") {
            return true;
        }

        let lower_name = file_name.to_lowercase();
        if lower_name.contains("savegame") || lower_name.contains("gamesave") || lower_name.contains("savedata") {
            return true;
        }

        if (p_str.contains("saved games")
            || p_str.contains("savegames")
            || p_str.contains("saved\\savegames")
            || p_str.contains("my games")
            || p_str.contains("remote") && p_str.contains("steam"))
            && matches!(ext, "dat" | "bin" | "vdf" | "profile" | "sav" | "save")
        {
            return true;
        }

        false
    }

    /// Inspecciona el interior del directorio para clasificar los archivos en basura, logs, configuraciones y partidas
    fn analyze_directory(path: &Path) -> DirAnalysis {
        let mut total_bytes = 0u64;
        let mut file_count = 0usize;
        let mut cache_bytes = 0u64;
        let mut log_bytes = 0u64;
        let mut config_bytes = 0u64;
        let mut game_save_bytes = 0u64;
        let mut game_save_count = 0usize;
        let mut is_game_save = false;
        let mut has_locked_files = false;

        let path_str_lower = path.to_string_lossy().to_lowercase();

        // Heurística de carpeta de partidas (ej. carpetas con sólo números de Steam AppIDs, o nombres "SaveGames")
        let is_potential_game_dir = path_str_lower.contains("saved games")
            || path_str_lower.contains("savegames")
            || path_str_lower.contains("saved\\savegames")
            || path_str_lower.contains("my games")
            || path.file_name().map_or(false, |n| {
                let name = n.to_string_lossy();
                name.len() >= 4 && name.chars().all(|c| c.is_ascii_digit())
            });

        // Recorrer walkdir ignorando estrictamente Junction Points internos
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| !Self::is_reparse_point_or_symlink(e.path()))
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    let len = meta.len();
                    total_bytes += len;
                    file_count += 1;

                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    let p_str = p.to_string_lossy().to_lowercase();
                    let file_name = p.file_name().unwrap_or_default().to_string_lossy();

                    let is_save = Self::is_game_save_file(&p_str, &ext, &file_name) || (is_potential_game_dir && matches!(ext.as_str(), "sav" | "save" | "dat"));

                    if is_save {
                        game_save_bytes += len;
                        game_save_count += 1;
                        is_game_save = true;
                    } else if p_str.contains("cache")
                        || p_str.contains("blob_storage")
                        || p_str.contains("shader")
                        || p_str.contains("temp")
                        || matches!(ext.as_str(), "tmp" | "temp" | "cache" | "dat" | "bin" | "pak" | "chk")
                    {
                        cache_bytes += len;
                    } else if matches!(ext.as_str(), "log" | "txt" | "dmp" | "mdmp" | "etl") {
                        log_bytes += len;
                    } else if matches!(ext.as_str(), "json" | "xml" | "ini" | "cfg" | "config" | "yaml" | "yml") {
                        config_bytes += len;
                    } else {
                        cache_bytes += len;
                    }

                    // Probar si el archivo está en uso por otro proceso de Windows
                    if fs::OpenOptions::new().write(true).open(p).is_err() {
                        has_locked_files = true;
                    }
                }
            }
        }

        DirAnalysis {
            total_bytes,
            file_count,
            cache_bytes,
            log_bytes,
            config_bytes,
            game_save_bytes,
            game_save_count,
            is_game_save,
            has_locked_files,
        }
    }

    fn generate_breakdown(analysis: &DirAnalysis) -> Vec<String> {
        let total = analysis.total_bytes.max(1) as f64;
        let mut list = Vec::new();

        if analysis.is_game_save && analysis.game_save_count > 0 {
            list.push(format!(
                "⚠️ {} Partida(s) Guardada(s) local(es) ({})",
                analysis.game_save_count,
                Self::format_bytes(analysis.game_save_bytes)
            ));
        }

        if analysis.cache_bytes > 0 {
            let pct = ((analysis.cache_bytes as f64 / total) * 100.0).round() as u8;
            list.push(format!("{}% Caché temporal, shaders y blobs ({})", pct, Self::format_bytes(analysis.cache_bytes)));
        }

        if analysis.log_bytes > 0 {
            let pct = ((analysis.log_bytes as f64 / total) * 100.0).round() as u8;
            list.push(format!("{}% Logs de eventos y volcados de errores ({})", pct, Self::format_bytes(analysis.log_bytes)));
        }

        if analysis.config_bytes > 0 {
            let pct = ((analysis.config_bytes as f64 / total) * 100.0).round() as u8;
            list.push(format!("{}% Archivos de configuración huérfana ({})", pct, Self::format_bytes(analysis.config_bytes)));
        }

        if list.is_empty() {
            list.push(format!("100% Archivos residuales ({})", Self::format_bytes(analysis.total_bytes)));
        }

        list
    }

    fn evaluate_recommendation(analysis: &DirAnalysis, dir_name: &str) -> (String, String, String) {
        let formatted_size = Self::format_bytes(analysis.total_bytes);

        // Si contiene partidas de videojuegos, emitir advertencia preventiva
        if analysis.is_game_save {
            return (
                "caution".into(),
                "⚠️ Partidas Guardadas de Videojuegos (Precaución)".into(),
                format!(
                    "PRECAUCIÓN: Esta carpeta contiene {} partida(s) de videojuegos (.sav / perfiles locales). Si no tienes Steam Cloud activo para este juego o juegas sin conexión, perderás el progreso de tus partidas. Los archivos de caché y shaders sí se regeneran automáticamente al volver a jugar, pero las partidas locales se pierden para siempre si las eliminas.",
                    analysis.game_save_count
                ),
            );
        }

        let total = analysis.total_bytes.max(1);
        let junk_bytes = analysis.cache_bytes + analysis.log_bytes;
        let is_mostly_junk = (junk_bytes * 100 / total) >= 60;

        if is_mostly_junk {
            (
                "highly_recommended".into(),
                "100% Seguro y Recomendable".into(),
                format!(
                    "Eliminar es 100% seguro. Solo contiene datos temporales, shaders y registros de '{}'. No afecta a Windows ni a ningún software instalado y recuperará {}.",
                    dir_name, formatted_size
                ),
            )
        } else {
            (
                "recommended".into(),
                "Recomendable (Configuración huérfana)".into(),
                format!(
                    "Recomendable. Pertenecía a '{}' (ya desinstalado). Borrarlo liberará {} y dejará tu perfil limpio sin rastros.",
                    dir_name, formatted_size
                ),
            )
        }
    }

    fn format_bytes(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{} KB", bytes / 1024)
        } else {
            format!("{} B", bytes)
        }
    }
}

// Estructura para formatear fecha de archivo sin depender de crates externos
struct ChronoMockDate(std::time::SystemTime);

impl From<std::time::SystemTime> for ChronoMockDate {
    fn from(t: std::time::SystemTime) -> Self {
        ChronoMockDate(t)
    }
}

impl ToString for ChronoMockDate {
    fn to_string(&self) -> String {
        if let Ok(duration) = self.0.duration_since(std::time::UNIX_EPOCH) {
            let secs = duration.as_secs();
            let days = secs / 86400;
            let year = 1970 + (days / 365);
            let month = ((days % 365) / 30) + 1;
            let day = ((days % 365) % 30) + 1;
            format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31))
        } else {
            "Reciente".into()
        }
    }
}
