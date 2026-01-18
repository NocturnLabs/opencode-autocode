package ui

import (
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// ScreenType represents different screens in the application
type ScreenType int

const (
	ScreenHome ScreenType = iota
	ScreenScaffold
	ScreenVibe
	ScreenEnhance
	ScreenConfig
	ScreenQuit
)

// Model represents main application model
type Model struct {
	currentScreen  ScreenType
	width          int
	height         int
	styles         *Styles
	homeScreen     *HomeScreen
	scaffoldScreen *ScaffoldScreen
	vibeScreen     *VibeScreen
	enhanceScreen  *EnhanceScreen
	configScreen   *ConfigScreen
}

// Init initializes model
func (m Model) Init() tea.Cmd {
	return nil
}

// Update handles incoming messages
func (m Model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height
		return m, nil

	case tea.KeyMsg:
		switch msg.String() {
		case "q", "ctrl+c":
			return m, tea.Quit
		}

		// Handle screen-specific input
		switch m.currentScreen {
		case ScreenHome:
			if m.homeScreen != nil {
				actionTaken, newScreen := m.homeScreen.Update(msg.String())
				if actionTaken {
					m.currentScreen = newScreen
				}
			}
		case ScreenScaffold:
			if m.scaffoldScreen != nil {
				actionTaken, newScreen := m.scaffoldScreen.Update(msg)
				if actionTaken {
					m.currentScreen = newScreen
				}
			}
		case ScreenVibe:
			if m.vibeScreen != nil {
				actionTaken, newScreen := m.vibeScreen.Update(msg)
				if actionTaken {
					m.currentScreen = newScreen
				}
			}
		case ScreenEnhance:
			if m.enhanceScreen != nil {
				actionTaken, newScreen := m.enhanceScreen.Update(msg)
				if actionTaken {
					m.currentScreen = newScreen
				}
			}
		case ScreenConfig:
			if m.configScreen != nil {
				actionTaken, newScreen := m.configScreen.Update(msg)
				if actionTaken {
					m.currentScreen = newScreen
				}
			}
		}
	}

	return m, nil
}

// View renders the UI
func (m Model) View() string {
	var content string

	switch m.currentScreen {
	case ScreenHome:
		if m.homeScreen != nil {
			content = m.homeScreen.View()
		} else {
			content = "Loading..."
		}
	case ScreenScaffold:
		if m.scaffoldScreen != nil {
			content = m.scaffoldScreen.View()
		} else {
			content = "Loading..."
		}
	case ScreenVibe:
		if m.vibeScreen != nil {
			content = m.vibeScreen.View()
		} else {
			content = "Loading..."
		}
	case ScreenEnhance:
		if m.enhanceScreen != nil {
			content = m.enhanceScreen.View()
		} else {
			content = "Loading..."
		}
	case ScreenConfig:
		if m.configScreen != nil {
			content = m.configScreen.View()
		} else {
			content = "Loading..."
		}
	case ScreenQuit:
		return ""
	default:
		content = "Screen not implemented yet"
	}

	// Center content if we have dimensions
	if m.width > 0 && m.height > 0 {
		return lipgloss.Place(m.width, m.height, lipgloss.Center, lipgloss.Center, content)
	}

	return content
}

// New creates a new application model
func New() Model {
	styles := DefaultStyles()
	homeScreen := NewHomeScreen(styles)
	scaffoldScreen := NewScaffoldScreen(styles)
	vibeScreen := NewVibeScreen(styles)
	enhanceScreen := NewEnhanceScreen(styles)
	configScreen := NewConfigScreen(styles)

	return Model{
		currentScreen:  ScreenHome,
		styles:         styles,
		homeScreen:     homeScreen,
		scaffoldScreen: scaffoldScreen,
		vibeScreen:     vibeScreen,
		enhanceScreen:  enhanceScreen,
		configScreen:   configScreen,
	}
}
