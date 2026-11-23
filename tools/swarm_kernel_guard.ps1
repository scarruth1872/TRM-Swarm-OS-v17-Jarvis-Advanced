# swarm_kernel_guard.ps1 — start-if-absent supervisor for the TRM council kernel and its relays.
#
# Run by the SwarmKernel scheduled task. Two paths:
#   FAST PATH  probe the five listening ports. All answering -> print one line, exit. NO WMI at all.
#   SLOW PATH  only a service whose port is dead reaches the WMI/start logic below.
#
# Why: measured before this change, 442-962 ms per run with THREE Get-CimInstance enumerations EVERY
# run, 1,440 times a day, even with everything healthy. The cost was the process scan, not the check.
# A health check should measure the service (the port), not enumerate every process on the machine.
#
# A named mutex makes an overlapping run (a slow start path against the next tick) exit immediately
# instead of racing to start the same service twice.

$ErrorActionPreference = 'Continue'
$repo = 'F:\Development sites\TRM-Swarm-OS-v2'
$exe  = Join-Path $repo 'trm_uefi_kernel\host_council_server\target\release\host_council_server.exe'
$logd = Join-Path $repo 'logs'
$tokf = 'C:\ProgramData\trm\council_token.txt'

# The hermes venv python.exe is a launcher shim that re-execs a second python, which makes one
# logical bridge look like two processes — so always use a CONCRETE interpreter path.
$script:PY311 = 'C:\Users\shawn\AppData\Local\Programs\Python\Python311\python.exe'

function Test-Port {
    param([string]$Target, [int]$Port, [int]$TimeoutMs = 400)
    $client = New-Object System.Net.Sockets.TcpClient
    try {
        $iar = $client.BeginConnect($Target, $Port, $null, $null)
        if (-not $iar.AsyncWaitHandle.WaitOne($TimeoutMs, $false)) { return $false }
        $client.EndConnect($iar)
        return $true
    } catch { return $false }
    finally { $client.Close() }
}

function Get-Python {
    param([string]$Preferred)
    if (Test-Path $Preferred) { return $Preferred }
    $cmd = Get-Command python.exe -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    return $null
}

function Start-Hidden {
    param([string]$FilePath, [string]$Arguments, [string]$Tag)
    # Start-Process REJECTS an empty -ArgumentList, and the failure must not take the other services
    # (or the kernel start below) down with it — so build the parameters conditionally and report.
    try {
        $stamp = Get-Date -Format 'yyyyMMdd_HHmmss'
        $params = @{
            FilePath               = $FilePath
            WorkingDirectory       = $repo
            WindowStyle            = 'Hidden'
            RedirectStandardOutput = (Join-Path $logd "${Tag}_$stamp.out.log")
            RedirectStandardError  = (Join-Path $logd "${Tag}_$stamp.err.log")
            PassThru               = $true
        }
        if ($Arguments) { $params['ArgumentList'] = $Arguments }
        $p = Start-Process @params
        Write-Output ("STARTED $Tag pid=" + $p.Id + " log=" + $params.RedirectStandardOutput)
        return $true
    } catch {
        Write-Output ("START FAILED $Tag : " + $_.Exception.Message)
        return $false
    }
}

