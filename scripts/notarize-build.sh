#!/bin/bash

# Script to build and notarize HandyGemini with Apple ID credentials
# Usage: ./scripts/notarize-build.sh

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
    echo "Error: No Developer ID Application certificate found in keychain."
    echo "Please install a Developer ID Application certificate or run: ./scripts/setup-developer-id.sh"
    exit 1
fi
echo ""

# Check if app-specific password is set
if [ -z "$APPLE_PASSWORD" ]; then
    echo "Error: APPLE_PASSWORD environment variable is not set"
    echo "Please set it in your .env file or with: export APPLE_PASSWORD='your-app-specific-password'"
    echo ""
    echo "To create an app-specific password:"
    echo "1. Go to https://appleid.apple.com"
    echo "2. Sign in with your Apple ID"
    echo "3. Go to 'Sign-In and Security' > 'App-Specific Passwords'"
    echo "4. Click 'Generate an app-specific password'"
    echo "5. Copy the password and use it for APPLE_PASSWORD"
    echo ""
    echo "Then run this script again."
    exit 1
fi

echo "Building and notarizing HandyGemini..."
echo "Apple ID: $APPLE_ID"
echo "Team ID: $APPLE_TEAM_ID"
echo ""

# Build with notarization
bun run tauri build

echo ""
echo "Build complete! Check the output above for notarization status."

# Restore original config if backup exists
if [ -f "$BACKUP_CONFIG" ]; then
    mv "$BACKUP_CONFIG" "$CONFIG_FILE"
    echo "Restored original tauri.conf.json"
fi
