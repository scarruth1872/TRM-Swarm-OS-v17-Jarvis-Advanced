# swarm_kernel_guard.ps1 — start-if-absent supervisor for the TRM council kernel.
# Run by the SwarmKernel scheduled task every minute. Idempotent: exits 0 if already running.

$repo = 'F:\Development sites\TRM-Swarm-OS-v2'
$exe  = Join-Path $repo 'trm_uefi_kernel\host_council_server\target\release\host_council_server.exe'
$logd = Join-Path $repo 'logs'
$tokf = 'C:\ProgramData\trm\council_token.txt'

if (-not (Test-Path $exe)) { Write-Output "EXE MISSING: $exe"; exit 2 }
if (-not (Test-Path $logd)) { New-Item -ItemType Directory -Path $logd -Force | Out-Null }

# --- tailnet relay: keep 100.93.191.43:8022 -> 127.0.0.1:8022 alive (tailnet-only exposure) ---
$bridgeScript = Join-Path $repo 'tools\tailnet_bridge.py'
if (Test-Path $bridgeScript) {
    $b = @(Get-CimInstance Win32_Process -Filter "Name='python.exe'" -ErrorAction SilentlyContinue |
           Where-Object { $_.CommandLine -like '*tailnet_bridge.py*' -and $_.CommandLine -notlike '*--port 3001*' })
    # Second guard run in the same second can slip past the process scan; also require the
    # tailnet port to be free, exactly like the kernel's loopback check below.
    $tailnetBusy = $false
    try {
        $t = New-Object System.Net.Sockets.TcpClient
        $t.Connect('100.93.191.43', 8022)
        $tailnetBusy = $t.Connected
        $t.Close()
    } catch { $tailnetBusy = $false }
    if ($b.Count -eq 0 -and -not $tailnetBusy) {
        # Use a CONCRETE interpreter path. The hermes venv python.exe is a launcher shim that
        # re-execs a second python, which makes one logical bridge look like two processes.
        $py = 'C:\Users\shawn\AppData\Local\Programs\Python\Python311\python.exe'
        if (-not (Test-Path $py)) { $py = (Get-Command python.exe -ErrorAction SilentlyContinue).Source }
        if (Test-Path $py) {
            $bstamp = Get-Date -Format 'yyyyMMdd_HHmmss'
            Start-Process -FilePath $py -ArgumentList ('"' + $bridgeScript + '"') -WorkingDirectory $repo -WindowStyle Hidden `
                -RedirectStandardOutput (Join-Path $logd "tailnet_bridge_$bstamp.out.log") `
                -RedirectStandardError  (Join-Path $logd "tailnet_bridge_$bstamp.err.log") | Out-Null
            Write-Output "BRIDGE STARTED"
        } else { Write-Output "BRIDGE SKIPPED: no python found" }
    } else {
        Write-Output ("BRIDGE ALIVE pid=" + $b[0].ProcessId)
    }
}

