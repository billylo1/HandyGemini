use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const REDIRECT_PORT: u16 = 8080;
const REDIRECT_URI: &str = "http://localhost:8080";

// OAuth2 client ID and secret for Gemini API
// These should be configured in Google Cloud Console
// Priority: Environment variables > Default placeholders
// Set GOOGLE_OAUTH_CLIENT_ID and GOOGLE_OAUTH_CLIENT_SECRET environment variables
// or update these defaults (not recommended for production)
fn get_client_id() -> String {
    std::env::var("GOOGLE_OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| "YOUR_CLIENT_ID_HERE".to_string())
}

fn get_client_secret() -> String {
    std::env::var("GOOGLE_OAUTH_CLIENT_SECRET")
        .unwrap_or_else(|_| "YOUR_CLIENT_SECRET_HERE".to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoogleAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>, // Unix timestamp
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoogleUserInfo {
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
}

/// Get stored Google auth tokens from settings
pub fn get_google_tokens(app: &AppHandle) -> Option<GoogleAuthTokens> {
    let store = app.store("settings.json").ok()?;
    let tokens_value = store.get("google_auth_tokens")?.clone();
    
    serde_json::from_value::<GoogleAuthTokens>(tokens_value).ok()
}

/// Save Google auth tokens to settings
pub fn save_google_tokens(app: &AppHandle, tokens: &GoogleAuthTokens) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let tokens_value = serde_json::to_value(tokens).map_err(|e| e.to_string())?;
    store.set("google_auth_tokens", tokens_value);
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Clear stored Google auth tokens
pub fn clear_google_tokens(app: &AppHandle) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.delete("google_auth_tokens");
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Check if we have valid Google auth tokens
pub fn has_valid_google_tokens(app: &AppHandle) -> bool {
    if let Some(tokens) = get_google_tokens(app) {
        // Check if token is expired
        if let Some(expires_at) = tokens.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if now < expires_at {
                return true;
            }
        } else {
            // If no expiration, assume valid (refresh token available)
            return !tokens.access_token.is_empty();
        }
    }
    false
}

/// Create OAuth2 client for Google
fn create_google_oauth_client(
    client_id: &str,
    client_secret: &str,
) -> Result<BasicClient, String> {
    let client_id = ClientId::new(client_id.to_string());
    let client_secret = ClientSecret::new(client_secret.to_string());
    let auth_url = AuthUrl::new(GOOGLE_AUTH_URL.to_string())
        .map_err(|e| format!("Invalid auth URL: {}", e))?;
    let token_url = TokenUrl::new(GOOGLE_TOKEN_URL.to_string())
        .map_err(|e| format!("Invalid token URL: {}", e))?;
    let redirect_url = RedirectUrl::new(REDIRECT_URI.to_string())
        .map_err(|e| format!("Invalid redirect URL: {}", e))?;

    Ok(BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url))
}

/// Start OAuth2 flow - returns the authorization URL
pub async fn start_google_oauth_flow(
    app: &AppHandle,
    client_id: Option<String>,
    client_secret: Option<String>,
) -> Result<String, String> {
    let client_id = client_id.unwrap_or_else(get_client_id);
    let client_secret = client_secret.unwrap_or_else(get_client_secret);

    if client_id == "YOUR_CLIENT_ID_HERE" || client_secret == "YOUR_CLIENT_SECRET_HERE" {
        return Err("Google OAuth client ID and secret must be configured. See GOOGLE_OAUTH_SETUP.md for instructions.".to_string());
    }

    let client = create_google_oauth_client(&client_id, &client_secret)?;

    // Generate PKCE verifier and challenge
    let (pkce_challenge, pkce_verifier) = oauth2::PkceCodeChallenge::new_random_sha256();
    
    // Store PKCE verifier for later use
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set(
        "google_oauth_pkce_verifier",
        serde_json::Value::String(pkce_verifier.secret().clone()),
    );
    store.save().map_err(|e| e.to_string())?;

    // Request scopes for user info (Gemini API doesn't require a specific scope)
    // The access token from OAuth can be used directly with Gemini API
    let scopes = vec![
        Scope::new("openid".to_string()),
        Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()),
        Scope::new("https://www.googleapis.com/auth/userinfo.profile".to_string()),
    ];

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge)
        .add_scopes(scopes)
        .url();

    Ok(auth_url.to_string())
}

