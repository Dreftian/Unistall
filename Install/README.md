# Instalador One-Click de Unistall (NSIS)

Este directorio contiene el script oficial de empaquetado para generar el instalador profesional **One-Click** de Windows para **Unistall**.

## Características del Instalador
1. **Instalación One-Click ultrarrápida**: Sin asistentes innecesarios; descomprime, registra e inicia en menos de 2 segundos.
2. **Modo Silencioso (`/S`)**: Permite despliegue corporativo automatizado sin interfaz gráfica.
3. **Registro completo en el sistema**: Añade la entrada correspondiente en el Registro de Windows (`HKLM\Software\Microsoft\Windows\CurrentVersion\Uninstall\Unistall`) con cálculo real del tamaño en disco (`EstimatedSize`).
4. **Desinstalación 100% Limpia**: Elimina binarios, accesos directos de Escritorio y Menú Inicio, y limpia las ramas de registro huérfanas.
5. **Detección de procesos activos**: Si Unistall está en ejecución al instalar o desinstalar, cierra el proceso de forma segura.

## Cómo compilar el instalador
1. Asegúrate de haber compilado el backend en modo release:
   ```cmd
   cd src-tauri
   cargo build --release
   ```
2. Ejecuta el script:
   ```cmd
   build_installer.bat
   ```
   O directamente con `makensis`:
   ```cmd
   "C:\Program Files (x86)\NSIS\makensis.exe" Unistall_Installer.nsi
   ```
