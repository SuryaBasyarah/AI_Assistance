-- Character/assistant knowledge is intentionally separate from user memory.
CREATE TABLE IF NOT EXISTS ai_knowledge (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    source_kind TEXT NOT NULL CHECK(source_kind IN ('official', 'story_summary', 'analysis')),
    source_reference TEXT,
    embedding_json TEXT,
    importance REAL NOT NULL DEFAULT 0.8,
    access_count INTEGER NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ai_knowledge_active_category
ON ai_knowledge(active, category);
