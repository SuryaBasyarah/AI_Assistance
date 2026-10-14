use crate::{
    database::repositories::{settings_repository, voice_profile_repository},
    error::{AppError, AppResult},
    models::VoiceList,
    services::{AppState, rvc_profile_service, rvc_service},
};
use std::time::Instant;
use tauri::State;

#[tauri::command]
pub fn list_voices(state: State<'_, AppState>) -> AppResult<VoiceList> {
    rvc_profile_service::discover(&state)
}

#[tauri::command]
pub async fn refresh_voices(state: State<'_, AppState>) -> AppResult<VoiceList> {
    let _ = rvc_service::refresh_remote(&state).await;
    rvc_profile_service::discover(&state)
}

#[tauri::command]
pub async fn select_voice(state: State<'_, AppState>, profile_id: String) -> AppResult<i64> {
    let list = rvc_profile_service::discover(&state)?;
    if !list.voices.iter().any(|p| p.id == profile_id) {
        return Err(AppError::from(format!(
            "Voice profile '{profile_id}' is not valid or enabled"
        )));
    }
    let started = Instant::now();
    rvc_service::load(&state, &profile_id).await?;
    let elapsed = started.elapsed().as_millis() as i64;
    let conn = state
        .db
        .0
        .lock()
        .map_err(|_| AppError::from("Database lock poisoned"))?;
    settings_repository::set(&conn, "selected_rvc_profile", &profile_id)?;
    Ok(elapsed)
}

#[tauri::command]
pub fn save_voice_settings(
    state: State<'_, AppState>,
    profile_id: String,
    pitch: i32,
    f0_method: String,
    index_rate: f32,
    protect: f32,
) -> AppResult<VoiceList> {
    if !(-24..=24).contains(&pitch)
        || !(0.0..=1.0).contains(&index_rate)
        || !(0.0..=0.5).contains(&protect)
    {
        return Err(AppError::from(
            "Voice settings are outside supported ranges",
        ));
    }
    {
        let conn = state
            .db
            .0
            .lock()
            .map_err(|_| AppError::from("Database lock poisoned"))?;
        voice_profile_repository::update(
            &conn,
            &profile_id,
            pitch,
            &f0_method,
            index_rate,
            protect,
        )?;
    }
    rvc_profile_service::discover(&state)
}

#[tauri::command]
pub fn open_voices_folder(state: State<'_, AppState>) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer.exe")
        .arg(&state.voices_root)
        .spawn()?;
    #[cfg(not(target_os = "windows"))]
    open::that(&state.voices_root).map_err(|e| AppError::from(e.to_string()))?;
    Ok(())
}
