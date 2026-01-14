# Google OAuth Setup Guide

This guide will help you set up Google OAuth credentials to enable Gemini AI features in HandyGemini.

## Prerequisites

- A Google account
- Access to [Google Cloud Console](https://console.cloud.google.com/)

## Step-by-Step Instructions

### 1. Create a Google Cloud Project

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Click the project dropdown at the top of the page
3. Click **"New Project"**
4. Enter a project name (e.g., "HandyGemini")
5. Click **"Create"**
6. Wait for the project to be created, then select it from the project dropdown

### 2. Enable Required APIs

1. In the Google Cloud Console, navigate to **"APIs & Services" > "Library"**
2. Search for **"Gemini API"** and click on it
3. Click **"Enable"** to enable the Gemini API
4. (Optional) Also enable **"Google+ API"** if you want to access user profile information

### 3. Configure OAuth Consent Screen

1. Navigate to **"APIs & Services" > "OAuth consent screen"**
2. Select **"External"** user type (unless you have a Google Workspace account)
3. Click **"Create"**
4. Fill in the required information:
   - **App name**: HandyGemini (or your preferred name)
   - **User support email**: Your email address
   - **Developer contact information**: Your email address
5. Click **"Save and Continue"**
6. On the **Scopes** page:
   - Click **"Add or Remove Scopes"**
   - Add the following scopes:
     - `openid` (OpenID Connect)
     - `https://www.googleapis.com/auth/userinfo.email` (User email address)
     - `https://www.googleapis.com/auth/userinfo.profile` (User profile information)
   - **Note**: Gemini API uses API keys (not OAuth) for authentication. See the settings UI to configure your API key.
   - Click **"Update"**, then **"Save and Continue"**
7. On the **Test users** page (if External):
   - Add your Google account email as a test user
   - Click **"Save and Continue"**
8. Review and click **"Back to Dashboard"**

### 4. Create OAuth 2.0 Credentials

1. Navigate to **"APIs & Services" > "Credentials"**
2. Click **"+ CREATE CREDENTIALS"** at the top
3. Select **"OAuth client ID"**
4. If prompted, select **"Desktop app"** as the application type
5. Fill in the form:
   - **Name**: HandyGemini Desktop (or your preferred name)
   - **Application type**: Desktop app
6. Click **"Create"**
7. **Important**: Copy both the **Client ID** and **Client secret** - you'll need these for the application
8. Click **"OK"**

### 5. Configure Redirect URI

The OAuth flow uses `http://localhost:8080` as the redirect URI. This is already configured in the code, but you should verify:

1. In the **Credentials** page, find your OAuth 2.0 Client ID
2. Click the edit icon (pencil) next to it
3. Under **"Authorized redirect URIs"**, ensure `http://localhost:8080` is listed
4. If not present, click **"+ ADD URI"** and add: `http://localhost:8080`
5. Click **"Save"**

### 6. Add Credentials to HandyGemini

The application supports reading credentials from environment variables (recommended) or you can update the default values directly.

#### Option A: Environment Variables (Recommended)

The application automatically reads `GOOGLE_OAUTH_CLIENT_ID` and `GOOGLE_OAUTH_CLIENT_SECRET` from environment variables.

**For Development:**
1. Create a `.env` file in the project root (if it doesn't exist)
2. Add the following:
   ```bash
   GOOGLE_OAUTH_CLIENT_ID=your-client-id-here.apps.googleusercontent.com
   GOOGLE_OAUTH_CLIENT_SECRET=your-client-secret-here
   ```
3. When running `bun run tauri dev`, the environment variables will be loaded automatically

**For Production/Build:**
Set the environment variables before building:
```bash
export GOOGLE_OAUTH_CLIENT_ID="your-client-id-here.apps.googleusercontent.com"
export GOOGLE_OAUTH_CLIENT_SECRET="your-client-secret-here"
bun run tauri build
```

#### Option B: Update Default Values (For Testing Only)

⚠️ **Warning**: This is not recommended for production as it exposes credentials in source code.

1. Open `src-tauri/src/google_auth.rs`
2. Find these lines:
   ```rust
   const DEFAULT_CLIENT_ID: &str = "YOUR_CLIENT_ID_HERE";
   const DEFAULT_CLIENT_SECRET: &str = "YOUR_CLIENT_SECRET_HERE";
   ```
3. Replace with your actual credentials:
   ```rust
   const DEFAULT_CLIENT_ID: &str = "your-actual-client-id.apps.googleusercontent.com";
   const DEFAULT_CLIENT_SECRET: &str = "your-actual-client-secret";
   ```

#### Option C: Configuration File (Recommended for Production)

1. Create a configuration file (e.g., `config.json`) in the app data directory
2. Store credentials there and read them at runtime
3. Add the config file to `.gitignore` to prevent committing credentials

### 7. Test the Setup

1. Build and run the application:
   ```bash
   bun run tauri dev
   ```
2. Navigate to **Settings > Post Process**
3. Scroll to the **Google Login** section
4. Click **"Sign in with Google"**
5. A browser window should open asking for permission
6. After granting permission, you should be redirected back and see your email displayed

## Troubleshooting

### "Invalid client" error
- Verify your Client ID and Client Secret are correct
- Ensure you copied the entire Client ID (it should end with `.apps.googleusercontent.com`)

### "Redirect URI mismatch" error
- Verify `http://localhost:8080` is added to Authorized redirect URIs in Google Cloud Console
- Ensure no trailing slashes or extra characters

### "Access blocked" error
- If using External app type, ensure your email is added as a test user
- The app may need to go through Google's verification process for production use

### Port 8080 already in use
- The OAuth callback server uses port 8080
- If another application is using this port, you can change it in:
  - `src-tauri/src/google_auth.rs` - Update `REDIRECT_PORT` and `REDIRECT_URI`
  - Google Cloud Console - Update the redirect URI to match

### Token refresh fails
- Ensure the OAuth consent screen has the correct scopes enabled
- Check that the refresh token is being stored correctly

## Security Best Practices

1. **Never commit credentials to version control**
   - Add `.env` to `.gitignore`
   - Use environment variables or secure configuration files

2. **Use separate credentials for development and production**
   - Create different OAuth clients for each environment

3. **Rotate credentials if compromised**
   - Immediately revoke and recreate credentials if exposed

4. **Limit OAuth consent screen to necessary scopes only**
   - Only request the minimum permissions needed

## Production Deployment

For production deployment:

1. **Publish your OAuth consent screen**:
   - Complete all required fields in the OAuth consent screen
   - Submit for Google's verification (required for External apps with sensitive scopes)
   - This process can take several days to weeks

2. **Use secure credential storage**:
   - Store credentials in environment variables or secure key management systems
   - Never hardcode credentials in source code

3. **Monitor usage**:
   - Set up quotas and alerts in Google Cloud Console
   - Monitor API usage to prevent unexpected costs

## Additional Resources

- [Google OAuth 2.0 Documentation](https://developers.google.com/identity/protocols/oauth2)
- [Gemini API Documentation](https://ai.google.dev/docs)
- [Google Cloud Console](https://console.cloud.google.com/)