function Invoke-Guard {
    if (-not (Test-Path $exe)) { Write-Output "EXE MISSING: $exe"; return 2 }
    if (-not (Test-Path $logd)) { New-Item -ItemType Directory -Path $logd -Force | Out-Null }

    $bridgeScript = Join-Path $repo 'tools\tailnet_bridge.py'
    $hasBridge = Test-Path $bridgeScript
    $tailnet = '100.93.191.43'

    # --- FAST PATH: every listener answering means there is nothing to supervise this tick ---
    $kernelUp   = Test-Port '127.0.0.1' 8022
    $proxyUp    = Test-Port '127.0.0.1' 3001
    $cpBridgeUp = Test-Port '127.0.0.1' 3002
    $jarvisUp   = Test-Port '127.0.0.1' 3000
    $tailKernel = if ($hasBridge) { Test-Port $tailnet 8022 } else { $true }
    $tailRelay  = if ($hasBridge) { Test-Port $tailnet 3001 } else { $true }

    if ($kernelUp -and $proxyUp -and $cpBridgeUp -and $jarvisUp -and $tailKernel -and $tailRelay) {
        Write-Output 'ALL LISTENING (8022,3000,3001,3002 + tailnet 8022,3001)'
        return 0
    }

    # --- tailnet relay: keep <tailnet>:8022 -> 127.0.0.1:8022 alive (tailnet-only exposure) ---
    if ($hasBridge) {
        if (-not $tailKernel) {
            $tailnetReachable = Test-Port $tailnet 8022 -TimeoutMs 800
            $b = @(Get-CimInstance Win32_Process -Filter "Name='python.exe'" -ErrorAction SilentlyContinue |
                   Where-Object { $_.CommandLine -like '*tailnet_bridge.py*' -and $_.CommandLine -notlike '*--port 3001*' })
            if ($b.Count -gt 0) {
                Write-Output ("BRIDGE ALIVE pid=" + $b[0].ProcessId)
            } elseif (-not $tailnetReachable) {
                # Reaching the tailnet is a prerequisite; starting a relay that cannot bind would just
                # fail every tick. Report the prerequisite instead of spinning up processes.
                Write-Output "BRIDGE SKIPPED: tailnet address $tailnet not reachable from this host"
            } else {
                $py = Get-Python $script:PY311
                if ($py) { Start-Hidden -FilePath $py -Arguments ('"' + $bridgeScript + '"') -Tag 'tailnet_bridge' }
                else { Write-Output 'BRIDGE SKIPPED: no python found' }
            }
        } else { Write-Output 'BRIDGE ALIVE (port answering)' }

        # --- companion relay: mobile app targets <tailnet-ip>:3001, the proxy binds loopback ---
        if (-not $tailRelay) {
            $c = @(Get-CimInstance Win32_Process -Filter "Name='python.exe'" -ErrorAction SilentlyContinue |
                   Where-Object { $_.CommandLine -like '*tailnet_bridge.py*' -and $_.CommandLine -like '*--port 3001*' })
            if ($c.Count -gt 0) {
                Write-Output ("COMPANION RELAY ALIVE pid=" + $c[0].ProcessId)
            } else {
                $pyc = Get-Python $script:PY311
                if ($pyc) {
                    Start-Hidden -FilePath $pyc -Tag 'companion_relay' `
                        -Arguments ('"' + $bridgeScript + '" --listen ' + $tailnet + ' --target 127.0.0.1 --port 3001 --no-auth')
                    Write-Output ("COMPANION RELAY STARTED ($tailnet" + ':3001 -> 127.0.0.1:3001)')
                }
            }
        } else { Write-Output 'COMPANION RELAY ALIVE (port answering)' }
    }

    # --- companion proxy :3001 (serves the mobile app's chat + status routes) ---
    if (-not $proxyUp) {
        $pxScript = Join-Path $repo 'trm_uefi_kernel\trm_companion_proxy.py'
        if (Test-Path $pxScript) {
            $px = @(Get-CimInstance Win32_Process -Filter "Name='python.exe'" -ErrorAction SilentlyContinue |
                    Where-Object { $_.CommandLine -like '*trm_companion_proxy.py*' })
            if ($px.Count -gt 0) {
                Write-Output ("COMPANION PROXY ALIVE pid=" + $px[0].ProcessId)
            } else {
                $pyp = Get-Python $script:PY311
                if ($pyp) { Start-Hidden -FilePath $pyp -Arguments ('-u "' + $pxScript + '"') -Tag 'companion_proxy' }
            }
        }
    } else { Write-Output 'COMPANION PROXY ALIVE (port answering)' }

    # --- companion bridge: microkernel fallback tier on :3002 (proxy 3001 -> 8022 -> fallback 3002) ---
    if (-not $cpBridgeUp) {
        $cpExe = Join-Path $repo 'trm_uefi_kernel\trm_companion_bridge\target\release\trm_companion_bridge.exe'
        if (Test-Path $cpExe) {
            $cp = @(Get-Process trm_companion_bridge -ErrorAction SilentlyContinue)
            if ($cp.Count -eq 0) { Start-Hidden -FilePath $cpExe -Arguments '' -Tag 'companion_bridge' }
            else { Write-Output ("COMPANION ALIVE pid=" + $cp[0].Id) }
        }
    } else { Write-Output 'COMPANION BRIDGE ALIVE (port answering)' }

    # --- Jarvis :3000 (UI + /api + kernel-grounded chat) ---
    # Supervised because it was NOT: when the Node server died it stayed dead until someone noticed,
    # while every other listener here self-healed. Started from a .bat, never from an inline command
    # whose arguments contain 'server.ts' — a pattern like that also matches the shell running it.
    if (-not $jarvisUp) {
        if (Test-Port '127.0.0.1' 3000) {
            Write-Output 'JARVIS ALIVE (port answering)'
        } else {
            $jarvisBat = Join-Path $repo 'tools\start_jarvis_3000.bat'
            if (Test-Path $jarvisBat) {
                try {
                    Start-Process -FilePath 'cmd.exe' -ArgumentList ('/c "' + $jarvisBat + '"') -WindowStyle Hidden | Out-Null
                    Write-Output 'JARVIS STARTED (:3000)'
                } catch { Write-Output ('JARVIS START FAILED: ' + $_.Exception.Message) }
            } else { Write-Output "JARVIS SKIPPED: $jarvisBat missing" }
        }
    } else { Write-Output 'JARVIS ALIVE (port answering)' }

    # --- kernel ---
    if ($kernelUp) { Write-Output 'ALIVE (port answering)'; return 0 }

    # Re-check rather than trust the fast-path sample: this is the only place a double-bind can happen.
    if (Test-Port '127.0.0.1' 8022) {
        Write-Output 'PORT 8022 BUSY by a non-managed listener; not starting'
        return 0
    }

    $env:TRM_REPO_ROOT = $repo
    # The kernel stays on loopback so local consumers (dashboard vite proxy, Jarvis) keep working.
    # Tailnet exposure is handled by tools/tailnet_bridge.py, which binds the tailnet address only.
    $env:TRM_BIND_HOST = '127.0.0.1'
    if (Test-Path $tokf) {
        $t = (Get-Content $tokf -Raw).Trim()
        if ($t) { $env:COUNCIL_AUTH_TOKEN = $t }
    }
    Start-Hidden -FilePath $exe -Arguments '' -Tag 'council_sup'
    return 0
}

# --- single-instance guard ---
$mutex = New-Object System.Threading.Mutex($false, 'Global\TRM_SwarmKernelGuard')
if (-not $mutex.WaitOne(0)) { Write-Output 'SKIP: previous guard run still active'; exit 0 }
try {
    Invoke-Guard
} catch {
    Write-Output ("GUARD ERROR: " + $_.Exception.Message)
    exit 1
} finally {
    try { $mutex.ReleaseMutex() } catch { }
    $mutex.Dispose()
}
exit 0
