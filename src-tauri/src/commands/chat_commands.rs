use crate::{
    database::repositories::{conversation_repository, settings_repository},
    error::{AppError, AppResult},
    models::{ChatResult, LatencySample},
    services::{
        AppState, latency_service, memory_extractor, openai_response_service, rag_service,
        rvc_profile_service, rvc_service, sbv2_service,
    },
};
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::Utc;
use std::time::Instant;
use tauri::State;

#[tauri::command]
pub fn save_api_key(api_key: String) -> AppResult<()> {
    openai_response_service::save_api_key(&api_key)
}

#[tauri::command]
pub fn api_key_configured() -> bool {
    openai_response_service::has_api_key()
}

#[tauri::command]
pub fn recent_chat(state: State<'_, AppState>) -> AppResult<Vec<crate::models::ChatTurn>> {
    let conn = state
        .db
        .0
        .lock()
        .map_err(|_| AppError::from("Database lock poisoned"))?;
    conversation_repository::recent(&conn, 40)
}

#[tauri::command]
pub async fn send_chat(
    state: State<'_, AppState>,
    message: String,
    mode: Option<String>,
    vad_to_stt_ms: Option<i64>,
) -> AppResult<ChatResult> {
    let message = message.trim().to_owned();
    if message.is_empty() {
        return Err(AppError::from("Message cannot be empty"));
    }
    let mut latency = LatencySample {
        created_at: Utc::now().to_rfc3339(),
        mode: mode.unwrap_or_else(|| "typed_companion".into()),
        vad_to_stt_ms,
        success: true,
        ..Default::default()
    };
    let recent = {
        let conn = state
            .db
            .0
            .lock()
            .map_err(|_| AppError::from("Database lock poisoned"))?;
        conversation_repository::recent(&conn, 12)?
    };
    // Seeded knowledge has no vectors until an API key is available. Batch-index any
    // missing user and AI rows once, then normal requests only embed the query.
    rag_service::backfill_embeddings(&state).await?;
    let started = Instant::now();
    let context = rag_service::retrieve(&state, &message, 5, 7).await?;
    latency.rag_ms = Some(started.elapsed().as_millis() as i64);
    let memories = context
        .user_memories
        .iter()
        .map(|m| m.memory.content.clone())
        .collect::<Vec<_>>();
    let ai_knowledge = context
        .ai_knowledge
        .iter()
        .map(|item| {
            format!(
                "[{} / {}] {}: {}",
                item.knowledge.source_kind,
                item.knowledge.category,
                item.knowledge.title,
                item.knowledge.content
            )
        })
        .collect::<Vec<_>>();
    let started = Instant::now();
    let response =
        openai_response_service::respond(&state, &message, &recent, &memories, &ai_knowledge)
            .await?;
    latency.llm_total_ms = Some(started.elapsed().as_millis() as i64);
    {
        let conn = state
            .db
            .0
            .lock()
            .map_err(|_| AppError::from("Database lock poisoned"))?;
        conversation_repository::add(&conn, "user", &message, None)?;
        conversation_repository::add(&conn, "assistant", &response.id, Some(&response.ja))?;
    }
    let _ = memory_extractor::extract_and_store(&state, &message).await;
    let list = rvc_profile_service::discover(&state)?;
    let profile = list
        .selected_profile
        .as_ref()
        .and_then(|id| list.voices.iter().find(|p| &p.id == id))
        .cloned();
    latency.rvc_profile_id = profile.as_ref().map(|p| p.id.clone());
    let mut warning = None;
    let mut final_audio = None;
    let started = Instant::now();
    match sbv2_service::synthesize_japanese(
        &state,
        &response.ja,
        sbv2_service::emotion_style(&response.emotion),
    )
    .await
    {
        Ok(raw) => {
            latency.tts_ms = Some(started.elapsed().as_millis() as i64);
            if let Some(p) = profile {
                let started = Instant::now();
                match rvc_service::convert(&state, &p, raw.clone()).await {
                    Ok(audio) => {
                        latency.rvc_ms = Some(started.elapsed().as_millis() as i64);
                        final_audio = Some(audio)
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
                            final_audio = Some(raw);
                            warning = Some(format!("RVC unavailable; raw SBV2 used: {e}"));
                        } else {
                            warning = Some(e.to_string());
                        }
                    }
                }
            } else {
                final_audio = Some(raw);
                warning = Some("No valid RVC voice selected; raw SBV2 used".into());
            }
        }
        Err(e) => {
            warning = Some(format!(
                "Text response ready, but speech is unavailable: {e}"
            ));
        }
    }
    let encoded = final_audio.map(|a| STANDARD.encode(a));
    let id = latency_service::record(&state, &latency)?;
    latency.id = Some(id);
    Ok(ChatResult {
        response,
        audio_base64: encoded,
        audio_mime: Some("audio/wav".into()),
        memories_used: memories,
        ai_knowledge_used: ai_knowledge,
        latency,
        voice_warning: warning,
    })
}

#[tauri::command]
pub async fn initialize_memory_embeddings(state: State<'_, AppState>) -> AppResult<usize> {
    rag_service::backfill_embeddings(&state).await
}
