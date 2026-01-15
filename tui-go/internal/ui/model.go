// Package ui provides the main Bubble Tea model for the TUI.
package ui

import (
	"github.com/charmbracelet/bubbles/spinner"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/yumlabs-tools/opencode-forger/tui-go/internal/ipc"
)

// Phase represents the current phase of the application.
type Phase int

const (
	PhaseConnecting Phase = iota
	PhaseSetupChoice
	PhaseModeSelection
	PhaseProgress
	PhasePrompt
	PhaseFinished
	PhaseError
)

// ModeOption represents a selectable mode option.
type ModeOption struct {
	ID          string
	Label       string
	Description string
}

// Model is the main Bubble Tea model for the TUI.
type Model struct {
	// IPC communication
	ipcClient *ipc.Client
	msgChan   chan *ipc.Message

	// UI state
	phase    Phase
	width    int
	height   int
	styles   *Styles
	spinner  spinner.Model
	ready    bool
	quitting bool
	err      error

	// Setup phase state
	hasExistingConfig bool
	setupChoice       int // 0: quick start, 1: configure
	reconfigure       bool

	// Mode selection state
	modes        []ModeOption
	selectedMode int

	// Progress state
	progressPhase   string
	progressCurrent int
	progressTotal   int
	progressMessage string

	// Prompt state
	currentPrompt *ipc.UserPromptPayload
	promptChoice  int

	// Log buffer (circular buffer of last N log lines)
	logs    []string
	maxLogs int

	// Result state
	finished bool
	success  bool
	message  string
}

// NewModel creates a new Model with the given IPC client.
func NewModel(client *ipc.Client) Model {
	s := spinner.New()
	s.Spinner = spinner.Dot

	return Model{
		ipcClient:    client,
		msgChan:      make(chan *ipc.Message, 100),
		phase:        PhaseConnecting,
		styles:       NewStyles(),
		spinner:      s,
		modes:        []ModeOption{},
		logs:         make([]string, 0, 100),
		maxLogs:      100,
		setupChoice:  0,
		selectedMode: 0,
		promptChoice: 0,
	}
}

// IpcMsg wraps an IPC message for the Bubble Tea message system.
type IpcMsg struct {
	Message *ipc.Message
}

// ErrorMsg represents an error that occurred.
type ErrorMsg struct {
	Err error
}

// Init initializes the model and starts the IPC read loop.
func (m Model) Init() tea.Cmd {
	return tea.Batch(
		m.spinner.Tick,
		m.listenForIpc(),
	)
}

// listenForIpc creates a command that reads from the IPC channel.
func (m Model) listenForIpc() tea.Cmd {
	return func() tea.Msg {
		// Start the IPC read loop in a goroutine
		go func() {
			if err := m.ipcClient.ReadLoop(m.msgChan); err != nil {
				m.msgChan <- nil // Signal error/close
			}
		}()

		// Wait for the first message
		msg := <-m.msgChan
		if msg == nil {
			return ErrorMsg{Err: nil}
		}
		return IpcMsg{Message: msg}
	}
}

// waitForIpc creates a command that waits for the next IPC message.
func (m Model) waitForIpc() tea.Cmd {
	return func() tea.Msg {
		msg := <-m.msgChan
		if msg == nil {
			return ErrorMsg{Err: nil}
		}
		return IpcMsg{Message: msg}
	}
}

// Update handles messages and updates the model state.
func (m Model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmds []tea.Cmd

	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height
		m.ready = true
		return m, nil

	case tea.KeyMsg:
		return m.handleKey(msg)

	case spinner.TickMsg:
		var cmd tea.Cmd
		m.spinner, cmd = m.spinner.Update(msg)
		cmds = append(cmds, cmd)

	case IpcMsg:
		return m.handleIpcMessage(msg.Message)

	case ErrorMsg:
		if msg.Err != nil {
			m.err = msg.Err
			m.phase = PhaseError
		}
		return m, nil
	}

	return m, tea.Batch(cmds...)
}

