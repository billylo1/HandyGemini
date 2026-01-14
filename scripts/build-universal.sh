#!/bin/bash

# Script to build HandyGemini as a macOS Universal Binary (ARM + Intel)
# Usage: ./scripts/build-universal.sh

set -e

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

echo "Building HandyGemini as Universal Binary (ARM64 + x86_64)..."
echo ""

# Check if Rust targets are installed
echo "Checking Rust targets..."
if ! rustup target list --installed | grep -q "aarch64-apple-darwin"; then
    echo "Installing aarch64-apple-darwin target..."
    rustup target add aarch64-apple-darwin
fi

if ! rustup target list --installed | grep -q "x86_64-apple-darwin"; then
    echo "Installing x86_64-apple-darwin target..."
    rustup target add x86_64-apple-darwin
fi

echo "✓ Rust targets ready"
echo ""

# Load environment variables from .env file if it exists
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Check if required environment variables are set
if [ -z "$APPLE_ID" ]; then
    echo "Error: APPLE_ID environment variable is not set"
    echo "Please set it in your .env file or with: export APPLE_ID='your-email@example.com'"
    exit 1
fi

if [ -z "$APPLE_TEAM_ID" ]; then
    echo "Error: APPLE_TEAM_ID environment variable is not set"
    echo "Please set it in your .env file or with: export APPLE_TEAM_ID='YOUR_TEAM_ID'"
    exit 1
fi

# Update signing identity in tauri.conf.json to match the team ID
CONFIG_FILE="src-tauri/tauri.conf.json"
BACKUP_CONFIG="${CONFIG_FILE}.build-backup"

# Create backup of original config
if [ ! -f "$BACKUP_CONFIG" ]; then
    cp "$CONFIG_FILE" "$BACKUP_CONFIG"
fi

# Find Developer ID certificate matching the team ID
echo "Finding Developer ID Application certificate for team $APPLE_TEAM_ID..."
DEVELOPER_ID=$(security find-identity -v -p codesigning | grep "Developer ID Application" | grep "$APPLE_TEAM_ID" | head -1 | sed 's/.*"\(.*\)".*/\1/')

if [ -z "$DEVELOPER_ID" ]; then
    # Fallback: find any Developer ID Application certificate
    echo "Warning: No certificate found for team $APPLE_TEAM_ID, trying to find any Developer ID Application certificate..."
    DEVELOPER_ID=$(security find-identity -v -p codesigning | grep "Developer ID Application" | head -1 | sed 's/.*"\(.*\)".*/\1/')
fi

if [ -n "$DEVELOPER_ID" ]; then
    # Update tauri.conf.json with the found certificate
    if command -v jq &> /dev/null; then
        jq '.bundle.macOS.signingIdentity = "'"$DEVELOPER_ID"'"' "$CONFIG_FILE" > "${CONFIG_FILE}.tmp" && mv "${CONFIG_FILE}.tmp" "$CONFIG_FILE"
        echo "Updated signingIdentity in tauri.conf.json: $DEVELOPER_ID"
    else
        # Fallback to sed
        sed -i '' "s|\"signingIdentity\": \".*\"|\"signingIdentity\": \"$DEVELOPER_ID\"|g" "$CONFIG_FILE"
        echo "Updated signingIdentity in tauri.conf.json using sed: $DEVELOPER_ID"
    fi
else
    echo "Warning: No Developer ID Application certificate found in keychain."
    echo "The build may fail if the signingIdentity in tauri.conf.json doesn't match your certificates."
    echo "You can manually set APPLE_SIGNING_IDENTITY or run: ./scripts/setup-developer-id.sh"
fi
echo ""

# Check if app-specific password is set (for notarization)
if [ -z "$APPLE_PASSWORD" ]; then
    echo "Warning: APPLE_PASSWORD not set. Build will proceed but notarization will be skipped."
    echo "Set APPLE_PASSWORD in .env file to enable notarization."
    echo ""
fi

echo "Building Universal Binary..."
echo "Apple ID: $APPLE_ID"
echo "Team ID: $APPLE_TEAM_ID"
echo ""

# Build with universal-apple-darwin target
# This will automatically build for both architectures and combine them
bun run tauri build --target universal-apple-darwin

echo ""
echo "✓ Universal Binary build complete!"
echo ""

# Restore original config if backup exists
if [ -f "$BACKUP_CONFIG" ]; then
    mv "$BACKUP_CONFIG" "$CONFIG_FILE"
    echo "Restored original tauri.conf.json"
    echo ""
fi

echo "The DMG should be located at:"
echo "  src-tauri/target/universal-apple-darwin/release/bundle/dmg/"
echo ""
echo "To verify the binary architecture, run:"
echo "  file src-tauri/target/universal-apple-darwin/release/bundle/macos/HandyGemini.app/Contents/MacOS/handy"
echo ""
echo "You should see both 'arm64' and 'x86_64' architectures listed."
echo ""
echo "Note: If you see a warning about TAURI_SIGNING_PRIVATE_KEY, it's optional"
echo "and only needed for updater signature verification. The build is still valid."
