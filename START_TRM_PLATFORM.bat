@echo off
setlocal EnableExtensions EnableDelayedExpansion
REM HWiNFO sensor feed (measured 2026-11-13: SVI2 vcore 1.27 V vs WMI's constant 1.1 V).
REM Without this the voltage routes report a nominal and no domain is measurable.
if not defined TRM_HWINFO_CSV set "TRM_HWINFO_CSV=C:\Users\shawn\AppData\Local\hermes\cache\scratch\hwi\hwinfo_live.csv"
title TRM Swarm OS - Platform Launcher

set ROOT_DIR=F:\Development sites\TRM-Swarm-OS-v2
set "PY_EXE=C:\Users\shawn\AppData\Local\Programs\Python\Python311\python.exe"
set "NPM_CMD=C:\Program Files\nodejs\npm.cmd"
set "LLAMA_EXE=F:\Development sites\ub3\llama-vulkan-win\llama-server.exe"
set "MODEL=%ROOT_DIR%\models\qwen2.5-1.5b-instruct-q4_k_m.gguf"
set "EMB_MODEL=C:/Users/shawn/.ollama/models/blobs/sha256-970aa74c0a90ef7482477cf803618e776e173c007bf957f635f1015bfcfef0e6"
:: Ternary deep tier. Measured 2026-11-05: loads on the RX 6700 XT in 5.28s at
:: --n-gpu-layers 99. voice.py probes :8591 and only routes deep-word questions
:: here, so an absent deep model degrades to :8587 instead of failing chat.
set "DEEP_MODEL=%ROOT_DIR%\Jarvis-Advanced-main\models\Bonsai-27B-Q1_0.gguf"
rem Qwen3-4B is registered in /models (qwen3-4b-instruct-q4_k_m, 4,022,468,096 params
rem measured from GGUF tensor dims). Without a launcher entry a reboot left :8595 down
rem and that manifest row unverified.
set "QWEN3_MODEL=%ROOT_DIR%\models\qwen3-4b-instruct-q4_k_m.gguf"
rem Instella-MoE needs a FORK build (csabakecskemeti/llama.cpp @ instella-moe) because
rem upstream llama.cpp has no InstellaMoE support. Both files live off-repo, so this
rem tier is optional: absent files print INFO and do not fail the boot.
set "INSTELLA_EXE=D:\llama-instella\llama.cpp\build\bin\llama-server.exe"
set "INSTELLA_MODEL=D:\q3\Instella-MoE-16B-A3B-Think.Q3_K_M.gguf"

:: ─── MODEL FLEET PROFILE ─────────────────────────────────────────────────────
:: MEASURED 2026-11-13: every llama-server started with --n-gpu-layers 99 reserves
:: far more COMMIT than the model file, because the Vulkan driver keeps host-visible
:: staging on top of VRAM:
::     :8587 qwen2.5-1.5b   1.04 GB file ->  9.84 GB commit
::     :8595 qwen3-4b       2.33 GB file -> 13.59 GB commit
::     :8597 qwen3-4b+LoRA  2.33 GB file ->  9.51 GB commit
::     :8591 Bonsai-27B                   ->  5.26 GB
::     :8596 Instella-16B-MoE             ->  8.41 GB
::     :8593 embeddings                   ->  0.35 GB
:: Starting the five this script owns cost 37.5 GB of commit on a 32 GB box: pagefile
:: 23.5 GB in use, 4.6 GB available, and the three oldest sat at a 3-8 MB working set
:: for days - fully paged out and still charged. So the default profile starts only
:: what the platform is actually calling; the rest are on demand.
::     lean     (default)  :8593 + :8596                      ~8.8 GB commit
::     standard            + :8591 + :8587  (voice deep tier)  ~23.9 GB
::     full                + :8595                             ~37.5 GB
:: :8597 (qwen3-4b + Jarvis LoRA) is not started by this script at all.
if not defined TRM_MODEL_PROFILE set "TRM_MODEL_PROFILE=lean"
:: A model that pushes the box into paging is worse than an absent model: it slows every
:: other service. Below this floor the launcher SKIPS the model and says why.
if not defined TRM_MIN_FREE_MB set "TRM_MIN_FREE_MB=6000"
:: One log PER RUN, not one fixed name. MEASURED DEFECT: the services started by a run
:: inherit this handle and keep it open for their whole lives, so the NEXT launch could
:: never reopen it and died with "launcher could not run (log not writable)" while every
:: service was healthy and listening. A timestamped name means a relaunch never contends
:: with its predecessor's children.
for /f %%t in ('powershell -NoProfile -Command "Get-Date -Format yyyyMMdd_HHmmss"') do set "_TS=%%t"
if not defined _TS set "_TS=%RANDOM%"
set "LOG=%ROOT_DIR%\logs\platform_launch_%_TS%.log"
:: The child writes its own exit code here. Must be set BEFORE the wrapper so
:: the child process inherits it. See :run_child for why ERRORLEVEL is not used.
set "RC_FILE=%TEMP%\trm_launch_rc.txt"

if "%TRM_LOGGED%"=="1" goto :dispatch
set TRM_LOGGED=1
if not exist "%ROOT_DIR%\logs" mkdir "%ROOT_DIR%\logs" 2>nul
:: A LOCKED LOG MUST NEVER ABORT THE LAUNCH. Probe writability first; if the
:: log is held by another process, fall back to a temp log and keep going.
:: No nested parentheses here on purpose: nesting a redirection inside an
:: if-block makes cmd emit a bogus "cannot find the drive specified".
call :log_probe
if errorlevel 1 goto :log_fallback
call :run_child "%LOG%"
exit /b %RC%

:log_fallback
set "LOG=%TEMP%\trm_platform_launch.log"
echo [WARN] primary log locked by another process - logging to %LOG%
call :run_child "%LOG%"
exit /b %RC%

:run_child
:: %1 = log file. Runs the real body with output redirected, then decides the
:: exit code from the child's sentinel instead of from ERRORLEVEL.
::
:: DEFECT FIX (1): A FAILED OUTPUT REDIRECTION LEAVES ERRORLEVEL AT 0.
:: If the log becomes unwritable between :log_probe and this line, cmd never
:: runs the child, prints the sharing violation to stderr, and ERRORLEVEL stays
:: 0 -- so `set RC=%ERRORLEVEL%` reported success for a launch that never
:: happened. ERRORLEVEL is therefore not evidence that the child ran. The child
:: writes its exit code to RC_FILE on every path (:finish); a missing sentinel
:: means the launch aborted, and that is a failure no matter what ERRORLEVEL says.
set "RC_FILE=%TEMP%\trm_launch_rc.txt"
del "%RC_FILE%" >nul 2>&1
"%COMSPEC%" /c "%~f0" 1>> "%~1" 2>&1
set "RC="
if exist "%RC_FILE%" for /f "usebackq delims=" %%r in ("%RC_FILE%") do set "RC=%%r"
if not defined RC (
    echo.
    echo === LAUNCH FAILED - the launcher body aborted before it could report ===
    echo === log: %~1 ===
    echo === This message used to say "log not writable", which sent the last investigation to the log file while
    echo === the real cause was a parse error in this script three lines above ^(a :: comment or a redirect inside
    echo === a parenthesised if-block^). If the log is empty, look at the script, not at the log.
    set "RC=1"
    exit /b 1
)
2>nul (>>"%~1" echo ==== done   %DATE% %TIME% rc=%RC% ====)
if not "%RC%"=="0" (
    echo.
    echo === LAUNCH FAILED - exit code %RC% ===
    echo === see %~1 ===
    exit /b %RC%
)
exit /b 0

:log_probe
:: exit 0 if the log is writable, 1 if it is locked. The append itself is the
:: test - opening then closing would race another writer.
2>nul (>>"%LOG%" echo ==== launch %DATE% %TIME% ====) && exit /b 0
exit /b 1

:dispatch
if "%TRM_SELFTEST%"=="1" goto :selftest
if "%TRM_MODELS_PLAN%"=="1" goto :models_plan
if "%TRM_CHECK_ONLY%"=="1" goto :check_only
goto :main
:: ============================================================
:: PORT VERIFICATION HELPERS (fix: stale process squatting a port)
:: ensure PORT EXPECTED_EXE DESC
:: If PORT is held by a process whose image != EXPECTED_EXE, it is a STALE
:: SQUATTER: report it loudly and kill it. Port occupancy alone is NOT proof
:: the right binary is serving.
:port_pid
set "_PID="
for /f "tokens=5" %%a in ('netstat -ano ^| findstr ":%_PORT% " ^| findstr LISTENING') do set "_PID=%%a"
exit /b 0

:verify
set "_PORT=%~1"
set "_EXE=%~2"
set "_DESC=%~3"
call :port_pid
if not defined _PID (
    echo       [DOWN]  %_DESC% - port %_PORT% free
    exit /b 1
)
set "_IMG="
for /f "tokens=1 delims=," %%b in ('tasklist /FI "PID eq %_PID%" /FO CSV /NH 2^>nul') do set "_IMG=%%~b"
if /i "%_IMG%"=="%_EXE%" (
    echo       [OK]    %_DESC% - pid %_PID% ^(%_IMG%^)
    exit /b 0
)
echo       [SQUAT] port %_PORT% held by '%_IMG%' pid %_PID%
echo       [SQUAT] expected '%_EXE%' - KILLING stale process
if not "%TRM_NO_KILL%"=="1" taskkill /PID %_PID% /F >nul 2>&1
ping -n 3 127.0.0.1 >nul 2>&1
exit /b 1

:confirm
:: %1=port %2=desc %3=rounds - each round is ~2s. Default 6 rounds (12s).
:: MEASURED DEFECT: the Council API (8099) needs far longer than 12s when all
:: nine services start together, because llama-server is still loading a model
:: and the heavy imports compete for CPU. A full launcher run reported
:: "8099 did NOT come up after 12s" and marked FAILED, while the very next smoke
:: test found 8099 healthy -- a false failure. Slow starters get more rounds.
set "_PORT=%~1"
set "_DESC=%~2"
set "_ROUNDS=%~3"
if not defined _ROUNDS set "_ROUNDS=6"
set /a _N=0
:confirm_loop
call :port_pid
if defined _PID (
    echo       [UP]    %_DESC% - pid %_PID%
    exit /b 0
)
set /a _N+=1
if %_N% GEQ %_ROUNDS% (
    rem NOT a failure claim. Measured 2026-11-09 at a real logon boot: every slow
    rem starter tripped these windows (8591 took over 90s, 8022 and the python
    rem services over 12s) and the SAME run's smoke test then found all ten
    rem healthy. The old text asserted "the started process failed to bind",
    rem which was false - it had simply not bound yet. Say what was measured and
    rem defer the verdict to the authoritative final gate.
    echo       [SLOW]  %_DESC% - not listening after %_ROUNDS% rounds of 2s - DEFERRED to the final gate
    exit /b 1
)
ping -n 3 127.0.0.1 >nul 2>&1
goto :confirm_loop

:verify_cmd
:: Like :verify but for python/node services - matches a MARKER in the command
:: line, because many python.exe processes share the same image name.
:: Cmdline is read with python+psutil: wmic is GONE from Win11 (verified) and
:: nested double quotes do not survive a for/f backquote command. An UNREADABLE
:: cmdline must NEVER kill - it reports UNVERIFIED and exits 0.
:: NOTE: this is a PRE-START identity check only. It is never sufficient for
:: success; every service is gated on :verify_http / :await_http below.
:: %1=port %2=marker %3=desc
set "_PORT=%~1"
set "_MARK=%~2"
set "_DESC=%~3"
call :port_pid
if not defined _PID (
    echo       [DOWN]  %_DESC% - port %_PORT% free
    exit /b 1
)
set "_CMD="
for /f "usebackq delims=" %%c in (`"%ROOT_DIR%\tools\proc_cmdline.bat" %_PID% 2^>nul`) do set "_CMD=%%c"
if not defined _CMD (
    echo       [UNVERIFIED] %_DESC% - pid %_PID% up, cmdline unreadable - NOT killing
    exit /b 0
)
set "_HIT=%TEMP%\trm_vc_%_PORT%.hit"
del "%_HIT%" >nul 2>&1
echo %_CMD%| findstr /i /c:"%_MARK%" >nul 2>&1 && echo MARK >"%_HIT%"
if exist "%_HIT%" (
    echo       [OK]    %_DESC% - pid %_PID% ^(matches '%_MARK%'^)
    del "%_HIT%" >nul 2>&1
    exit /b 0
)
echo       [SQUAT] port %_PORT% held by wrong process pid %_PID%
echo       [SQUAT] cmd: %_CMD%
if not "%TRM_NO_KILL%"=="1" taskkill /PID %_PID% /F >nul 2>&1
ping -n 3 127.0.0.1 >nul 2>&1
exit /b 1

:probe_http
:: Silent readiness probe. Sets _REASON on failure. No killing: a service that
:: is merely slow to boot must not be shot in the back.
:: %1=port %2=path %3=marker
set "_PORT=%~1"
set "_PATH=%~2"
set "_MARK=%~3"
set "_REASON="
rem NO PORT->PID GATE. Measured 2026-11-09: netstat on this host reported pid 0
rem for 8022 and 3000, named the COUNCIL's pid for 5183, and listed no LISTENING
rem row at all for seven services that answered HTTP 200. A port probe that
rem cannot be trusted must not be a precondition for the HTTP probe, which is
rem the one that carries identity (marker in the body).
set "_F=%TEMP%\trm_vh_%_PORT%.txt"
del "%_F%" >nul 2>&1
set "_HTTPCODE=000"
for /f "usebackq delims=" %%h in (`curl.exe -s -m 6 -o "%_F%" -w "%%{http_code}" "http://127.0.0.1:%_PORT%%_PATH%" 2^>nul`) do set "_HTTPCODE=%%h"
if not "%_HTTPCODE%"=="200" (
    if "%_HTTPCODE%"=="000" set "_REASON=no HTTP answer on port %_PORT%"
    if not "%_HTTPCODE%"=="000" set "_REASON=HTTP %_HTTPCODE% from %_PATH%"
    del "%_F%" >nul 2>&1
    exit /b 1
)
set "_SZ=0"
if exist "%_F%" for %%A in ("%_F%") do set "_SZ=%%~zA"
if "%_SZ%"=="0" (
    set "_REASON=empty body from %_PATH%"
    del "%_F%" >nul 2>&1
    exit /b 1
)
findstr /i /c:"%_MARK%" "%_F%" >nul 2>&1
if errorlevel 1 (
    set "_REASON=%_PATH% does not serve '%_MARK%'"
    del "%_F%" >nul 2>&1
    exit /b 1
)
del "%_F%" >nul 2>&1
exit /b 0

:verify_http
:: DEFECT FIX (2): readiness is HTTP, and an HTTP failure is a FAILURE.
:: Port-listen is NOT proof the ROUTE exists: an old binary can listen on the
:: right port and 404 or serve stale JSON. The old version printed [WARN] and
:: exited 0 for a non-200, an empty body, or no body at all -- so a dead route
:: still counted as a healthy service. Every one of those is now exit 1.
:: The body goes to a temp FILE and findstr reads the file, so HTML containing
:: < > & | cannot break the comparison.
:: %1=port %2=path %3=marker %4=desc
set "_PORT=%~1"
set "_PATH=%~2"
set "_MARK=%~3"
set "_DESC=%~4"
call :probe_http "%~1" "%~2" "%~3"
if not errorlevel 1 (
    echo       [OK]    %_DESC% - port %_PORT% %_PATH% served '%_MARK%'
    exit /b 0
)
echo       [FAIL]  %_DESC% - port %_PORT% %_PATH%: %_REASON%
exit /b 1

:await_http
:: Retry the HTTP gate while a freshly started service boots. Prints exactly
:: one line, so a cold start does not spam the log with [DOWN] noise.
:: %1=port %2=path %3=marker %4=desc %5=tries
set "_PORT=%~1"
set "_PATH=%~2"
set "_MARK=%~3"
set "_DESC=%~4"
set "_TRIES=%~5"
if "%_TRIES%"=="" set "_TRIES=8"
set "_N=0"
:await_loop
call :probe_http "%~1" "%~2" "%~3"
if not errorlevel 1 (
    echo       [OK]    %_DESC% - port %_PORT% %_PATH% served '%_MARK%'
    exit /b 0
)
set /a _N+=1
if %_N% GEQ %_TRIES% (
    echo       [FAIL]  %_DESC% - port %_PORT% %_PATH% not ready after %_TRIES% tries: %_REASON%
    exit /b 1
)
ping -n 2 127.0.0.1 >nul 2>&1
goto :await_loop

:svc_already
:: "Is the RIGHT service already answering?" - the boot path's only UP test.
:: Quiet by design: one line, no killing, no PID reasoning. A service is up when
:: it answers HTTP 200 with its own marker in the body; anything else is DOWN and
:: gets started. This replaced a netstat-based check that declared healthy
:: services down, spawned four vite instances, and killed the council by pid.
:: %1=port %2=path %3=marker %4=desc
set "_PORT=%~1"
set "_PATH=%~2"
set "_MARK=%~3"
set "_DESC=%~4"
call :probe_http "%~1" "%~2" "%~3"
if not errorlevel 1 (
    echo       [OK]    %_DESC% - already serving %_PATH% as '%_MARK%'
    exit /b 0
)
echo       [DOWN]  %_DESC% - %_REASON%
exit /b 1

:start_8587
:: Kept OUT of the parenthesised if-block on purpose: a redirection inside a ( ... ) block is parsed as a
:: block terminator, which aborted the launcher with ". was unexpected at this time." after the readiness
:: probe, leaving 8587 down while the error message blamed the log file.
start "TRM-Inference" /min "%LLAMA_EXE%" --model "%MODEL%" --host 127.0.0.1 --port 8587 --n-gpu-layers 99 --ctx-size 16384 --threads 12 --parallel 4 --ubatch-size 1024 --flash-attn on > "%ROOT_DIR%\logs\model_8587.log" 2>&1
exit /b 0

:want_model
:: Should THIS model start? %1=port %2=allowed profiles (space separated) %3=label
:: Two reasons to refuse, both printed: the profile does not include it, or the box does
:: not have TRM_MIN_FREE_MB free. Silence would look like a failure; a skip that says why
:: is a measurement. exit 0 = start it, 1 = skip it.
set "_WM_PORT=%~1"
set "_WM_PROFILES=%~2"
set "_WM_LABEL=%~3"
set "_WM_OK="
for %%q in (%_WM_PROFILES%) do if /i "%%q"=="%TRM_MODEL_PROFILE%" set "_WM_OK=1"
if not defined _WM_OK (
    echo       [SKIP]  %_WM_LABEL% ^(:%_WM_PORT%^) - profile "%TRM_MODEL_PROFILE%" does not include it
    echo               ^(on demand: start_model.bat ^<alias^>  ^|  or set TRM_MODEL_PROFILE=standard^|full^)
    exit /b 1
)
call :free_mb
if %_FREE% LSS %TRM_MIN_FREE_MB% (
    echo       [SKIP]  %_WM_LABEL% ^(:%_WM_PORT%^) - %_FREE% MB free, below TRM_MIN_FREE_MB=%TRM_MIN_FREE_MB%
    exit /b 1
)
echo       [MEM]   %_FREE% MB free - %_WM_LABEL% allowed by profile "%TRM_MODEL_PROFILE%"
exit /b 0

:free_mb
:: MEASURED, never assumed: FreePhysicalMemory from the OS, in MB.
set "_FREE=0"
for /f "usebackq delims=" %%a in (`powershell -NoProfile -Command "[math]::Round((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory/1024)"`) do set "_FREE=%%a"
exit /b 0

:start_council
:: G91 LAUNCHER OWNS THE PID. `start /min` returns no pid, so "restart the council" had no handle and
:: fell back to taskkill, which needs an interactive approval on this host and timed out repeatedly.
:: Start-Process -PassThru returns the id, which is recorded in run\council.pid; STOP_TRM_PLATFORM.bat
:: reads it. The stop path itself is the process's own POST /platform/shutdown, so a restart needs no
:: kill at all -- the fallback kill exists only for a council that is too wedged to answer.
if not exist "%ROOT_DIR%\run" mkdir "%ROOT_DIR%\run" 2>nul
del "%ROOT_DIR%\run\council.pid" >nul 2>&1
set "_CPID="
for /f "usebackq delims=" %%p in (`powershell -NoProfile -Command "$p = Start-Process -FilePath '%ROOT_DIR%\trm_uefi_kernel\host_council_server\target\release\host_council_server.exe' -WindowStyle Hidden -PassThru; $p.Id"`) do set "_CPID=%%p"
if not defined _CPID (
    echo       [WARN]  Council Server - Start-Process returned no pid; falling back to start /min
    start "TRM-Council-Server" /min "%ROOT_DIR%\trm_uefi_kernel\host_council_server\target\release\host_council_server.exe"
    exit /b 0
)
>"%ROOT_DIR%\run\council.pid" echo %_CPID%
echo       [PID]   Council Server - pid %_CPID% recorded in run\council.pid
exit /b 0

:main
:: FAILED tracks any step that was started but never became READY. A launch
:: that aborts must NOT report success to the caller.
:: Keep the port/path/marker triples below in lock-step with SERVICES in
:: tools/smoke_test_platform.py -- that script is the same gate for CI.
::
:: MEASURED DEFECT FIXED 2026-11-09 - the UP/DOWN test was netstat-based.
:: On this host the port -> PID mapping is unreliable: port 5183 mapped to the
:: COUNCIL's pid, 8022 and 3000 mapped to pid 0, and seven live services had no
:: LISTENING row at all while answering HTTP 200. Consequences measured in one
:: run: every already-healthy service was declared DOWN, the launcher started
:: DUPLICATES (four vite instances on 5183), and :verify_cmd's kill path shot
:: the healthy council because 5183's row named its pid. The boot path now
:: identifies a service ONLY by what it answers: HTTP 200 whose body carries a
:: service-specific marker. Nothing is killed. A service that answers is up,
:: whatever netstat thinks.
set "FAILED=0"
rem KERNEL UI PORT. v20 binds a second listener on :5183 and serves /ui/ from the kernel
rem (main.rs:2638-2660, "the dashboard has lived on :5183"). Both it and vite bind 5183, so
rem whichever answers is a coin toss: measured both ways today - vite HTML before the duplicate
rem kill, kernel JSON after it. The kernel listener moves to 5184 so vite owns 5183 outright;
rem the Rust UI stays reachable at http://127.0.0.1:5184/ui/, and 0 disables it. Children
rem inherit this environment, so no quoting is needed at the start line.
set "TRM_UI_PORT=5184"
rem G81 CADENCE. The platform runs itself: 8 worker threads inside the council binary doing real work
rem over repository state (ddr_scan / gate_replay / aura_identify / aura_eval / featurize). Measured:
rem 8.48% -> 46.11% mean CPU utilisation, 2.04 -> 11.07 busy cores of 24, 0 errors. Requests stay
rem answerable (cadence adds ~4-33% latency to routes that already shell out to WMI). Its own stop path
rem is POST /platform/cadence?action=stop, and /platform/cadence reports jobs, errors and per-agent
rem ownership. Set to 0 to start the platform without the supervisor.
set "TRM_CADENCE_WORKERS=8"
echo [1/9] Council Server (8022)
call :svc_already 8022 /health "Native Rust Sovereign Kernel" "Council Server"
if errorlevel 1 (
    call :start_council
    call :await_http 8022 /health "Native Rust Sovereign Kernel" "Council Server" 60
    if errorlevel 1 set "FAILED=1"
)

echo [2/9] Companion Bridge (3002)
call :svc_already 3002 /health "TRM Sovereign Companion Bridge" "Companion Bridge"
if errorlevel 1 (
    start "TRM-Companion-Bridge" /min "%ROOT_DIR%\trm_uefi_kernel\trm_companion_bridge\target\release\trm_companion_bridge.exe"
    call :await_http 3002 /health "TRM Sovereign Companion Bridge" "Companion Bridge" 40
    if errorlevel 1 set "FAILED=1"
)

echo [3/9] Companion Proxy (3001)
call :svc_already 3001 /health "TRM Sovereign Companion Bridge" "Companion Proxy"
if errorlevel 1 (
    start "TRM-Companion-Proxy" /min "%PY_EXE%" -u "%ROOT_DIR%\trm_uefi_kernel\trm_companion_proxy.py"
    call :await_http 3001 /health "TRM Sovereign Companion Bridge" "Companion Proxy" 40
    if errorlevel 1 set "FAILED=1"
)

echo [4/9] Jarvis (3000)
call :svc_already 3000 /health "jarvis" "Jarvis"
if errorlevel 1 (
    cd /d "%ROOT_DIR%\Jarvis-Advanced-main"
    start "TRM-Jarvis" /min cmd /c ""%NPM_CMD%" run dev"
    cd /d "%ROOT_DIR%"
    call :await_http 3000 /health "jarvis" "Jarvis" 60
    if errorlevel 1 set "FAILED=1"
)

echo [5/9] Inference Engine Vulkan (8587)
call :want_model 8587 "standard full" "Inference Engine 1.5B"
if errorlevel 1 goto :skip_8587
call :svc_already 8587 /health "ok" "Inference Engine"
if errorlevel 1 (
    if not exist "%LLAMA_EXE%" (
        echo       [FAIL]  llama-server.exe missing - chat will be templated
        set "FAILED=1"
    ) else (
        rem DEFECT FIXED 2026-11-13: --batch-size 2 with --parallel 4 aborts this build with
        rem GGML_ASSERT(n_outputs_max <= cparams.n_outputs_max). Measured in the foreground:
        rem with the flag the server dies before loading; without it "model loaded" in 2.17 s.
        rem Output goes to a log, because a model that dies silently looks like a slow load.
        call :start_8587
        call :await_http 8587 /health "ok" "Inference Engine" 90
        if errorlevel 1 (
            echo       [LOG]   %ROOT_DIR%\logs\model_8587.log
            set "FAILED=1"
        )
    )
)
:skip_8587

echo [5b/9] Deep Tier - Bonsai 27B Q1_0 ternary (8591)
:: 3.80 GB ternary GGUF, measured 5.28s load at --n-gpu-layers 99 on a warm
:: disk; far longer on a cold logon boot, so it gets the longest window. This is
:: the tier voice.py routes explain/why/design/prove questions to. Without it
:: those questions still answer, from the 1.5B on 8587. No parentheses inside
:: this block: a paren in a parenthesised if-block breaks the cmd parser.
call :want_model 8591 "standard full" "Deep Tier Bonsai-27B"
if errorlevel 1 goto :skip_8591
call :svc_already 8591 /health "ok" "Deep Tier"
if errorlevel 1 (
    if not exist "%DEEP_MODEL%" (
        echo       [FAIL]  Bonsai-27B-Q1_0.gguf missing - deep tier unavailable
        set "FAILED=1"
    ) else (
        start "TRM-DeepTier" /min "%LLAMA_EXE%" --model "%DEEP_MODEL%" --host 127.0.0.1 --port 8591 --n-gpu-layers 99 --ctx-size 2048 --threads 8
        call :await_http 8591 /health "ok" "Deep Tier" 120
        if errorlevel 1 set "FAILED=1"
    )
)
:skip_8591

echo [5c/9] Qwen3-4B Q4_K_M (8595)
call :want_model 8595 "full" "Qwen3-4B"
if errorlevel 1 goto :skip_8595
call :svc_already 8595 /health "ok" "Qwen3-4B"
if errorlevel 1 (
    if not exist "%QWEN3_MODEL%" (
        echo       [WARN]  qwen3-4b-instruct-q4_k_m.gguf missing - :8595 down, manifest row unverified
    ) else (
        start "TRM-Qwen3-4B" /min "%LLAMA_EXE%" --model "%QWEN3_MODEL%" --host 127.0.0.1 --port 8595 --n-gpu-layers 99 --ctx-size 4096 --threads 8
        call :await_http 8595 /health "ok" "Qwen3-4B" 90
        if errorlevel 1 set "FAILED=1"
    )
)
:skip_8595

echo [5d/9] Instella-MoE-16B Q3_K_M (8596, fork build, optional)
call :want_model 8596 "lean standard full" "Instella-MoE-16B"
if errorlevel 1 goto :skip_8596
call :svc_already 8596 /health "ok" "Instella-MoE"
if errorlevel 1 (
    if exist "%INSTELLA_EXE%" (
        if exist "%INSTELLA_MODEL%" (
            start "TRM-Instella" /min "%INSTELLA_EXE%" --model "%INSTELLA_MODEL%" --host 127.0.0.1 --port 8596 --n-gpu-layers 99 --ctx-size 4096 --threads 8
            call :await_http 8596 /health "ok" "Instella-MoE" 120
        ) else (
            echo       [INFO]  Instella GGUF absent - independent deep tier off
        )
    ) else (
        echo       [INFO]  instella-moe fork binary absent - independent deep tier off
    )
)
:skip_8596

:: [6/9] RETIRED — tools/trm_server.py (port 8588) served an ARC/TRM brain that no
:: launcher step, Rust crate or frontend calls. It was verified unreferenced: the only
:: mentions in the tree were agent chatter in .swarm/mailboxes/*. Retired to reclaim memory.
:: Re-enable by running: %PY_EXE% "%ROOT_DIR%\tools\trm_server.py"

echo [7/9] LCARS Dashboard (5183)
:: Marker is the vite HMR hook on '/', NOT /health: the dashboard proxies
:: /health and /system/* to the council, so a proxied 200 says the council is
:: up, not that the dashboard is. Probed on a dead dashboard, /health answered
:: with the council's JSON and passed. '/' proves vite itself is serving.
call :svc_already 5183 / "react-refresh" "LCARS Dashboard"
if errorlevel 1 (
    cd /d "%ROOT_DIR%\dashboard"
    start "TRM-LCARS-Dashboard" /min cmd /c ""%NPM_CMD%" run dev"
    cd /d "%ROOT_DIR%"
    call :await_http 5183 / "react-refresh" "LCARS Dashboard" 60
    if errorlevel 1 set "FAILED=1"
)

:: [8/9] RETIRED — the Python Council API (8099) is no longer part of the boot.
:: The Rust council retrieves its own evidence in-process (G48, evidence_index.rs reads
:: data/council_evidence_index.json directly). Measured: with 8099 up a safe hardening
:: directive approved A12/R1; with it down the old code path rejected it A1/R13; with the
:: native index it approves A12/R1 again with 8099 stopped. Nothing else in the tree calls 8099.
:: Re-enable if ever needed: set SWARM_PORT=8099 ^&^& set PYTHONPATH=%ROOT_DIR% ^&^& %PY_EXE% -m swarm_v2.app_v2

echo [9/9] Embeddings Server (8593)
:: Without this the semantic domain router silently degrades to keyword_fallback
:: after a reboot. Same llama-server binary as 8587, --embeddings mode.
call :want_model 8593 "lean standard full" "Embeddings"
if errorlevel 1 goto :skip_8593
call :svc_already 8593 /health "ok" "Embeddings"
if errorlevel 1 (
    if not exist "%LLAMA_EXE%" (
        echo       [FAIL]  llama-server.exe missing - semantic router will use keyword_fallback
        set "FAILED=1"
    ) else (
        start "TRM-Embeddings" /min "%LLAMA_EXE%" --model "%EMB_MODEL%" --host 127.0.0.1 --port 8593 --embeddings --ctx-size 2048 --threads 6
        call :await_http 8593 /health "ok" "Embeddings" 60
        if errorlevel 1 set "FAILED=1"
    )
)
:skip_8593

:: FINAL GATE 2026-11-03: the per-service checks above match marker SUBSTRINGS, and
:: a substring is not proof of an API. Measured: Jarvis /health served
:: "<!doctype html>" and the marker "Jarvis" matched the page title, so the check
:: passed while the route was broken. tools/smoke_test_platform.py is the single
:: source of truth because it also requires JSON from every JSON service. Run it
:: here so the launcher cannot declare success on a page that merely mentions the
:: service name.
if exist "%ROOT_DIR%\tools\smoke_test_platform.py" call :final_gate

echo.
if "%FAILED%"=="1" (
    echo === PLATFORM NOT READY - one or more services failed their HTTP check ===
    echo === see %LOG% ===
    set "_RC=1"
    goto :finish
)
echo === PLATFORM SERVICES INITIALIZED ===
echo  Model fleet  profile=%TRM_MODEL_PROFILE%   (on demand: start_model.bat embed^|instella^|bonsai^|1.5b^|4b^|jarvis)
echo  Dashboard  http://localhost:5183
echo  Council    http://localhost:8022
echo  Jarvis     http://localhost:3000
echo  Companion  http://localhost:3001
echo  TRM svc    http://localhost:8588
echo  Embeddings http://localhost:8593
start http://localhost:5183
ping -n 5 127.0.0.1 >nul 2>&1
set "_RC=0"
goto :finish

:models_plan
:: Side-effect-free check of the model gate: prints what the CURRENT profile would start
:: and stops. Exists because "the launcher no longer starts six models" has to be provable
:: without booting nine services and watching memory for ten minutes.
echo === MODEL FLEET PLAN (profile=%TRM_MODEL_PROFILE%, floor=%TRM_MIN_FREE_MB% MB) ===
call :free_mb
echo       free: %_FREE% MB
call :want_model 8593 "lean standard full" "Embeddings"
call :want_model 8596 "lean standard full" "Instella-MoE-16B"
call :want_model 8591 "standard full" "Deep Tier Bonsai-27B"
call :want_model 8587 "standard full" "Inference Engine 1.5B"
call :want_model 8595 "full" "Qwen3-4B"
echo === END PLAN (nothing was started) ===
set "_RC=0"
goto :finish

:check_only
echo --- CHECK ONLY ---
set "CHK_FAIL=0"
call :verify 8022 host_council_server.exe "Council"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_http 8022 /health HEALTHY "Council"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_http 8022 /arc/pool unified_transistor_fabric "Council"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_http 8022 /arc/status "arc/status" "Council"
if errorlevel 1 set "CHK_FAIL=1"
call :verify 3002 trm_companion_bridge.exe "Bridge"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_http 3002 /health "TRM Sovereign Companion Bridge" "Bridge"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_cmd 3001 trm_companion_proxy "Proxy"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_http 3001 /health "TRM Sovereign Companion Bridge" "Proxy"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_cmd 3000 server.ts "Jarvis"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_http 3000 /health "Jarvis" "Jarvis"
if errorlevel 1 set "CHK_FAIL=1"
call :verify 8587 llama-server.exe "Inference"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_http 8587 /health "ok" "Inference"
if errorlevel 1 set "CHK_FAIL=1"
call :verify 8591 llama-server.exe "Deep Tier"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_http 8591 /health "ok" "Deep Tier"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_cmd 8588 trm_server "TRM Svc"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_http 8588 /health "trm7m" "TRM Svc"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_cmd 5183 vite "Dashboard"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_http 5183 /health "HEALTHY" "Dashboard"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_cmd 8099 app_v2 "Council API"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_http 8099 /health "v5_online" "Council API"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_http 8099 /council/status rust_kernel_bound "Council API"
if errorlevel 1 set "CHK_FAIL=1"
call :verify 8593 llama-server.exe "Embeddings"
if errorlevel 1 set "CHK_FAIL=1"
call :verify_http 8593 /health "ok" "Embeddings"
if errorlevel 1 set "CHK_FAIL=1"

:: Same final gate as :main. The marker checks above cannot tell an API from a page,
:: so check-only must not report PASS on its own say-so either.
if exist "%ROOT_DIR%\tools\smoke_test_platform.py" (
    echo --- FINAL READINESS GATE: tools\smoke_test_platform.py ---
    "%PY_EXE%" "%ROOT_DIR%\tools\smoke_test_platform.py"
    if errorlevel 1 set "CHK_FAIL=1"
)
if "%CHK_FAIL%"=="1" (
    echo --- RESULT: FAIL - one or more checks failed ---
    set "_RC=1"
    goto :finish
)
echo --- RESULT: PASS ---
set "_RC=0"
goto :finish

:selftest
:: Deliberately WRONG markers. A verifier that only ever prints OK is useless:
:: this proves the failure path fires. Always run with TRM_NO_KILL=1.
:: The selftest PASSES only if all three negative checks correctly FAILED --
:: otherwise the verifier itself is broken and that is a failure too.
echo --- SELF TEST (wrong markers, expect FAIL) ---
set "ST_BAD=0"
call :verify_cmd 8022 WRONG_MARKER "selftest"
if errorlevel 1 set /a ST_BAD+=1
call :verify_http 8022 /health WRONG_MARKER "selftest"
if errorlevel 1 set /a ST_BAD+=1
call :verify_http 8022 /nope "x" "selftest"
if errorlevel 1 set /a ST_BAD+=1
if "%ST_BAD%"=="3" (
    echo --- SELF TEST PASS - 3/3 negative checks failed as expected ---
    set "_RC=0"
    goto :finish
)
echo --- SELF TEST FAIL - verifier accepted %ST_BAD%/3 bad inputs, expected 0/3 ---
set "_RC=1"
goto :finish

:finish
:: Single exit point. Writes the sentinel the parent wrapper reads, so a launch
:: that dies before reaching here is detectable instead of silently "successful".
:: Callers set _RC then goto here. Reading %~1 would be wrong: it is the
:: script's own argument, not a resume code. No _RC at all means fail closed.
if not defined _RC set "_RC=1"
>"%RC_FILE%" echo %_RC%
if not "%_RC%"=="0" echo === LAUNCH FAILED - one or more services are not ready ===
exit /b %_RC%

:final_gate
:: The final gate is AUTHORITATIVE. FAILED was otherwise sticky: a slow starter that
:: trips an early timeout (measured: "Council API - port 8099 did NOT come up after
:: 12s") sets FAILED=1 and nothing ever cleared it. The same run then printed
:: "10/10 services healthy / RESULT: PASS" from the smoke test and still declared
:: "PLATFORM NOT READY" and exited 1. A launcher that fails on a healthy platform is
:: worse than no launcher: it trains the operator to ignore it, and it is why the
:: logon shortcut looked like it did nothing at all.
echo.
echo --- FINAL READINESS GATE: tools\smoke_test_platform.py ---
:: COLD-BOOT HEADROOM. Measured 2026-11-09 at a real logon boot: the per-service
:: windows all tripped, yet the platform was fully healthy minutes later, because
:: a cold start loads a 28 MB council binary plus a 31 MB K3 blob and two
:: llama.cpp models off a cold disk while nine services compete for CPU. One
:: smoke call at that moment answered while services were still settling, so the
:: gate gets retries instead of a single shot. The gate stays AUTHORITATIVE; the
:: retries only give the measurement time to be true.
set "GATE_N=0"
set "GATE_MAX=12"
:gate_attempt
set /a GATE_N+=1
"%PY_EXE%" "%ROOT_DIR%\tools\smoke_test_platform.py"
if not errorlevel 1 goto :gate_pass
if %GATE_N% GEQ %GATE_MAX% goto :gate_fail
echo       [WAIT]  services still settling - retry %GATE_N%/%GATE_MAX% in 10s
ping -n 11 127.0.0.1 >nul 2>&1
goto :gate_attempt

:gate_fail
echo       [FAIL]  final readiness gate still reports unhealthy services after %GATE_N% attempts
set "FAILED=1"
exit /b 1

:gate_pass
echo       [OK]    final readiness gate: all services healthy - clears earlier timeouts
set "FAILED=0"
exit /b 0
