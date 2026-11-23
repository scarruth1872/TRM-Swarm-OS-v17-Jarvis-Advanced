@echo off
rem start_dashboard_5183.bat - the LCARS dashboard (vite dev server, :5183).
rem
rem The kernel's health matrix has reported "LCARS dashboard unreachable on port 5183" as its only
rem anomaly, because nothing ever started this. Started from a file so the guard can call it and so no
rem inline process filter has to name this script.
setlocal
cd /d "F:\Development sites\TRM-Swarm-OS-v2\dashboard"
set "PYTHONPATH="
set "PYTHONHOME="
set "NODE_NO_WARNINGS=1"
if not exist "node_modules" (
  echo dashboard: node_modules missing - run npm install first
  exit /b 1
)
echo. >> "F:\Development sites\TRM-Swarm-OS-v2\logs\dashboard_5183.log"
echo ==== start %DATE% %TIME% ==== >> "F:\Development sites\TRM-Swarm-OS-v2\logs\dashboard_5183.log"
call npm run dev -- --host 127.0.0.1 >> "F:\Development sites\TRM-Swarm-OS-v2\logs\dashboard_5183.log" 2>&1
