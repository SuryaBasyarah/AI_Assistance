use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LatencySample {
    pub id: Option<i64>,
    pub created_at: String,
    pub mode: String,
    pub rvc_profile_id: Option<String>,
    pub vad_to_stt_ms: Option<i64>,
    pub rag_ms: Option<i64>,
    pub llm_total_ms: Option<i64>,
    pub tts_ms: Option<i64>,
    pub rvc_ms: Option<i64>,
    pub voice_switch_ms: Option<i64>,
    pub post_speech_ttfa_ms: Option<i64>,
    pub success: bool,
    pub error_stage: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceHealth {
    pub name: String,
    pub ready: bool,
    pub detail: String,
    pub latency_ms: Option<i64>,
}
