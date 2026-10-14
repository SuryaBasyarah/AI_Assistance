use crate::{
    error::{AppError, AppResult},
    services::{AppState, openai_response_service},
};
use reqwest::multipart::{Form, Part};
use serde_json::Value;

pub async fn transcribe_recording(
    state: &AppState,
    audio: Vec<u8>,
    mime: &str,
) -> AppResult<String> {
    let extension = if mime.contains("webm") {
        "webm"
    } else if mime.contains("ogg") {
        "ogg"
    } else {
        "wav"
    };
    let part = Part::bytes(audio)
        .file_name(format!("speech.{extension}"))
        .mime_str(mime)
        .map_err(|e| AppError::from(e.to_string()))?;
    let form = Form::new()
        .part("file", part)
        .text("model", "gpt-transcribe")
        .text("language", "id");
    let response = state
        .http
        .post("https://api.openai.com/v1/audio/transcriptions")
        .bearer_auth(openai_response_service::api_key()?)
        .multipart(form)
        .send()
        .await?;
    let status = response.status();
    let value: Value = response.json().await?;
    if !status.is_success() {
        return Err(AppError::from(format!(
            "Transcription failed ({status}): {}",
            value
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("unknown error")
        )));
    }
    value
        .get("text")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| AppError::from("Transcription returned no text"))
}
