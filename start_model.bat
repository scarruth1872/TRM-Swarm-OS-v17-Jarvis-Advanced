@echo off
setlocal enabledelayedexpansion
title TRM Swarm OS - Start a model on demand

rem ============================================================================
rem start_model.bat <alias>  - start ONE llama-server, with the memory guard.
rem
rem WHY THIS EXISTS
rem   The launcher starts a lean fleet by default (embeddings + Instella). Every
rem   llama-server with --n-gpu-layers 99 reserves 4-10x its model file in commit
rem   because the Vulkan driver keeps host-visible staging on top of VRAM:
rem       qwen2.5-1.5b   1.04 GB file ->  9.84 GB commit
rem       qwen3-4b       2.33 GB file -> 13.59 GB commit
rem       qwen3-4b+LoRA  2.33 GB file ->  9.51 GB commit
rem       Bonsai-27B                  ->  5.26 GB
rem       Instella-16B-MoE            ->  8.41 GB
rem       embeddings                  ->  0.35 GB
rem   Starting five of those cost 37.5 GB of commit on a 32 GB box (23.5 GB of
rem   pagefile in use, 4.6 GB available) and the ones nobody called sat at a 3-8 MB
rem   working set for days. So a model is started when it is wanted, not at boot.
rem
rem The model's own stdout goes to logs\model_<port>.log. A model that dies on a bad flag
rem looks exactly like a slow load from the outside - measured 2026-11-13: --batch-size 2 with
rem --parallel 4 aborts with GGML_ASSERT(n_outputs_max <= cparams.n_outputs_max) before loading,
rem and the launcher reported only "did not answer".
rem
rem FAST FLAGS (measured 2026-11-13, see docs/PHASE-6-GAPS.md G96): -fa on -ub 128 -b 512.
rem On a model that fits in VRAM this took Bonsai-27B Q1_0 from 13.20 to 23.72 tok/s; on the
rem oversized Qwen3.8-27B (10.87 GB file on a 12 GB card) from 5.16 to 5.95. The Instella alias
rem keeps plain flags: it runs the fork build, which was not benchmarked for -fa.
rem
rem ALIASES: vote (8587, gpt-oss-20b = the council's vote model) · embed (8593) · instella (8596)
rem          bonsai (8591) · 1.5b (8598) · 4b (8595) · jarvis (8597, qwen3-4b + LoRA)
rem
rem EXIT: 0 already serving or now healthy · 1 failed to come up
rem       2 refused (not enough free memory) · 3 usage error
rem ============================================================================

set "ROOT_DIR=F:\Development sites\TRM-Swarm-OS-v2"
set "LLAMA_EXE=F:\Development sites\ub3\llama-vulkan-win\llama-server.exe"
set "INSTELLA_EXE=D:\llama-instella\llama.cpp\build\bin\llama-server.exe"
rem Bonsai 2 binaries, measured 2026-11-13 (docs/PHASE-6-GAPS.md G97):
rem   PTQ1_0 (5.54 GB) needs the PrismML fork's ternary kernels - the fork DOES ship win-vulkan-x64.
rem   Q2_0   (7.10 GB) needs stock llama.cpp >= b11556; older builds load it and emit garbage.
set "PRISM_EXE=F:\Development sites\llama-vulkan-prism\llama-server.exe"
set "B12360_EXE=F:\Development sites\llama-vulkan-b12360\llama-server.exe"
rem The council's vote model runs on the VENDOR build, not the ub3 build: that is the binary the
rem council was measured against (task TRM-VoteModel-8587, tools/start_vote_model_8587.bat).
set "VOTE_EXE=%ROOT_DIR%\vendor\llama.cpp\build-win\bin\llama-server.exe"

set "M_1.5B=%ROOT_DIR%\models\qwen2.5-1.5b-instruct-q4_k_m.gguf"
set "M_4B=%ROOT_DIR%\models\qwen3-4b-instruct-q4_k_m.gguf"
set "M_BONSAI=%ROOT_DIR%\Jarvis-Advanced-main\models\Bonsai-27B-Q1_0.gguf"
set "M_INSTELLA=D:\q3\Instella-MoE-16B-A3B-Think.Q3_K_M.gguf"
set "M_EMBED=C:/Users/shawn/.ollama/models/blobs/sha256-970aa74c0a90ef7482477cf803618e776e173c007bf957f635f1015bfcfef0e6"
set "M_LORA=%ROOT_DIR%\Jarvis-Advanced-main\models\jarvis_qwen3_lora.gguf"
set "M_BONSAI2_PTQ=E:\Hugginface models\Ternary-Bonsai-2-27B-PTQ1_0.gguf"
set "M_BONSAI2_Q2=E:\Hugginface models\Ternary-Bonsai-2-27B-Q2_0.gguf"
set "M_BONSAI2_2B=E:\Hugginface models\Ternary-Bonsai-2-2B-Q2_0.gguf"
set "M_BONSAI2_9B=E:\Hugginface models\Ternary-Bonsai-2-9B-Q2_0.gguf"
set "M_GPTOSS=F:\models\gpt-oss-20b\gpt-oss-20b-MXFP4.gguf"

rem Measured as MB free physically, not as a guess. Override for a deliberate squeeze.
if not defined TRM_MIN_FREE_MB set "TRM_MIN_FREE_MB=6000"

set "ALIAS=%~1"
if "%ALIAS%"=="" goto :usage

set "PORT="
set "LABEL="
set "EXE=%LLAMA_EXE%"
set "MODEL="
set "EXTRA="
if /i "%ALIAS%"=="embed"    (set "PORT=8593" & set "LABEL=Embeddings"       & set "MODEL=%M_EMBED%"   & set "EXTRA=--embeddings --ctx-size 2048 --threads 6")
if /i "%ALIAS%"=="instella" (set "PORT=8596" & set "LABEL=Instella-MoE-16B" & set "MODEL=%M_INSTELLA%" & set "EXE=%INSTELLA_EXE%" & set "EXTRA=--n-gpu-layers 99 -fa on -ub 128 -b 512 --ctx-size 16384 --parallel 4 --threads 8")
if /i "%ALIAS%"=="bonsai"   (set "PORT=8591" & set "LABEL=Bonsai-27B"       & set "MODEL=%M_BONSAI%"  & set "EXTRA=--n-gpu-layers 99 -fa on -ub 128 -b 512 --ctx-size 2048 --threads 8")
if /i "%ALIAS%"=="vote"     (set "PORT=8587" & set "LABEL=Council-Vote-gpt-oss-20b" & set "MODEL=%M_GPTOSS%" & set "EXE=%VOTE_EXE%" & set "EXTRA=--ctx-size 4096 --threads 12")
rem 1.5b used to live on 8587. Since 2026-11-22 that port belongs to the council's VOTE model
rem (task TRM-VoteModel-8587), so the 1.5B moved to 8598. Leaving the old mapping here would have made
rem 'start_model.bat 1.5b' report [OK] already serving while the port answered with a 20B.
if /i "%ALIAS%"=="1.5b"     (set "PORT=8598" & set "LABEL=Inference-1.5B"   & set "MODEL=%M_1.5B%"    & set "EXTRA=--n-gpu-layers 99 -fa on -ub 128 -b 512 --ctx-size 4096 --threads 12 --parallel 4")
if /i "%ALIAS%"=="4b"       (set "PORT=8595" & set "LABEL=Qwen3-4B"         & set "MODEL=%M_4B%"      & set "EXTRA=--n-gpu-layers 99 --ctx-size 4096 --threads 8")
if /i "%ALIAS%"=="bonsai2"  (set "PORT=8612" & set "LABEL=Bonsai2-27B-PTQ1_0" & set "MODEL=%M_BONSAI2_PTQ%" & set "EXE=%PRISM_EXE%" & set "EXTRA=--n-gpu-layers 99 -fa on -ub 128 -b 512 --ctx-size 32768 --parallel 8 --threads 8")
if /i "%ALIAS%"=="bonsai2q" (set "PORT=8613" & set "LABEL=Bonsai2-27B-Q2_0"   & set "MODEL=%M_BONSAI2_Q2%"  & set "EXE=%B12360_EXE%" & set "EXTRA=--n-gpu-layers 99 -fa on -ub 128 -b 512 --ctx-size 8192 --threads 8")
if /i "%ALIAS%"=="bonsai2b" (set "PORT=8730" & set "LABEL=Bonsai2-2B-Q2_0"   & set "MODEL=%M_BONSAI2_2B%"  & set "EXE=%B12360_EXE%" & set "EXTRA=--n-gpu-layers 99 -fa on -ub 128 -b 512 --ctx-size 32768 --parallel 8 --threads 8")
if /i "%ALIAS%"=="bonsai9b" (set "PORT=8731" & set "LABEL=Bonsai2-9B-Q2_0"   & set "MODEL=%M_BONSAI2_9B%"  & set "EXE=%B12360_EXE%" & set "EXTRA=--n-gpu-layers 99 -fa on -ub 128 -b 512 --ctx-size 16384 --parallel 4 --threads 8")
if /i "%ALIAS%"=="jarvis"   (set "PORT=8597" & set "LABEL=Jarvis+LoRA"      & set "MODEL=%M_4B%"      & set "EXTRA=--lora %M_LORA% --n-gpu-layers 99 -fa on -ub 128 -b 512 --ctx-size 16384 --parallel 1 --threads 8 --alias jarvis")

if not defined PORT goto :usage
echo === start_model: %ALIAS%  ^(%LABEL%, port %PORT%^) ===

if not exist "%EXE%" (
    echo   [FAIL]  server binary missing: %EXE%
    exit /b 1
)
if not exist "%MODEL%" (
    echo   [FAIL]  model file missing: %MODEL%
    exit /b 1
)

call :probe %PORT%
if not errorlevel 1 (
    echo   [OK]    already serving on :%PORT% - nothing started
    echo           expected here: %LABEL%
    echo           %MODEL%
    echo           If that is NOT what answers, another model owns this port - check before trusting it.
    exit /b 0
)

call :free_mb
if %_FREE% LSS %TRM_MIN_FREE_MB% (
    echo   [REFUSED] %_FREE% MB free, floor is %TRM_MIN_FREE_MB% MB.
    echo             This model reserves 4-10x its file in commit; starting it here would
    echo             push the whole box into paging. Free memory first, or start it with
    echo             TRM_MIN_FREE_MB raised only if you accept the squeeze.
    exit /b 2
)
echo   [MEM]   %_FREE% MB free - starting

if not exist "%ROOT_DIR%\logs" mkdir "%ROOT_DIR%\logs" 2>nul
if not exist "%ROOT_DIR%\run" mkdir "%ROOT_DIR%\run" 2>nul
set "MLOG=%ROOT_DIR%\logs\model_%PORT%.log"
:: The ps1 starts the server DIRECTLY (not through cmd.exe) so the pid in run\model_<port>.pid is
:: the process that holds the VRAM. Measured 2026-11-13: the cmd.exe wrapper pid killed the wrapper
:: and left an 8.2 GB orphan still serving - stop reported exit 0 while the port stayed up.
::
:: DETACH WITH A PID (G91 pattern). `start /min cmd /c ...` looked fine but MEASURED: a caller that
:: invokes this script from PowerShell blocks until the model exits, because the child stays attached
:: to the caller's console - observed as a 400 s hang on a server that was already serving. Start-Process
:: -PassThru returns immediately AND gives the pid, which is then recorded like run\council.pid.
set "_MPID="
set "MPID_FILE=%ROOT_DIR%\run\model_%PORT%.pid"
rem MEASURED: capturing the launcher's stdout with for/f makes the caller wait for the server to
rem exit (a 400 s hang on a model that was already serving). So the launcher is fired off with
rem `start` and the pid is read from the pid file it writes - the caller waits for a FILE, not a process.
del "%MPID_FILE%" 2>nul
start "TRM-start-%ALIAS%" /min powershell -NoProfile -ExecutionPolicy Bypass -File "%ROOT_DIR%\tools\start_model.ps1" -Exe "%EXE%" -Model "%MODEL%" -Port %PORT% -Extra "%EXTRA%" -Log "%MLOG%" -PidFile "%MPID_FILE%"
set "_WAIT=0"
:wait_pid
if exist "%MPID_FILE%" goto have_pid
set /a _WAIT+=1
if %_WAIT% GEQ 25 goto no_pid
%SystemRoot%\System32\ping.exe -n 2 127.0.0.1 >nul 2>&1
goto wait_pid
:have_pid
set /p _MPID=<"%MPID_FILE%"
:no_pid
if defined _MPID echo   [PID]   %_MPID% recorded in run\model_%PORT%.pid

call :await %PORT%
if errorlevel 1 (
    echo   [FAIL]  :%PORT% did not answer /health within 150 s
    echo   [LOG]   %MLOG%
    powershell -NoProfile -Command "Get-Content -Tail 8 '%MLOG%','%MLOG%.err' -ErrorAction SilentlyContinue" 2>nul
    exit /b 1
)
call :free_mb
echo   [UP]    %LABEL% on :%PORT% - %_FREE% MB free afterwards
exit /b 0

:probe
for /f "usebackq delims=" %%h in (`curl.exe -s -m 4 -o NUL -w "%%{http_code}" "http://127.0.0.1:%~1/health" 2^>nul`) do set "_C=%%h"
if "%_C%"=="200" exit /b 0
exit /b 1

:await
set /a _N=0
:await_loop
call :probe %~1
if not errorlevel 1 exit /b 0
set /a _N+=1
if %_N% GEQ 30 exit /b 1
ping -n 5 127.0.0.1 >nul 2>&1
goto :await_loop

:free_mb
set "_FREE=0"
for /f "usebackq delims=" %%a in (`powershell -NoProfile -Command "[math]::Round((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory/1024)"`) do set "_FREE=%%a"
exit /b 0

:usage
echo.
echo  start_model.bat ^<alias^>       start one model, with the memory guard
echo.
echo    vote      8587  gpt-oss-20b MXFP4 - THE COUNCIL'S VOTE MODEL (task TRM-VoteModel-8587)
echo    embed     8593  embeddings, ~0.35 GB commit
echo    instella  8596  Instella-MoE-16B, ~8.4 GB
echo    bonsai    8591  Bonsai-27B ternary deep tier, ~5.3 GB
echo    1.5b      8598  qwen2.5-1.5b chat, ~9.8 GB (was 8587 before the vote model took that port)
echo    4b        8595  qwen3-4b, ~13.6 GB
echo    jarvis    8597  qwen3-4b + LoRA, ~9.5 GB
echo    bonsai2   8612  Bonsai 2 27B ternary PTQ1_0, 5.5 GB, 8 slots (G100: 312 tok/s at 2B), prism fork
echo    bonsai2q  8613  Bonsai 2 27B Q2_0, 7.1 GB file, NEEDS stock llama.cpp >= b11556
echo    bonsai2b  8730  Bonsai 2 2B Q2_0, 0.55 GB, 8 slots - the small ternary tier
echo    bonsai9b  8731  Bonsai 2 9B Q2_0, 2.56 GB, 4 slots
echo.
echo  Floor: TRM_MIN_FREE_MB=%TRM_MIN_FREE_MB% MB free. Below it the start is refused
echo  rather than allowed to push the box into paging. The launcher starts a lean
echo  fleet only (see TRM_MODEL_PROFILE in START_TRM_PLATFORM.bat).
exit /b 3
