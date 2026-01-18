package ui

import (
	"fmt"
	"strings"
	"time"

	tea "github.com/charmbracelet/bubbletea"
)

// EnhanceStep represents the current step in the enhance flow.
type EnhanceStep int

const (
	EnhanceIdle EnhanceStep = iota
	EnhanceDiscovering
	EnhanceReview
	EnhanceImplementing
	EnhanceDone
)

// EnhancementItem represents a discovered enhancement for display.
type EnhancementItem struct {
	Name        string
	Description string
	Difficulty  string
	Priority    string
	Impact      string
	Approved    bool
}

// EnhanceScreen handles the enhancement discovery and implementation UI.
type EnhanceScreen struct {
	styles        *Styles
	step          EnhanceStep
	enhancements  []EnhancementItem
	selectedIndex int
	output        []string
	scrollOffset  int
	startTime     time.Time
	errorMsg      string
	statusMsg     string
}

// NewEnhanceScreen creates a new enhance screen with the given styles.
func NewEnhanceScreen(styles *Styles) *EnhanceScreen {
	return &EnhanceScreen{
		styles:       styles,
		step:         EnhanceIdle,
		enhancements: []EnhancementItem{},
		output:       make([]string, 0, 100),
	}
}

// Update handles input for the enhance screen.
func (e *EnhanceScreen) Update(msg tea.Msg) (bool, ScreenType) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "esc":
			if e.step == EnhanceIdle || e.step == EnhanceDone {
				return true, ScreenHome
			}
			// Cancel current operation
			e.step = EnhanceIdle
			e.statusMsg = "Operation cancelled"
			return false, ScreenEnhance

		case "enter":
			return e.handleEnter()

		case "d":
			if e.step == EnhanceIdle {
				e.startDiscovery()
				return false, ScreenEnhance
			}

		case "i":
			if e.step == EnhanceReview && len(e.enhancements) > 0 {
				e.startImplementation()
				return false, ScreenEnhance
			}

		case "a":
			if e.step == EnhanceReview && len(e.enhancements) > 0 {
				e.toggleApproval()
				return false, ScreenEnhance
			}

		case "up", "k":
			if e.step == EnhanceReview && e.selectedIndex > 0 {
				e.selectedIndex--
			} else if e.scrollOffset > 0 {
				e.scrollOffset--
			}

		case "down", "j":
			if e.step == EnhanceReview && e.selectedIndex < len(e.enhancements)-1 {
				e.selectedIndex++
			} else {
				maxScroll := len(e.output) - 10
				if maxScroll > 0 && e.scrollOffset < maxScroll {
					e.scrollOffset++
				}
			}

		case "q":
			if e.step == EnhanceIdle || e.step == EnhanceDone {
				return true, ScreenHome
			}
		}
	}

	return false, ScreenEnhance
}

// handleEnter handles the enter key based on current step.
func (e *EnhanceScreen) handleEnter() (bool, ScreenType) {
	switch e.step {
	case EnhanceIdle:
		e.startDiscovery()
	case EnhanceReview:
		if len(e.enhancements) > 0 {
			e.toggleApproval()
		}
	case EnhanceDone:
		e.step = EnhanceIdle
		e.enhancements = []EnhancementItem{}
		e.output = []string{}
		e.errorMsg = ""
		e.statusMsg = ""
	}
	return false, ScreenEnhance
}

// startDiscovery begins the enhancement discovery process.
func (e *EnhanceScreen) startDiscovery() {
	e.step = EnhanceDiscovering
	e.startTime = time.Now()
	e.output = []string{}
	e.statusMsg = "Discovering enhancements..."
	e.AddOutput("Starting enhancement discovery...")
	// In a real implementation, this would trigger the enhancer
}

// startImplementation begins implementing approved enhancements.
func (e *EnhanceScreen) startImplementation() {
	// Count approved enhancements
	approved := 0
	for _, enh := range e.enhancements {
		if enh.Approved {
			approved++
		}
	}

	if approved == 0 {
		e.statusMsg = "No enhancements approved. Press 'a' to approve."
		return
	}

	e.step = EnhanceImplementing
	e.startTime = time.Now()
	e.statusMsg = fmt.Sprintf("Implementing %d enhancement(s)...", approved)
	e.AddOutput(fmt.Sprintf("Starting implementation of %d enhancement(s)...", approved))
}

