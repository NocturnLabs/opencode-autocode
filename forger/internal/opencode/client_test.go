package opencode

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

// MockOutputHandler for testing
type MockOutputHandler struct {
	outputs   []string
	errors    []string
	completed bool
}

func (m *MockOutputHandler) OnOutput(line string) {
	m.outputs = append(m.outputs, line)
}

func (m *MockOutputHandler) OnError(error string) {
	m.errors = append(m.errors, error)
}

func (m *MockOutputHandler) OnComplete() {
	m.completed = true
}

func TestNewClient(t *testing.T) {
	client := New("opencode")
	assert.Equal(t, "opencode", client.binPath)
	assert.InDelta(t, 30*60.0, client.timeout.Seconds(), 0.1) // Default 30 min
}

func TestSetModel(t *testing.T) {
	client := New("opencode")
	client.SetModel("custom/model")
	assert.Equal(t, "custom/model", client.model)
}

func TestSetTimeout(t *testing.T) {
	client := New("opencode")
	client.SetTimeout(5 * time.Minute) // 5 minutes
	assert.InDelta(t, 5*60.0, client.timeout.Seconds(), 0.1)
}

func TestGetModelDefault(t *testing.T) {
	client := New("opencode")
	assert.Equal(t, "opencode/glm-4.7-free", client.getModel())
}

func TestGetModelCustom(t *testing.T) {
	client := New("opencode")
	client.model = "custom/model"
	assert.Equal(t, "custom/model", client.getModel())
}

func TestTruncate(t *testing.T) {
	long := "abcdefghijklmnopqrstuvwxyz"
	result := truncateString(long, 10)
	assert.Equal(t, "abcdefghij", result)
}

func TestTruncateShort(t *testing.T) {
	short := "abc"
	result := truncateString(short, 10)
	assert.Equal(t, "abc", result)
}

func truncateString(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen]
}
