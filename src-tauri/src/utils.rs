use crate::managers::audio::AudioRecordingManager;
use crate::managers::transcription::TranscriptionManager;
use crate::shortcut;
use crate::ManagedToggleState;
use log::{info, warn};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

// Re-export all utility modules for easy access
// pub use crate::audio_feedback::*;
pub use crate::clipboard::*;
pub use crate::overlay::*;
pub use crate::tray::*;

/// Centralized cancellation function that can be called from anywhere in the app.
/// Handles cancelling both recording and transcription operations and updates UI state.
pub fn cancel_current_operation(app: &AppHandle) {
    info!("Initiating operation cancellation...");

    // Unregister the cancel shortcut asynchronously
    shortcut::unregister_cancel_shortcut(app);

    // First, reset all shortcut toggle states.
    // This is critical for non-push-to-talk mode where shortcuts toggle on/off
    let toggle_state_manager = app.state::<ManagedToggleState>();
    if let Ok(mut states) = toggle_state_manager.lock() {
        states.active_toggles.values_mut().for_each(|v| *v = false);
    } else {
        warn!("Failed to lock toggle state manager during cancellation");
    }

    // Cancel any ongoing recording
    let audio_manager = app.state::<Arc<AudioRecordingManager>>();
    audio_manager.cancel_recording();

    // Update tray icon and hide overlay
    change_tray_icon(app, crate::tray::TrayIconState::Idle);
    hide_recording_overlay(app);

    // Unload model if immediate unload is enabled
    let tm = app.state::<Arc<TranscriptionManager>>();
    tm.maybe_unload_immediately("cancellation");

    info!("Operation cancellation completed - returned to idle state");
}

/// Check if using the Wayland display server protocol
#[cfg(target_os = "linux")]
pub fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|v| v.to_lowercase() == "wayland")
            .unwrap_or(false)
}

/// Get the user's public IP address with caching
/// Uses ipify.org API to fetch the IP, caches it in the app state
pub async fn get_user_ip_address(app: &tauri::AppHandle) -> Option<String> {
    use std::sync::Arc;
    use std::sync::Mutex;
    use tauri::Manager;
    
    // Get cached IP from app state
    let cached_ip = app.state::<Arc<Mutex<Option<String>>>>();
    if let Ok(ip_mutex) = cached_ip.lock() {
        if let Some(ref ip) = *ip_mutex {
            return Some(ip.clone());
        }
    }
    
    // Fetch IP from ipify.org
    let client = reqwest::Client::new();
    match client
        .get("https://api.ipify.org?format=text")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(ip) => {
                        let ip = ip.trim().to_string();
                        // Cache the IP in app state
                        if let Ok(mut ip_mutex) = cached_ip.lock() {
                            *ip_mutex = Some(ip.clone());
                        }
                        Some(ip)
                    }
                    Err(e) => {
                        log::warn!("Failed to read IP address response: {}", e);
                        None
                    }
                }
            } else {
                log::warn!("Failed to fetch IP address: HTTP {}", response.status());
                None
            }
        }
        Err(e) => {
            log::warn!("Failed to fetch IP address: {}", e);
            None
        }
    }
}