// toggleApproval toggles approval status of the selected enhancement.
func (e *EnhanceScreen) toggleApproval() {
	if e.selectedIndex >= 0 && e.selectedIndex < len(e.enhancements) {
		e.enhancements[e.selectedIndex].Approved = !e.enhancements[e.selectedIndex].Approved
	}
}

// View renders the enhance screen.
func (e *EnhanceScreen) View() string {
	switch e.step {
	case EnhanceIdle:
		return e.viewIdle()
	case EnhanceDiscovering:
		return e.viewDiscovering()
	case EnhanceReview:
		return e.viewReview()
	case EnhanceImplementing:
		return e.viewImplementing()
	case EnhanceDone:
		return e.viewDone()
	}
	return ""
}

// viewIdle renders the idle/start state.
func (e *EnhanceScreen) viewIdle() string {
	var sb strings.Builder

	sb.WriteString(e.styles.Title.Render("Enhancement Mode"))
	sb.WriteString("\n\n")
	sb.WriteString(e.styles.Subtitle.Render("Discover and implement project improvements"))
	sb.WriteString("\n\n")

	sb.WriteString(e.styles.Body.Render("Enhancement mode helps you:"))
	sb.WriteString("\n\n")
	sb.WriteString("  • Discover potential improvements for your project\n")
	sb.WriteString("  • Review and approve suggested enhancements\n")
	sb.WriteString("  • Implement approved changes automatically\n")
	sb.WriteString("\n\n")

	sb.WriteString(e.styles.Highlight.Render("[d] Start Discovery"))
	sb.WriteString("  ")
	sb.WriteString(e.styles.Muted.Render("[q] Back to menu"))
	sb.WriteString("\n\n")

	if e.statusMsg != "" {
		sb.WriteString(e.styles.Muted.Render(e.statusMsg))
	}

	return sb.String()
}

// viewDiscovering renders the discovery in-progress state.
func (e *EnhanceScreen) viewDiscovering() string {
	var sb strings.Builder

	sb.WriteString(e.styles.Title.Render("Enhancement Mode"))
	sb.WriteString("\n\n")
	sb.WriteString(e.styles.Subtitle.Render("Discovering enhancements..."))
	sb.WriteString("\n\n")

	// Elapsed time
	elapsed := time.Since(e.startTime)
	sb.WriteString(fmt.Sprintf("Elapsed: %s\n\n", elapsed.Round(time.Second)))

	// Output
	sb.WriteString(e.styles.Subtitle.Render("Live Output"))
	sb.WriteString("\n\n")
	sb.WriteString(e.viewOutput())
	sb.WriteString("\n\n")

	sb.WriteString(e.styles.Muted.Render("[esc] Cancel"))

	return sb.String()
}

// viewReview renders the enhancement review state.
func (e *EnhanceScreen) viewReview() string {
	var sb strings.Builder

	sb.WriteString(e.styles.Title.Render("Enhancement Mode"))
	sb.WriteString("\n\n")
	sb.WriteString(e.styles.Subtitle.Render("Review Proposed Enhancements"))
	sb.WriteString("\n\n")

	if len(e.enhancements) == 0 {
		sb.WriteString(e.styles.Muted.Render("No enhancements discovered."))
		sb.WriteString("\n\n")
		sb.WriteString(e.styles.Highlight.Render("[d] Try Again"))
		sb.WriteString("  ")
		sb.WriteString(e.styles.Muted.Render("[q] Back to menu"))
		return sb.String()
	}

	// Count approved
	approved := 0
	for _, enh := range e.enhancements {
		if enh.Approved {
			approved++
		}
	}
	sb.WriteString(fmt.Sprintf("Found %d enhancement(s), %d approved\n\n", len(e.enhancements), approved))

	// List enhancements
	for i, enh := range e.enhancements {
		prefix := "  "
		if i == e.selectedIndex {
			prefix = "> "
		}

		status := "[ ]"
		if enh.Approved {
			status = "[✓]"
		}

		var style = e.styles.MenuItem
		if i == e.selectedIndex {
			style = e.styles.MenuItemSelected
		}

		line := fmt.Sprintf("%s%s %s (%s, %s)", prefix, status, enh.Name, enh.Priority, enh.Difficulty)
		sb.WriteString(style.Render(line))
		sb.WriteString("\n")

		// Show description for selected item
		if i == e.selectedIndex && enh.Description != "" {
			sb.WriteString(e.styles.Muted.Render(fmt.Sprintf("      %s", enh.Description)))
			sb.WriteString("\n")
		}
	}

	sb.WriteString("\n")
	sb.WriteString(e.styles.Highlight.Render("[a] Toggle Approve"))
	sb.WriteString("  ")
	sb.WriteString(e.styles.Highlight.Render("[i] Implement Approved"))
	sb.WriteString("  ")
	sb.WriteString(e.styles.Muted.Render("[q] Back"))

	if e.statusMsg != "" {
		sb.WriteString("\n\n")
		sb.WriteString(e.styles.Muted.Render(e.statusMsg))
	}

	return sb.String()
}

