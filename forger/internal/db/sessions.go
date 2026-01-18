package db

import (
	"database/sql"
	"time"
)

// Session represents an autonomous session
type Session struct {
	ID             int
	SessionNumber  int
	StartedAt      time.Time
	CompletedAt    sql.NullTime
	FeaturesBefore int
	FeaturesAfter  int
	Status         string
}

// SessionEvent represents an event in a session
type SessionEvent struct {
	ID        int
	SessionID int
	EventType string
	Message   string
	Timestamp time.Time
}

// SessionRepository handles session database operations
type SessionRepository struct {
	db *sql.DB
}

// NewSessionRepository creates a new session repository
func NewSessionRepository(db *sql.DB) *SessionRepository {
	return &SessionRepository{db: db}
}

// Create creates a new session
func (r *SessionRepository) Create(session *Session) error {
	result, err := r.db.Exec(`
		INSERT INTO sessions (session_number, started_at, completed_at, features_before, features_after, status)
		VALUES (?, ?, ?, ?, ?, ?)
	`, session.SessionNumber, session.StartedAt, session.CompletedAt,
		session.FeaturesBefore, session.FeaturesAfter, session.Status)
	if err != nil {
		return err
	}

	id, err := result.LastInsertId()
	if err != nil {
		return err
	}

	session.ID = int(id)
	return nil
}

// GetByID retrieves a session by ID
func (r *SessionRepository) GetByID(id int) (*Session, error) {
	var s Session
	var completedAt sql.NullTime

	err := r.db.QueryRow(`
		SELECT id, session_number, started_at, completed_at, features_before, features_after, status
		FROM sessions WHERE id = ?
	`, id).Scan(
		&s.ID, &s.SessionNumber, &s.StartedAt, &completedAt,
		&s.FeaturesBefore, &s.FeaturesAfter, &s.Status,
	)
	if err != nil {
		return nil, err
	}

	s.CompletedAt = completedAt
	return &s, nil
}

// GetCurrent retrieves the most recent session
func (r *SessionRepository) GetCurrent() (*Session, error) {
	var s Session
	var completedAt sql.NullTime

	err := r.db.QueryRow(`
		SELECT id, session_number, started_at, completed_at, features_before, features_after, status
		FROM sessions ORDER BY id DESC LIMIT 1
	`).Scan(
		&s.ID, &s.SessionNumber, &s.StartedAt, &completedAt,
		&s.FeaturesBefore, &s.FeaturesAfter, &s.Status,
	)
	if err != nil {
		return nil, err
	}

	s.CompletedAt = completedAt
	return &s, nil
}

// Update updates an existing session
func (r *SessionRepository) Update(session *Session) error {
	_, err := r.db.Exec(`
		UPDATE sessions
		SET session_number = ?, started_at = ?, completed_at = ?, features_before = ?, features_after = ?, status = ?
		WHERE id = ?
	`, session.SessionNumber, session.StartedAt, session.CompletedAt,
		session.FeaturesBefore, session.FeaturesAfter, session.Status, session.ID)
	return err
}

// Complete marks a session as completed
func (r *SessionRepository) Complete(id int, featuresAfter int) error {
	_, err := r.db.Exec(`
		UPDATE sessions SET completed_at = datetime('now'), features_after = ?, status = 'completed'
		WHERE id = ?
	`, featuresAfter, id)
	return err
}

// AddEvent adds an event to a session
func (r *SessionRepository) AddEvent(event *SessionEvent) error {
	result, err := r.db.Exec(`
		INSERT INTO session_events (session_id, event_type, message, timestamp)
		VALUES (?, ?, ?, ?)
	`, event.SessionID, event.EventType, event.Message, event.Timestamp)
	if err != nil {
		return err
	}

	id, err := result.LastInsertId()
	if err != nil {
		return err
	}

	event.ID = int(id)
	return nil
}

// GetEvents retrieves all events for a session
func (r *SessionRepository) GetEvents(sessionID int) ([]SessionEvent, error) {
	rows, err := r.db.Query(`
		SELECT id, session_id, event_type, message, timestamp
		FROM session_events WHERE session_id = ? ORDER BY timestamp
	`, sessionID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var events []SessionEvent
	for rows.Next() {
		var e SessionEvent
		if err := rows.Scan(&e.ID, &e.SessionID, &e.EventType, &e.Message, &e.Timestamp); err != nil {
			return nil, err
		}
		events = append(events, e)
	}

	if err := rows.Err(); err != nil {
		return nil, err
	}

	return events, nil
}
