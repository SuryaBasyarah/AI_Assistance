use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: i64,
    pub content: String,
    pub importance: f64,
    pub access_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct RankedMemory {
    pub memory: Memory,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiKnowledge {
    pub id: i64,
    pub slug: String,
    pub category: String,
    pub title: String,
    pub content: String,
    pub source_kind: String,
    pub source_reference: Option<String>,
    pub importance: f64,
    pub access_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct RankedAiKnowledge {
    pub knowledge: AiKnowledge,
    pub score: f64,
}

#[derive(Debug, Clone, Default)]
pub struct RagContext {
    pub user_memories: Vec<RankedMemory>,
    pub ai_knowledge: Vec<RankedAiKnowledge>,
}
