#!/bin/bash

# Script to build HandyGemini for Linux
# Usage: ./scripts/build-linux.sh [bundle-type]
# Bundle types: deb, appimage, rpm, or all (default: all)

set -e

BUNDLE_TYPE="${1:-all}"

echo "Building HandyGemini for Linux..."
echo ""

# Check if we're on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "Warning: This script is designed for Linux. You're running on: $OSTYPE"
    echo "For macOS/Windows, use GitHub Actions to build Linux binaries."
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

# Check if Rust Linux target is installed
echo "Checking Rust Linux target..."
if ! rustup target list --installed | grep -q "x86_64-unknown-linux-gnu"; then
    echo "Installing x86_64-unknown-linux-gnu target..."
    rustup target add x86_64-unknown-linux-gnu
fi

echo "✓ Rust target ready"
echo ""

# Check for required Linux dependencies
echo "Checking Linux build dependencies..."
MISSING_DEPS=()

if ! command -v pkg-config &> /dev/null; then
    MISSING_DEPS+=("pkg-config")
fi

if ! pkg-config --exists gtk+-3.0 2>/dev/null; then
    MISSING_DEPS+=("libgtk-3-dev")
fi

if ! pkg-config --exists webkit2gtk-4.1 2>/dev/null; then
    MISSING_DEPS+=("libwebkit2gtk-4.1-dev")
fi

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo "Warning: Missing dependencies: ${MISSING_DEPS[*]}"
    echo "Install with:"
    echo "  Ubuntu/Debian: sudo apt install ${MISSING_DEPS[*]} libasound2-dev libssl-dev libvulkan-dev"
    echo "  Fedora/RHEL: sudo dnf install gtk3-devel webkit2gtk4.1-devel alsa-lib-devel openssl-devel vulkan-devel"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo ""

# Load environment variables from .env file if it exists
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Set environment variables for Linux build
export WHISPER_NO_AVX="${WHISPER_NO_AVX:-ON}"
export WHISPER_NO_AVX2="${WHISPER_NO_AVX2:-ON}"

echo "Building Linux packages..."
echo "Bundle type: $BUNDLE_TYPE"
echo ""

# Build for Linux
case "$BUNDLE_TYPE" in
    deb)
        bun run tauri build --target x86_64-unknown-linux-gnu --bundles deb
        ;;
    appimage)
        bun run tauri build --target x86_64-unknown-linux-gnu --bundles appimage
        ;;
    rpm)
        bun run tauri build --target x86_64-unknown-linux-gnu --bundles rpm
        ;;
    all)
        bun run tauri build --target x86_64-unknown-linux-gnu --bundles deb,appimage,rpm
        ;;
    *)
        echo "Error: Invalid bundle type: $BUNDLE_TYPE"
        echo "Valid types: deb, appimage, rpm, all"
        exit 1
        ;;
esac

echo ""
echo "✓ Linux build complete!"
echo ""
echo "The packages should be located at:"
echo "  src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/deb/HandyGemini_*.deb"
echo "  src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/appimage/HandyGemini_*.AppImage"
echo "  src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/rpm/HandyGemini_*.rpm"
echo ""
echo "Note: Linux builds may require GPG signing for package distribution."
echo "Configure GPG signing in tauri.conf.json if needed."
