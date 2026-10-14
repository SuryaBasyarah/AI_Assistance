CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS conversations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    role TEXT NOT NULL,
    content_id TEXT NOT NULL,
    content_ja TEXT,
    created_at TEXT NOT NULL
);

-- User-owned durable memories. AI character knowledge belongs in ai_knowledge.
CREATE TABLE IF NOT EXISTS memories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    embedding_json TEXT,
    importance REAL NOT NULL DEFAULT 0.7,
    access_count INTEGER NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 1,
    superseded_by INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS latency_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT NOT NULL,
    mode TEXT NOT NULL,
    rvc_profile_id TEXT,
    vad_to_stt_ms INTEGER,
    rag_ms INTEGER,
    llm_total_ms INTEGER,
    tts_ms INTEGER,
    rvc_ms INTEGER,
    voice_switch_ms INTEGER,
    post_speech_ttfa_ms INTEGER,
    success INTEGER NOT NULL,
    error_stage TEXT
);
