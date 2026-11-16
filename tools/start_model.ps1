# start_model.ps1 — detach one llama-server, return its pid.
#
# WHY A FILE AND NOT A ONE-LINER: the batch launcher needed this call inside a for/f
# backtick block, where every quote has to be escaped twice. The escaped form was not
# readable and did not work. A .ps1 takes the values as parameters, so there is one
# level of quoting in total and the same script can be tested on its own.
#
# Usage:
#   powershell -File start_model.ps1 -Exe <server.exe> -Model <gguf> -Port 8591 `
#       -Extra "--n-gpu-layers 99 --ctx-size 2048 --threads 8" -Log <file> -PidFile <file>
#
# Prints the child pid on stdout (nothing else), so the caller can capture it. Writes
# the same pid to -PidFile so a later stop does not have to guess from netstat.

param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$Model,
    [Parameter(Mandatory = $true)][int]$Port,
    [string]$Extra = "",
    [Parameter(Mandatory = $true)][string]$Log,
    [string]$PidFile = ""
)

$ErrorActionPreference = 'Stop'

$logDir = Split-Path -Parent $Log
if ($logDir -and -not (Test-Path $logDir)) { New-Item -ItemType Directory -Path $logDir -Force | Out-Null }

# Start-Process cannot redirect stdout and stderr to the SAME file, and the first version
# therefore wrapped the server in cmd.exe — which made the RECORDED PID the wrapper's, not the
# server's. MEASURED 2026-11-13: run\model_8596.pid held 28424 while the real server was 23792
# with parent 28424; stopping the recorded pid killed the wrapper and left an 8.2 GB orphan
# holding the GPU. A stop path that reports success while the service keeps running is the G91
# class of defect, so the server is now started DIRECTLY and the log is split in two files
# (<Log> for stdout, <Log>.err for stderr). Both are reported when the server dies early.
$ErrLog = "$Log.err"

# MEASURED DEFECT 2026-11-13: Start-Process joins -ArgumentList with spaces and does not quote, so
# "E:\Hugginface models\X.gguf" arrived as two arguments and the server died with
# "invalid argument: models\X.gguf". Every token containing whitespace is now quoted, and -Extra is
# tokenised with quote awareness so a --lora path with spaces survives too.
$argList = @('--model', $Model, '--host', '127.0.0.1', '--port', "$Port")
if ($Extra.Trim()) {
    $argList += ([regex]::Matches($Extra, '"[^"]*"|\S+') | ForEach-Object { $_.Value.Trim('"') })
}
$argLine = ($argList | ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }) -join ' '

$p = Start-Process -FilePath $Exe -ArgumentList $argLine `
    -RedirectStandardOutput $Log -RedirectStandardError $ErrLog `
    -WindowStyle Hidden -PassThru

# A server that dies on a bad flag exits in under a second. Say so here, in its own words,
# instead of returning a pid for a process that is already gone.
Start-Sleep -Seconds 3
if ($p.HasExited) {
    Write-Output "EXITED $($p.ExitCode)"
    foreach ($f in @($ErrLog, $Log)) {
        if (Test-Path $f) { [Console]::Error.WriteLine((Get-Content -Tail 6 $f) -join "`n") }
    }
    exit 1
}
if ($PidFile) { Set-Content -Path $PidFile -Value $p.Id -Encoding ascii }
Write-Output $p.Id