/// Handle OAuth2 callback and exchange authorization code for tokens
pub async fn handle_google_oauth_callback(
    app: &AppHandle,
    code: String,
    client_id: Option<String>,
    client_secret: Option<String>,
) -> Result<GoogleAuthTokens, String> {
    let client_id = client_id.unwrap_or_else(get_client_id);
    let client_secret = client_secret.unwrap_or_else(get_client_secret);

    if client_id == "YOUR_CLIENT_ID_HERE" || client_secret == "YOUR_CLIENT_SECRET_HERE" {
        return Err("Google OAuth client ID and secret must be configured. See GOOGLE_OAUTH_SETUP.md for instructions.".to_string());
    }

    let client = create_google_oauth_client(&client_id, &client_secret)?;

    // Retrieve stored PKCE verifier
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let pkce_verifier_value = store
        .get("google_oauth_pkce_verifier")
        .ok_or("PKCE verifier not found")?;
    let pkce_verifier_str = pkce_verifier_value
        .as_str()
        .ok_or("PKCE verifier is not a string")?;
    let pkce_verifier = oauth2::PkceCodeVerifier::new(pkce_verifier_str.to_string());

    // Exchange authorization code for tokens
    let auth_code = AuthorizationCode::new(code);
    let token_result = client
        .exchange_code(auth_code)
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .map_err(|e| format!("Token exchange failed: {}", e))?;

    // Calculate expiration time
    let expires_at = token_result
        .expires_in()
        .map(|duration| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + duration.as_secs()
        });

    let tokens = GoogleAuthTokens {
        access_token: token_result.access_token().secret().clone(),
        refresh_token: token_result.refresh_token().map(|rt| rt.secret().clone()),
        expires_at,
    };

    // Save tokens
    save_google_tokens(app, &tokens)?;

    // Clear PKCE verifier
    store.delete("google_oauth_pkce_verifier");
    store.save().map_err(|e| e.to_string())?;

    Ok(tokens)
}

/// Refresh Google access token using refresh token
pub async fn refresh_google_token(
    app: &AppHandle,
    client_id: Option<String>,
    client_secret: Option<String>,
) -> Result<GoogleAuthTokens, String> {
    let tokens = get_google_tokens(app)
        .ok_or("No stored tokens found")?;
    
    let refresh_token = tokens
        .refresh_token
        .as_ref()
        .ok_or("No refresh token available")?;

    let client_id = client_id.unwrap_or_else(get_client_id);
    let client_secret = client_secret.unwrap_or_else(get_client_secret);

    if client_id == "YOUR_CLIENT_ID_HERE" || client_secret == "YOUR_CLIENT_SECRET_HERE" {
        return Err("Google OAuth client ID and secret must be configured. See GOOGLE_OAUTH_SETUP.md for instructions.".to_string());
    }

    let client = create_google_oauth_client(&client_id, &client_secret)?;
    let refresh_token = oauth2::RefreshToken::new(refresh_token.clone());

    let token_result = client
        .exchange_refresh_token(&refresh_token)
        .request_async(async_http_client)
        .await
        .map_err(|e| format!("Token refresh failed: {}", e))?;

    let expires_at = token_result
        .expires_in()
        .map(|duration| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + duration.as_secs()
        });

    let new_tokens = GoogleAuthTokens {
        access_token: token_result.access_token().secret().clone(),
        refresh_token: token_result
            .refresh_token()
            .map(|rt| rt.secret().clone())
            .or(tokens.refresh_token),
        expires_at,
    };

    save_google_tokens(app, &new_tokens)?;
    Ok(new_tokens)
}

/// Get current access token, refreshing if necessary
pub async fn get_valid_access_token(
    app: &AppHandle,
    client_id: Option<String>,
    client_secret: Option<String>,
) -> Result<String, String> {
    let tokens = get_google_tokens(app)
        .ok_or("No stored tokens found")?;

    // Check if token is expired
    let needs_refresh = if let Some(expires_at) = tokens.expires_at {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= expires_at
    } else {
        false
    };

    if needs_refresh {
        let refreshed = refresh_google_token(app, client_id, client_secret).await?;
        Ok(refreshed.access_token)
    } else {
        Ok(tokens.access_token)
    }
}

/// Get user info from Google
pub async fn get_google_user_info(app: &AppHandle) -> Result<GoogleUserInfo, String> {
    let access_token = get_valid_access_token(app, None, None).await?;

    let client = reqwest::Client::new();
    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user info: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to get user info: {}", response.status()));
    }

    let user_info: GoogleUserInfo = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse user info: {}", e))?;

    Ok(user_info)
}
