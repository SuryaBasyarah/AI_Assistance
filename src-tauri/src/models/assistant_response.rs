use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantResponse {
    pub ja: String,
    pub id: String,
    pub emotion: String,
    pub intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTurn {
    pub id: i64,
    pub role: String,
    pub content_id: String,
    pub content_ja: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResult {
    pub response: AssistantResponse,
    pub audio_base64: Option<String>,
    pub audio_mime: Option<String>,
    pub memories_used: Vec<String>,
    pub ai_knowledge_used: Vec<String>,
    pub latency: super::LatencySample,
    pub voice_warning: Option<String>,
}
