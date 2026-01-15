// Package ui provides view rendering methods for the Bubble Tea model.
package ui

import (
	"fmt"
	"strings"
)

// viewConnecting renders the connecting/initialization phase.
func (m Model) viewConnecting() string {
	s := m.styles

	header := s.Header.Render(
		s.Title.Render("OpenCode Forger") + "  " +
			s.MutedText.Render(m.spinner.View()+" Connecting to engine..."),
	)

	content := s.Content.Render(
		s.MutedText.Render("Waiting for the Rust engine to initialize..."),
	)

	footer := s.Footer.Render("Press q to quit")

	return header + "\n" + content + "\n" + footer
}

// viewSetupChoice renders the initial setup choice screen.
func (m Model) viewSetupChoice() string {
	s := m.styles

	// Header with progress
	progress := fmt.Sprintf("Step 1 of 2  %s %s", SymbolRunning, SymbolPending)
	header := s.Header.Render(
		s.Title.Render("OpenCode Forger") + strings.Repeat(" ", 10) +
			s.MutedText.Render(progress),
	)

	var content string
	if m.hasExistingConfig {
		// Reconfigure yes/no choice
		yesStyle := s.MenuItem
		noStyle := s.MenuItem
		if m.reconfigure {
			yesStyle = s.MenuItemSelected
		} else {
			noStyle = s.MenuItemSelected
		}

		content = s.Content.Render(
			s.Title.Render("Existing configuration found.") + "\n\n" +
				s.Label.Render("Reconfigure? ") +
				yesStyle.Render("[Yes]") + " / " + noStyle.Render("[No]") + "\n\n" +
				s.MutedText.Render("←/→ toggle, Enter confirm"),
		)
	} else {
		// Quick start vs Configure choice
		var items strings.Builder
		choices := []string{"Quick start (use defaults)", "Configure settings first"}

		for i, choice := range choices {
			prefix := SymbolUnselected + " "
			style := s.MenuItem
			if i == m.setupChoice {
				prefix = SymbolSelected + " "
				style = s.MenuItemSelected
			}
			items.WriteString(prefix + style.Render(choice) + "\n")
		}

		content = s.Content.Render(
			s.Title.Render("Setup Mode") + "\n\n" +
				items.String(),
		)
	}

	footer := s.Footer.Render("↑↓ Navigate  Enter Select  q Quit")

	return header + "\n" + content + "\n" + footer
}

// viewModeSelection renders the mode selection screen.
func (m Model) viewModeSelection() string {
	s := m.styles

	// Header with progress
	progress := fmt.Sprintf("Step 2 of 2  %s %s", SymbolComplete, SymbolRunning)
	header := s.Header.Render(
		s.Title.Render("OpenCode Forger") + strings.Repeat(" ", 10) +
			s.MutedText.Render(progress),
	)

	// Mode cards
	var modes strings.Builder
	for i, mode := range m.modes {
		isSelected := i == m.selectedMode
		boxStyle := s.Box
		if isSelected {
			boxStyle = s.BoxFocused
		}

		prefix := SymbolUnselected + " "
		labelStyle := s.MenuItem
		if isSelected {
			prefix = SymbolSelected + " "
			labelStyle = s.MenuItemSelected
		}

		modeCard := boxStyle.Render(
			prefix + labelStyle.Render(mode.Label) + "\n" +
				"   " + s.MutedText.Render(mode.Description),
		)
		modes.WriteString(modeCard + "\n")
	}

	content := s.Content.Render(
		s.Title.Render("Select Project Mode") + "\n\n" +
			modes.String(),
	)

	footer := s.Footer.Render("↑↓ Navigate  Enter Select  q Quit")

	return header + "\n" + content + "\n" + footer
}

// viewProgress renders the progress/working screen.
func (m Model) viewProgress() string {
	s := m.styles

	header := s.Header.Render(
		s.Title.Render("OpenCode Forger") + "  " +
			s.StatusIndicator.Render(m.spinner.View()+" Working..."),
	)

	// Progress bar
	var progressBar string
	if m.progressTotal > 0 {
		pct := float64(m.progressCurrent) / float64(m.progressTotal)
		barWidth := 40
		filled := int(pct * float64(barWidth))
		empty := barWidth - filled

		bar := s.ProgressFilled.Render(strings.Repeat("█", filled)) +
			s.ProgressBar.Render(strings.Repeat("░", empty))

		progressBar = fmt.Sprintf("\n%s [%s] %d/%d (%.0f%%)\n",
			m.progressPhase, bar, m.progressCurrent, m.progressTotal, pct*100)
	}

	// Current message
	message := ""
	if m.progressMessage != "" {
		message = s.Label.Render(m.progressMessage) + "\n\n"
	}

	// Recent logs
	var logs strings.Builder
	logs.WriteString(s.Subtitle.Render("Recent Activity:") + "\n")
	start := 0
	if len(m.logs) > 10 {
		start = len(m.logs) - 10
	}
	for i := start; i < len(m.logs); i++ {
		logs.WriteString(s.MutedText.Render("  "+m.logs[i]) + "\n")
	}

	content := s.Content.Render(
		progressBar +
			message +
			logs.String(),
	)

	footer := s.Footer.Render("q Cancel")

	return header + "\n" + content + "\n" + footer
}

// viewPrompt renders a user prompt screen.
func (m Model) viewPrompt() string {
	s := m.styles

	header := s.Header.Render(
		s.Title.Render("OpenCode Forger"),
	)

	if m.currentPrompt == nil {
		return header + "\n" + s.Content.Render("Loading...")
	}

	var options strings.Builder
	for i, opt := range m.currentPrompt.Options {
		prefix := SymbolUnselected + " "
		style := s.MenuItem
		if i == m.promptChoice {
			prefix = SymbolSelected + " "
			style = s.MenuItemSelected
		}
		options.WriteString(prefix + style.Render(opt) + "\n")
	}

	content := s.Content.Render(
		s.Title.Render(m.currentPrompt.Message) + "\n\n" +
			options.String(),
	)

	footer := s.Footer.Render("↑↓ Navigate  Enter Select  q Cancel")

	return header + "\n" + content + "\n" + footer
}

// viewFinished renders the completion screen.
func (m Model) viewFinished() string {
	s := m.styles

	statusIcon := SymbolSuccess
	statusStyle := s.SuccessText
	statusMsg := "Complete!"
	if !m.success {
		statusIcon = SymbolError
		statusStyle = s.ErrorText
		statusMsg = "Failed"
	}

	header := s.Header.Render(
		s.Title.Render("OpenCode Forger") + "  " +
			statusStyle.Render(statusIcon+" "+statusMsg),
	)

	message := m.message
	if message == "" {
		if m.success {
			message = "Project scaffolding completed successfully!"
		} else {
			message = "An error occurred during scaffolding."
		}
	}

	content := s.Content.Render(
		statusStyle.Render(message) + "\n\n" +
			s.MutedText.Render("Press any key to exit."),
	)

	return header + "\n" + content
}

// viewError renders the error screen.
func (m Model) viewError() string {
	s := m.styles

	header := s.Header.Render(
		s.Title.Render("OpenCode Forger") + "  " +
			s.ErrorText.Render(SymbolError+" Error"),
	)

	errMsg := "An unexpected error occurred."
	if m.err != nil {
		errMsg = m.err.Error()
	} else if m.message != "" {
		errMsg = m.message
	}

	content := s.Content.Render(
		s.ErrorText.Render(errMsg) + "\n\n" +
			s.MutedText.Render("Press q to quit."),
	)

	return header + "\n" + content
}
