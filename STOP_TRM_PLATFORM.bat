@echo off
setlocal EnableExtensions
rem ============================================================================
rem  TRM SWARM OS - PLATFORM SHUTDOWN
rem
rem  This script no longer decides anything. It calls tools\platform_stop.py,
rem  which stops each service by an exact handle and then RE-PROBES every port
rem  over HTTP; the exit code is that verification.
rem
rem  The previous version read `netstat -ano` to find the pid holding a port.
rem  MEASURED 2026-11-13: that table has NO rows for this platform's ports even
rem  while they serve HTTP (netstat :8593 and :3000 return nothing; TCP table
rem  shows only TimeWait/FinWait1 owned by pid 0). It reported "[DOWN] port
rem  free" for a live embed server, "[STOPPED] Jarvis (pid 20424)" after killing
rem  a wrapper, and printed "TRM platform stopped (council verified down)"
rem  while :8593 and :3000 were still answering. Also missing entirely: :8612,
rem  :8620, :8730 - the three processes holding ~10.7 GB of the 12 GB card.
rem  Those four defects are why a reboot could leave the GPU occupied.
rem
rem  Verification, not assertion: platform_stop.py ends with "TRM platform
rem  stopped - every port re-probed and silent." or "NOT STOPPED: ..." and rc=1.
rem ============================================================================

set "ROOT_DIR=%~dp0"
set "PY=%ROOT_DIR%.venv\Scripts\python.exe"
if not exist "%PY%" set "PY=C:\Users\shawn\AppData\Local\Programs\Python\Python311\python.exe"
if not exist "%PY%" set "PY=python"

if not exist "%ROOT_DIR%scratch\stop" mkdir "%ROOT_DIR%scratch\stop"
"%PY%" -u "%ROOT_DIR%tools\platform_stop.py" %*
set "RC=%ERRORLEVEL%"
echo.
if "%RC%"=="0" (echo Shutdown verified: every TRM port is silent.) else (echo Shutdown INCOMPLETE - see NOT STOPPED above.)
exit /b %RC%
