package ui

import (
	"strings"

	"github.com/charmbracelet/lipgloss"
)

// HomeScreen represents the main menu screen.
type HomeScreen struct {
	styles *Styles
	cursor int
}

// MenuItem represents a menu item with label and action.
type MenuItem struct {
	Label  string
	Action string
}

// NewHomeScreen creates a new home screen with the given styles.
func NewHomeScreen(styles *Styles) *HomeScreen {
	return &HomeScreen{
		styles: styles,
		cursor: 0,
	}
}

func (h *HomeScreen) Update(msg string) (bool, ScreenType) {
	switch msg {
	case "up", "k":
		if h.cursor > 0 {
			h.cursor--
		}
	case "down", "j":
		if h.cursor < len(h.getMenu())-1 {
			h.cursor++
		}
	case "enter", " ":
		return true, h.executeSelection()
	}
	return false, ScreenHome
}

func (h *HomeScreen) executeSelection() ScreenType {
	menu := h.getMenu()
	if h.cursor >= len(menu) {
		return ScreenHome
	}

	switch menu[h.cursor].Action {
	case "new":
		return ScreenScaffold
	case "scaffold":
		return ScreenScaffold
	case "vibe":
		return ScreenVibe
	case "enhance":
		return ScreenEnhance
	case "config":
		return ScreenConfig
	case "quit":
		return ScreenQuit
	}
	return ScreenHome
}

func (h *HomeScreen) getMenu() []MenuItem {
	return []MenuItem{
		{Label: "> New Project (AI generates spec from idea)", Action: "new"},
		{Label: "  Scaffold (use existing spec)", Action: "scaffold"},
		{Label: "  Vibe (start autonomous coding loop)", Action: "vibe"},
		{Label: "  Enhance (discover improvements)", Action: "enhance"},
		{Label: "  Settings", Action: "config"},
		{Label: "  Quit", Action: "quit"},
	}
}

func (h *HomeScreen) View() string {
	menu := h.getMenu()

	var sb strings.Builder

	sb.WriteString(h.styles.Title.Render("OpenCode Forger"))
	sb.WriteString("\n\n")
	sb.WriteString(h.styles.Subtitle.Render("Main Menu"))
	sb.WriteString("\n\n")

	for i, item := range menu {
		var style lipgloss.Style
		if i == h.cursor {
			style = h.styles.MenuItemSelected
		} else {
			style = h.styles.MenuItem
		}
		sb.WriteString(style.Render(item.Label))
		sb.WriteString("\n")
	}

	sb.WriteString("\n")
	sb.WriteString(h.styles.Muted.Render("Press ↑/↓ to navigate, Enter to select, q to quit"))

	return sb.String()
}
