# macOS Notarization Setup Guide

This guide explains how to set up GitHub Actions secrets for macOS build notarization in HandyGemini.

## Required Secrets

To enable notarization in the macOS build workflow, you need to set up the following GitHub repository secrets:

### 1. `APPLE_ID`
- **Description**: Your Apple ID email address
- **Example**: `your-email@example.com`
- **How to get it**: Your Apple ID email (the one you use to sign in to appleid.apple.com)

### 2. `APPLE_PASSWORD`
- **Description**: App-specific password (NOT your regular Apple ID password)
- **Example**: `abcd-efgh-ijkl-mnop`
- **How to create**:
  1. Go to https://appleid.apple.com
  2. Sign in with your Apple ID
  3. Navigate to **Sign-In and Security** > **App-Specific Passwords**
  4. Click **Generate an app-specific password**
  5. Give it a label (e.g., "GitHub Actions Notarization")
  6. Copy the generated password (format: `xxxx-xxxx-xxxx-xxxx`)
  7. **Important**: You can only see this password once, so save it immediately

### 3. `APPLE_TEAM_ID`
- **Description**: Your Apple Developer Team ID
- **Example**: `X5J5T5UT6J`
- **How to find it**:
  1. Go to https://developer.apple.com/account
  2. Sign in with your Apple ID
  3. Your Team ID is displayed at the top right of the page (under your name)
  4. It's a 10-character alphanumeric string

### 4. `APPLE_CERTIFICATE` (Optional)
- **Description**: Base64-encoded Developer ID Application certificate (.p12 file)
- **When needed**: Only if you need to import a certificate that's not already in the GitHub Actions runner's keychain
- **How to create**:
  1. Export your Developer ID Application certificate from Keychain Access:
     - Open Keychain Access
     - Find "Developer ID Application: Your Name (TEAM_ID)"
     - Right-click > Export
     - Save as .p12 file
     - Set a password for the export
  2. Encode to Base64:
     ```bash
     base64 -i YourCertificate.p12 | pbcopy
     ```
  3. The encoded string is what you paste into the secret

### 5. `APPLE_CERTIFICATE_PASSWORD` (Optional)
- **Description**: Password used when exporting the .p12 certificate
- **When needed**: Only if you set `APPLE_CERTIFICATE`
- **Example**: The password you entered when exporting the certificate

### 6. `TAURI_SIGNING_PRIVATE_KEY` (Optional)
- **Description**: Private key for Tauri updater signing
- **When needed**: Only if you want to generate updater artifacts (for auto-updates)
- **How to generate**:
  ```bash
  # Generate a new key pair
  openssl genrsa -out private_key.pem 2048
  openssl rsa -in private_key.pem -pubout -out public_key.pem
  
  # The private key content goes into the secret
  cat private_key.pem
  ```
- **Note**: The public key should match the `pubkey` in `src-tauri/tauri.conf.json`

## Setting Up Secrets in GitHub

### Method 1: Via GitHub Web Interface

1. Go to your repository: https://github.com/billylo1/HandyGemini
2. Click **Settings** (top menu)
3. In the left sidebar, click **Secrets and variables** > **Actions**
4. Click **New repository secret**
5. Enter the secret name (e.g., `APPLE_ID`)
6. Enter the secret value
7. Click **Add secret**
8. Repeat for all required secrets

### Method 2: Via GitHub CLI

```bash
# Set each secret
gh secret set APPLE_ID --body "your-email@example.com"
gh secret set APPLE_PASSWORD --body "abcd-efgh-ijkl-mnop"
gh secret set APPLE_TEAM_ID --body "X5J5T5UT6J"

# Optional secrets
gh secret set APPLE_CERTIFICATE --body "$(base64 -i YourCertificate.p12)"
gh secret set APPLE_CERTIFICATE_PASSWORD --body "your-cert-password"
gh secret set TAURI_SIGNING_PRIVATE_KEY --body "$(cat private_key.pem)"
```

## Verifying Your Setup

1. **Check your secrets are set**:
   ```bash
   gh secret list
   ```

2. **Test the workflow**:
   - Go to Actions > Build macOS
   - Click "Run workflow"
   - Enable "Enable notarization"
   - Run the workflow
   - Check the logs to verify notarization succeeded

## Troubleshooting

### "No Developer ID Application certificate found"
- **Solution**: The GitHub Actions runner should have certificates available, but if not, you may need to:
  1. Set up `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD` to import your certificate
  2. Or ensure your certificate is available in the runner's keychain

### "Invalid credentials"
- **Solution**: 
  - Double-check your `APPLE_ID` and `APPLE_PASSWORD`
  - Make sure you're using an app-specific password, not your regular Apple ID password
  - Verify your Apple ID has access to the Developer Program

### "Team ID mismatch"
- **Solution**: 
  - Verify `APPLE_TEAM_ID` matches your actual Team ID
  - Check that your certificate matches the Team ID

### Notarization fails
- **Solution**:
  - Check the workflow logs for specific error messages
  - Verify all required secrets are set correctly
  - Ensure your Apple Developer account is active and in good standing

## Security Best Practices

1. **Never commit secrets to the repository** - Always use GitHub Secrets
2. **Use app-specific passwords** - Never use your main Apple ID password
3. **Rotate secrets periodically** - Especially if you suspect they've been compromised
4. **Limit access** - Only grant repository access to trusted collaborators
5. **Monitor usage** - Check GitHub Actions logs regularly for any suspicious activity

## Additional Resources

- [Apple Developer Documentation](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Tauri Notarization Guide](https://tauri.app/v1/guides/distribution/notarization)
- [GitHub Actions Secrets Documentation](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
