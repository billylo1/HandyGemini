#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use crate::apple_intelligence;
use crate::audio_feedback::{play_feedback_sound, play_feedback_sound_blocking, SoundType};
use crate::managers::audio::AudioRecordingManager;
use crate::managers::gemini_conversation::GeminiConversationManager;
use crate::managers::history::HistoryManager;
use crate::managers::transcription::TranscriptionManager;
use crate::gemini_client;
use crate::settings::{get_settings, AppSettings, APPLE_INTELLIGENCE_PROVIDER_ID};
use crate::shortcut;
use crate::tray::{change_tray_icon, TrayIconState};
use crate::utils::{self, show_recording_overlay, show_transcribing_overlay};
use crate::gemini_popup;
use crate::ManagedToggleState;
use ferrous_opencc::{config::BuiltinConfig, OpenCC};
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tauri::AppHandle;
use tauri::Manager;

// Shortcut Action Trait
pub trait ShortcutAction: Send + Sync {
    fn start(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str);
    fn stop(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str);
}

// Helper function to capture screenshot (active window or full screen based on settings)
async fn capture_screenshot(app: &AppHandle) -> Option<Vec<u8>> {
    let settings = get_settings(app);
    
    match settings.screenshot_mode {
        crate::settings::ScreenshotMode::ActiveWindow => {
            #[cfg(target_os = "macos")]
            {
                capture_active_window_macos().await
            }
            #[cfg(not(target_os = "macos"))]
            {
                warn!("Active window capture not yet implemented on this platform, falling back to full screen");
                capture_full_screen_screenshot().await
            }
        }
        crate::settings::ScreenshotMode::FullScreen => {
            capture_full_screen_screenshot().await
        }
    }
}

