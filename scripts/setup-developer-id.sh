#!/bin/bash

# Script to find and configure Developer ID Application certificate
# Run this after installing the Developer ID certificate from Apple Developer portal

set -e

echo "Searching for Developer ID Application certificate..."

# Find Developer ID Application certificate
DEVELOPER_ID=$(security find-identity -v -p codesigning | grep "Developer ID Application" | head -1 | sed 's/.*"\(.*\)".*/\1/')

if [ -z "$DEVELOPER_ID" ]; then
    echo "Error: Developer ID Application certificate not found."
    echo ""
    echo "Please:"
    echo "1. Go to https://developer.apple.com/account/resources/certificates/list"
    echo "2. Click '+' to create a new certificate"
    echo "3. Select 'Developer ID Application' under Software"
    echo "4. Follow the prompts to create and download the certificate"
    echo "5. Double-click the downloaded .cer file to install it in Keychain"
    echo "6. Run this script again"
    exit 1
fi

echo "Found Developer ID certificate: $DEVELOPER_ID"
echo ""

# Update tauri.conf.json
CONFIG_FILE="src-tauri/tauri.conf.json"
BACKUP_FILE="${CONFIG_FILE}.backup"

# Create backup
cp "$CONFIG_FILE" "$BACKUP_FILE"
echo "Created backup: $BACKUP_FILE"

# Update signingIdentity (using jq if available, otherwise sed)
if command -v jq &> /dev/null; then
    jq '.bundle.macOS.signingIdentity = "'"$DEVELOPER_ID"'"' "$CONFIG_FILE" > "${CONFIG_FILE}.tmp" && mv "${CONFIG_FILE}.tmp" "$CONFIG_FILE"
    echo "Updated tauri.conf.json using jq"
else
    # Fallback to sed (less reliable but works)
    sed -i '' "s|\"signingIdentity\": \"-\"|\"signingIdentity\": \"$DEVELOPER_ID\"|g" "$CONFIG_FILE"
    echo "Updated tauri.conf.json using sed"
fi

echo ""
echo "✓ Configuration updated!"
echo "You can now rebuild with: bun run tauri build"
echo "Or use: ./scripts/notarize-build.sh"