// handleKey processes keyboard input based on the current phase.
func (m Model) handleKey(msg tea.KeyMsg) (tea.Model, tea.Cmd) {
	switch msg.String() {
	case "q", "ctrl+c":
		m.quitting = true
		// Send cancel command to Rust
		_ = m.ipcClient.SendCommand(ipc.CommandCancel, ipc.CancelPayload{Reason: "user quit"})
		return m, tea.Quit

	case "up", "k":
		m.navigateUp()

	case "down", "j":
		m.navigateDown()

	case "left", "h":
		if m.phase == PhaseSetupChoice && m.hasExistingConfig {
			m.reconfigure = true
		}

	case "right", "l":
		if m.phase == PhaseSetupChoice && m.hasExistingConfig {
			m.reconfigure = false
		}

	case "y", "Y":
		if m.phase == PhaseSetupChoice && m.hasExistingConfig {
			m.reconfigure = true
			return m.confirmSetupChoice()
		}

	case "n", "N":
		if m.phase == PhaseSetupChoice && m.hasExistingConfig {
			m.reconfigure = false
			return m.confirmSetupChoice()
		}

	case "enter":
		return m.handleEnter()
	}

	return m, nil
}

// navigateUp moves the selection cursor up.
func (m *Model) navigateUp() {
	switch m.phase {
	case PhaseSetupChoice:
		if !m.hasExistingConfig && m.setupChoice > 0 {
			m.setupChoice--
		}
	case PhaseModeSelection:
		if m.selectedMode > 0 {
			m.selectedMode--
		}
	case PhasePrompt:
		if m.promptChoice > 0 {
			m.promptChoice--
		}
	}
}

// navigateDown moves the selection cursor down.
func (m *Model) navigateDown() {
	switch m.phase {
	case PhaseSetupChoice:
		if !m.hasExistingConfig && m.setupChoice < 1 {
			m.setupChoice++
		}
	case PhaseModeSelection:
		if m.selectedMode < len(m.modes)-1 {
			m.selectedMode++
		}
	case PhasePrompt:
		if m.currentPrompt != nil && m.promptChoice < len(m.currentPrompt.Options)-1 {
			m.promptChoice++
		}
	}
}

// handleEnter processes the enter key based on current phase.
func (m Model) handleEnter() (tea.Model, tea.Cmd) {
	switch m.phase {
	case PhaseSetupChoice:
		return m.confirmSetupChoice()

	case PhaseModeSelection:
		if len(m.modes) > 0 {
			mode := m.modes[m.selectedMode]
			err := m.ipcClient.SendCommand(ipc.CommandSelectMode, ipc.SelectModePayload{
				ModeID: mode.ID,
			})
			if err != nil {
				m.err = err
				m.phase = PhaseError
			} else {
				m.phase = PhaseProgress
			}
		}
		return m, m.waitForIpc()

	case PhasePrompt:
		return m.handlePromptSelection()
	}

	return m, nil
}

// confirmSetupChoice confirms the setup phase selection.
func (m Model) confirmSetupChoice() (tea.Model, tea.Cmd) {
	// Send configuration choice to Rust
	shouldConfigure := m.setupChoice == 1 || (m.hasExistingConfig && m.reconfigure)
	err := m.ipcClient.SendCommand(ipc.CommandConfirm, ipc.ConfirmPayload{
		PromptID:  "setup_choice",
		Confirmed: shouldConfigure,
	})
	if err != nil {
		m.err = err
		m.phase = PhaseError
		return m, nil
	}

	m.phase = PhaseModeSelection
	return m, m.waitForIpc()
}

