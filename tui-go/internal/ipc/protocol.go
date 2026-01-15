// Package ipc defines the JSON-RPC-style protocol for communication between
// the Rust engine and the Go TUI client.
package ipc

// ProtocolVersion is the current version of the IPC protocol.
// Both sides should verify this matches to avoid incompatibilities.
const ProtocolVersion = "1.0.0"

// Direction indicates whether a message flows from Rust to Go or vice versa.
type Direction string

const (
	DirectionRustToGo Direction = "rust->go"
	DirectionGoToRust Direction = "go->rust"
)

// MessageType classifies messages as either events (notifications) or commands (requests).
type MessageType string

const (
	MessageTypeEvent   MessageType = "event"
	MessageTypeCommand MessageType = "command"
)

// Message is the envelope for all IPC messages exchanged over stdin/stdout.
// Messages are newline-delimited JSON (NDJSON).
type Message struct {
	ProtocolVersion string      `json:"protocol_version"`
	Direction       Direction   `json:"direction"`
	Type            MessageType `json:"type"`
	Name            string      `json:"name"`
	Payload         interface{} `json:"payload,omitempty"`
}

// Event names (Rust -> Go)
const (
	EventEngineReady    = "EngineReady"
	EventLogLine        = "LogLine"
	EventProgressUpdate = "ProgressUpdate"
	EventUserPrompt     = "UserPrompt"
	EventFinished       = "Finished"
	EventError          = "Error"
	EventModeList       = "ModeList"
	EventConfigLoaded   = "ConfigLoaded"
)

// Command names (Go -> Rust)
const (
	CommandStartVibe       = "StartVibe"
	CommandHandleSelection = "HandleSelection"
	CommandCancel          = "Cancel"
	CommandOpenConfig      = "OpenConfig"
	CommandRetry           = "Retry"
	CommandSelectMode      = "SelectMode"
	CommandConfirm         = "Confirm"
)

// --- Event Payloads (Rust -> Go) ---

// EngineReadyPayload is sent when the Rust engine is ready to accept commands.
type EngineReadyPayload struct {
	Version string `json:"version"`
	WorkDir string `json:"work_dir"`
}

// LogLinePayload represents a log message from the engine.
type LogLinePayload struct {
	Level     string `json:"level"`
	Message   string `json:"message"`
	Timestamp string `json:"timestamp,omitempty"`
}

// ProgressUpdatePayload indicates progress on a task.
type ProgressUpdatePayload struct {
	Phase      string  `json:"phase"`
	Current    int     `json:"current"`
	Total      int     `json:"total"`
	Message    string  `json:"message,omitempty"`
	Percentage float64 `json:"percentage,omitempty"`
}

// UserPromptPayload asks the user to make a selection or provide input.
type UserPromptPayload struct {
	PromptID    string   `json:"prompt_id"`
	PromptType  string   `json:"prompt_type"` // "select", "confirm", "input"
	Message     string   `json:"message"`
	Options     []string `json:"options,omitempty"`
	Default     string   `json:"default,omitempty"`
	AllowCancel bool     `json:"allow_cancel,omitempty"`
}

// FinishedPayload signals that the engine has completed its work.
type FinishedPayload struct {
	Success bool   `json:"success"`
	Message string `json:"message,omitempty"`
}

// ErrorPayload reports an error from the engine.
type ErrorPayload struct {
	Code    string `json:"code,omitempty"`
	Message string `json:"message"`
	Fatal   bool   `json:"fatal,omitempty"`
}

// ModeListPayload provides the available interactive modes.
type ModeListPayload struct {
	Modes []ModeInfo `json:"modes"`
}

// ModeInfo describes an interactive mode option.
type ModeInfo struct {
	ID          string `json:"id"`
	Label       string `json:"label"`
	Description string `json:"description"`
}

// ConfigLoadedPayload indicates configuration has been loaded.
type ConfigLoadedPayload struct {
	HasExistingConfig bool   `json:"has_existing_config"`
	ConfigPath        string `json:"config_path,omitempty"`
}

// --- Command Payloads (Go -> Rust) ---

// SelectModePayload tells the engine which mode the user selected.
type SelectModePayload struct {
	ModeID string `json:"mode_id"`
}

// HandleSelectionPayload responds to a UserPrompt with the user's choice.
type HandleSelectionPayload struct {
	PromptID string `json:"prompt_id"`
	Value    string `json:"value"`
	Index    int    `json:"index,omitempty"`
}

// ConfirmPayload responds to a confirm prompt.
type ConfirmPayload struct {
	PromptID  string `json:"prompt_id"`
	Confirmed bool   `json:"confirmed"`
}

// CancelPayload requests cancellation of the current operation.
type CancelPayload struct {
	Reason string `json:"reason,omitempty"`
}

// StartVibePayload initiates the vibe loop.
type StartVibePayload struct {
	Limit       int  `json:"limit,omitempty"`
	SingleModel bool `json:"single_model,omitempty"`
	Parallel    int  `json:"parallel,omitempty"`
}
