// Package supervisor implements the autonomous coding loop (vibe mode).
// It manages OpenCode sessions, tracks progress, and handles retries.
package supervisor

import (
	"time"
)

// Action represents supervisor actions
type Action int

const (
	ActionContinue Action = iota
	ActionStop
	ActionRetry
)

// State represents supervisor state
type State struct {
	Iteration         int
	ConsecutiveErrors int
	NoProgressCount   int
	LastRunSuccess    bool
	AlternativeCount  int
	LastError         string
}

// Settings represents supervisor configuration
type Settings struct {
	MaxIterations  int
	SessionTimeout time.Duration
	IdleTimeout    time.Duration
	Model          string
	MaxRetries     int
	MaxNoProgress  int
}

// NewState creates a new supervisor state
func NewState() *State {
	return &State{
		LastRunSuccess: true,
	}
}

// NewSettings creates default supervisor settings
func NewSettings() *Settings {
	return &Settings{
		MaxIterations:  0, // Unlimited
		SessionTimeout: 15 * time.Minute,
		IdleTimeout:    10 * time.Minute,
		MaxRetries:     3,
		MaxNoProgress:  10,
	}
}

// DetermineAction determines the next action based on state
func DetermineAction(state *State, settings *Settings, hasPassingTests bool) Action {
	// Check if we should stop
	if settings.MaxIterations > 0 && state.Iteration > settings.MaxIterations {
		return ActionStop
	}

	// Check max consecutive errors
	if state.ConsecutiveErrors >= settings.MaxRetries {
		return ActionStop
	}

	// Check max iterations without progress
	if settings.MaxNoProgress > 0 && state.NoProgressCount >= settings.MaxNoProgress {
		return ActionStop
	}

	// If we have passing tests, we made progress
	if hasPassingTests {
		state.NoProgressCount = 0
	}

	return ActionContinue
}

// ShouldRetry determines if we should retry on error
func ShouldRetry(state *State, settings *Settings) bool {
	// If we haven't made progress after many tries, stop
	if settings.MaxNoProgress > 0 && state.NoProgressCount >= settings.MaxNoProgress {
		return false
	}

	// Retry if we haven't hit max retries
	return state.ConsecutiveErrors < settings.MaxRetries
}

// IncrementError increments error counters
func (s *State) IncrementError() {
	s.ConsecutiveErrors++
	s.NoProgressCount++
	s.LastRunSuccess = false
}

// IncrementSuccess increments success counters
func (s *State) IncrementSuccess() {
	s.ConsecutiveErrors = 0
	s.NoProgressCount = 0
	s.LastRunSuccess = true
	s.Iteration++
}

// GetLastError returns the last error
func (s *State) GetLastError() string {
	return s.LastError
}

// SetError sets the last error
func (s *State) SetError(err string) {
	s.LastError = err
}

// GetIteration returns current iteration count
func (s *State) GetIteration() int {
	return s.Iteration
}
