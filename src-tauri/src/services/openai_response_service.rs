use crate::{
    error::{AppError, AppResult},
    models::{AssistantResponse, ChatTurn},
    services::AppState,
};
use serde_json::{Value, json};

const KEYRING_SERVICE: &str = "Haruya Virtual Companion";
const KEYRING_USER: &str = "openai_api_key";
pub const CHAT_MODEL: &str = "gpt-5.6-sol";

pub fn save_api_key(key: &str) -> AppResult<()> {
    let key = key.trim();
    if !key.starts_with("sk-") || key.len() < 20 {
        return Err(AppError::from("That does not look like an OpenAI API key."));
    }
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| AppError::from(e.to_string()))?
        .set_password(key)
        .map_err(|e| AppError::from(e.to_string()))?;
    Ok(())
}

pub fn has_api_key() -> bool {
    api_key().is_ok()
}

pub fn api_key() -> AppResult<String> {
    if let Ok(v) = std::env::var("OPENAI_API_KEY") {
        if !v.trim().is_empty() {
            return Ok(v);
        }
    }
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| AppError::from(e.to_string()))?
        .get_password()
        .map_err(|_| AppError::from("OpenAI API key is not configured. Add it in Settings."))
}

fn output_text(value: &Value) -> Option<String> {
    value
        .get("output")?
        .as_array()?
        .iter()
        .flat_map(|item| {
            item.get("content")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .find_map(|part| {
            if part.get("type").and_then(Value::as_str) == Some("output_text") {
                part.get("text").and_then(Value::as_str).map(str::to_owned)
            } else {
                None
            }
        })
}

pub async fn respond(
    state: &AppState,
    message: &str,
    recent: &[ChatTurn],
    user_memories: &[String],
    ai_knowledge: &[String],
) -> AppResult<AssistantResponse> {
    let history = recent
        .iter()
        .map(|t| {
            format!(
                "{}: {}{}",
                t.role,
                t.content_id,
                t.content_ja
                    .as_ref()
                    .map(|v| format!(" / {v}"))
                    .unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let memory = if user_memories.is_empty() {
        "No relevant user memory.".into()
    } else {
        user_memories
            .iter()
            .map(|m| format!("- {m}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let identity = if ai_knowledge.is_empty() {
        "No additional character knowledge was retrieved.".into()
    } else {
        ai_knowledge
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let input = format!(
        "RECENT CHAT:\n{history}\n\nRELEVANT USER MEMORY:\n{memory}\n\nRELEVANT AI CHARACTER KNOWLEDGE:\n{identity}\n\nCURRENT USER MESSAGE (Indonesian):\n{message}"
    );
    let body = json!({
        "model": CHAT_MODEL,
        "instructions":"You are a warm virtual companion modeled on Akiyama Mizuki from Project SEKAI. Understand Indonesian input. Reply with natural, conversational Japanese suitable for spoken delivery and an accurate Indonesian subtitle. The ja field must contain Japanese speech only, with no Indonesian translation, citations, database slugs, labels, or metadata. The id field must contain only the Indonesian translation. Keep the Japanese under 90 characters, concise, playful, perceptive, and friendly. Use AI character knowledge to maintain Mizuki's personality and history, while respecting official-versus-interpretive distinctions. Use user memory only when relevant and never invent personal facts. Do not claim that unofficial identity interpretations are officially confirmed. Return only the requested structure.",
        "input":input,
        "text":{"format":{"type":"json_schema","name":"companion_response","strict":true,"schema":{
            "type":"object","properties":{
                "ja":{"type":"string","maxLength":90},"id":{"type":"string"},
                "emotion":{"type":"string","enum":["neutral","warm","happy","excited","sad","concerned","calm"]},
                "intensity":{"type":"number","minimum":0,"maximum":1}
            },"required":["ja","id","emotion","intensity"],"additionalProperties":false
        }}}
    });
    let response = state
        .http
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(api_key()?)
        .json(&body)
        .send()
        .await?;
    let status = response.status();
    let value: Value = response.json().await?;
    if !status.is_success() {
        return Err(AppError::from(format!(
            "OpenAI response failed ({status}): {}",
            value
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("unknown error")
        )));
    }
    let text =
        output_text(&value).ok_or_else(|| AppError::from("OpenAI returned no text output"))?;
    let mut parsed: AssistantResponse = serde_json::from_str(&text)?;
    parsed.intensity = parsed.intensity.clamp(0.0, 1.0);
    Ok(parsed)
}

pub async fn embedding(state: &AppState, text: &str) -> AppResult<Vec<f32>> {
    let mut vectors = embedding_batch(state, &[text.to_owned()]).await?;
    vectors
        .pop()
        .ok_or_else(|| AppError::from("Embedding response missing vector"))
}

pub async fn embedding_batch(state: &AppState, texts: &[String]) -> AppResult<Vec<Vec<f32>>> {
    let body = json!({"model":"text-embedding-3-small","input":texts,"encoding_format":"float","dimensions":512});
    let response = state
        .http
        .post("https://api.openai.com/v1/embeddings")
        .bearer_auth(api_key()?)
        .json(&body)
        .send()
        .await?;
    let status = response.status();
    let value: Value = response.json().await?;
    if !status.is_success() {
        return Err(AppError::from(format!("Embedding failed ({status})")));
    }
    let data = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::from("Embedding response missing data"))?;
    data.iter()
        .map(|item| {
            serde_json::from_value(
                item.get("embedding")
                    .cloned()
                    .ok_or_else(|| AppError::from("Embedding response missing vector"))?,
            )
            .map_err(Into::into)
        })
        .collect()
}
