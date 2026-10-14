CREATE TABLE IF NOT EXISTS voice_profiles (
    profile_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    pitch INTEGER NOT NULL DEFAULT 0,
    f0_method TEXT NOT NULL DEFAULT 'rmvpe',
    index_rate REAL NOT NULL DEFAULT 0.75,
    protect REAL NOT NULL DEFAULT 0.33,
    enabled INTEGER NOT NULL DEFAULT 1,
    last_seen_at TEXT,
    updated_at TEXT NOT NULL
);

INSERT INTO settings(key, value, updated_at)
VALUES ('selected_rvc_profile', 'mizuki', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
ON CONFLICT(key) DO NOTHING;

INSERT INTO settings(key, value, updated_at)
VALUES ('allow_raw_tts_fallback', 'true', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
ON CONFLICT(key) DO NOTHING;
