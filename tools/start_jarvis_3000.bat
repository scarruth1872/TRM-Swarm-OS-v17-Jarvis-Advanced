@echo off
rem start_jarvis_3000.bat - start the Jarvis Node/TS server on :3000.
rem
rem Exists as a FILE on purpose: a process filter that matches 'server.ts' also matches the shell
rem whose own command line contains that string, so pkill-by-commandline from an inline command
rem kills the thing running it. Launching from a bat keeps the filter out of every process line.
setlocal
cd /d "F:\Development sites\TRM-Swarm-OS-v2\Jarvis-Advanced-main"
rem The Hermes tool kernel exports PYTHONPATH pointing at its own venv; nothing here needs it.
set "PYTHONPATH="
set "PYTHONHOME="
set "NODE_NO_WARNINGS=1"
echo. >> "F:\Development sites\TRM-Swarm-OS-v2\logs\jarvis_3000.log"
echo ==== start %DATE% %TIME% ==== >> "F:\Development sites\TRM-Swarm-OS-v2\logs\jarvis_3000.log"
npx tsx server.ts >> "F:\Development sites\TRM-Swarm-OS-v2\logs\jarvis_3000.log" 2>&1
