@echo off
title Disable TRM Swarm OS Auto-Start
echo Disabling Auto-Start on Windows Login...

del "%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\TRM Swarm OS.lnk" 2>nul

echo [SUCCESS] Auto-start disabled.
pause
