@echo off
setlocal enabledelayedexpansion
title Compilador de Instalador One-Click de Unistall

echo ==============================================================================
echo             COMPILADOR DEL INSTALADOR ONE-CLICK - UNISTALL
echo ==============================================================================
echo.

cd /d "%~dp0.."

REM 1. Compilar frontend y backend con assets embebidos si se solicita o no existe el binario
if not exist "src-tauri\target\release\unistall.exe" (
    echo [1/2] Compilando aplicacion para produccion (assets embebidos)...
    where npx >nul 2>nul
    if !errorlevel! equ 0 (
        call npx tauri build --no-bundle
    ) else (
        call node "%~dp0..\node_modules\@tauri-apps\cli\tauri.js" build --no-bundle
    )
    if !errorlevel! neq 0 (
        echo [ERROR] Fallo al compilar la aplicacion.
        pause
        exit /b 1
    )
) else (
    echo [1/2] Binario nativo de produccion detectado en src-tauri\target\release\unistall.exe
)

cd /d "%~dp0"
echo.
echo [2/2] Buscando compilador NSIS (makensis.exe)...

set "MAKENSIS="

where makensis >nul 2>nul
if %errorlevel% equ 0 (
    set "MAKENSIS=makensis"
    goto :FOUND
)

if exist "C:\Program Files (x86)\NSIS\makensis.exe" (
    set "MAKENSIS=C:\Program Files (x86)\NSIS\makensis.exe"
    goto :FOUND
)

if exist "C:\Program Files\NSIS\makensis.exe" (
    set "MAKENSIS=C:\Program Files\NSIS\makensis.exe"
    goto :FOUND
)

echo [ERROR] No se encontro makensis.exe en el sistema.
echo Por favor asegurese de tener instalado NSIS (Nullsoft Scriptable Install System).
echo.
pause
exit /b 1

:FOUND
echo [OK] NSIS detectado en: "!MAKENSIS!"
echo.
echo Compilando Unistall_Installer.nsi...
"!MAKENSIS!" "%~dp0Unistall_Installer.nsi"

if %errorlevel% equ 0 (
    echo.
    echo ==============================================================================
    echo [EXITO] Instalador One-Click generado satisfactoriamente en:
    echo        %~dp0Unistall_Setup_v1.0.0.exe
    echo ==============================================================================
) else (
    echo.
    echo [ERROR] Ocurrio un fallo durante la compilacion de NSIS.
)

echo.
pause
