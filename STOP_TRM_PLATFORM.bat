@echo off
title TRM Swarm OS - Platform Shutdown

echo ======================================================================
echo             TRM SWARM OS v19 - PLATFORM SHUTDOWN
echo ======================================================================
echo.

echo Stopping TRM services...

taskkill /F /IM host_council_server.exe 2>nul
if %errorlevel% equ 0 echo  [STOPPED] host_council_server.exe

taskkill /F /IM trm_companion_bridge.exe 2>nul
if %errorlevel% equ 0 echo  [STOPPED] trm_companion_bridge.exe

for /f "tokens=5" %%a in ('netstat -aon ^| findstr :3001 ^| findstr LISTENING') do (
    taskkill /F /PID %%a 2>nul
    echo  [STOPPED] Companion Proxy (PID %%a)
)

for /f "tokens=5" %%a in ('netstat -aon ^| findstr :5183 ^| findstr LISTENING') do (
    taskkill /F /PID %%a 2>nul
    echo  [STOPPED] Dashboard Dev Server (PID %%a)
)

echo.
echo TRM services stopped.
timeout /t 3
