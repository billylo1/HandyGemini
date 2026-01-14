use log::debug;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize)]
struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inline_data: Option<InlineData>,
}

#[derive(Debug, Serialize)]
struct InlineData {
    mime_type: String,
    data: String, // base64 encoded
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Debug, Deserialize)]
struct CandidateContent {
    parts: Vec<Part>,
}

#[derive(Debug, Deserialize)]
struct Part {
    text: Option<String>,
}

/// Convert audio samples to base64 WAV
fn audio_to_base64_wav(audio: &[f32], sample_rate: u32) -> Result<String, String> {
    use hound::{WavSpec, WavWriter};
    use std::io::Cursor;

    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buffer = Vec::new();
    {
        let mut writer = WavWriter::new(Cursor::new(&mut buffer), spec)
            .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

        for &sample in audio {
            // Convert f32 (-1.0 to 1.0) to i16
            let int_sample = (sample * i16::MAX as f32) as i16;
            writer
                .write_sample(int_sample)
                .map_err(|e| format!("Failed to write sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV: {}", e))?;
    }

    Ok(general_purpose::STANDARD.encode(&buffer))
}

/// Send text and optional context (images, audio) to Gemini API for answers
pub async fn ask_gemini(
    _app: &AppHandle,
    text: &str,
    model: &str,
    api_key: &str,
    context_images: Option<Vec<Vec<u8>>>, // Raw image bytes (will be base64 encoded)
    context_audio: Option<Vec<f32>>,      // Optional audio context
    sample_rate: Option<u32>,
) -> Result<String, String> {
    if api_key.is_empty() {
        return Err("Gemini API key is not configured".to_string());
    }

    // Build parts for the request
    let mut parts = Vec::new();

    // Add text
    parts.push(GeminiPart {
        text: Some(text.to_string()),
        inline_data: None,
    });

    // Add images if provided
    if let Some(images) = context_images {
        for image_bytes in images {
            // Detect image type (simplified - assume PNG or JPEG)
            let mime_type = if image_bytes.len() >= 8 && &image_bytes[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
                "image/png"
            } else if image_bytes.len() >= 3 && &image_bytes[0..3] == [0xFF, 0xD8, 0xFF] {
                "image/jpeg"
            } else {
                "image/png" // Default fallback
            };

            parts.push(GeminiPart {
                text: None,
                inline_data: Some(InlineData {
                    mime_type: mime_type.to_string(),
                    data: general_purpose::STANDARD.encode(&image_bytes),
                }),
            });
        }
    }

    // Add audio if provided
    if let Some(audio) = context_audio {
        let sample_rate = sample_rate.unwrap_or(16000);
        let audio_base64 = audio_to_base64_wav(&audio, sample_rate)?;
        parts.push(GeminiPart {
            text: None,
            inline_data: Some(InlineData {
                mime_type: "audio/wav".to_string(),
                data: audio_base64,
            }),
        });
    }

    // Build request
    let request_body = serde_json::json!({
        "contents": [{
            "parts": parts
        }],
        "generationConfig": {
            "temperature": 0.7,
            "maxOutputTokens": 8192
        }
    });

    // Build headers
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Make request with API key as query parameter (recommended for Gemini API)
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    debug!("Sending request to Gemini API: {} with {} parts", url, parts.len());

    let response = client
        .post(&url)
        .headers(headers)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        return Err(format!(
            "Gemini API request failed with status {}: {}",
            status, error_text
        ));
    }

    let gemini_response: GeminiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Gemini response: {}", e))?;

    // Extract text from response
    let answer = gemini_response
        .candidates
        .first()
        .and_then(|c| c.content.parts.first())
        .and_then(|p| p.text.clone())
        .ok_or_else(|| "No text in Gemini response".to_string())?;

    Ok(answer)
}
