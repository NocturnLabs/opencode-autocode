package supervisor

import (
	"fmt"

	"github.com/yum-inc/opencode-forger/internal/db"
)

// Tracker handles progress tracking in the database
type Tracker struct {
	featureRepo    *db.FeatureRepository
	sessionRepo    *db.SessionRepository
	currentSession *db.Session
}

// NewTracker creates a new progress tracker
func NewTracker(featureRepo *db.FeatureRepository, sessionRepo *db.SessionRepository) *Tracker {
	return &Tracker{
		featureRepo: featureRepo,
		sessionRepo: sessionRepo,
	}
}

// StartSession creates and starts a new session
func (t *Tracker) StartSession(sessionNumber int) error {
	session := &db.Session{
		SessionNumber: sessionNumber,
		Status:        "running",
	}

	if err := t.sessionRepo.Create(session); err != nil {
		return fmt.Errorf("failed to create session: %w", err)
	}

	t.currentSession = session
	return nil
}

// CompleteSession marks the current session as completed.
// It fetches the current passing count and stores it.
func (t *Tracker) CompleteSession() error {
	if t.currentSession == nil {
		return nil
	}

	featuresAfter, err := t.featureRepo.GetPassingCount()
	if err != nil {
		return fmt.Errorf("failed to get passing count: %w", err)
	}

	if err := t.sessionRepo.Complete(t.currentSession.ID, featuresAfter); err != nil {
		return fmt.Errorf("failed to complete session: %w", err)
	}

	return nil
}

// UpdateFeatureSuccess updates a feature as passing
func (t *Tracker) UpdateFeatureSuccess(featureID int, verificationCommand string) error {
	// Update passes count
	if err := t.featureRepo.IncrementPasses(featureID); err != nil {
		return fmt.Errorf("failed to increment passes: %w", err)
	}

	// Log session event
	t.logEvent("feature_pass", fmt.Sprintf("Feature %d passed verification", featureID))
	return nil
}

// UpdateFeatureError updates a feature with an error
func (t *Tracker) UpdateFeatureError(featureID int, errorMsg string) error {
	// Update error
	if err := t.featureRepo.UpdateError(featureID, errorMsg); err != nil {
		return fmt.Errorf("failed to update error: %w", err)
	}

	// Log session event
	t.logEvent("feature_error", fmt.Sprintf("Feature %d failed: %s", featureID, errorMsg))
	return nil
}

// GetPassingCount returns the number of passing features
func (t *Tracker) GetPassingCount() (int, error) {
	return t.featureRepo.GetPassingCount()
}

// GetFailingCount returns the number of failing features
func (t *Tracker) GetFailingCount() (int, error) {
	return t.featureRepo.GetFailingCount()
}

// GetAllFeatures returns all features
func (t *Tracker) GetAllFeatures() ([]db.Feature, error) {
	return t.featureRepo.GetAll()
}

// GetNextFeature returns the next feature to work on (failing first)
func (t *Tracker) GetNextFeature() (*db.Feature, error) {
	features, err := t.featureRepo.GetAll()
	if err != nil {
		return nil, err
	}

	// Return first failing feature
	for _, feature := range features {
		if feature.Passes == 0 {
			return &feature, nil
		}
	}

	// If all passing, return nil (loop complete)
	return nil, nil
}

// logEvent logs a session event
func (t *Tracker) logEvent(eventType, message string) error {
	if t.currentSession == nil {
		return nil
	}

	event := &db.SessionEvent{
		SessionID: t.currentSession.ID,
		EventType: eventType,
		Message:   message,
	}

	return t.sessionRepo.AddEvent(event)
}

// AddEvent adds a custom event to the current session
func (t *Tracker) AddEvent(eventType, message string) error {
	return t.logEvent(eventType, message)
}
