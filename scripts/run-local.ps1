$ErrorActionPreference = 'Stop'
Push-Location (Join-Path $PSScriptRoot '..')
try {
    npm run build --prefix apps/desktop
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    cargo build -p simulator-api-flow-script-worker
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    cargo run -p simulator-api-flow-server -- @args
    exit $LASTEXITCODE
} finally {
    Pop-Location
}
