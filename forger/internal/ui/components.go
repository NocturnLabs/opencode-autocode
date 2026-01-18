package ui

import (
	"strings"
	"time"
)

// ErrorOverlay provides a reusable error display component for TUI screens.
type ErrorOverlay struct {
	styles    *Styles
	visible   bool
	title     string
	message   string
	details   string
	timestamp time.Time
	canRetry  bool
}

// NewErrorOverlay creates a new error overlay component.
func NewErrorOverlay(styles *Styles) *ErrorOverlay {
	return &ErrorOverlay{
		styles: styles,
	}
}

// Show displays an error with the given title and message.
func (e *ErrorOverlay) Show(title, message string) {
	e.visible = true
	e.title = title
	e.message = message
	e.details = ""
	e.timestamp = time.Now()
	e.canRetry = false
}

// ShowWithDetails displays an error with additional details.
func (e *ErrorOverlay) ShowWithDetails(title, message, details string) {
	e.visible = true
	e.title = title
	e.message = message
	e.details = details
	e.timestamp = time.Now()
	e.canRetry = false
}

// ShowRetryable displays an error that can be retried.
func (e *ErrorOverlay) ShowRetryable(title, message string) {
	e.visible = true
	e.title = title
	e.message = message
	e.details = ""
	e.timestamp = time.Now()
	e.canRetry = true
}

// Hide hides the error overlay.
func (e *ErrorOverlay) Hide() {
	e.visible = false
}

// IsVisible returns whether the overlay is currently visible.
func (e *ErrorOverlay) IsVisible() bool {
	return e.visible
}

// CanRetry returns whether the error is retryable.
func (e *ErrorOverlay) CanRetry() bool {
	return e.canRetry
}

// View renders the error overlay.
func (e *ErrorOverlay) View() string {
	if !e.visible {
		return ""
	}

	var sb strings.Builder

	// Error box
	sb.WriteString("\n")
	sb.WriteString(e.styles.Error.Render("╭─────────────────────────────────────────╮"))
	sb.WriteString("\n")
	sb.WriteString(e.styles.Error.Render("│ "))
	sb.WriteString(e.styles.Error.Render("⚠ " + e.title))
	sb.WriteString(e.styles.Error.Render(strings.Repeat(" ", 40-len(e.title)-3)))
	sb.WriteString(e.styles.Error.Render("│"))
	sb.WriteString("\n")
	sb.WriteString(e.styles.Error.Render("├─────────────────────────────────────────┤"))
	sb.WriteString("\n")

	// Message (wrap long lines)
	lines := wrapText(e.message, 39)
	for _, line := range lines {
		padding := 39 - len(line)
		if padding < 0 {
			padding = 0
		}
		sb.WriteString(e.styles.Error.Render("│ "))
		sb.WriteString(line)
		sb.WriteString(strings.Repeat(" ", padding))
		sb.WriteString(e.styles.Error.Render(" │"))
		sb.WriteString("\n")
	}

	// Details if present
	if e.details != "" {
		sb.WriteString(e.styles.Error.Render("│                                         │"))
		sb.WriteString("\n")
		detailLines := wrapText(e.details, 39)
		for _, line := range detailLines {
			padding := 39 - len(line)
			if padding < 0 {
				padding = 0
			}
			sb.WriteString(e.styles.Error.Render("│ "))
			sb.WriteString(e.styles.Muted.Render(line))
			sb.WriteString(strings.Repeat(" ", padding))
			sb.WriteString(e.styles.Error.Render(" │"))
			sb.WriteString("\n")
		}
	}

	sb.WriteString(e.styles.Error.Render("├─────────────────────────────────────────┤"))
	sb.WriteString("\n")

	// Actions
	if e.canRetry {
		sb.WriteString(e.styles.Error.Render("│ "))
		sb.WriteString(e.styles.Highlight.Render("[r] Retry"))
		sb.WriteString("  ")
		sb.WriteString(e.styles.Muted.Render("[esc] Dismiss"))
		sb.WriteString(strings.Repeat(" ", 10))
		sb.WriteString(e.styles.Error.Render(" │"))
	} else {
		sb.WriteString(e.styles.Error.Render("│ "))
		sb.WriteString(e.styles.Muted.Render("[esc] Dismiss"))
		sb.WriteString(strings.Repeat(" ", 24))
		sb.WriteString(e.styles.Error.Render(" │"))
	}
	sb.WriteString("\n")
	sb.WriteString(e.styles.Error.Render("╰─────────────────────────────────────────╯"))
	sb.WriteString("\n")

	return sb.String()
}

