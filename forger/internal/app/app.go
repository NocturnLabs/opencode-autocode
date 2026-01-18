package app

import (
	tea "github.com/charmbracelet/bubbletea"
	"github.com/yum-inc/opencode-forger/internal/ui"
)

// Run is the entry point for the application
func Run() error {
	// Create initial model
	model := ui.New()

	// Create Bubble Tea program
	p := tea.NewProgram(model, tea.WithAltScreen())

	if _, err := p.Run(); err != nil {
		return err
	}

	return nil
}
