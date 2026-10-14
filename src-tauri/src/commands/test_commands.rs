use crate::{
    database::repositories::settings_repository,
    error::{AppError, AppResult},
    models::{LatencySample, RvcProfile},
    services::{AppState, latency_service, rvc_profile_service, rvc_service, sbv2_service},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::Utc;
use serde::Serialize;
use std::time::Instant;
use tauri::State;

#[derive(Serialize)]
pub struct VoiceTestResult {
    pub audio_base64: String,
    pub audio_mime: String,
    pub latency: LatencySample,
    pub warning: Option<String>,
}

fn chosen_profile(state: &AppState, profile_id: Option<&str>) -> AppResult<Option<RvcProfile>> {
    let list = rvc_profile_service::discover(state)?;
    let id = profile_id.map(str::to_owned).or(list.selected_profile);
    Ok(id.and_then(|id| list.voices.into_iter().find(|p| p.id == id)))
}

#[tauri::command]
pub async fn direct_voice_test(
    state: State<'_, AppState>,
    text: String,
    profile_id: Option<String>,
    bypass_rvc: bool,
) -> AppResult<VoiceTestResult> {
    if text.trim().is_empty() {
        return Err(AppError::from("Enter Japanese text to preview"));
    }
    let profile = chosen_profile(&state, profile_id.as_deref())?;
    let mut sample = LatencySample {
        created_at: Utc::now().to_rfc3339(),
        mode: "direct_voice".into(),
        rvc_profile_id: profile.as_ref().map(|p| p.id.clone()),
        success: true,
        ..Default::default()
    };
    let start = Instant::now();
    let raw = sbv2_service::synthesize_japanese(&state, &text, "Neutral").await?;
    sample.tts_ms = Some(start.elapsed().as_millis() as i64);
    let (audio, warning) = if bypass_rvc {
        (raw, Some("RVC bypassed; playing raw SBV2 output".into()))
    } else if let Some(p) = profile {
        let start = Instant::now();
        match rvc_service::convert(&state, &p, raw.clone()).await {
            Ok(v) => {
                sample.rvc_ms = Some(start.elapsed().as_millis() as i64);
                (v, p.warning)
            }
            Err(e) => {
                let fallback = {
                    let conn = state
                        .db
                        .0
                        .lock()
                        .map_err(|_| AppError::from("Database lock poisoned"))?;
                    settings_repository::get(&conn, "allow_raw_tts_fallback")?.as_deref()
                        == Some("true")
                };
                if fallback {
                    (raw, Some(format!("RVC unavailable; playing raw SBV2: {e}")))
                } else {
                    return Err(e);
                }
            }
        }
    } else {
        (
            raw,
            Some("No valid RVC profile selected; playing raw SBV2".into()),
        )
    };
    let id = latency_service::record(&state, &sample)?;
    sample.id = Some(id);
    Ok(VoiceTestResult {
        audio_base64: STANDARD.encode(audio),
        audio_mime: "audio/wav".into(),
        latency: sample,
        warning,
    })
}

#[tauri::command]
pub fn latency_history(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> AppResult<Vec<LatencySample>> {
    latency_service::recent(&state, limit.unwrap_or(50).clamp(1, 500))
}
