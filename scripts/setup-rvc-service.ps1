$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$runtime = Join-Path $root "rvc-runtime"
$python = Join-Path $runtime ".venv\Scripts\python.exe"
if (-not (Test-Path -LiteralPath $python)) { python -m venv (Join-Path $runtime ".venv") }
& $python -m pip install --upgrade pip
& $python -m pip install -r (Join-Path $runtime "requirements.txt")
Write-Host "RVC profile service ready. Connect rvc_backend.py to your installed RVC v2 runtime."
