use crate::input;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize};

#[cfg(not(target_os = "macos"))]
use log::debug;

#[cfg(not(target_os = "macos"))]
use tauri::WebviewWindowBuilder;

#[cfg(target_os = "macos")]
use tauri::WebviewUrl;

#[cfg(target_os = "macos")]
use tauri_nspanel::{tauri_panel, CollectionBehavior, PanelBuilder, PanelLevel};

#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(GeminiPopupPanel {
        config: {
            can_become_key_window: false,
            is_floating_panel: true
        }
    })
}

const POPUP_WIDTH: f64 = 600.0;
const POPUP_HEIGHT: f64 = 400.0;

fn get_monitor_with_cursor(app_handle: &AppHandle) -> Option<tauri::Monitor> {
    if let Some(mouse_location) = input::get_cursor_position(app_handle) {
        if let Ok(monitors) = app_handle.available_monitors() {
            for monitor in monitors {
                let is_within = is_mouse_within_monitor(
                    mouse_location,
                    &monitor.position(),
                    &monitor.size(),
                );
                if is_within {
                    return Some(monitor);
                }
            }
        }
    }

    app_handle.primary_monitor().ok().flatten()
}

fn is_mouse_within_monitor(
    mouse_pos: (i32, i32),
    monitor_pos: &PhysicalPosition<i32>,
    monitor_size: &PhysicalSize<u32>,
) -> bool {
    let (mouse_x, mouse_y) = mouse_pos;
    let PhysicalPosition {
        x: monitor_x,
        y: monitor_y,
    } = *monitor_pos;
    let PhysicalSize {
        width: monitor_width,
        height: monitor_height,
    } = *monitor_size;

    mouse_x >= monitor_x
        && mouse_x < monitor_x + monitor_width as i32
        && mouse_y >= monitor_y
        && mouse_y < monitor_y + monitor_height as i32
}

fn calculate_popup_position(app_handle: &AppHandle) -> Option<(f64, f64)> {
    if let Some(monitor) = get_monitor_with_cursor(app_handle) {
        let monitor_size = monitor.size();
        let monitor_pos = monitor.position();

        // Position popup in bottom right corner with some padding
        let padding = 20.0;
        let x = monitor_pos.x as f64 + monitor_size.width as f64 - POPUP_WIDTH - padding;
        let y = monitor_pos.y as f64 + monitor_size.height as f64 - POPUP_HEIGHT - padding;

        Some((x, y))
    } else {
        None
    }
}

/// Creates the Gemini popup window and keeps it hidden by default
#[cfg(not(target_os = "macos"))]
pub fn create_gemini_popup(app_handle: &AppHandle) {
    if let Some((x, y)) = calculate_popup_position(app_handle) {
        match WebviewWindowBuilder::new(
            app_handle,
            "gemini_popup",
            tauri::WebviewUrl::App("src/gemini-popup/index.html".into()),
        )
        .title("Gemini Response")
        .position(x, y)
        .resizable(true)
        .inner_size(POPUP_WIDTH, POPUP_HEIGHT)
        .min_inner_size(POPUP_WIDTH, POPUP_HEIGHT)
        .max_inner_size(1200.0, 800.0)
        .shadow(true)
        .maximizable(false)
        .minimizable(true)
        .closable(true)
        .decorations(true)
        .always_on_top(true)
        .skip_taskbar(false)
        .transparent(false)
        .focused(true)
        .visible(false)
        .build()
        {
            Ok(_window) => {
                log::info!("Gemini popup window created successfully (hidden)");
            }
            Err(e) => {
                log::error!("Failed to create Gemini popup window: {}", e);
            }
        }
    }
}

