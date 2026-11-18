<#
  model_tiers.ps1 - bring up one model tier on this box.

    .\model_tiers.ps1 status      what is resident and on which port
    .\model_tiers.ps1 gate        gpt-oss-20b MXFP4   -> :8587  (INFER_PORT, veto tier)
    .\model_tiers.ps1 analysis    Instella 16B Q4_K_M -> :8596  (TRM_INFER_PORT, analysis tier)

  Only one tier is resident at a time: 11.28 GiB + 9.99 GiB does not fit an 11.2 GiB budget, and
  co-loading measures as GPU thrash rather than throughput. The script stops the other tier first.

  No -ngl 99 for gpt-oss: this build aborts with
  "failed to fit params to free device memory: n_gpu_layers already set by user to 99".
#>
param(
    [Parameter(Position = 0)]
    [ValidateSet('status', 'gate', 'analysis', 'stop')]
    [string]$Action = 'status'
)

$ErrorActionPreference = 'Stop'
$Root   = Split-Path -Parent $PSScriptRoot
$Server = Join-Path $Root 'vendor\llama.cpp\build-win\bin\llama-server.exe'
$LogDir = Join-Path $env:LOCALAPPDATA 'hermes\cache\scratch'

$Tiers = @{
    gate     = @{ Port = 8587; Model = 'F:/models/gpt-oss-20b/gpt-oss-20b-MXFP4.gguf';        Tag = 'gpt-oss-20b' }
    analysis = @{ Port = 8596; Model = 'F:/models/instella-moe-think-Q4_K_M.gguf';             Tag = 'instella-16b-a3b' }
}

function Get-Resident {
    Get-CimInstance Win32_Process -Filter "Name='llama-server.exe'" | ForEach-Object {
        $p = '-'
        if ($_.CommandLine -match '--port (\d+)') { $p = $matches[1] }
        $m = '-'
        if ($_.CommandLine -match '--model (\S+)') { $m = $matches[1] }
        [pscustomobject]@{ Pid = $_.ProcessId; Port = $p; Model = $m }
    }
}

function Wait-Health([int]$Port, [int]$TimeoutSec = 240) {
    $deadline = (Get-Date).AddSeconds($TimeoutSec)
    while ((Get-Date) -lt $deadline) {
        try {
            $r = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/health" -TimeoutSec 3 -UseBasicParsing
            if ($r.StatusCode -eq 200) { return $true }
        } catch { }
        Start-Sleep -Seconds 4
    }
    return $false
}

function Stop-All {
    foreach ($p in Get-Resident) {
        Write-Host "stopping pid $($p.Pid) on :$($p.Port)"
        # graceful first; llama-server on Windows refuses WM_CLOSE ("can only be terminated forcefully"),
        # so the graceful attempt is EXPECTED to fail - it must not terminate the script.
        try { & taskkill /PID $p.Pid 2>&1 | Out-Null } catch { }
        Start-Sleep -Seconds 2
        if (Get-Process -Id $p.Pid -ErrorAction SilentlyContinue) {
            try { & taskkill /PID $p.Pid /F 2>&1 | Out-Null } catch { }
        }
    }
    Start-Sleep -Seconds 3
}

switch ($Action) {
    'status' {
        $r = Get-Resident
        if (-not $r) { Write-Host 'no llama-server resident' }
        else { $r | Format-Table -AutoSize | Out-String | Write-Host }
        foreach ($port in 8587, 8596) {
            $state = 'down'
            try {
                $resp = Invoke-WebRequest -Uri "http://127.0.0.1:$port/health" -TimeoutSec 3 -UseBasicParsing
                if ($resp.StatusCode -eq 200) { $state = 'ok' }
            } catch { }
            Write-Host ("  :{0} health={1}" -f $port, $state)
        }
    }
    'stop' { Stop-All; Write-Host 'stopped' }
    default {
        if (-not (Test-Path $Server)) { throw "server not found: $Server" }
        $t = $Tiers[$Action]
        if (-not (Test-Path $t.Model)) { throw "model not found: $($t.Model)" }
        Stop-All
        $log  = Join-Path $LogDir ("tier_" + $t.Tag + ".log")
        $args = @('--model', $t.Model, '--host', '127.0.0.1', '--port', $t.Port,
                  '--ctx-size', '4096', '--threads', '12')
        $proc = Start-Process -FilePath $Server -ArgumentList $args -PassThru `
                              -RedirectStandardOutput $log -RedirectStandardError ($log + '.err') `
                              -WindowStyle Hidden
        Write-Host "started $($t.Tag) pid=$($proc.Id) on :$($t.Port)  log=$log"
        if (Wait-Health -Port $t.Port) {
            Write-Host "READY :$($t.Port)  ($($t.Tag))"
        } else {
            Write-Host "FAILED to become healthy on :$($t.Port)"
            Get-Content $log -Tail 20 -ErrorAction SilentlyContinue
            exit 1
        }
    }
}