// Helper function to capture full screen screenshot
async fn capture_full_screen_screenshot() -> Option<Vec<u8>> {
    use screenshots::Screen;
    
    // Get all screens
    let screens = match Screen::all() {
        Ok(screens) => screens,
        Err(e) => {
            warn!("Failed to get screens: {}", e);
            return None;
        }
    };
    
    // Try to capture the primary screen (or first screen)
    let screen = screens.first()?;
    
    match screen.capture() {
        Ok(image) => {
            // Use the to_png() method to get PNG bytes directly
            match image.to_png(None) {
                Ok(png_data) => Some(png_data),
                Err(e) => {
                    warn!("Failed to convert screenshot to PNG: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            warn!("Failed to capture screenshot: {}", e);
            None
        }
    }
}

// macOS-specific active window capture using AppleScript + screencapture
#[cfg(target_os = "macos")]
async fn capture_active_window_macos() -> Option<Vec<u8>> {
    use std::process::Command;
    
    // First, get the active window bounds using AppleScript
    // Use position and size separately as bounds may not be available for all windows
    let applescript = r#"
        tell application "System Events"
            try
                set frontApp to first application process whose frontmost is true
                
                -- Try to get the frontmost window - use different methods as fallback
                set frontWindow to missing value
                try
                    set frontWindow to front window of frontApp
                on error
                    try
                        -- If front window fails, try first window
                        set frontWindow to first window of frontApp
                    on error
                        -- If that fails, try getting window 1
                        set frontWindow to window 1 of frontApp
                    end try
                end try
                
                if frontWindow is missing value then
                    return "ERROR: No window found"
                end if
                
                -- Get position and size separately (more reliable than bounds)
                set windowPosition to position of frontWindow
                set windowSize to size of frontWindow
                set x to item 1 of windowPosition
                set y to item 2 of windowPosition
                set w to item 1 of windowSize
                set h to item 2 of windowSize
                
                -- Return as {left, top, right, bottom}
                return {x, y, x + w, y + h}
            on error errorMessage
                return "ERROR: " & errorMessage
            end try
        end tell
    "#;
    
    let bounds_output = match Command::new("osascript")
        .arg("-e")
        .arg(applescript)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if output.status.success() && !stdout.starts_with("ERROR:") {
                stdout
            } else {
                let error_msg = if stdout.starts_with("ERROR:") {
                    stdout
                } else {
                    format!("{}: {}", stderr, stdout)
                };
                warn!("Failed to get window bounds: {}", error_msg);
                // Fallback to full screen capture if we can't get window bounds
                warn!("Falling back to full screen capture");
                return capture_full_screen_screenshot().await;
            }
        }
        Err(e) => {
            warn!("Failed to execute osascript: {}", e);
            // Fallback to full screen capture
            return capture_full_screen_screenshot().await;
        }
    };
    
    // Parse bounds: AppleScript returns "{left, top, right, bottom}"
    let bounds: Vec<i32> = bounds_output
        .trim_matches(|c| c == '{' || c == '}')
        .split(", ")
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    
    if bounds.len() != 4 {
        warn!("Invalid window bounds format: {}", bounds_output);
        // Fallback to full screen capture
        return capture_full_screen_screenshot().await;
    }
    
    let left = bounds[0];
    let top = bounds[1];
    let right = bounds[2];
    let bottom = bounds[3];
    
    // Validate bounds
    if right <= left || bottom <= top {
        warn!("Invalid window bounds: left={}, top={}, right={}, bottom={}", left, top, right, bottom);
        return capture_full_screen_screenshot().await;
    }
    
    // Calculate width and height
    let width = right - left;
    let height = bottom - top;
    
    // Use screencapture -R to capture the specific region
    // Format: -R"x,y,width,height" where x,y is top-left corner
    let temp_file = std::env::temp_dir().join(format!("handy_screenshot_{}.png", std::process::id()));
    let region_arg = format!("-R{},{},{},{}", left, top, width, height);
    
    match Command::new("screencapture")
        .arg("-x") // No sound
        .arg(&region_arg) // Capture specific region
        .arg("-t") // Format: png
        .arg("png") // PNG format
        .arg(temp_file.to_str().unwrap())
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                // Read the file
                match std::fs::read(&temp_file) {
                    Ok(data) => {
                        // Clean up temp file
                        let _ = std::fs::remove_file(&temp_file);
                        Some(data)
                    }
                    Err(e) => {
                        warn!("Failed to read screenshot file: {}", e);
                        capture_full_screen_screenshot().await
                    }
                }
            } else {
                warn!("screencapture command failed: {:?}", String::from_utf8_lossy(&output.stderr));
                // Fallback to full screen capture
                capture_full_screen_screenshot().await
            }
        }
        Err(e) => {
            warn!("Failed to execute screencapture: {}", e);
            // Fallback to full screen capture
            capture_full_screen_screenshot().await
        }
    }
}

// Helper function to check if Ctrl is in the shortcut string or if screenshot flag is set
fn should_capture_screenshot(shortcut_str: &str) -> bool {
    // Check for the SCREENSHOT flag we append when Ctrl is pressed
    if shortcut_str.contains("|SCREENSHOT") {
        return true;
    }
    // Fallback: check if Ctrl is in the shortcut string
    let shortcut_lower = shortcut_str.to_lowercase();
    shortcut_lower.contains("ctrl") || shortcut_lower.contains("control")
}

// Transcribe Action
struct TranscribeAction;

