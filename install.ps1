Write-Host "🐾 Installing Tomodachi..." -ForegroundColor Cyan

# Check for Rust
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Cargo not found! Please install Rust (rustup.rs) first." -ForegroundColor Red
    exit 1
}

Write-Host "Building Tomodachi (release mode)..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "Installing shell hooks and startup scripts..." -ForegroundColor Yellow
$clientExe = Join-Path $PWD "target\release\tomodachi-client.exe"
& $clientExe install

Write-Host "Starting Tomodachi daemon..." -ForegroundColor Yellow
$daemonExe = Join-Path $PWD "target\release\tomodachi-daemon.exe"
Start-Process $daemonExe -WindowStyle Hidden

Write-Host "Done! Tomodachi is now running in your system tray." -ForegroundColor Green
Write-Host "Please restart your terminal to activate the shell hooks." -ForegroundColor Cyan
