use base64::Engine;
use image::{ImageBuffer, RgbaImage};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS,
};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::UI::Shell::{
    SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, GetIconInfo, PrivateExtractIconsW, HICON, ICONINFO,
};

static ICON_CACHE: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

pub struct IconExtractor;

impl IconExtractor {
    /// Obtiene el icono en Base64 (PNG) de la aplicación, usando caché en memoria
    pub fn get_app_icon_base64(display_icon: Option<&str>, install_location: Option<&Path>) -> Option<String> {
        let raw_target = Self::resolve_target_path(display_icon, install_location)?;

        // Consultar caché primero
        if let Ok(mut guard) = ICON_CACHE.lock() {
            let cache = guard.get_or_insert_with(HashMap::new);
            if let Some(cached) = cache.get(&raw_target) {
                return Some(cached.clone());
            }
        }

        // Extraer icono nativo
        let b64 = Self::extract_icon_as_png_base64(&raw_target)?;

        // Guardar en caché
        if let Ok(mut guard) = ICON_CACHE.lock() {
            if let Some(cache) = guard.as_mut() {
                cache.insert(raw_target, b64.clone());
            }
        }

        Some(b64)
    }

    /// Resuelve la ruta del archivo y opcionalmente el índice de recurso (ej. "app.exe,0")
    fn resolve_target_path(display_icon: Option<&str>, install_location: Option<&Path>) -> Option<String> {
        if let Some(icon_str) = display_icon {
            let clean = icon_str.trim().trim_matches('"');
            let parts: Vec<&str> = clean.split(',').collect();
            let file_part = parts[0].trim();
            if Path::new(file_part).exists() {
                return Some(clean.to_string());
            }
        }

        // Buscar ejecutable principal en la carpeta de instalación
        if let Some(dir) = install_location {
            if dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file() {
                            if let Some(ext) = p.extension() {
                                if ext.eq_ignore_ascii_case("exe") {
                                    return Some(p.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Extrae el icono nativo mediante PrivateExtractIconsW o SHGetFileInfoW y lo convierte a PNG Base64
    fn extract_icon_as_png_base64(target_str: &str) -> Option<String> {
        let parts: Vec<&str> = target_str.split(',').collect();
        let file_path = parts[0].trim().trim_matches('"');
        let icon_index: i32 = if parts.len() > 1 {
            parts[1].trim().parse().unwrap_or(0)
        } else {
            0
        };

        if !Path::new(file_path).exists() {
            return None;
        }

        let mut hicon: HICON = HICON::default();

        // 1. Intentar con PrivateExtractIconsW
        let mut filename_buf = [0u16; 260];
        let path_u16: Vec<u16> = file_path.encode_utf16().collect();
        let copy_len = path_u16.len().min(259);
        filename_buf[..copy_len].copy_from_slice(&path_u16[..copy_len]);

        let mut icon_id = 0u32;
        let mut hicon_arr = [HICON::default()];

        unsafe {
            let count = PrivateExtractIconsW(
                &filename_buf,
                icon_index,
                32,
                32,
                Some(&mut hicon_arr),
                Some(&mut icon_id as *mut u32),
                0,
            );

            if count > 0 && !hicon_arr[0].is_invalid() {
                hicon = hicon_arr[0];
            }

            // 2. Si falló, recurrir a SHGetFileInfoW (altamente fiable con el shell de Windows)
            if hicon.is_invalid() {
                let path_hstr = HSTRING::from(file_path);
                let mut sfi = SHFILEINFOW::default();
                let res = SHGetFileInfoW(
                    PCWSTR(path_hstr.as_ptr()),
                    FILE_FLAGS_AND_ATTRIBUTES(0),
                    Some(&mut sfi),
                    std::mem::size_of::<SHFILEINFOW>() as u32,
                    SHGFI_ICON | SHGFI_LARGEICON,
                );

                if res != 0 && !sfi.hIcon.is_invalid() {
                    hicon = sfi.hIcon;
                }
            }
        }

        if hicon.is_invalid() {
            return None;
        }

        // Convertir HICON a RGBA y luego a PNG Base64
        let base64_png = Self::hicon_to_base64_png(hicon);

        unsafe {
            let _ = DestroyIcon(hicon);
        }

        base64_png
    }

    /// Convierte un HICON Win32 a un buffer PNG codificado en Base64
    fn hicon_to_base64_png(hicon: HICON) -> Option<String> {
        unsafe {
            let mut icon_info = ICONINFO::default();
            if GetIconInfo(hicon, &mut icon_info).is_err() {
                return None;
            }

            let hdc = CreateCompatibleDC(None);
            let width = 32i32;
            let height = 32i32;

            let mut bi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // Orientación top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let pixel_count = (width * height * 4) as usize;
            let mut raw_pixels = vec![0u8; pixel_count];

            let lines = GetDIBits(
                hdc,
                icon_info.hbmColor,
                0,
                height as u32,
                Some(raw_pixels.as_mut_ptr() as *mut _),
                &mut bi,
                DIB_RGB_COLORS,
            );

            // Liberar objetos GDI
            let _ = DeleteObject(icon_info.hbmColor);
            let _ = DeleteObject(icon_info.hbmMask);
            let _ = DeleteDC(hdc);

            if lines == 0 {
                return None;
            }

            // En Windows DIB los bytes vienen como BGRA; convertirlos a RGBA
            let mut has_alpha = false;
            for chunk in raw_pixels.chunks_exact_mut(4) {
                chunk.swap(0, 2); // B <-> R
                if chunk[3] > 0 {
                    has_alpha = true;
                }
            }

            // Si todos los bytes de alfa son 0, fijar alfa en 255
            if !has_alpha {
                for chunk in raw_pixels.chunks_exact_mut(4) {
                    chunk[3] = 255;
                }
            }

            // Codificar imagen a PNG
            let img: RgbaImage = ImageBuffer::from_raw(width as u32, height as u32, raw_pixels)?;
            let mut png_bytes = Vec::new();
            img.write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .ok()?;

            Some(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
        }
    }
}