/// Creates the Gemini popup panel and keeps it hidden by default (macOS)
#[cfg(target_os = "macos")]
pub fn create_gemini_popup(app_handle: &AppHandle) {
    if let Some((x, y)) = calculate_popup_position(app_handle) {
        match PanelBuilder::<_, GeminiPopupPanel>::new(app_handle, "gemini_popup")
            .url(WebviewUrl::App("src/gemini-popup/index.html".into()))
            .title("Gemini Response")
            .position(tauri::Position::Logical(tauri::LogicalPosition { x, y }))
            .level(PanelLevel::Floating)
            .size(tauri::Size::Logical(tauri::LogicalSize {
                width: POPUP_WIDTH,
                height: POPUP_HEIGHT,
            }))
            .has_shadow(true)
            .transparent(false)
            .no_activate(false)
            .corner_radius(10.0)
            .with_window(|w| w.decorations(true).transparent(false))
            .collection_behavior(
                CollectionBehavior::new()
                    .can_join_all_spaces()
                    .full_screen_auxiliary(),
            )
            .build()
        {
            Ok(panel) => {
                let _ = panel.hide();
                log::info!("Gemini popup panel created successfully (hidden)");
            }
            Err(e) => {
                log::error!("Failed to create Gemini popup panel: {}", e);
            }
        }
    }
}

/// Shows the Gemini popup window with response text
pub fn show_gemini_popup(app_handle: &AppHandle, response: String) {
    log::info!("Showing Gemini popup with response (length: {} chars)", response.len());
    
    if let Some(popup_window) = app_handle.get_webview_window("gemini_popup") {
        log::info!("Gemini popup window found, showing it");
        // Update position before showing
        if let Some((x, y)) = calculate_popup_position(app_handle) {
            let _ = popup_window
                .set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
        }

        let _ = popup_window.show();
        let _ = popup_window.set_focus();

        // Use eval to directly set the response in the window's React state
        // This bypasses the event system which seems to have timing issues
        let response_for_eval = response.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r");
        let js_code = format!(
            r#"
            (function() {{
                if (window.__geminiResponseHandler) {{
                    window.__geminiResponseHandler("{}");
                }} else {{
                    // Store for when handler is ready
                    window.__pendingGeminiResponse = "{}";
                    // Also try to dispatch a custom event
                    window.dispatchEvent(new CustomEvent('gemini-response', {{ detail: "{}" }}));
                }}
            }})();
            "#,
            response_for_eval, response_for_eval, response_for_eval
        );
        
        if let Err(e) = popup_window.eval(&js_code) {
            log::error!("Failed to eval response into window: {}", e);
        } else {
            log::info!("Successfully evaluated response into window ({} chars)", response.len());
        }
        
        // Also emit via Tauri events as fallback
        let response_clone = response.clone();
        let window_label = popup_window.label().to_string();
        let app_handle_clone = app_handle.clone();
        
        tauri::async_runtime::spawn(async move {
            for delay_ms in [100, 300, 500, 1000] {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                if let Some(window) = app_handle_clone.get_webview_window(&window_label) {
                    let _ = window.emit("show-response", response_clone.clone());
                }
            }
        });
    } else {
        log::warn!("Gemini popup window not found, creating it...");
        create_gemini_popup(app_handle);
        // Try again after a short delay
        std::thread::sleep(std::time::Duration::from_millis(200));
        if let Some(popup_window) = app_handle.get_webview_window("gemini_popup") {
            log::info!("Gemini popup window created, showing it");
            if let Some((x, y)) = calculate_popup_position(app_handle) {
                let _ = popup_window
                    .set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
            }
            let _ = popup_window.show();
            let _ = popup_window.set_focus();
            
            // Wait for window to be ready, then emit event multiple times
            let response_clone = response.clone();
            let window_label = popup_window.label().to_string();
            let app_handle_clone = app_handle.clone();
            
            // Emit after delays to ensure React is mounted
            tauri::async_runtime::spawn(async move {
                // Try multiple times with increasing delays
                for delay_ms in [200, 500, 1000, 1500] {
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    if let Some(window) = app_handle_clone.get_webview_window(&window_label) {
                        log::info!("Emitting show-response event after {}ms delay (after create), response length: {}", delay_ms, response_clone.len());
                        if let Err(e) = window.emit("show-response", response_clone.clone()) {
                            log::warn!("Failed to emit show-response event after {}ms (after create): {}", delay_ms, e);
                        } else {
                            log::info!("Successfully emitted show-response event after {}ms (after create) ({} chars)", delay_ms, response_clone.len());
                        }
                    }
                }
            });
        } else {
            log::error!("Failed to create Gemini popup window");
        }
    }
}

/// Hides the Gemini popup window
#[allow(dead_code)]
pub fn hide_gemini_popup(app_handle: &AppHandle) {
    if let Some(popup_window) = app_handle.get_webview_window("gemini_popup") {
        let _ = popup_window.hide();
    }
}
