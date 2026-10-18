; ==============================================================================
; UNISTALL - ONE-CLICK HIGH-PERFORMANCE WINDOWS INSTALLER SCRIPT (NSIS 3.x)
; ==============================================================================

Unicode true
SetCompressor /SOLID lzma
RequestExecutionLevel admin

!define PRODUCT_NAME "Unistall"
!define PRODUCT_VERSION "1.0.0"
!define PRODUCT_PUBLISHER "Unistall Software"
!define PRODUCT_WEB_SITE "https://github.com/Unistall/Unistall"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\Unistall.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\Unistall"
!define PRODUCT_UNINST_ROOT_KEY "HKLM"
!define MAIN_EXE "Unistall.exe"

; Interfaz Moderna MUI2
!include "MUI2.nsh"
!include "FileFunc.nsh"

; Definiciones de UI y Colores Fluent
!define MUI_ABORTWARNING
!define MUI_ICON "..\src-tauri\icons\icon.ico"
!define MUI_UNICON "..\src-tauri\icons\icon.ico"

; Páginas del Instalador (Configuración One-Click / Rápida)
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\${MAIN_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Iniciar Unistall ahora"
!insertmacro MUI_PAGE_FINISH

; Páginas del Desinstalador
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; Idioma
!insertmacro MUI_LANGUAGE "Spanish"
!insertmacro MUI_LANGUAGE "English"

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "..\Install\Unistall_Setup_v${PRODUCT_VERSION}.exe"
InstallDir "$PROGRAMFILES64\Unistall"
InstallDirRegKey HKLM "${PRODUCT_DIR_REGKEY}" ""
ShowInstDetails nevershow
ShowUnInstDetails nevershow

; ==============================================================================
; SECCIÓN PRINCIPAL DE INSTALACIÓN
; ==============================================================================
Section "MainSection" SEC01
    SetOutPath "$INSTDIR"
    SetOverwrite on

    ; Empaquetar el binario nativo compilado en Rust (release)
    File "/oname=${MAIN_EXE}" "..\src-tauri\target\release\unistall.exe"
    
    ; Acceso directo en Menú Inicio
    CreateDirectory "$SMPROGRAMS\Unistall"
    CreateShortcut "$SMPROGRAMS\Unistall\Unistall.lnk" "$INSTDIR\${MAIN_EXE}" "" "$INSTDIR\${MAIN_EXE}" 0
    CreateShortcut "$SMPROGRAMS\Unistall\Desinstalar Unistall.lnk" "$INSTDIR\uninst.exe" "" "$INSTDIR\uninst.exe" 0

    ; Acceso directo en el Escritorio
    CreateShortcut "$DESKTOP\Unistall.lnk" "$INSTDIR\${MAIN_EXE}" "" "$INSTDIR\${MAIN_EXE}" 0

    ; Creación del Desinstalador
    WriteUninstaller "$INSTDIR\uninst.exe"

    ; Registro en Windows (Panel de Control y Configuración de Aplicaciones)
    WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\${MAIN_EXE}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayName" "${PRODUCT_NAME}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninst.exe"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "QuietUninstallString" "$INSTDIR\uninst.exe /S"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\${MAIN_EXE}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
    WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoModify" 1
    WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoRepair" 1

    ; Cálculo de tamaño en KB para EstimatedSize
    ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
    IntFmt $0 "0x%08X" $0
    WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "EstimatedSize" "$0"
SectionEnd

; ==============================================================================
; SECCIÓN DE DESINSTALACIÓN LIMPIA AL 100%
; ==============================================================================
Section Uninstall
    ; Cerrar procesos activos de Unistall si están abiertos
    nsExec::Exec 'taskkill /F /IM ${MAIN_EXE}'

    ; Eliminar accesos directos
    Delete "$DESKTOP\Unistall.lnk"
    Delete "$SMPROGRAMS\Unistall\Unistall.lnk"
    Delete "$SMPROGRAMS\Unistall\Desinstalar Unistall.lnk"
    RMDir "$SMPROGRAMS\Unistall"

    ; Eliminar archivos de la carpeta de instalación
    Delete "$INSTDIR\${MAIN_EXE}"
    Delete "$INSTDIR\uninst.exe"
    RMDir /r "$INSTDIR"

    ; Limpiar claves del Registro de Windows
    DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}"
    DeleteRegKey HKLM "${PRODUCT_DIR_REGKEY}"
    DeleteRegKey /ifempty HKCU "Software\Unistall"
    DeleteRegKey /ifempty HKLM "Software\Unistall"

    SetAutoClose true
SectionEnd
