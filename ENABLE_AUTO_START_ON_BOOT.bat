@echo off
title Enable TRM Swarm OS Auto-Start
echo Enabling Auto-Start on Windows Login...

set TARGET=F:\Development sites\TRM-Swarm-OS-v2\START_TRM_PLATFORM.bat
set SHORTCUT=%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\TRM Swarm OS.lnk

powershell -NoProfile -Command "$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('%SHORTCUT%'); $Shortcut.TargetPath = '%TARGET%'; $Shortcut.WorkingDirectory = 'F:\Development sites\TRM-Swarm-OS-v2'; $Shortcut.Description = 'Start TRM Swarm OS Platform'; $Shortcut.Save()"

echo [SUCCESS] Auto-start enabled! The platform will launch automatically on Windows login.
pause