// viewImplementing renders the implementation in-progress state.
func (e *EnhanceScreen) viewImplementing() string {
	var sb strings.Builder

	sb.WriteString(e.styles.Title.Render("Enhancement Mode"))
	sb.WriteString("\n\n")
	sb.WriteString(e.styles.Subtitle.Render("Implementing enhancements..."))
	sb.WriteString("\n\n")

	// Elapsed time
	elapsed := time.Since(e.startTime)
	sb.WriteString(fmt.Sprintf("Elapsed: %s\n\n", elapsed.Round(time.Second)))

	// Show which enhancements are being implemented
	for _, enh := range e.enhancements {
		if enh.Approved {
			sb.WriteString(fmt.Sprintf("  → %s\n", enh.Name))
		}
	}
	sb.WriteString("\n")

	// Output
	sb.WriteString(e.styles.Subtitle.Render("Live Output"))
	sb.WriteString("\n\n")
	sb.WriteString(e.viewOutput())
	sb.WriteString("\n\n")

	sb.WriteString(e.styles.Muted.Render("[esc] Cancel"))

	return sb.String()
}

// viewDone renders the completion state.
func (e *EnhanceScreen) viewDone() string {
	var sb strings.Builder

	sb.WriteString(e.styles.Title.Render("Enhancement Mode"))
	sb.WriteString("\n\n")

	if e.errorMsg != "" {
		sb.WriteString(e.styles.Error.Render("Error: " + e.errorMsg))
	} else {
		sb.WriteString(e.styles.Success.Render("✓ Enhancement session complete!"))
	}
	sb.WriteString("\n\n")

	// Summary
	implemented := 0
	for _, enh := range e.enhancements {
		if enh.Approved {
			implemented++
		}
	}

	if implemented > 0 {
		sb.WriteString(fmt.Sprintf("Implemented %d enhancement(s)\n\n", implemented))
	}

	sb.WriteString(e.styles.Highlight.Render("[enter] Start New Session"))
	sb.WriteString("  ")
	sb.WriteString(e.styles.Muted.Render("[q] Back to menu"))

	return sb.String()
}

// viewOutput renders the output log area.
func (e *EnhanceScreen) viewOutput() string {
	if len(e.output) == 0 {
		return e.styles.Muted.Render("Waiting for output...")
	}

	// Show last 10 lines
	start := len(e.output) - 10
	if start < 0 {
		start = 0
	}

	visible := e.output[start:]
	var sb strings.Builder

	for _, line := range visible {
		sb.WriteString(line)
		sb.WriteString("\n")
	}

	return sb.String()
}

// AddOutput adds a line to the output log.
func (e *EnhanceScreen) AddOutput(line string) {
	timestamp := time.Now().Format("15:04:05")
	e.output = append(e.output, fmt.Sprintf("[%s] %s", timestamp, line))
}

// SetEnhancements sets the discovered enhancements.
func (e *EnhanceScreen) SetEnhancements(enhancements []EnhancementItem) {
	e.enhancements = enhancements
	e.selectedIndex = 0
}

// SetStep sets the current step.
func (e *EnhanceScreen) SetStep(step EnhanceStep) {
	e.step = step
}

// SetError sets an error message.
func (e *EnhanceScreen) SetError(msg string) {
	e.errorMsg = msg
}

// SetStatus sets a status message.
func (e *EnhanceScreen) SetStatus(msg string) {
	e.statusMsg = msg
}

// CompleteDiscovery transitions to review step with discovered enhancements.
func (e *EnhanceScreen) CompleteDiscovery(enhancements []EnhancementItem) {
	e.enhancements = enhancements
	e.step = EnhanceReview
	e.statusMsg = ""
}

// CompleteImplementation transitions to done step.
func (e *EnhanceScreen) CompleteImplementation() {
	e.step = EnhanceDone
	e.statusMsg = ""
}
