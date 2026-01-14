use crate::gemini_client;
use tauri::{AppHandle, Manager};

/// Ask Gemini a question with optional context (images, audio)
#[tauri::command]
#[specta::specta]
pub async fn ask_gemini(
    app: AppHandle,
    text: String,
    model: String,
    api_key: String,
    context_images: Option<Vec<Vec<u8>>>, // Base64 encoded or raw image bytes
    context_audio: Option<Vec<f32>>,      // Optional audio context
    sample_rate: Option<u32>,
) -> Result<String, String> {
    // Get conversation history
    let conv_mgr = app.state::<std::sync::Arc<crate::managers::gemini_conversation::GeminiConversationManager>>();
    let conversation_history: Vec<gemini_client::ConversationMessage> = conv_mgr
        .get_history()
        .into_iter()
        .map(|msg| gemini_client::ConversationMessage {
            role: msg.role,
            text: msg.text,
        })
        .collect();
    
    let response = gemini_client::ask_gemini(
        &app,
        &text,
        &model,
        &api_key,
        context_images,
        context_audio,
        sample_rate,
        Some(conversation_history),
    )
    .await?;
    
    // Return just the answer for backward compatibility with existing code
    Ok(response.answer)
}

/// Clear Gemini conversation history
#[tauri::command]
#[specta::specta]
pub fn clear_gemini_history(app: AppHandle) -> Result<(), String> {
    let conv_mgr = app.state::<std::sync::Arc<crate::managers::gemini_conversation::GeminiConversationManager>>();
    conv_mgr.clear();
    Ok(())
}