async fn maybe_post_process_transcription(
    settings: &AppSettings,
    transcription: &str,
) -> Option<String> {
    if !settings.post_process_enabled {
        return None;
    }

    let provider = match settings.active_post_process_provider().cloned() {
        Some(provider) => provider,
        None => {
            debug!("Post-processing enabled but no provider is selected");
            return None;
        }
    };

    let model = settings
        .post_process_models
        .get(&provider.id)
        .cloned()
        .unwrap_or_default();

    if model.trim().is_empty() {
        debug!(
            "Post-processing skipped because provider '{}' has no model configured",
            provider.id
        );
        return None;
    }

    let selected_prompt_id = match &settings.post_process_selected_prompt_id {
        Some(id) => id.clone(),
        None => {
            debug!("Post-processing skipped because no prompt is selected");
            return None;
        }
    };

    let prompt = match settings
        .post_process_prompts
        .iter()
        .find(|prompt| prompt.id == selected_prompt_id)
    {
        Some(prompt) => prompt.prompt.clone(),
        None => {
            debug!(
                "Post-processing skipped because prompt '{}' was not found",
                selected_prompt_id
            );
            return None;
        }
    };

    if prompt.trim().is_empty() {
        debug!("Post-processing skipped because the selected prompt is empty");
        return None;
    }

    debug!(
        "Starting LLM post-processing with provider '{}' (model: {})",
        provider.id, model
    );

    // Replace ${output} variable in the prompt with the actual text
    let processed_prompt = prompt.replace("${output}", transcription);
    debug!("Processed prompt length: {} chars", processed_prompt.len());

    if provider.id == APPLE_INTELLIGENCE_PROVIDER_ID {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            if !apple_intelligence::check_apple_intelligence_availability() {
                debug!("Apple Intelligence selected but not currently available on this device");
                return None;
            }

            let token_limit = model.trim().parse::<i32>().unwrap_or(0);
            return match apple_intelligence::process_text(&processed_prompt, token_limit) {
                Ok(result) => {
                    if result.trim().is_empty() {
                        debug!("Apple Intelligence returned an empty response");
                        None
                    } else {
                        debug!(
                            "Apple Intelligence post-processing succeeded. Output length: {} chars",
                            result.len()
                        );
                        Some(result)
                    }
                }
                Err(err) => {
                    error!("Apple Intelligence post-processing failed: {}", err);
                    None
                }
            };
        }

        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        {
            debug!("Apple Intelligence provider selected on unsupported platform");
            return None;
        }
    }

    let api_key = settings
        .post_process_api_keys
        .get(&provider.id)
        .cloned()
        .unwrap_or_default();

    // Send the chat completion request
    match crate::llm_client::send_chat_completion(&provider, api_key, &model, processed_prompt)
        .await
    {
        Ok(Some(content)) => {
            debug!(
                "LLM post-processing succeeded for provider '{}'. Output length: {} chars",
                provider.id,
                content.len()
            );
            Some(content)
        }
        Ok(None) => {
            error!("LLM API response has no content");
            None
        }
        Err(e) => {
            error!(
                "LLM post-processing failed for provider '{}': {}. Falling back to original transcription.",
                provider.id,
                e
            );
            None
        }
    }
}

async fn maybe_convert_chinese_variant(
    settings: &AppSettings,
    transcription: &str,
) -> Option<String> {
    // Check if language is set to Simplified or Traditional Chinese
    let is_simplified = settings.selected_language == "zh-Hans";
    let is_traditional = settings.selected_language == "zh-Hant";

    if !is_simplified && !is_traditional {
        debug!("selected_language is not Simplified or Traditional Chinese; skipping translation");
        return None;
    }

    debug!(
        "Starting Chinese translation using OpenCC for language: {}",
        settings.selected_language
    );

    // Use OpenCC to convert based on selected language
    let config = if is_simplified {
        // Convert Traditional Chinese to Simplified Chinese
        BuiltinConfig::Tw2sp
    } else {
        // Convert Simplified Chinese to Traditional Chinese
        BuiltinConfig::S2twp
    };

    match OpenCC::from_config(config) {
        Ok(converter) => {
            let converted = converter.convert(transcription);
            debug!(
                "OpenCC translation completed. Input length: {}, Output length: {}",
                transcription.len(),
                converted.len()
            );
            Some(converted)
        }
        Err(e) => {
            error!("Failed to initialize OpenCC converter: {}. Falling back to original transcription.", e);
            None
        }
    }
}