// handlePromptSelection handles a prompt response.
func (m Model) handlePromptSelection() (tea.Model, tea.Cmd) {
	if m.currentPrompt == nil {
		return m, nil
	}

	var value string
	if len(m.currentPrompt.Options) > 0 && m.promptChoice < len(m.currentPrompt.Options) {
		value = m.currentPrompt.Options[m.promptChoice]
	}

	err := m.ipcClient.SendCommand(ipc.CommandHandleSelection, ipc.HandleSelectionPayload{
		PromptID: m.currentPrompt.PromptID,
		Value:    value,
		Index:    m.promptChoice,
	})
	if err != nil {
		m.err = err
		m.phase = PhaseError
		return m, nil
	}

	m.currentPrompt = nil
	m.phase = PhaseProgress
	return m, m.waitForIpc()
}

// handleIpcMessage processes incoming IPC messages from the Rust engine.
func (m Model) handleIpcMessage(msg *ipc.Message) (tea.Model, tea.Cmd) {
	if msg == nil {
		return m, nil
	}

	switch msg.Name {
	case ipc.EventEngineReady:
		m.phase = PhaseSetupChoice
		if payload, err := ipc.ParsePayload[ipc.EngineReadyPayload](msg.Payload); err == nil {
			m.addLog("Engine ready: " + payload.Version)
		}

	case ipc.EventConfigLoaded:
		if payload, err := ipc.ParsePayload[ipc.ConfigLoadedPayload](msg.Payload); err == nil {
			m.hasExistingConfig = payload.HasExistingConfig
		}

	case ipc.EventModeList:
		if payload, err := ipc.ParsePayload[ipc.ModeListPayload](msg.Payload); err == nil {
			m.modes = make([]ModeOption, len(payload.Modes))
			for i, mode := range payload.Modes {
				m.modes[i] = ModeOption{
					ID:          mode.ID,
					Label:       mode.Label,
					Description: mode.Description,
				}
			}
			m.phase = PhaseModeSelection
		}

	case ipc.EventLogLine:
		if payload, err := ipc.ParsePayload[ipc.LogLinePayload](msg.Payload); err == nil {
			m.addLog(payload.Message)
		}

	case ipc.EventProgressUpdate:
		if payload, err := ipc.ParsePayload[ipc.ProgressUpdatePayload](msg.Payload); err == nil {
			m.progressPhase = payload.Phase
			m.progressCurrent = payload.Current
			m.progressTotal = payload.Total
			m.progressMessage = payload.Message
		}

	case ipc.EventUserPrompt:
		if payload, err := ipc.ParsePayload[ipc.UserPromptPayload](msg.Payload); err == nil {
			m.currentPrompt = payload
			m.promptChoice = 0
			m.phase = PhasePrompt
		}

	case ipc.EventFinished:
		if payload, err := ipc.ParsePayload[ipc.FinishedPayload](msg.Payload); err == nil {
			m.finished = true
			m.success = payload.Success
			m.message = payload.Message
			m.phase = PhaseFinished
		}

	case ipc.EventError:
		if payload, err := ipc.ParsePayload[ipc.ErrorPayload](msg.Payload); err == nil {
			m.addLog("Error: " + payload.Message)
			if payload.Fatal {
				m.err = nil // Use message field instead
				m.message = payload.Message
				m.phase = PhaseError
			}
		}
	}

	return m, m.waitForIpc()
}

// addLog adds a log message to the circular buffer.
func (m *Model) addLog(msg string) {
	if len(m.logs) >= m.maxLogs {
		m.logs = m.logs[1:]
	}
	m.logs = append(m.logs, msg)
}

// View renders the current state of the model.
func (m Model) View() string {
	if !m.ready {
		return "\n  Initializing..."
	}

	if m.quitting {
		return "\n  Goodbye!\n"
	}

	switch m.phase {
	case PhaseConnecting:
		return m.viewConnecting()
	case PhaseSetupChoice:
		return m.viewSetupChoice()
	case PhaseModeSelection:
		return m.viewModeSelection()
	case PhaseProgress:
		return m.viewProgress()
	case PhasePrompt:
		return m.viewPrompt()
	case PhaseFinished:
		return m.viewFinished()
	case PhaseError:
		return m.viewError()
	default:
		return "Unknown phase"
	}
}
