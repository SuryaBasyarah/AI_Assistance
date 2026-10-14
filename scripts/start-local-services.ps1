$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$sbv2 = Join-Path $root "TTS\sbv2\Style-Bert-VITS2"
$sbv2Python = Join-Path $sbv2 "venv\Scripts\python.exe"
$rvc = Join-Path $root "rvc-runtime"
$rvcPython = Join-Path $rvc ".venv\Scripts\python.exe"
if (-not (Test-Path -LiteralPath $sbv2Python)) { throw "SBV2 environment not found at $sbv2Python" }
if (-not (Test-Path -LiteralPath $rvcPython)) { throw "RVC service environment missing. Run: npm run setup:rvc" }
$env:PYTHONIOENCODING = "utf-8"
Start-Process -FilePath $sbv2Python -ArgumentList "server_fastapi.py", "--cpu" -WorkingDirectory $sbv2 -WindowStyle Hidden
Start-Process -FilePath $rvcPython -ArgumentList "rvc_server.py" -WorkingDirectory $rvc -WindowStyle Hidden
Write-Host "Started SBV2 on 127.0.0.1:5000 and RVC profile service on 127.0.0.1:5200."
