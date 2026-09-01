@echo off
setlocal enabledelayedexpansion
title TRM Swarm OS - Platform Launcher

echo ======================================================================
echo             TRM SWARM OS v19 - PLATFORM LAUNCHER
echo ======================================================================
echo.

set ROOT_DIR=F:\Development sites\TRM-Swarm-OS-v2
cd /d "%ROOT_DIR%"

:: 1. Check & Launch Ollama (Port 11434)
echo [1/5] Checking Ollama Service (Port 11434)...
netstat -ano | findstr :11434 | findstr LISTENING >nul
if %errorlevel% equ 0 (
    echo       Ollama is already running.
) else (
    echo       Starting Ollama...
    start "TRM-Ollama" /min "C:\Users\shawn\AppData\Local\Programs\Ollama\ollama.exe" serve
    timeout /t 2 /nobreak >nul
)

:: 2. Check & Launch Host Council Server (Port 8022)
echo [2/5] Checking Host Council Server (Port 8022)...
netstat -ano | findstr :8022 | findstr LISTENING >nul
if %errorlevel% equ 0 (
    echo       Council Server is already running.
) else (
    echo       Starting Council Server (Release Binary)...
    start "TRM-Council-Server" /min "%ROOT_DIR%\trm_uefi_kernel\host_council_server\target\release\host_council_server.exe"
    timeout /t 1 /nobreak >nul
)

:: 3. Check & Launch Companion Bridge (Port 3002)
echo [3/5] Checking Companion Bridge (Port 3002)...
netstat -ano | findstr :3002 | findstr LISTENING >nul
if %errorlevel% equ 0 (
    echo       Companion Bridge is already running.
) else (
    echo       Starting Companion Bridge (Release Binary)...
    start "TRM-Companion-Bridge" /min "%ROOT_DIR%\trm_uefi_kernel\trm_companion_bridge\target\release\trm_companion_bridge.exe"
    timeout /t 1 /nobreak >nul
)

:: 4. Check & Launch Companion Proxy (Port 3001)
echo [4/5] Checking Companion Proxy (Port 3001)...
netstat -ano | findstr :3001 | findstr LISTENING >nul
if %errorlevel% equ 0 (
    echo       Companion Proxy is already running.
) else (
    echo       Starting Companion Proxy (Python)...
    start "TRM-Companion-Proxy" /min python -u "%ROOT_DIR%\trm_uefi_kernel\trm_companion_proxy.py"
    timeout /t 1 /nobreak >nul
)

:: 5. Check & Launch Dashboard (Port 5183)
echo [5/5] Checking LCARS Dashboard (Port 5183)...
netstat -ano | findstr :5183 | findstr LISTENING >nul
if %errorlevel% equ 0 (
    echo       Dashboard is already running.
) else (
    echo       Starting Dashboard (npm run dev)...
    cd /d "%ROOT_DIR%\dashboard"
    start "TRM-LCARS-Dashboard" /min cmd /c "npm run dev"
    cd /d "%ROOT_DIR%"
    timeout /t 2 /nobreak >nul
)

echo.
echo ======================================================================
echo              PLATFORM SERVICES INITIALIZED
echo ======================================================================
echo.
echo  - Dashboard URL:       http://localhost:5183
echo  - Council Server:      http://localhost:8022
echo  - Companion API:       http://localhost:3001
echo  - Neural Ollama:       http://localhost:11434
echo.
echo Opening Dashboard in default browser...
start http://localhost:5183
echo Done.
timeout /t 4
