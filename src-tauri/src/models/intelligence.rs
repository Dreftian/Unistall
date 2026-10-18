pub struct AppIntelligence;

pub struct AppMetadataInfo {
    pub description: String,
    pub category: String,
    pub is_critical: bool,
    pub risk_warning: Option<String>,
}

impl AppIntelligence {
    /// Evalúa el propósito de la app, su categoría y si es un componente crítico del sistema
    pub fn evaluate(name: &str, publisher: Option<&str>) -> AppMetadataInfo {
        let name_lower = name.to_lowercase();
        let pub_lower = publisher.unwrap_or("").to_lowercase();

        // 1. COMPONENTES CRÍTICOS DEL SISTEMA (Runtimes, Drivers, Core Windows)
        if name_lower.contains("visual c++") || name_lower.contains("vcredist") {
            return AppMetadataInfo {
                description: "Bibliotecas de tiempo de ejecución de C++ esenciales para miles de aplicaciones de Windows.".into(),
                category: "Librerías de Sistema".into(),
                is_critical: true,
                risk_warning: Some("PELIGRO: Desinstalar este componente provocará que decenas de juegos y aplicaciones dejen de abrir arrojando errores como 'MSVCP140.dll / VCRUNTIME140.dll no encontrado'.".into()),
            };
        }

        if name_lower.contains(".net") || name_lower.contains("dotnet") {
            return AppMetadataInfo {
                description: "Plataforma de ejecución de Microsoft .NET para aplicaciones y servicios administrados.".into(),
                category: "Framework del Sistema".into(),
                is_critical: true,
                risk_warning: Some("ADVERTENCIA: Muchos programas empresariales, herramientas del sistema y utilidades dependen de .NET y dejarán de funcionar si se elimina.".into()),
            };
        }

        if name_lower.contains("directx") {
            return AppMetadataInfo {
                description: "Colección de APIs multimedia de Microsoft para aceleración gráfica, sonido y videojuegos.".into(),
                category: "Componente Gráfico".into(),
                is_critical: true,
                risk_warning: Some("PELIGRO: Los videojuegos y el software de renderizado 3D perderán aceleración o no podrán iniciar.".into()),
            };
        }

        if name_lower.contains("webview2") || name_lower.contains("edge webview") {
            return AppMetadataInfo {
                description: "Motor embebido de Microsoft Edge utilizado por aplicaciones modernas para renderizar interfaces web nativas.".into(),
                category: "Componente de Sistema".into(),
                is_critical: true,
                risk_warning: Some("ADVERTENCIA: Aplicaciones modernas como Teams, Office y diversas herramientas de terceros dependen de WebView2 para mostrar su interfaz gráfica.".into()),
            };
        }

        if name_lower.contains("nvidia") || name_lower.contains("geforce") || name_lower.contains("radeon") || name_lower.contains("amd software") || name_lower.contains("intel graphics") {
            return AppMetadataInfo {
                description: "Controladores gráficos y panel de administración de la tarjeta de video.".into(),
                category: "Controlador de Hardware".into(),
                is_critical: true,
                risk_warning: Some("ADVERTENCIA: Podrías perder la resolución nativa de tu pantalla, la tasa de refresco alta (Hz) y el rendimiento en juegos.".into()),
            };
        }

        if name_lower.contains("realtek") {
            return AppMetadataInfo {
                description: "Controlador de audio o red para hardware integrado en la placa base.".into(),
                category: "Controlador de Hardware".into(),
                is_critical: true,
                risk_warning: Some("ADVERTENCIA: Podrías quedarte sin sonido en tus altavoces/auriculares o perder conectividad de red.".into()),
            };
        }

        // 2. NAVEGADORES WEB
        if name_lower.contains("chrome") {
            return AppMetadataInfo {
                description: "Navegador web de Google rápido, seguro y con sincronización en la nube.".into(),
                category: "Navegador Web".into(),
                is_critical: false,
                risk_warning: None,
            };
        }
        if name_lower.contains("firefox") {
            return AppMetadataInfo {
                description: "Navegador web libre y de código abierto enfocado en la privacidad desarrollado por Mozilla.".into(),
                category: "Navegador Web".into(),
                is_critical: false,
                risk_warning: None,
            };
        }
        if name_lower.contains("brave") {
            return AppMetadataInfo {
                description: "Navegador web con bloqueo integrado de anuncios y rastreadores enfocado en privacidad.".into(),
                category: "Navegador Web".into(),
                is_critical: false,
                risk_warning: None,
            };
        }
        if name_lower.contains("edge") {
            return AppMetadataInfo {
                description: "Navegador web oficial de Microsoft integrado en el ecosistema de Windows.".into(),
                category: "Navegador Web".into(),
                is_critical: false,
                risk_warning: None,
            };
        }

        // 3. DESARROLLO Y PROGRAMACIÓN
        if name_lower.contains("visual studio code") || name_lower.contains("vscode") {
            return AppMetadataInfo {
                description: "Editor de código fuente ligero y potente con soporte para extensiones y depuración.".into(),
                category: "Desarrollo de Software".into(),
                is_critical: false,
                risk_warning: None,
            };
        }
        if name_lower.contains("visual studio") {
            return AppMetadataInfo {
                description: "Entorno de desarrollo integrado (IDE) completo de Microsoft para C++, C#, .NET y web.".into(),
                category: "Entorno de Desarrollo (IDE)".into(),
                is_critical: false,
                risk_warning: None,
            };
        }
        if name_lower.contains("git") {
            return AppMetadataInfo {
                description: "Sistema de control de versiones distribuido para rastrear cambios en código fuente.".into(),
                category: "Herramienta de Desarrollo".into(),
                is_critical: false,
                risk_warning: None,
            };
        }
        if name_lower.contains("docker") {
            return AppMetadataInfo {
                description: "Plataforma de virtualización y contenedores para desplegar aplicaciones de forma aislada.".into(),
                category: "Virtualización & Servidores".into(),
                is_critical: false,
                risk_warning: None,
            };
        }
        if name_lower.contains("python") {
            return AppMetadataInfo {
                description: "Intérprete del lenguaje de programación Python para scripting y ciencia de datos.".into(),
                category: "Lenguaje de Programación".into(),
                is_critical: false,
                risk_warning: None,
            };
        }
        if name_lower.contains("node") {
            return AppMetadataInfo {
                description: "Entorno de ejecución de JavaScript del lado del servidor basado en el motor V8.".into(),
                category: "Entorno de Programación".into(),
                is_critical: false,
                risk_warning: None,
            };
        }

        // 4. VIDEOJUEGOS Y ENTRETENIMIENTO
        if name_lower.contains("steam") {
            return AppMetadataInfo {
                description: "Plataforma líder de distribución digital de videojuegos y comunidad creada por Valve.".into(),
                category: "Plataforma de Videojuegos".into(),
                is_critical: false,
                risk_warning: None,
            };
        }
        if name_lower.contains("discord") {
            return AppMetadataInfo {
                description: "Aplicación de mensajería instantánea, chat de voz y comunidades virtuales.".into(),
                category: "Comunicación & Chat".into(),
                is_critical: false,
                risk_warning: None,
            };
        }
        if name_lower.contains("epic games") {
            return AppMetadataInfo {
                description: "Tienda y lanzador de videojuegos de Epic Games.".into(),
                category: "Plataforma de Videojuegos".into(),
                is_critical: false,
                risk_warning: None,
            };
        }
        if name_lower.contains("spotify") {
            return AppMetadataInfo {
                description: "Servicio de reproducción de música, podcasts y audiolibros en streaming.".into(),
                category: "Música & Streaming".into(),
                is_critical: false,
                risk_warning: None,
            };
        }

        // 5. OFIMÁTICA Y PRODUCTIVIDAD
        if name_lower.contains("microsoft office") || name_lower.contains("office 365") || name_lower.contains("word") || name_lower.contains("excel") {
            return AppMetadataInfo {
                description: "Suite ofimática de productividad empresarial para documentos, hojas de cálculo y presentaciones.".into(),
                category: "Suite Ofimática".into(),
                is_critical: false,
                risk_warning: None,
            };
        }
        if name_lower.contains("notion") {
            return AppMetadataInfo {
                description: "Espacio de trabajo todo en uno para notas, bases de datos y gestión de proyectos.".into(),
                category: "Productividad & Notas".into(),
                is_critical: false,
                risk_warning: None,
            };
        }

        // 6. UTILIDADES Y COMPRESIÓN
        if name_lower.contains("7-zip") || name_lower.contains("winrar") {
            return AppMetadataInfo {
                description: "Herramienta de compresión y descompresión de archivos (ZIP, RAR, 7Z, TAR).".into(),
                category: "Compresión de Archivos".into(),
                is_critical: false,
                risk_warning: None,
            };
        }
        if name_lower.contains("vlc") {
            return AppMetadataInfo {
                description: "Reproductor multimedia universal de código abierto compatible con todos los códecs de video y audio.".into(),
                category: "Reproductor Multimedia".into(),
                is_critical: false,
                risk_warning: None,
            };
        }

        // 7. INFERENCIA POR EDITOR O GENERAL
        let desc = if !pub_lower.is_empty() {
            format!("Aplicación para Windows desarrollada por {}.", publisher.unwrap_or("un editor verificado"))
        } else {
            "Aplicación instalada para el sistema operativo Windows.".into()
        };

        AppMetadataInfo {
            description: desc,
            category: "Aplicación de Escritorio".into(),
            is_critical: false,
            risk_warning: None,
        }
    }
}