// wrapText wraps text to the specified width.
func wrapText(text string, width int) []string {
	if len(text) <= width {
		return []string{text}
	}

	var lines []string
	words := strings.Fields(text)
	currentLine := ""

	for _, word := range words {
		if len(currentLine)+len(word)+1 <= width {
			if currentLine != "" {
				currentLine += " "
			}
			currentLine += word
		} else {
			if currentLine != "" {
				lines = append(lines, currentLine)
			}
			currentLine = word
		}
	}

	if currentLine != "" {
		lines = append(lines, currentLine)
	}

	return lines
}

// StatusBar provides a reusable status bar component.
type StatusBar struct {
	styles  *Styles
	message string
	msgType string // "info", "success", "warning", "error"
	expires time.Time
}

// NewStatusBar creates a new status bar component.
func NewStatusBar(styles *Styles) *StatusBar {
	return &StatusBar{
		styles: styles,
	}
}

// SetInfo sets an info message.
func (s *StatusBar) SetInfo(message string) {
	s.message = message
	s.msgType = "info"
	s.expires = time.Now().Add(5 * time.Second)
}

// SetSuccess sets a success message.
func (s *StatusBar) SetSuccess(message string) {
	s.message = message
	s.msgType = "success"
	s.expires = time.Now().Add(5 * time.Second)
}

// SetWarning sets a warning message.
func (s *StatusBar) SetWarning(message string) {
	s.message = message
	s.msgType = "warning"
	s.expires = time.Now().Add(10 * time.Second)
}

// SetError sets an error message.
func (s *StatusBar) SetError(message string) {
	s.message = message
	s.msgType = "error"
	s.expires = time.Now().Add(10 * time.Second)
}

// Clear clears the status bar.
func (s *StatusBar) Clear() {
	s.message = ""
}

// View renders the status bar.
func (s *StatusBar) View() string {
	// Check if message has expired
	if time.Now().After(s.expires) {
		s.message = ""
	}

	if s.message == "" {
		return ""
	}

	var style = s.styles.Muted
	prefix := ""

	switch s.msgType {
	case "success":
		style = s.styles.Success
		prefix = "✓ "
	case "warning":
		style = s.styles.Highlight
		prefix = "⚠ "
	case "error":
		style = s.styles.Error
		prefix = "✗ "
	case "info":
		style = s.styles.Muted
		prefix = "ℹ "
	}

	return style.Render(prefix + s.message)
}

// ProgressIndicator provides a simple progress indicator.
type ProgressIndicator struct {
	styles  *Styles
	current int
	total   int
	label   string
	showPct bool
}

// NewProgressIndicator creates a new progress indicator.
func NewProgressIndicator(styles *Styles) *ProgressIndicator {
	return &ProgressIndicator{
		styles:  styles,
		showPct: true,
	}
}

// Set sets the current progress.
func (p *ProgressIndicator) Set(current, total int) {
	p.current = current
	p.total = total
}

// SetLabel sets the progress label.
func (p *ProgressIndicator) SetLabel(label string) {
	p.label = label
}

// View renders the progress indicator.
func (p *ProgressIndicator) View() string {
	if p.total <= 0 {
		return ""
	}

	var sb strings.Builder

	// Label
	if p.label != "" {
		sb.WriteString(p.styles.Muted.Render(p.label))
		sb.WriteString(" ")
	}

	// Progress bar
	width := 30
	pct := float64(p.current) / float64(p.total)
	filled := int(pct * float64(width))

	sb.WriteString("[")
	for i := 0; i < width; i++ {
		if i < filled {
			sb.WriteString(p.styles.Success.Render("█"))
		} else {
			sb.WriteString(p.styles.Muted.Render("░"))
		}
	}
	sb.WriteString("]")

	// Percentage
	if p.showPct {
		sb.WriteString(p.styles.Muted.Render(strings.Repeat(" ", 1)))
		sb.WriteString(p.styles.Highlight.Render(strings.TrimSpace(strings.Repeat(" ", 3) + string(rune('0'+int(pct*100)/10)) + string(rune('0'+int(pct*100)%10)) + "%")))
	}

	return sb.String()
}
