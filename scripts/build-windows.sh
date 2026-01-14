#!/bin/bash

# Script to build HandyGemini for Windows
# Note: This script is designed to be run on a Windows machine or via GitHub Actions
# For local Windows builds, use the PowerShell version: scripts/build-windows.ps1
# Usage: ./scripts/build-windows.sh

set -e

echo "Building HandyGemini for Windows..."
echo ""
echo "Note: Windows builds require:"
echo "  - Windows machine with Visual Studio Build Tools"
echo "  - Rust toolchain with x86_64-pc-windows-msvc target"
echo "  - Or use GitHub Actions workflow"
echo ""

# Check if we're on Windows (Git Bash or WSL)
if [[ "$OSTYPE" != "msys" && "$OSTYPE" != "win32" && "$OSTYPE" != "cygwin" ]]; then
    echo "Warning: This script is designed for Windows. You're running on: $OSTYPE"
    echo "For macOS/Linux, use GitHub Actions to build Windows binaries."
    echo ""
    echo "To build via GitHub Actions:"
    echo "  1. Push your changes to GitHub"
    echo "  2. Go to Actions > Release workflow"
    echo "  3. Run workflow manually (workflow_dispatch)"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Add common bun installation paths to PATH
export PATH="$HOME/.bun/bin:/usr/local/bin:$PATH"

# Find bun executable
if ! command -v bun &> /dev/null; then
    if [ -f "$HOME/.bun/bin/bun" ]; then
        export PATH="$HOME/.bun/bin:$PATH"
    else
        echo "Error: bun not found. Please install bun or add it to your PATH."
        exit 1
    fi
fi

# Check if Rust Windows target is installed
echo "Checking Rust Windows target..."
if ! rustup target list --installed | grep -q "x86_64-pc-windows-msvc"; then
    echo "Installing x86_64-pc-windows-msvc target..."
    rustup target add x86_64-pc-windows-msvc
fi

echo "✓ Rust target ready"
echo ""

# Load environment variables from .env file if it exists
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

echo "Building Windows installer..."
echo ""

# Build for Windows
# Tauri will create both MSI and NSIS (Setup.exe) installers
bun run tauri build --target x86_64-pc-windows-msvc

echo ""
echo "✓ Windows build complete!"
echo ""
echo "The installers should be located at:"
echo "  src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/HandyGemini_*.msi"
echo "  src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/HandyGemini_*.exe"
echo ""
echo "Note: Windows code signing requires Azure Trusted Signing credentials."
echo "The signCommand in tauri.conf.json is configured for the base Handy project."
echo "You may need to update it with your own signing credentials."
