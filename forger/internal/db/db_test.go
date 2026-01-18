package db

import (
	"database/sql"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestDatabaseNew(t *testing.T) {
	// Create temp directory
	tmpDir, err := os.MkdirTemp("", "forger-db-test")
	require.NoError(t, err)
	defer os.RemoveAll(tmpDir)

	dbPath := filepath.Join(tmpDir, "test.db")

	// Create database
	db, err := New(dbPath)
	require.NoError(t, err)
	defer db.Close()

	// Verify file exists
	_, err = os.Stat(dbPath)
	assert.NoError(t, err)
}

func TestFeatureRepository(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "forger-db-test")
	require.NoError(t, err)
	defer os.RemoveAll(tmpDir)

	db, err := New(filepath.Join(tmpDir, "test.db"))
	require.NoError(t, err)
	defer db.Close()

	repo := NewFeatureRepository(db.DB())

	// Create feature
	feature := &Feature{
		Category:            "core",
		Description:         "Test feature",
		Passes:              0,
		VerificationCommand: "go test",
	}

	err = repo.Create(feature)
	require.NoError(t, err)
	assert.Greater(t, feature.ID, 0)

	// Get by ID
	retrieved, err := repo.GetByID(feature.ID)
	require.NoError(t, err)
	assert.Equal(t, "Test feature", retrieved.Description)
	assert.Equal(t, "core", retrieved.Category)

	// Get all
	all, err := repo.GetAll()
	require.NoError(t, err)
	assert.Len(t, all, 1)

	// Increment passes
	err = repo.IncrementPasses(feature.ID)
	require.NoError(t, err)

	retrieved, err = repo.GetByID(feature.ID)
	require.NoError(t, err)
	assert.Equal(t, 1, retrieved.Passes)

	// Get passing count
	count, err := repo.GetPassingCount()
	require.NoError(t, err)
	assert.Equal(t, 1, count)

	// Get failing count
	count, err = repo.GetFailingCount()
	require.NoError(t, err)
	assert.Equal(t, 0, count)

	// Update error
	err = repo.UpdateError(feature.ID, "test error")
	require.NoError(t, err)

	retrieved, err = repo.GetByID(feature.ID)
	require.NoError(t, err)
	assert.True(t, retrieved.LastError.Valid)
	assert.Equal(t, "test error", retrieved.LastError.String)

	// Delete
	err = repo.Delete(feature.ID)
	require.NoError(t, err)

	_, err = repo.GetByID(feature.ID)
	assert.Error(t, err)
}

func TestSessionRepository(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "forger-db-test")
	require.NoError(t, err)
	defer os.RemoveAll(tmpDir)

	db, err := New(filepath.Join(tmpDir, "test.db"))
	require.NoError(t, err)
	defer db.Close()

	repo := NewSessionRepository(db.DB())

	// Create session
	session := &Session{
		SessionNumber:  1,
		StartedAt:      time.Now(),
		FeaturesBefore: 5,
		FeaturesAfter:  0,
		Status:         "running",
	}

	err = repo.Create(session)
	require.NoError(t, err)
	assert.Greater(t, session.ID, 0)

	// Get by ID
	retrieved, err := repo.GetByID(session.ID)
	require.NoError(t, err)
	assert.Equal(t, 1, retrieved.SessionNumber)
	assert.Equal(t, "running", retrieved.Status)

	// Get current
	current, err := repo.GetCurrent()
	require.NoError(t, err)
	assert.Equal(t, session.ID, current.ID)

	// Complete session
	err = repo.Complete(session.ID, 10)
	require.NoError(t, err)

	retrieved, err = repo.GetByID(session.ID)
	require.NoError(t, err)
	assert.Equal(t, "completed", retrieved.Status)
	assert.Equal(t, 10, retrieved.FeaturesAfter)

	// Add event
	event := &SessionEvent{
		SessionID: session.ID,
		EventType: "test_event",
		Message:   "Test message",
		Timestamp: time.Now(),
	}

	err = repo.AddEvent(event)
	require.NoError(t, err)
	assert.Greater(t, event.ID, 0)

	// Get events
	events, err := repo.GetEvents(session.ID)
	require.NoError(t, err)
	assert.Len(t, events, 1)
	assert.Equal(t, "test_event", events[0].EventType)
}

func TestKnowledgeRepository(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "forger-db-test")
	require.NoError(t, err)
	defer os.RemoveAll(tmpDir)

	db, err := New(filepath.Join(tmpDir, "test.db"))
	require.NoError(t, err)
	defer db.Close()

	repo := NewKnowledgeRepository(db.DB())

	// Create knowledge entry
	knowledge := &Knowledge{
		Key:      "test_key",
		Value:    "test_value",
		Category: "test",
		Description: sql.NullString{
			String: "Test description",
			Valid:  true,
		},
	}

	err = repo.Create(knowledge)
	require.NoError(t, err)

	// Get by key
	retrieved, err := repo.GetByKey("test_key")
	require.NoError(t, err)
	assert.Equal(t, "test_value", retrieved.Value)
	assert.Equal(t, "test", retrieved.Category)

	// Get all
	all, err := repo.GetAll()
	require.NoError(t, err)
	assert.Len(t, all, 1)

	// Update
	knowledge.Value = "updated_value"
	err = repo.Update(knowledge)
	require.NoError(t, err)

	retrieved, err = repo.GetByKey("test_key")
	require.NoError(t, err)
	assert.Equal(t, "updated_value", retrieved.Value)

	// Delete
	err = repo.Delete("test_key")
	require.NoError(t, err)

	_, err = repo.GetByKey("test_key")
	assert.Error(t, err)
}

func TestMetaRepository(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "forger-db-test")
	require.NoError(t, err)
	defer os.RemoveAll(tmpDir)

	db, err := New(filepath.Join(tmpDir, "test.db"))
	require.NoError(t, err)
	defer db.Close()

	repo := NewMetaRepository(db.DB())

	// Set value
	err = repo.Set("test_key", "test_value")
	require.NoError(t, err)

	// Get value
	value, err := repo.Get("test_key")
	require.NoError(t, err)
	assert.Equal(t, "test_value", value)

	// Get non-existent key returns empty string
	value, err = repo.Get("nonexistent")
	require.NoError(t, err)
	assert.Equal(t, "", value)

	// Update existing value
	err = repo.Set("test_key", "updated_value")
	require.NoError(t, err)

	value, err = repo.Get("test_key")
	require.NoError(t, err)
	assert.Equal(t, "updated_value", value)

	// Get all
	all, err := repo.GetAll()
	require.NoError(t, err)
	assert.Len(t, all, 1)

	// Delete
	err = repo.Delete("test_key")
	require.NoError(t, err)

	value, err = repo.Get("test_key")
	require.NoError(t, err)
	assert.Equal(t, "", value)
}
