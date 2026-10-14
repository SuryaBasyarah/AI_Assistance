use crate::{
    database::repositories::latency_repository,
    error::{AppError, AppResult},
    models::LatencySample,
    services::AppState,
};

pub fn record(state: &AppState, sample: &LatencySample) -> AppResult<i64> {
    let conn = state
        .db
        .0
        .lock()
        .map_err(|_| AppError::from("Database lock poisoned"))?;
    latency_repository::add(&conn, sample)
}
pub fn recent(state: &AppState, limit: i64) -> AppResult<Vec<LatencySample>> {
    let conn = state
        .db
        .0
        .lock()
        .map_err(|_| AppError::from("Database lock poisoned"))?;
    latency_repository::recent(&conn, limit)
}