impl ShortcutAction for TranscribeAction {
    fn start(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str) {
        let start_time = Instant::now();
        info!("TranscribeAction::start called for binding: {} with shortcut: {}", binding_id, shortcut_str);
        debug!("TranscribeAction::start called for binding: {}", binding_id);

        // Load model in the background
        let tm = app.state::<Arc<TranscriptionManager>>();
        tm.initiate_model_load();

        let binding_id = binding_id.to_string();
        change_tray_icon(app, TrayIconState::Recording);
        show_recording_overlay(app);

        let rm = app.state::<Arc<AudioRecordingManager>>();

        // Get the microphone mode to determine audio feedback timing
        let settings = get_settings(app);
        let is_always_on = settings.always_on_microphone;
        debug!("Microphone mode - always_on: {}", is_always_on);

        let mut recording_started = false;
        if is_always_on {
            // Always-on mode: Play audio feedback immediately, then apply mute after sound finishes
            debug!("Always-on mode: Playing audio feedback immediately");
            let rm_clone = Arc::clone(&rm);
            let app_clone = app.clone();
            // The blocking helper exits immediately if audio feedback is disabled,
            // so we can always reuse this thread to ensure mute happens right after playback.
            std::thread::spawn(move || {
                play_feedback_sound_blocking(&app_clone, SoundType::Start);
                rm_clone.apply_mute();
            });

            recording_started = rm.try_start_recording(&binding_id);
            debug!("Recording started: {}", recording_started);
            if recording_started {
                // Play ready sound after a short delay to ensure mic is actually capturing
                let app_clone = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    play_feedback_sound(&app_clone, SoundType::Ready);
                });
            }
        } else {
            // On-demand mode: Start recording first, then play audio feedback, then apply mute
            // This allows the microphone to be activated before playing the sound
            debug!("On-demand mode: Starting recording first, then audio feedback");
            let recording_start_time = Instant::now();
            if rm.try_start_recording(&binding_id) {
                recording_started = true;
                debug!("Recording started in {:?}", recording_start_time.elapsed());
                // Small delay to ensure microphone stream is active
                let app_clone = app.clone();
                let rm_clone = Arc::clone(&rm);
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    debug!("Handling delayed audio feedback/mute sequence");
                    // Helper handles disabled audio feedback by returning early, so we reuse it
                    // to keep mute sequencing consistent in every mode.
                    play_feedback_sound_blocking(&app_clone, SoundType::Start);
                    rm_clone.apply_mute();
                    // Play ready sound after mic is ready (additional delay)
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    play_feedback_sound(&app_clone, SoundType::Ready);
                });
            } else {
                debug!("Failed to start recording");
            }
        }

        if recording_started {
            // Dynamically register the cancel shortcut in a separate task to avoid deadlock
            shortcut::register_cancel_shortcut(app);
        }

        debug!(
            "TranscribeAction::start completed in {:?}",
            start_time.elapsed()
        );
    }

    fn stop(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str) {
        // Unregister the cancel shortcut when transcription stops
        shortcut::unregister_cancel_shortcut(app);

        let stop_time = Instant::now();
        debug!("TranscribeAction::stop called for binding: {}", binding_id);

        let ah = app.clone();
        let rm = Arc::clone(&app.state::<Arc<AudioRecordingManager>>());
        let tm = Arc::clone(&app.state::<Arc<TranscriptionManager>>());
        let hm = Arc::clone(&app.state::<Arc<HistoryManager>>());

        // Check if using Gemini audio transcription - if so, skip local transcription overlay
        let settings = get_settings(&ah);
        let using_gemini_audio = settings.gemini_enabled 
            && !settings.gemini_api_key.is_empty() 
            && settings.gemini_send_audio;

        if !using_gemini_audio {
            // Only show transcribing overlay if using local transcription
            change_tray_icon(app, TrayIconState::Transcribing);
            show_transcribing_overlay(app);
        }

        // Unmute before playing audio feedback so the stop sound is audible
        rm.remove_mute();

        // Play audio feedback for recording stop
        play_feedback_sound(app, SoundType::Stop);

        let binding_id = binding_id.to_string(); // Clone binding_id for the async task
        let shortcut_str = shortcut_str.to_string(); // Clone shortcut_str for the async task

        tauri::async_runtime::spawn(async move {
            // Check if screenshot should be captured (Ctrl was pressed)
            // Remove the SCREENSHOT flag from shortcut_str for logging
            let clean_shortcut = shortcut_str.replace("|SCREENSHOT", "");
            let screenshot = if should_capture_screenshot(&shortcut_str) {
                info!("Ctrl detected in shortcut '{}', capturing screenshot", clean_shortcut);
                capture_screenshot(&ah).await
            } else {
                None
            };
            let binding_id = binding_id.clone(); // Clone for the inner async task
            debug!(
                "Starting async transcription task for binding: {}",
                binding_id
            );

            let stop_recording_time = Instant::now();
            if let Some(samples) = rm.stop_recording(&binding_id) {
                debug!(
                    "Recording stopped and samples retrieved in {:?}, sample count: {}",
                    stop_recording_time.elapsed(),
                    samples.len()
                );

                // Check if we should send audio directly to Gemini (skip local transcription)
                let settings_for_audio_check = get_settings(&ah);
                let send_audio_directly = settings_for_audio_check.gemini_enabled 
                    && !settings_for_audio_check.gemini_api_key.is_empty() 
                    && settings_for_audio_check.gemini_send_audio;

                let transcription_time = Instant::now();
                let samples_clone = samples.clone(); // Clone for history saving
                let samples_for_gemini = samples.clone(); // Clone for potential Gemini audio sending
                
                // If sending audio directly to Gemini, skip local transcription and send immediately
                if send_audio_directly {
                    info!("Sending audio directly to Gemini, skipping local transcription");
                    
                    // Show "Sending to Gemini" status on overlay
                    utils::show_gemini_sending_overlay(&ah);
                    
                    let ah_clone = ah.clone();
                    let gemini_model = settings_for_audio_check.gemini_model.clone();
                    let gemini_api_key = settings_for_audio_check.gemini_api_key.clone();
                    
                    // Get conversation manager and history
                    let conv_mgr = Arc::clone(&ah.state::<Arc<GeminiConversationManager>>());
                    let conversation_history: Vec<gemini_client::ConversationMessage> = conv_mgr
                        .get_history()
                        .into_iter()
                        .map(|msg| gemini_client::ConversationMessage {
                            role: msg.role.clone(),
                            text: msg.text.clone(),
                        })
                        .collect();
                    
                    let audio_samples = samples_for_gemini.clone();
                    let conv_mgr_clone = Arc::clone(&conv_mgr);
                    let screenshot_for_gemini = screenshot.clone();
                    tauri::async_runtime::spawn(async move {
                        // Prepare context images if screenshot was captured
                        let context_images = screenshot_for_gemini.map(|img| vec![img]);
                        
                        match gemini_client::ask_gemini(
                            &ah_clone,
                            "", // Empty text when sending audio
                            &gemini_model,
                            &gemini_api_key,
                            context_images, // Screenshot if Ctrl was pressed
                            Some(audio_samples), // Send audio samples
                            Some(16000), // Sample rate (16kHz, standard for Whisper)
                            Some(conversation_history.clone()),
                        )
                        .await
                        {
                            Ok(gemini_response_data) => {
                                info!("Received Gemini response from audio (answer length: {} chars)", gemini_response_data.answer.len());
                                
                                // Show "Answer is ready" status before hiding
                                utils::show_gemini_ready_overlay(&ah_clone);
                                
                                // Small delay to show "ready" status, then hide overlay
                                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                
                                // Hide overlay and update tray icon when response is received
                                utils::hide_recording_overlay(&ah_clone);
                                change_tray_icon(&ah_clone, TrayIconState::Idle);
                                
                                // Get transcription from Gemini
                                let question_text = gemini_response_data.transcription
                                    .as_ref()
                                    .map(|t| t.clone())
                                    .unwrap_or_else(|| "Audio transcription".to_string());
                                
                                // Add to conversation history
                                conv_mgr_clone.add_user_message(question_text.clone());
                                conv_mgr_clone.add_model_message(gemini_response_data.answer.clone());
                                
                                // Format response to include Gemini's transcription and answer
                                let formatted_response = format!("**Q:** {}\n\n**A:** {}", question_text, gemini_response_data.answer);
                                // Show Gemini popup with formatted response
                                gemini_popup::show_gemini_popup(&ah_clone, formatted_response);
                            }
                            Err(e) => {
                                error!("Failed to get Gemini response from audio: {}", e);
                                // Hide overlay and update tray icon on error too
                                utils::hide_recording_overlay(&ah_clone);
                                change_tray_icon(&ah_clone, TrayIconState::Idle);
                            }
                        }
                    });
                    
                    // Still save to history in background (with empty transcription since we're using Gemini)
                    let hm_clone = Arc::clone(&hm);
                    let samples_for_history = samples_clone.clone();
                    tauri::async_runtime::spawn(async move {
                        // Save with empty transcription - Gemini will provide the transcription
                        if let Err(e) = hm_clone
                            .save_transcription(
                                samples_for_history,
                                "".to_string(), // Empty transcription when using Gemini audio
                                None,
                                None,
                            )
                            .await
                        {
                            error!("Failed to save transcription to history: {}", e);
                        }
                    });
                    
                    return; // Exit early, don't do local transcription
                }
                
                // Otherwise, do local transcription as before
                match tm.transcribe(samples) {
                    Ok(transcription) => {
                        debug!(
                            "Transcription completed in {:?}: '{}'",
                            transcription_time.elapsed(),
                            transcription
                        );
                        if !transcription.is_empty() {
                            let settings = get_settings(&ah);
                            let mut final_text = transcription.clone();
                            let mut post_processed_text: Option<String> = None;
                            let mut post_process_prompt: Option<String> = None;

                            // First, check if Chinese variant conversion is needed
                            if let Some(converted_text) =
                                maybe_convert_chinese_variant(&settings, &transcription).await
                            {
                                final_text = converted_text.clone();
                                post_processed_text = Some(converted_text);
                            }
                            // Then apply regular post-processing if enabled
                            else if let Some(processed_text) =
                                maybe_post_process_transcription(&settings, &transcription).await
                            {
                                final_text = processed_text.clone();
                                post_processed_text = Some(processed_text);

                                // Get the prompt that was used
                                if let Some(prompt_id) = &settings.post_process_selected_prompt_id {
                                    if let Some(prompt) = settings
                                        .post_process_prompts
                                        .iter()
                                        .find(|p| &p.id == prompt_id)
                                    {
                                        post_process_prompt = Some(prompt.prompt.clone());
                                    }
                                }
                            }

                            // Save to history with post-processed text and prompt
                            let hm_clone = Arc::clone(&hm);
                            let transcription_for_history = transcription.clone();
                            let samples_for_history = samples_clone.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = hm_clone
                                    .save_transcription(
                                        samples_for_history,
                                        transcription_for_history,
                                        post_processed_text,
                                        post_process_prompt,
                                    )
                                    .await
                                {
                                    error!("Failed to save transcription to history: {}", e);
                                }
                            });

                            // Send to Gemini if enabled
                            info!("Gemini setting check: enabled={}, model={}, send_audio={}", settings.gemini_enabled, settings.gemini_model, settings.gemini_send_audio);
                            let gemini_enabled = settings.gemini_enabled && !settings.gemini_api_key.is_empty();
                            if gemini_enabled {
                                let ah_clone = ah.clone();
                                let gemini_model = settings.gemini_model.clone();
                                let gemini_api_key = settings.gemini_api_key.clone();
                                let send_audio = settings.gemini_send_audio;
                                
                                // Get conversation manager and history
                                let conv_mgr = Arc::clone(&ah.state::<Arc<GeminiConversationManager>>());
                                let conversation_history: Vec<gemini_client::ConversationMessage> = conv_mgr
                                    .get_history()
                                    .into_iter()
                                    .map(|msg| gemini_client::ConversationMessage {
                                        role: msg.role.clone(),
                                        text: msg.text.clone(),
                                    })
                                    .collect();
                                
                                if send_audio {
                                    // Send audio directly to Gemini for server-side transcription
                                    info!("Gemini send_audio enabled, sending audio samples to Gemini");
                                    
                                    // Show "Sending to Gemini" status on overlay
                                    utils::show_gemini_sending_overlay(&ah);
                                    
                                    let audio_samples = samples_for_gemini.clone();
                                    let conv_mgr_clone = Arc::clone(&conv_mgr);
                                    let screenshot_for_gemini = screenshot.clone();
                                    tauri::async_runtime::spawn(async move {
                                        // Prepare context images if screenshot was captured
                                        let context_images = screenshot_for_gemini.map(|img| vec![img]);
                                        
                                        match gemini_client::ask_gemini(
                                            &ah_clone,
                                            "", // Empty text when sending audio
                                            &gemini_model,
                                            &gemini_api_key,
                                            context_images, // Screenshot if Ctrl was pressed
                                            Some(audio_samples), // Send audio samples
                                            Some(16000), // Sample rate (16kHz, standard for Whisper)
                                            Some(conversation_history.clone()),
                                        )
                                        .await
                                        {
                                            Ok(gemini_response_data) => {
                                                info!("Received Gemini response from audio (answer length: {} chars)", gemini_response_data.answer.len());
                                                
                                                // Show "Answer is ready" status before hiding
                                                utils::show_gemini_ready_overlay(&ah_clone);
                                                
                                                // Small delay to show "ready" status, then hide overlay
                                                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                                
                                                // Hide overlay and update tray icon when response is received
                                                utils::hide_recording_overlay(&ah_clone);
                                                change_tray_icon(&ah_clone, TrayIconState::Idle);
                                                
                                                // Get transcription (from Gemini or use local as fallback)
                                                let question_text = gemini_response_data.transcription
                                                    .as_ref()
                                                    .map(|t| t.clone())
                                                    .unwrap_or_else(|| transcription.clone());
                                                
                                                // Add to conversation history
                                                conv_mgr_clone.add_user_message(question_text.clone());
                                                conv_mgr_clone.add_model_message(gemini_response_data.answer.clone());
                                                
                                                // Format response to include Gemini's transcription and answer
                                                let formatted_response = format!("**Q:** {}\n\n**A:** {}", question_text, gemini_response_data.answer);
                                                // Show Gemini popup with formatted response
                                                gemini_popup::show_gemini_popup(&ah_clone, formatted_response);
                                            }
                                            Err(e) => {
                                                error!("Failed to get Gemini response from audio: {}", e);
                                                // Hide overlay and update tray icon on error too
                                                utils::hide_recording_overlay(&ah_clone);
                                                change_tray_icon(&ah_clone, TrayIconState::Idle);
                                            }
                                        }
                                    });
                                } else {
                                    // Send transcribed text to Gemini
                                    info!("Gemini is enabled, sending transcription to Gemini");
                                    
                                    // Show "Sending to Gemini" status on overlay
                                    utils::show_gemini_sending_overlay(&ah);
                                    
                                    let transcription_for_gemini = transcription.clone();
                                    let conv_mgr_clone = Arc::clone(&conv_mgr);
                                    let screenshot_for_gemini = screenshot.clone();
                                    tauri::async_runtime::spawn(async move {
                                        info!("Sending transcription to Gemini: {}", transcription_for_gemini);
                                        
                                        // Add user message to conversation history
                                        conv_mgr_clone.add_user_message(transcription_for_gemini.clone());
                                        
                                        // Prepare context images if screenshot was captured
                                        let context_images = screenshot_for_gemini.map(|img| vec![img]);
                                        
                                        match gemini_client::ask_gemini(
                                            &ah_clone,
                                            &transcription_for_gemini,
                                            &gemini_model,
                                            &gemini_api_key,
                                            context_images, // Screenshot if Ctrl was pressed
                                            None, // No audio context for now
                                            None, // No sample rate
                                            Some(conversation_history.clone()),
                                        )
                                        .await
                                        {
                                            Ok(gemini_response_data) => {
                                                info!("Received Gemini response (answer length: {} chars)", gemini_response_data.answer.len());
                                                
                                                // Show "Answer is ready" status before hiding
                                                utils::show_gemini_ready_overlay(&ah_clone);
                                                
                                                // Small delay to show "ready" status, then hide overlay
                                                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                                
                                                // Hide overlay and update tray icon when response is received
                                                utils::hide_recording_overlay(&ah_clone);
                                                change_tray_icon(&ah_clone, TrayIconState::Idle);
                                                
                                                // Add model response to conversation history
                                                conv_mgr_clone.add_model_message(gemini_response_data.answer.clone());
                                                
                                                // Format response to include question and answer
                                                let formatted_response = format!("**Q:** {}\n\n**A:** {}", transcription_for_gemini, gemini_response_data.answer);
                                                // Show Gemini popup with formatted response
                                                gemini_popup::show_gemini_popup(&ah_clone, formatted_response);
                                            }
                                            Err(e) => {
                                                error!("Failed to get Gemini response: {}", e);
                                                // Hide overlay and update tray icon on error too
                                                utils::hide_recording_overlay(&ah_clone);
                                                change_tray_icon(&ah_clone, TrayIconState::Idle);
                                            }
                                        }
                                    });
                                }
                            } else {
                                info!("Gemini is disabled, skipping Gemini API call");
                            }

                            // Paste the final text (either processed or original) - skip if Gemini is enabled
                            if !gemini_enabled {
                                let ah_clone = ah.clone();
                                let paste_time = Instant::now();
                                ah.run_on_main_thread(move || {
                                    match utils::paste(final_text, ah_clone.clone()) {
                                        Ok(()) => debug!(
                                            "Text pasted successfully in {:?}",
                                            paste_time.elapsed()
                                        ),
                                        Err(e) => error!("Failed to paste transcription: {}", e),
                                    }
                                    // Hide the overlay after transcription is complete
                                    utils::hide_recording_overlay(&ah_clone);
                                    change_tray_icon(&ah_clone, TrayIconState::Idle);
                                })
                                .unwrap_or_else(|e| {
                                    error!("Failed to run paste on main thread: {:?}", e);
                                    utils::hide_recording_overlay(&ah);
                                    change_tray_icon(&ah, TrayIconState::Idle);
                                });
                            } else {
                                info!("Gemini is enabled, skipping paste - overlay and tray icon will be hidden when Gemini response is received");
                                // Don't hide overlay/tray icon here - they will be hidden in the async task callbacks
                                // when the Gemini response is received (or on error)
                            }
                        } else {
                            utils::hide_recording_overlay(&ah);
                            change_tray_icon(&ah, TrayIconState::Idle);
                        }
                    }
                    Err(err) => {
                        debug!("Global Shortcut Transcription error: {}", err);
                        utils::hide_recording_overlay(&ah);
                        change_tray_icon(&ah, TrayIconState::Idle);
                    }
                }
            } else {
                debug!("No samples retrieved from recording stop");
                utils::hide_recording_overlay(&ah);
                change_tray_icon(&ah, TrayIconState::Idle);
            }

            // Clear toggle state now that transcription is complete
            if let Ok(mut states) = ah.state::<ManagedToggleState>().lock() {
                states.active_toggles.insert(binding_id, false);
            }
        });

        debug!(
            "TranscribeAction::stop completed in {:?}",
            stop_time.elapsed()
        );
    }
}

