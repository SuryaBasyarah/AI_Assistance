use crate::{
    database::repositories::memory_repository,
    error::{AppError, AppResult},
    services::{AppState, openai_response_service},
};
use serde_json::{Value, json};

pub async fn extract_and_store(state: &AppState, user_message: &str) -> AppResult<usize> {
    let body = json!({
        "model": openai_response_service::CHAT_MODEL,
        "instructions":"Extract only durable personal facts or preferences explicitly stated by the user. Do not infer. Ignore greetings, transient plans, and questions. Return an empty facts array when none exist.",
        "input":user_message,
        "text":{"format":{"type":"json_schema","name":"durable_memory","strict":true,"schema":{"type":"object","properties":{"facts":{"type":"array","items":{"type":"string"}}},"required":["facts"],"additionalProperties":false}}}
    });
    let response = state
        .http
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(openai_response_service::api_key()?)
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        return Ok(0);
    }
    let value: Value = response.json().await?;
    let text = value
        .get("output")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|v| {
            v.get("content")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .find_map(|v| v.get("text").and_then(Value::as_str))
        .unwrap_or("{\"facts\":[]}");
    let parsed: Value = serde_json::from_str(text)?;
    let facts = parsed
        .get("facts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut count = 0;
    for fact in facts
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .take(5)
    {
        let vector = openai_response_service::embedding(state, &fact).await.ok();
        let conn = state
            .db
            .0
            .lock()
            .map_err(|_| AppError::from("Database lock poisoned"))?;
        memory_repository::add_if_new(&conn, &fact, vector.as_deref())?;
        count += 1;
    }
    Ok(count)
}
