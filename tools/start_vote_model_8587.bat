@echo off
REM :8587 = the COUNCIL VOTE MODEL. host_council_server calls this for llm_agent_vote().
REM gpt-oss-20b MXFP4, 4 slots, 4096 ctx, 12 threads, loopback only.
REM
REM WHY A TASK AND NOT start_model.bat: start_model.bat is the on-demand launcher and its alias map
REM still says "1.5b (8587)" -- it would put a 1.5B where the council's vote model lives. This file
REM pins the exact command line the council is measured against.
REM
REM COMMIT BUDGET, checked before creating this task rather than assumed: the 20B file is 12.1 GB and
REM the running process holds 1,049 MB working set with NO --n-gpu-layers flag (mmap + CPU threads),
REM against 38.1 GB commit used of a 69.9 GB limit. That is why this is safe at logon where a
REM 99-layer GPU model would not be (the start_model.bat header measured 4-10x file size in commit).
REM
REM Foreground inside the task, same trap as the embedder: `start /min` returns before the child has a
REM console and the task dies 0x80070002 with nothing listening.
setlocal
set "ROOT=F:\Development sites\TRM-Swarm-OS-v2"
set "EXE=%ROOT%\vendor\llama.cpp\build-win\bin\llama-server.exe"
set "MODEL=F:\models\gpt-oss-20b\gpt-oss-20b-MXFP4.gguf"
set "LOG=%ROOT%\logs\model_8587.log"

if not exist "%EXE%" exit /b 1
if not exist "%MODEL%" exit /b 1
if not exist "%ROOT%\logs" mkdir "%ROOT%\logs"

REM Already serving? Exit clean instead of racing a second binder onto the port.
powershell -NoProfile -Command "try { $null = Invoke-WebRequest 'http://127.0.0.1:8587/health' -TimeoutSec 3; exit 0 } catch { exit 1 }" >nul 2>&1
if not errorlevel 1 exit /b 0

"%EXE%" --model "%MODEL%" --host 127.0.0.1 --port 8587 --ctx-size 4096 --threads 12 >> "%LOG%" 2>&1
exit /b %ERRORLEVEL%
