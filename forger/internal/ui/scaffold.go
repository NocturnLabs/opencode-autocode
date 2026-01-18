package ui

import (
	"strings"

	"github.com/charmbracelet/bubbles/textarea"
	tea "github.com/charmbracelet/bubbletea"
)

type ScaffoldStep int

const (
	ScaffoldInputIdea ScaffoldStep = iota
	ScaffoldGenerating
	ScaffoldReview
	ScaffoldConfirm
	ScaffoldDone
)

type ScaffoldScreen struct {
	styles      *Styles
	step        ScaffoldStep
	ideaInput   textarea.Model
	projectName string
	specPreview string
	errorMsg    string
}

func NewScaffoldScreen(styles *Styles) *ScaffoldScreen {
	ti := textarea.New()
	ti.Placeholder = "Describe your project idea..."
	ti.SetHeight(10)
	ti.SetWidth(60)
	ti.Focus()

	return &ScaffoldScreen{
		styles:    styles,
		step:      ScaffoldInputIdea,
		ideaInput: ti,
	}
}

func (s *ScaffoldScreen) Update(msg tea.Msg) (bool, ScreenType) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "esc":
			return true, ScreenHome
		case "enter":
			if s.step == ScaffoldInputIdea && s.ideaInput.Value() != "" {
				s.step = ScaffoldGenerating
				// TODO: Trigger spec generation
				return true, ScreenHome
			}
		}

		// Handle textarea input
		var cmd tea.Cmd
		s.ideaInput, cmd = s.ideaInput.Update(msg)
		_ = cmd // Ignore command for now
		return false, ScreenHome
	}

	return false, ScreenHome
}

func (s *ScaffoldScreen) View() string {
	switch s.step {
	case ScaffoldInputIdea:
		return s.viewInputIdea()
	case ScaffoldGenerating:
		return s.viewGenerating()
	case ScaffoldReview:
		return s.viewReview()
	case ScaffoldConfirm:
		return s.viewConfirm()
	case ScaffoldDone:
		return s.viewDone()
	}
	return ""
}

func (s *ScaffoldScreen) viewInputIdea() string {
	var sb strings.Builder

	sb.WriteString(s.styles.Title.Render("New Project"))
	sb.WriteString("\n\n")
	sb.WriteString(s.styles.Subtitle.Render("Describe your project idea"))
	sb.WriteString("\n\n")
	sb.WriteString(s.ideaInput.View())
	sb.WriteString("\n\n")
	sb.WriteString(s.styles.Muted.Render("Press Enter to generate spec, Esc to cancel"))

	return sb.String()
}

func (s *ScaffoldScreen) viewGenerating() string {
	var sb strings.Builder

	sb.WriteString(s.styles.Title.Render("New Project"))
	sb.WriteString("\n\n")
	sb.WriteString(s.styles.Subtitle.Render("Generating specification..."))
	sb.WriteString("\n\n")
	sb.WriteString("This may take a moment. Please wait...")

	return sb.String()
}

func (s *ScaffoldScreen) viewReview() string {
	var sb strings.Builder

	sb.WriteString(s.styles.Title.Render("New Project"))
	sb.WriteString("\n\n")
	sb.WriteString(s.styles.Subtitle.Render("Review Generated Specification"))
	sb.WriteString("\n\n")

	// Truncate spec preview for display
	preview := s.specPreview
	if len(preview) > 1000 {
		preview = preview[:1000] + "\n... (truncated)"
	}

	sb.WriteString(s.styles.Body.Render(preview))
	sb.WriteString("\n\n")
	sb.WriteString(s.styles.Highlight.Render("[Enter] Proceed to scaffold  [Esc] Cancel"))

	return sb.String()
}

func (s *ScaffoldScreen) viewConfirm() string {
	var sb strings.Builder

	sb.WriteString(s.styles.Title.Render("New Project"))
	sb.WriteString("\n\n")
	sb.WriteString(s.styles.Subtitle.Render("Scaffolding Project..."))
	sb.WriteString("\n\n")

	// Show progress items
	items := []string{
		"✓ Generated specification",
		"✓ Created .forger/ directory",
		"✓ Created .opencode/ directory",
		"✓ Wrote forger.toml",
		"✓ Wrote opencode.json",
		"✓ Wrote AGENTS.md",
	}

	for _, item := range items {
		sb.WriteString(s.styles.Success.Render(item))
		sb.WriteString("\n")
	}

	sb.WriteString("\n")
	sb.WriteString(s.styles.Success.Render("Project scaffolded successfully!"))

	return sb.String()
}

func (s *ScaffoldScreen) viewDone() string {
	var sb strings.Builder

	sb.WriteString(s.styles.Title.Render("New Project"))
	sb.WriteString("\n\n")
	sb.WriteString(s.styles.Success.Render("✓ Project created successfully!"))
	sb.WriteString("\n\n")
	sb.WriteString(s.styles.Body.Render("You can now run 'forger vibe' to start the autonomous coding loop."))
	sb.WriteString("\n\n")
	sb.WriteString(s.styles.Highlight.Render("[Enter] Return to main menu"))

	return sb.String()
}
