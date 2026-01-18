// Package db provides SQLite database access for the forger application.
// It includes repositories for features, sessions, knowledge base, and metadata.
package db

import (
	"database/sql"
	"fmt"
	"os"
	"path/filepath"

	_ "github.com/mattn/go-sqlite3"
)

const schema = `
-- Features table (replaces feature_list.json)
CREATE TABLE IF NOT EXISTS features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category TEXT NOT NULL,
    description TEXT NOT NULL UNIQUE,
    passes INTEGER DEFAULT 0,
    verification_command TEXT,
    last_error TEXT,
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

-- Key-Value metadata storage (e.g. for discord_message_id)
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Agent Knowledge Base (Persistent Facts)
CREATE TABLE IF NOT EXISTS knowledge (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    category TEXT DEFAULT 'general',
    description TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Trigger to update updated_at on knowledge changes
CREATE TRIGGER IF NOT EXISTS update_knowledge_timestamp
    AFTER UPDATE ON knowledge
    FOR EACH ROW
BEGIN
    UPDATE knowledge SET updated_at = datetime('now') WHERE key = NEW.key;
END;

-- Active Instances (Control Panel)
CREATE TABLE IF NOT EXISTS instances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pid INTEGER NOT NULL,
    role TEXT NOT NULL, -- 'supervisor', 'worker', 'web'
    start_time TEXT DEFAULT (datetime('now')),
    status TEXT DEFAULT 'running', -- 'running', 'stopped', 'error'
    log_path TEXT,
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Trigger to update updated_at on instance changes
CREATE TRIGGER IF NOT EXISTS update_instances_timestamp
    AFTER UPDATE ON instances
    FOR EACH ROW
BEGIN
    UPDATE instances SET updated_at = datetime('now') WHERE id = NEW.id;
END;
`

// Database represents a SQLite database connection
type Database struct {
	db *sql.DB
}

// New creates a new database connection and initializes the schema
func New(path string) (*Database, error) {
	// Create directory if needed
	dir := filepath.Dir(path)
	if dir != "." {
		if err := os.MkdirAll(dir, 0755); err != nil {
			return nil, fmt.Errorf("failed to create database directory: %w", err)
		}
	}

	// Open database connection
	db, err := sql.Open("sqlite3", path)
	if err != nil {
		return nil, fmt.Errorf("failed to open database: %w", err)
	}

	// Initialize schema
	if _, err := db.Exec(schema); err != nil {
		db.Close()
		return nil, fmt.Errorf("failed to initialize schema: %w", err)
	}

	return &Database{db: db}, nil
}

// Close closes the database connection
func (d *Database) Close() error {
	return d.db.Close()
}

// DB returns the underlying sql.DB for use by repositories
func (d *Database) DB() *sql.DB {
	return d.db
}
