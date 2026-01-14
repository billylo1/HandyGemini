# PowerShell script to build HandyGemini for Windows
# Usage: .\scripts\build-windows.ps1

$ErrorActionPreference = "Stop"

Write-Host "Building HandyGemini for Windows..." -ForegroundColor Cyan
Write-Host ""

# Check if bun is available
if (-not (Get-Command bun -ErrorAction SilentlyContinue)) {
    Write-Host "Error: bun not found. Please install bun or add it to your PATH." -ForegroundColor Red
    exit 1
}

# Check if Rust Windows target is installed
Write-Host "Checking Rust Windows target..."
$installedTargets = rustup target list --installed
if ($installedTargets -notmatch "x86_64-pc-windows-msvc") {
    Write-Host "Installing x86_64-pc-windows-msvc target..."
    rustup target add x86_64-pc-windows-msvc
}

Write-Host "✓ Rust target ready" -ForegroundColor Green
Write-Host ""

# Load environment variables from .env file if it exists
if (Test-Path ".env") {
    Get-Content ".env" | ForEach-Object {
        if ($_ -match '^([^#][^=]+)=(.*)$') {
            $name = $matches[1].Trim()
            $value = $matches[2].Trim()
            [Environment]::SetEnvironmentVariable($name, $value, "Process")
        }
    }
}

Write-Host "Building Windows installer..." -ForegroundColor Cyan
Write-Host ""

# Build for Windows
# Tauri will create both MSI and NSIS (Setup.exe) installers
bun run tauri build --target x86_64-pc-windows-msvc

Write-Host ""
Write-Host "✓ Windows build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "The installers should be located at:" -ForegroundColor Cyan
Write-Host "  src-tauri\target\x86_64-pc-windows-msvc\release\bundle\msi\HandyGemini_*.msi"
Write-Host "  src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\HandyGemini_*.exe"
Write-Host ""
Write-Host "Note: Windows code signing requires Azure Trusted Signing credentials." -ForegroundColor Yellow
Write-Host "The signCommand in tauri.conf.json is configured for the base Handy project." -ForegroundColor Yellow
Write-Host "You may need to update it with your own signing credentials." -ForegroundColor Yellow
