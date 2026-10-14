use crate::{
    error::{AppError, AppResult},
    models::ServiceHealth,
    services::{AppState, openai_realtime_stt},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::Value;
use std::time::Instant;
use tauri::State;

#[tauri::command]
pub async fn transcribe_audio(
    state: State<'_, AppState>,
    audio_base64: String,
    mime_type: String,
) -> AppResult<String> {
    let bytes = STANDARD
        .decode(audio_base64)
        .map_err(|_| AppError::from("Invalid recorded audio"))?;
    openai_realtime_stt::transcribe_recording(&state, bytes, &mime_type).await
}

#[tauri::command]
pub async fn service_health(state: State<'_, AppState>) -> AppResult<Vec<ServiceHealth>> {
    let mut results = Vec::new();
    for (name, url) in [
        ("Style-Bert-VITS2", format!("{}/docs", state.sbv2_base_url)),
        ("RVC", format!("{}/health", state.rvc_base_url)),
    ] {
        let started = Instant::now();
        let response = state
            .http
            .get(&url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await;
        let (ready, detail) = match response {
            Ok(r) if name == "RVC" && r.status().is_success() => {
                let value: Value = r.json().await.unwrap_or_default();
                let ready = value.get("ready").and_then(Value::as_bool).unwrap_or(false);
                let device = value
                    .get("device")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let profile = value
                    .get("current_profile")
                    .and_then(Value::as_str)
                    .unwrap_or("none");
                (ready, format!("device={device}, profile={profile}"))
            }
            Ok(r) => (r.status().is_success(), format!("HTTP {}", r.status())),
            Err(e) => (false, e.to_string()),
        };
        results.push(ServiceHealth {
            name: name.into(),
            ready,
            detail,
            latency_ms: Some(started.elapsed().as_millis() as i64),
        });
    }
    results.push(ServiceHealth {
        name: "OpenAI key".into(),
        ready: crate::services::openai_response_service::has_api_key(),
        detail: if crate::services::openai_response_service::has_api_key() {
            "Stored securely".into()
        } else {
            "Not configured".into()
        },
        latency_ms: None,
    });
    Ok(results)
}