// Cancel Action
struct CancelAction;

impl ShortcutAction for CancelAction {
    fn start(&self, app: &AppHandle, _binding_id: &str, _shortcut_str: &str) {
        utils::cancel_current_operation(app);
    }

    fn stop(&self, _app: &AppHandle, _binding_id: &str, _shortcut_str: &str) {
        // Nothing to do on stop for cancel
    }
}

// Test Action
struct TestAction;

impl ShortcutAction for TestAction {
    fn start(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str) {
        log::info!(
            "Shortcut ID '{}': Started - {} (App: {})", // Changed "Pressed" to "Started" for consistency
            binding_id,
            shortcut_str,
            app.package_info().name
        );
    }

    fn stop(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str) {
        log::info!(
            "Shortcut ID '{}': Stopped - {} (App: {})", // Changed "Released" to "Stopped" for consistency
            binding_id,
            shortcut_str,
            app.package_info().name
        );
    }
}

// Static Action Map
pub static ACTION_MAP: Lazy<HashMap<String, Arc<dyn ShortcutAction>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        "transcribe".to_string(),
        Arc::new(TranscribeAction) as Arc<dyn ShortcutAction>,
    );
    map.insert(
        "cancel".to_string(),
        Arc::new(CancelAction) as Arc<dyn ShortcutAction>,
    );
    map.insert(
        "test".to_string(),
        Arc::new(TestAction) as Arc<dyn ShortcutAction>,
    );
    map
});
