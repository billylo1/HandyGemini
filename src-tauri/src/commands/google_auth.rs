use crate::google_auth;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Emitter};
use tauri_plugin_opener::OpenerExt;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct GoogleAuthStatus {
    pub is_authenticated: bool,
    pub email: Option<String>,
    pub name: Option<String>,
}

/// Start Google OAuth flow
/// Opens browser and starts a local HTTP server to handle the callback
#[tauri::command]
#[specta::specta]
pub async fn start_google_oauth(
    app: AppHandle,
    client_id: Option<String>,
    client_secret: Option<String>,
) -> Result<String, String> {
    debug!("Starting Google OAuth flow");

    // Start the authorization URL generation
    let auth_url = google_auth::start_google_oauth_flow(&app, client_id.clone(), client_secret.clone())
        .await?;

    // Start HTTP server for callback in background
    let app_clone = app.clone();
    let client_id_clone = client_id.clone();
    let client_secret_clone = client_secret.clone();

    tokio::spawn(async move {
        if let Err(e) = start_oauth_callback_server(app_clone, client_id_clone, client_secret_clone).await {
            error!("OAuth callback server error: {}", e);
        }
    });

    // Open browser with auth URL
    app.opener()
        .open_url(&auth_url, None::<String>)
        .map_err(|e| format!("Failed to open browser: {}", e))?;

    info!("Opened browser for Google OAuth. Waiting for callback...");

    // Return immediately - the callback server will emit events
    Ok("OAuth flow started. Please complete authentication in your browser.".to_string())
}

/// Start a simple HTTP server to handle OAuth callback
async fn start_oauth_callback_server(
    app: AppHandle,
    client_id: Option<String>,
    client_secret: Option<String>,
) -> Result<String, String> {
    use hyper::service::service_fn;
    use hyper_util::rt::TokioExecutor;
    use hyper_util::rt::TokioIo;
    use hyper_util::server::conn::auto::Builder;
    use http_body_util::Full;
    use hyper::body::Bytes;
    use hyper::{Request, Response, StatusCode};
    use std::convert::Infallible;
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .map_err(|e| format!("Failed to bind to port 8080: {}", e))?;

    info!("OAuth callback server started on http://localhost:8080");

    // Accept one connection
    let (stream, _) = listener
        .accept()
        .await
        .map_err(|e| format!("Failed to accept connection: {}", e))?;

    let io = TokioIo::new(stream);

    let service = service_fn(move |req: Request<hyper::body::Incoming>| {
        let app = app.clone();
        let client_id = client_id.clone();
        let client_secret = client_secret.clone();

        async move {
            let uri = req.uri().clone();
            let query = uri.query().unwrap_or("");

            // Parse query parameters
            let url = format!("http://localhost:8080/?{}", query);
            let parsed_url = match Url::parse(&url) {
                Ok(u) => u,
                Err(_) => {
                    return Ok::<_, Infallible>(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Full::new(Bytes::from("Invalid URL")))
                        .unwrap());
                }
            };

            if let Some(code) = parsed_url.query_pairs().find(|(k, _)| k == "code") {
                let code_value = code.1.to_string();
                
                // Exchange code for tokens in background
                let app_clone = app.clone();
                let client_id_clone = client_id.clone();
                let client_secret_clone = client_secret.clone();
                tokio::spawn(async move {
                    match google_auth::handle_google_oauth_callback(
                        &app_clone,
                        code_value,
                        client_id_clone,
                        client_secret_clone,
                    )
                    .await
                    {
                        Ok(_) => {
                            info!("Successfully authenticated with Google");
                            app_clone.emit("google-auth-success", ()).ok();
                        }
                        Err(e) => {
                            error!("Failed to exchange authorization code: {}", e);
                            app_clone.emit("google-auth-error", e).ok();
                        }
                    }
                });

                // Return success page
                let html = r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>Authentication Successful</title>
                        <style>
                            body {
                                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                                display: flex;
                                justify-content: center;
                                align-items: center;
                                height: 100vh;
                                margin: 0;
                                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                            }
                            .container {
                                background: white;
                                padding: 2rem;
                                border-radius: 10px;
                                box-shadow: 0 10px 25px rgba(0,0,0,0.2);
                                text-align: center;
                            }
                            h1 { color: #4CAF50; margin: 0 0 1rem 0; }
                            p { color: #666; }
                        </style>
                    </head>
                    <body>
                        <div class="container">
                            <h1>✓ Authentication Successful</h1>
                            <p>You can close this window and return to Handy.</p>
                        </div>
                    </body>
                    </html>
                "#;
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Full::new(Bytes::from(html)))
                    .unwrap())
            } else if let Some(error) = parsed_url.query_pairs().find(|(k, _)| k == "error") {
                let error_value = error.1.to_string();
                error!("OAuth error: {}", error_value);
                app.emit("google-auth-error", error_value.clone()).ok();
                
                let html = format!(r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>Authentication Failed</title>
                        <style>
                            body {{
                                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                                display: flex;
                                justify-content: center;
                                align-items: center;
                                height: 100vh;
                                margin: 0;
                                background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
                            }}
                            .container {{
                                background: white;
                                padding: 2rem;
                                border-radius: 10px;
                                box-shadow: 0 10px 25px rgba(0,0,0,0.2);
                                text-align: center;
                            }}
                            h1 {{ color: #f5576c; margin: 0 0 1rem 0; }}
                            p {{ color: #666; }}
                        </style>
                    </head>
                    <body>
                        <div class="container">
                            <h1>✗ Authentication Failed</h1>
                            <p>{}</p>
                            <p>You can close this window and try again.</p>
                        </div>
                    </body>
                    </html>
                "#, error_value);
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Full::new(Bytes::from(html)))
                    .unwrap())
            } else {
                Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Full::new(Bytes::from("No authorization code or error in callback")))
                    .unwrap())
            }
        }
    });

    Builder::new(TokioExecutor::new())
        .serve_connection(io, service)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok("Callback received".to_string())
}

/// Check Google authentication status
#[tauri::command]
#[specta::specta]
pub async fn get_google_auth_status(app: AppHandle) -> Result<GoogleAuthStatus, String> {
    let is_authenticated = google_auth::has_valid_google_tokens(&app);

    if is_authenticated {
        match google_auth::get_google_user_info(&app).await {
            Ok(user_info) => Ok(GoogleAuthStatus {
                is_authenticated: true,
                email: Some(user_info.email),
                name: user_info.name,
            }),
            Err(e) => {
                error!("Failed to get user info: {}", e);
                Ok(GoogleAuthStatus {
                    is_authenticated: false,
                    email: None,
                    name: None,
                })
            }
        }
    } else {
        Ok(GoogleAuthStatus {
            is_authenticated: false,
            email: None,
            name: None,
        })
    }
}

/// Log out from Google
#[tauri::command]
#[specta::specta]
pub fn logout_google(app: AppHandle) -> Result<(), String> {
    google_auth::clear_google_tokens(&app)?;
    info!("Logged out from Google");
    Ok(())
}

/// Get current Google access token
#[tauri::command]
#[specta::specta]
pub async fn get_google_access_token(
    app: AppHandle,
    client_id: Option<String>,
    client_secret: Option<String>,
) -> Result<String, String> {
    google_auth::get_valid_access_token(&app, client_id, client_secret).await
}
