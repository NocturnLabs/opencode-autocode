// Package ui provides the Bubble Tea TUI components and views.
package ui

import "github.com/charmbracelet/lipgloss"

// Theme defines the color palette for the TUI.
type Theme struct {
	Primary    lipgloss.Color
	Secondary  lipgloss.Color
	Accent     lipgloss.Color
	Muted      lipgloss.Color
	Border     lipgloss.Color
	Success    lipgloss.Color
	Warning    lipgloss.Color
	Error      lipgloss.Color
	Background lipgloss.Color
}

// DefaultTheme returns the default color theme.
func DefaultTheme() Theme {
	return Theme{
		Primary:    lipgloss.Color("39"),  // Bright blue
		Secondary:  lipgloss.Color("245"), // Gray
		Accent:     lipgloss.Color("213"), // Pink/magenta
		Muted:      lipgloss.Color("240"), // Dark gray
		Border:     lipgloss.Color("238"), // Darker gray
		Success:    lipgloss.Color("82"),  // Green
		Warning:    lipgloss.Color("214"), // Orange
		Error:      lipgloss.Color("196"), // Red
		Background: lipgloss.Color("0"),   // Black
	}
}

// Styles contains pre-configured lipgloss styles for UI elements.
type Styles struct {
	Theme Theme

	// Layout styles
	App         lipgloss.Style
	Header      lipgloss.Style
	Footer      lipgloss.Style
	Content     lipgloss.Style
	ContentArea lipgloss.Style

	// Text styles
	Title       lipgloss.Style
	Subtitle    lipgloss.Style
	Label       lipgloss.Style
	MutedText   lipgloss.Style
	SuccessText lipgloss.Style
	WarningText lipgloss.Style
	ErrorText   lipgloss.Style

	// Interactive elements
	MenuItem         lipgloss.Style
	MenuItemSelected lipgloss.Style
	Button           lipgloss.Style
	ButtonActive     lipgloss.Style

	// Progress and status
	ProgressBar     lipgloss.Style
	ProgressFilled  lipgloss.Style
	StatusIndicator lipgloss.Style

	// Borders and containers
	Box        lipgloss.Style
	BoxFocused lipgloss.Style
}

// NewStyles creates a new Styles instance with the default theme.
func NewStyles() *Styles {
	return NewStylesWithTheme(DefaultTheme())
}

// NewStylesWithTheme creates a new Styles instance with a custom theme.
func NewStylesWithTheme(theme Theme) *Styles {
	s := &Styles{Theme: theme}

	// Layout styles
	s.App = lipgloss.NewStyle()

	s.Header = lipgloss.NewStyle().
		BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(theme.Border).
		Padding(0, 1).
		MarginBottom(1)

	s.Footer = lipgloss.NewStyle().
		BorderStyle(lipgloss.NormalBorder()).
		BorderForeground(theme.Border).
		BorderTop(true).
		BorderBottom(false).
		BorderLeft(false).
		BorderRight(false).
		Padding(0, 1).
		Foreground(theme.Muted)

	s.Content = lipgloss.NewStyle().
		Padding(1, 2)

	s.ContentArea = lipgloss.NewStyle().
		Padding(0, 1)

	// Text styles
	s.Title = lipgloss.NewStyle().
		Bold(true).
		Foreground(theme.Primary)

	s.Subtitle = lipgloss.NewStyle().
		Foreground(theme.Secondary)

	s.Label = lipgloss.NewStyle().
		Foreground(theme.Secondary)

	s.MutedText = lipgloss.NewStyle().
		Foreground(theme.Muted)

	s.SuccessText = lipgloss.NewStyle().
		Foreground(theme.Success)

	s.WarningText = lipgloss.NewStyle().
		Foreground(theme.Warning)

	s.ErrorText = lipgloss.NewStyle().
		Foreground(theme.Error)

	// Interactive elements
	s.MenuItem = lipgloss.NewStyle().
		Padding(0, 2).
		Foreground(theme.Muted)

	s.MenuItemSelected = lipgloss.NewStyle().
		Padding(0, 2).
		Bold(true).
		Foreground(theme.Primary)

	s.Button = lipgloss.NewStyle().
		Padding(0, 2).
		BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(theme.Border)

	s.ButtonActive = lipgloss.NewStyle().
		Padding(0, 2).
		Bold(true).
		BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(theme.Primary).
		Foreground(theme.Primary)

	// Progress and status
	s.ProgressBar = lipgloss.NewStyle().
		Foreground(theme.Muted)

	s.ProgressFilled = lipgloss.NewStyle().
		Foreground(theme.Accent)

	s.StatusIndicator = lipgloss.NewStyle().
		Foreground(theme.Accent)

	// Borders and containers
	s.Box = lipgloss.NewStyle().
		BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(theme.Border).
		Padding(1, 2).
		MarginBottom(1)

	s.BoxFocused = lipgloss.NewStyle().
		BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(theme.Accent).
		Padding(1, 2).
		MarginBottom(1)

	return s
}

// Symbols for UI elements
const (
	SymbolSelected   = "›"
	SymbolUnselected = " "
	SymbolRunning    = "◐"
	SymbolPending    = "○"
	SymbolComplete   = "●"
	SymbolSuccess    = "✓"
	SymbolError      = "✗"
	SymbolWarning    = "⚠"
	SymbolInfo       = "ℹ"
	SymbolArrowUp    = "↑"
	SymbolArrowDown  = "↓"
)
