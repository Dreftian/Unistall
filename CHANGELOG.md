# Changelog

Todas las novedades, correcciones y mejoras notables de **Unistall** se documentarán en este archivo.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y este proyecto se adhiere a [Semantic Versioning](https://semver.org/lang/es/).

---

## [1.0.0] - 2026-10-18

### 🚀 Añadido
- **Motor de Escaneo Multi-Fuente en Paralelo**:
  - Win32 (64-bit y WoW6432Node 32-bit).
  - Aplicaciones por usuario (`HKCU`).
  - UWP / Windows Store / MSIX a través de Win32 AppModel API nativo sin subprocesos de PowerShell.
  - Soporte nativo para Scoop, Chocolatey y Winget.
- **Cazador Heurístico de Restos (Deep Cleaner)**:
  - Búsqueda de archivos y carpetas huérfanas en `%AppData%`, `%LocalAppData%`, `%ProgramData%` y `%Temp%`.
  - Detección de claves huérfanas en el Registro (`HKCU\Software` y `HKLM\Software`).
  - Algoritmo de puntuación de confianza (*Confidence Score* 0-100%) para evitar falsos positivos.
  - Integración con **Windows Restart Manager API** para desbloquear archivos en uso o programar su borrado en reinicio (`MoveFileExW`).
- **Gestor de Inicio (Startup Manager)**:
  - Enumeración y control de programas de inicio en `HKCU\Run` y `HKLM\Run`.
- **Asistente de Desinstalación (Wizard)**:
  - Flujo guiado de desinstalación con detección automática de restos post-desinstalación.
- **Interfaz Fluent Design (Windows 11)**:
  - Soporte Mica/Acrylic con selector de tema oscuro y claro.
  - Virtual Scrolling optimizado para más de 10,000 programas a 120 FPS fluidos.
  - Búsqueda en tiempo real por nombre, editor o categoría.
- **Empaquetado y Distribución**:
  - Instalador asistido NSIS Modern UI 2 (`Unistall_Setup_v1.0.0.exe`) con modo silencioso (`/S`).
  - Binario portable autónomo (`unistall.exe`).
