package ui

import (
	"fmt"
	"strings"
	"time"

	tea "github.com/charmbracelet/bubbletea"
)

type VibeScreen struct {
	styles         *Styles
	currentFeature string
	passingCount   int
	totalCount     int
	sessionCount   int
	iterationCount int
	output         []string
	scrollOffset   int
}

func NewVibeScreen(styles *Styles) *VibeScreen {
	return &VibeScreen{
		styles:       styles,
		output:       make([]string, 0, 100),
		scrollOffset: 0,
	}
}

func (v *VibeScreen) Update(msg tea.Msg) (bool, ScreenType) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "q", "ctrl+c":
			return true, ScreenQuit
		case "s":
			// Stop vibe loop
			return true, ScreenHome
		}

		// Handle scrolling
		switch msg.String() {
		case "up", "k":
			if v.scrollOffset > 0 {
				v.scrollOffset--
			}
		case "down", "j":
			maxScroll := len(v.output) - 15
			if maxScroll < 0 {
				maxScroll = 0
			}
			if v.scrollOffset < maxScroll {
				v.scrollOffset++
			}
		}
	}

	return false, ScreenVibe
}

func (v *VibeScreen) View() string {
	var sb strings.Builder

	// Header
	sb.WriteString(v.styles.Header.Render("Vibe - Autonomous Coding Loop"))
	sb.WriteString("\n\n")

	// Stats
	sb.WriteString(v.viewStats())
	sb.WriteString("\n\n")

	// Output area
	sb.WriteString(v.styles.Subtitle.Render("Live Output"))
	sb.WriteString("\n\n")
	sb.WriteString(v.viewOutput())

	return sb.String()
}

func (v *VibeScreen) viewStats() string {
	var sb strings.Builder

	// Progress bar
	percentage := 0.0
	if v.totalCount > 0 {
		percentage = float64(v.passingCount) / float64(v.totalCount) * 100.0
	}

	sb.WriteString(fmt.Sprintf("Features: %d/%d passing (%.0f%%)\n",
		v.passingCount, v.totalCount, percentage))
	sb.WriteString(fmt.Sprintf("Session: #%d | Iteration: %d\n\n",
		v.sessionCount, v.iterationCount))

	// Progress bar
	barWidth := 40
	filled := int(percentage / 100.0 * float64(barWidth))
	sb.WriteString("[")
	for i := 0; i < barWidth; i++ {
		if i < filled {
			sb.WriteString("=")
		} else {
			sb.WriteString(" ")
		}
	}
	sb.WriteString("]")

	return sb.String()
}

func (v *VibeScreen) viewOutput() string {
	if len(v.output) == 0 {
		return v.styles.Muted.Render("Waiting for output...")
	}

	// Show last 15 lines of output
	start := len(v.output) - 15
	if start < 0 {
		start = 0
	}

	visible := v.output[start:]
	var sb strings.Builder

	for _, line := range visible {
		sb.WriteString(line)
		sb.WriteString("\n")
	}

	return sb.String()
}

func (v *VibeScreen) AddOutput(line string) {
	// Add timestamp
	timestamp := time.Now().Format("15:04:05")
	timestamped := fmt.Sprintf("[%s] %s", timestamp, line)
	v.output = append(v.output, timestamped)

	// Auto-scroll to bottom
	if len(v.output) > 15 {
		v.scrollOffset = len(v.output) - 15
	}
}

func (v *VibeScreen) UpdateStats(passing, total, session, iteration int) {
	v.passingCount = passing
	v.totalCount = total
	v.sessionCount = session
	v.iterationCount = iteration
}

func (v *VibeScreen) GetPassingCount() int {
	return v.passingCount
}

func (v *VibeScreen) GetTotalCount() int {
	return v.totalCount
}

func (v *VibeScreen) SetCurrentFeature(description string) {
	v.currentFeature = description
}

func (v *VibeScreen) AddError(msg string) {
	timestamp := time.Now().Format("15:04:05")
	timestamped := fmt.Sprintf("[%s] ERROR: %s", timestamp, msg)
	v.output = append(v.output, timestamped)

	// Auto-scroll to bottom
	if len(v.output) > 15 {
		v.scrollOffset = len(v.output) - 15
	}
}
