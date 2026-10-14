use crate::{
    error::{AppError, AppResult},
    models::RvcProfile,
    services::AppState,
};

pub async fn load(state: &AppState, profile: &str) -> AppResult<()> {
    let response = state
        .http
        .post(format!("{}/voices/load", state.rvc_base_url))
        .query(&[("profile", profile)])
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(AppError::from(format!(
            "RVC could not load {profile}: {}",
            response.text().await.unwrap_or_default()
        )));
    }
    Ok(())
}

pub async fn refresh_remote(state: &AppState) -> AppResult<()> {
    let response = state
        .http
        .post(format!("{}/voices/refresh", state.rvc_base_url))
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(AppError::from(format!(
            "RVC refresh failed: {}",
            response.status()
        )));
    }
    Ok(())
}

pub async fn convert(state: &AppState, profile: &RvcProfile, wav: Vec<u8>) -> AppResult<Vec<u8>> {
    let response = state
        .http
        .post(format!("{}/convert", state.rvc_base_url))
        .query(&[
            ("profile", profile.id.clone()),
            ("pitch", profile.pitch.to_string()),
            ("f0_method", profile.f0_method.clone()),
            ("index_rate", profile.index_rate.to_string()),
            ("resample_sr", "0".into()),
            ("rms_mix_rate", "1.0".into()),
            ("protect", profile.protect.to_string()),
        ])
        .header("content-type", "audio/wav")
        .body(wav)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(AppError::from(format!(
            "RVC conversion failed: {}",
            response.text().await.unwrap_or_default()
        )));
    }
    Ok(response.bytes().await?.to_vec())
}
