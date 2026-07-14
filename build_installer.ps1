Write-Host "🐾 Building Tomodachi Standalone Installer..." -ForegroundColor Cyan

Write-Host "1. Building Daemon and Client..." -ForegroundColor Yellow
cargo build --release -p tomodachi-daemon -p tomodachi-client

if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to build daemon and client!" -ForegroundColor Red
    exit 1
}

Write-Host "2. Building Installer..." -ForegroundColor Yellow
# The installer statically includes the binaries we just built
cargo build --release -p tomodachi-installer

if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to build installer!" -ForegroundColor Red
    exit 1
}

$installerPath = Join-Path $PWD "target\release\tomodachi-installer.exe"
$outputPath = Join-Path $PWD "tomodachi-setup.exe"

Copy-Item $installerPath $outputPath -Force

Write-Host "✅ Done!" -ForegroundColor Green
Write-Host "Your standalone installer is ready at: $outputPath" -ForegroundColor Cyan
Write-Host "You can share this single .exe file with anyone, they don't need Rust!" -ForegroundColor Cyan
