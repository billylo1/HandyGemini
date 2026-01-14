use crate::gemini_client;
use tauri::AppHandle;

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
    gemini_client::ask_gemini(
        &app,
        &text,
        &model,
        &api_key,
        context_images,
        context_audio,
        sample_rate,
    )
    .await
}
