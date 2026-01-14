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

/// Response containing both transcription (if audio was sent) and answer
#[derive(Debug)]
pub struct GeminiResponseData {
    pub transcription: Option<String>,
    pub answer: String,
}

/// Conversation message for history
#[derive(Debug, Clone)]
pub struct ConversationMessage {
    pub role: String, // "user" or "model"
    pub text: String,
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
    conversation_history: Option<Vec<ConversationMessage>>, // Previous conversation messages
) -> Result<GeminiResponseData, String> {
    if api_key.is_empty() {
        return Err("Gemini API key is not configured".to_string());
    }

    // Build parts for the request
    let mut parts = Vec::new();

    // Check if we have images before processing
    let has_images = context_images.is_some();
    
    // Screenshot instruction to focus on the biggest canvas area
    let screenshot_instruction = "When analyzing the screenshot, focus on the biggest canvas area only. Ignore UI elements, menus, and sidebars.";

    // Add text with screenshot instruction if images are present
    if !text.is_empty() {
        let text_content = if has_images {
            format!("{}\n\n{}", screenshot_instruction, text)
        } else {
            text.to_string()
        };
        parts.push(GeminiPart {
            text: Some(text_content),
            inline_data: None,
        });
    } else if has_images {
        // If no text but images are present, add instruction as a separate part
        parts.push(GeminiPart {
            text: Some(screenshot_instruction.to_string()),
            inline_data: None,
        });
    }

    // Ensure we have at least one part (text or audio)
    if parts.is_empty() && context_audio.is_none() && context_images.is_none() {
        return Err("At least one of text, audio, or images must be provided".to_string());
    }

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

    // Check if we have audio before moving it
    let has_audio = context_audio.is_some();

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

    // When sending audio without text, add an instruction as a text part
    if has_audio && text.is_empty() {
        // Add instruction as a text part to format the response
        parts.push(GeminiPart {
            text: Some("Please transcribe the audio first, then provide your response. Format your response as:\n\nTranscription: [the transcribed text]\n\nResponse: [your answer]".to_string()),
            inline_data: None,
        });
    }

    // Build conversation history in Gemini format
    let mut contents = Vec::new();
    
    // Add conversation history if provided
    if let Some(history) = conversation_history {
        for msg in history {
            contents.push(serde_json::json!({
                "role": msg.role,
                "parts": [{
                    "text": msg.text
                }]
            }));
        }
    }
    
    // Add current message
    contents.push(serde_json::json!({
        "role": "user",
        "parts": parts
    }));

    let request_body = serde_json::json!({
        "contents": contents,
        "generationConfig": {
            "temperature": 0.7,
            "maxOutputTokens": 8192
        },
        "tools": [{
            "googleSearch": {}
        }]
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

    debug!("Gemini response structure: candidates={}", gemini_response.candidates.len());
    
    // Extract text from response - check all parts
    let response_text = gemini_response
        .candidates
        .first()
        .and_then(|c| {
            debug!("Candidate has {} parts", c.content.parts.len());
            // Try to get text from all parts and concatenate
            let texts: Vec<String> = c.content.parts
                .iter()
                .filter_map(|p| {
                    if let Some(text) = &p.text {
                        debug!("Found text part: {} chars", text.len());
                    }
                    p.text.clone()
                })
                .collect();
            if texts.is_empty() {
                None
            } else {
                Some(texts.join("\n"))
            }
        })
        .ok_or_else(|| {
            let debug_info = format!("No text in Gemini response. Candidates: {}, Parts in first candidate: {}", 
                gemini_response.candidates.len(),
                gemini_response.candidates.first().map(|c| c.content.parts.len()).unwrap_or(0));
            debug!("{}", debug_info);
            debug_info
        })?;
    
    debug!("Extracted response text: {} chars, preview: {}", response_text.len(), response_text.chars().take(200).collect::<String>());

    // If we sent audio, try to extract transcription from the response
    let (transcription, answer) = if has_audio && text.is_empty() {
        debug!("Parsing audio response, looking for transcription format");
        // Try to parse "Transcription: ... Response: ..." format
        if let Some(transcription_start) = response_text.find("Transcription:") {
            debug!("Found 'Transcription:' marker at position {}", transcription_start);
            let transcription_end = response_text[transcription_start..].find("\n\nResponse:").or_else(|| response_text[transcription_start..].find("\nResponse:"));
            if let Some(end) = transcription_end {
                let transcription_text = response_text[transcription_start + "Transcription:".len()..transcription_start + end].trim().to_string();
                let answer_start = transcription_start + end;
                let answer_text = if response_text[answer_start..].starts_with("\n\nResponse:") {
                    response_text[answer_start + "\n\nResponse:".len()..].trim().to_string()
                } else {
                    response_text[answer_start + "\nResponse:".len()..].trim().to_string()
                };
                debug!("Extracted transcription: {} chars, answer: {} chars", transcription_text.len(), answer_text.len());
                (Some(transcription_text), answer_text)
            } else {
                // Fallback: if format doesn't match, assume entire response is the answer
                debug!("No 'Response:' marker found, using entire response as answer");
                (None, response_text)
            }
        } else {
            // No transcription marker found, return entire response as answer
            debug!("No 'Transcription:' marker found, using entire response as answer");
            (None, response_text)
        }
    } else {
        // No audio sent, no transcription
        (None, response_text)
    };
    
    if answer.is_empty() {
        debug!("WARNING: Answer is empty after parsing!");
    }

    Ok(GeminiResponseData {
        transcription,
        answer,
    })
}
