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

# Set defaults if not in .env
export APPLE_ID="${APPLE_ID:-billy@evergreen-labs.org}"
export APPLE_TEAM_ID="${APPLE_TEAM_ID:-X5J5T5UT6J}"

# Check if app-specific password is set
if [ -z "$APPLE_PASSWORD" ]; then
    echo "Error: APPLE_PASSWORD environment variable is not set"
    echo "Please set it with: export APPLE_PASSWORD='your-app-specific-password'"
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
