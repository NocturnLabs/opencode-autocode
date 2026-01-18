package ui

import (
	"github.com/charmbracelet/lipgloss"
)

// Styles holds all the styling constants
type Styles struct {
	// Base colors
	primaryColor   lipgloss.Color
	secondaryColor lipgloss.Color
	errorColor     lipgloss.Color
	successColor   lipgloss.Color
	warningColor   lipgloss.Color

	// Text styles
	Title     lipgloss.Style
	Subtitle  lipgloss.Style
	Body      lipgloss.Style
	Muted     lipgloss.Style
	Highlight lipgloss.Style
	Error     lipgloss.Style
	Success   lipgloss.Style

	// UI components styles
	Header           lipgloss.Style
	Footer           lipgloss.Style
	MenuItem         lipgloss.Style
	MenuItemSelected lipgloss.Style
	Button           lipgloss.Style
	ButtonSelected   lipgloss.Style
	Border           lipgloss.Style
}

// DefaultStyles returns the default styling
func DefaultStyles() *Styles {
	primary := lipgloss.Color("86")    // Cyan
	secondary := lipgloss.Color("147") // Lilac
	error := lipgloss.Color("196")     // Red
	success := lipgloss.Color("42")    // Green
	warning := lipgloss.Color("226")   // Yellow

	return &Styles{
		primaryColor:   primary,
		secondaryColor: secondary,
		errorColor:     error,
		successColor:   success,
		warningColor:   warning,

		// Text styles
		Title: lipgloss.NewStyle().
			Bold(true).
			Foreground(primary).
			MarginTop(1).
			MarginBottom(1),

		Subtitle: lipgloss.NewStyle().
			Bold(true).
			Foreground(secondary).
			MarginBottom(1),

		Body: lipgloss.NewStyle().
			Foreground(lipgloss.Color("241")),

		Muted: lipgloss.NewStyle().
			Foreground(lipgloss.Color("244")),

		Highlight: lipgloss.NewStyle().
			Bold(true).
			Foreground(primary),

		Error: lipgloss.NewStyle().
			Foreground(error),

		Success: lipgloss.NewStyle().
			Foreground(success),

		// UI components styles
		Header: lipgloss.NewStyle().
			Bold(true).
			Foreground(primary).
			Padding(0, 1).
			MarginBottom(1),

		Footer: lipgloss.NewStyle().
			Foreground(lipgloss.Color("244")),

		MenuItem: lipgloss.NewStyle().
			Foreground(lipgloss.Color("241")),

		MenuItemSelected: lipgloss.NewStyle().
			Bold(true).
			Foreground(primary),

		Button: lipgloss.NewStyle().
			Foreground(lipgloss.Color("241")),

		ButtonSelected: lipgloss.NewStyle().
			Bold(true).
			Foreground(primary),

		Border: lipgloss.NewStyle().
			Border(lipgloss.RoundedBorder()).
			BorderForeground(lipgloss.Color("238")),
	}
}
