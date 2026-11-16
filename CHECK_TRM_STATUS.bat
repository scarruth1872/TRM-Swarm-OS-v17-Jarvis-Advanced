@echo off
title TRM Swarm OS - Status Check
echo ======================================================================
echo             TRM SWARM OS v19 - SERVICE STATUS CHECK
echo ======================================================================
echo.

echo [Port 11434 - Ollama]
netstat -ano | findstr :11434 | findstr LISTENING >nul
if %errorlevel% equ 0 (echo  Status: ONLINE) else (echo  Status: OFFLINE)

echo.
echo [Port 8022 - Council Server]
netstat -ano | findstr :8022 | findstr LISTENING >nul
if %errorlevel% equ 0 (echo  Status: ONLINE) else (echo  Status: OFFLINE)

echo.
echo [Port 3002 - Companion Bridge]
netstat -ano | findstr :3002 | findstr LISTENING >nul
if %errorlevel% equ 0 (echo  Status: ONLINE) else (echo  Status: OFFLINE)

echo.
echo [Port 3001 - Companion Proxy]
netstat -ano | findstr :3001 | findstr LISTENING >nul
if %errorlevel% equ 0 (echo  Status: ONLINE) else (echo  Status: OFFLINE)

echo.
echo [Port 5183 - LCARS Dashboard]
netstat -ano | findstr :5183 | findstr LISTENING >nul
if %errorlevel% equ 0 (echo  Status: ONLINE) else (echo  Status: OFFLINE)

echo.
echo ======================================================================
pause