# --- companion relay: the mobile app targets <tailnet-ip>:3001, but the proxy binds loopback ---
if (Test-Path $bridgeScript) {
    $c = @(Get-CimInstance Win32_Process -Filter "Name='python.exe'" -ErrorAction SilentlyContinue |
           Where-Object { $_.CommandLine -like '*tailnet_bridge.py*' -and $_.CommandLine -like '*--port 3001*' })
    $cBusy = $false
    try {
        $c1 = New-Object System.Net.Sockets.TcpClient
        $c1.Connect('100.93.191.43', 3001)
        $cBusy = $c1.Connected
        $c1.Close()
    } catch { $cBusy = $false }
    if ($c.Count -eq 0 -and -not $cBusy) {
        $pyc = 'C:\Users\shawn\AppData\Local\Programs\Python\Python311\python.exe'
        if (-not (Test-Path $pyc)) { $pyc = (Get-Command python.exe -ErrorAction SilentlyContinue).Source }
        if (Test-Path $pyc) {
            $ccstamp = Get-Date -Format 'yyyyMMdd_HHmmss'
            Start-Process -FilePath $pyc -ArgumentList ('"' + $bridgeScript + '" --listen 100.93.191.43 --target 127.0.0.1 --port 3001 --no-auth') `
                -WorkingDirectory $repo -WindowStyle Hidden `
                -RedirectStandardOutput (Join-Path $logd "companion_relay_$ccstamp.out.log") `
                -RedirectStandardError  (Join-Path $logd "companion_relay_$ccstamp.err.log") | Out-Null
            Write-Output "COMPANION RELAY STARTED (100.93.191.43:3001 -> 127.0.0.1:3001)"
        }
    } else {
        Write-Output ("COMPANION RELAY ALIVE pid=" + $(if ($c.Count) { $c[0].ProcessId } else { 'port-busy' }))
    }
}

# --- companion proxy :3001 (serves the mobile app's chat + status routes) ---
$pxScript = Join-Path $repo 'trm_uefi_kernel\trm_companion_proxy.py'
if (Test-Path $pxScript) {
    $px = @(Get-CimInstance Win32_Process -Filter "Name='python.exe'" -ErrorAction SilentlyContinue |
            Where-Object { $_.CommandLine -like '*trm_companion_proxy.py*' })
    $pxBusy = $false
    try {
        $p1 = New-Object System.Net.Sockets.TcpClient
        $p1.Connect('127.0.0.1', 3001)
        $pxBusy = $p1.Connected
        $p1.Close()
    } catch { $pxBusy = $false }
    if ($px.Count -eq 0 -and -not $pxBusy) {
        $pyp = 'C:\Users\shawn\AppData\Local\Programs\Python\Python311\python.exe'
        if (-not (Test-Path $pyp)) { $pyp = (Get-Command python.exe -ErrorAction SilentlyContinue).Source }
        if (Test-Path $pyp) {
            $pstamp = Get-Date -Format 'yyyyMMdd_HHmmss'
            Start-Process -FilePath $pyp -ArgumentList ('-u "' + $pxScript + '"') -WorkingDirectory $repo -WindowStyle Hidden `
                -RedirectStandardOutput (Join-Path $logd "companion_proxy_$pstamp.out.log") `
                -RedirectStandardError  (Join-Path $logd "companion_proxy_$pstamp.err.log") | Out-Null
            Write-Output "COMPANION PROXY STARTED"
        }
    } else {
        Write-Output ("COMPANION PROXY ALIVE pid=" + $(if ($px.Count) { $px[0].ProcessId } else { 'port-busy' }))
    }
}

# --- companion bridge: microkernel fallback tier on :3002 (proxy 3001 -> 8022 -> fallback 3002) ---
$cpExe = Join-Path $repo 'trm_uefi_kernel\trm_companion_bridge\target\release\trm_companion_bridge.exe'
if (Test-Path $cpExe) {
    $cp = @(Get-Process trm_companion_bridge -ErrorAction SilentlyContinue)
    if ($cp.Count -eq 0) {
        $cstamp = Get-Date -Format 'yyyyMMdd_HHmmss'
        Start-Process -FilePath $cpExe -WorkingDirectory $repo -WindowStyle Hidden `
            -RedirectStandardOutput (Join-Path $logd "companion_bridge_$cstamp.out.log") `
            -RedirectStandardError  (Join-Path $logd "companion_bridge_$cstamp.err.log") | Out-Null
        Write-Output "COMPANION BRIDGE STARTED"
    } else {
        Write-Output ("COMPANION ALIVE pid=" + $cp[0].Id)
    }
}

# --- kernel ---
# Already running? Nothing to do.
$running = @(Get-Process host_council_server -ErrorAction SilentlyContinue)
if ($running.Count -gt 0) {
    Write-Output ("ALIVE pid=" + $running[0].Id)
    exit 0
}

# Something else answering on 8022? Do not double-bind.
$portBusy = $false
try {
    $c = New-Object System.Net.Sockets.TcpClient
    $c.Connect('127.0.0.1', 8022)
    $portBusy = $c.Connected
    $c.Close()
} catch { $portBusy = $false }
if ($portBusy) { Write-Output 'PORT 8022 BUSY by a non-managed listener; not starting'; exit 0 }

$env:TRM_REPO_ROOT = $repo
# Kernel stays on loopback so local consumers (dashboard vite proxy, Jarvis) keep working.
# Tailnet exposure is handled by tools/tailnet_bridge.py, which binds 100.93.191.43:8022 ONLY.
$env:TRM_BIND_HOST = '127.0.0.1'
if (Test-Path $tokf) {
    $t = (Get-Content $tokf -Raw).Trim()
    if ($t) { $env:COUNCIL_AUTH_TOKEN = $t }
}

$stamp = Get-Date -Format 'yyyyMMdd_HHmmss'
$out = Join-Path $logd "council_sup_$stamp.out.log"
$err = Join-Path $logd "council_sup_$stamp.err.log"

$p = Start-Process -FilePath $exe -WorkingDirectory $repo -WindowStyle Hidden `
     -RedirectStandardOutput $out -RedirectStandardError $err -PassThru

Write-Output ("STARTED pid=" + $p.Id + " log=" + $out)
exit 0
