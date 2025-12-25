//! Database schema definition

/// SQL schema for all tables
pub const SCHEMA: &str = r#"
-- Features table (replaces feature_list.json)
CREATE TABLE IF NOT EXISTS features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category TEXT NOT NULL,
    description TEXT NOT NULL UNIQUE,
    passes INTEGER DEFAULT 0,
    verification_command TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Feature verification steps
CREATE TABLE IF NOT EXISTS feature_steps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id INTEGER NOT NULL,
    step_order INTEGER NOT NULL,
    step_text TEXT NOT NULL,
    FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE
);

-- Autonomous sessions
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_number INTEGER NOT NULL,
    started_at TEXT DEFAULT (datetime('now')),
    completed_at TEXT,
    features_before INTEGER DEFAULT 0,
    features_after INTEGER DEFAULT 0,
    status TEXT DEFAULT 'running'
);

-- Session events/logs
CREATE TABLE IF NOT EXISTS session_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    message TEXT,
    timestamp TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_features_passes ON features(passes);
CREATE INDEX IF NOT EXISTS idx_features_category ON features(category);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_session_events_session ON session_events(session_id);

-- Trigger to update updated_at on feature changes
CREATE TRIGGER IF NOT EXISTS update_feature_timestamp
    AFTER UPDATE ON features
    FOR EACH ROW
BEGIN
    UPDATE features SET updated_at = datetime('now') WHERE id = NEW.id;
END;
"#;
