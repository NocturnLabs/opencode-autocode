package ui

import (
	"fmt"
	"strconv"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/yum-inc/opencode-forger/internal/config"
)

// ConfigField represents an editable configuration field.
type ConfigField struct {
	Label       string
	Key         string
	Value       string
	Type        string // "string", "int", "bool"
	Description string
}

// ConfigScreen handles configuration editing.
type ConfigScreen struct {
	styles     *Styles
	fields     []ConfigField
	cursor     int
	editing    bool
	editBuffer string
	config     *config.Config
	configPath string
	statusMsg  string
	hasChanges bool
}

// NewConfigScreen creates a new config screen with the given styles.
func NewConfigScreen(styles *Styles) *ConfigScreen {
	return &ConfigScreen{
		styles:     styles,
		fields:     []ConfigField{},
		configPath: "forger.toml",
	}
}

// LoadConfig loads configuration and populates fields.
func (c *ConfigScreen) LoadConfig() error {
	cfg, err := config.Load(c.configPath)
	if err != nil {
		return err
	}
	c.config = cfg
	c.populateFields()
	c.hasChanges = false
	return nil
}

// populateFields creates editable fields from config.
func (c *ConfigScreen) populateFields() {
	c.fields = []ConfigField{
		// Models section
		{
			Label:       "Default Model",
			Key:         "models.default",
			Value:       c.config.Models.Default,
			Type:        "string",
			Description: "Model for interactive sessions",
		},
		{
			Label:       "Autonomous Model",
			Key:         "models.autonomous",
			Value:       c.config.Models.Autonomous,
			Type:        "string",
			Description: "Model for autonomous coding loop",
		},
		// Autonomous section
		{
			Label:       "Session Timeout (min)",
			Key:         "autonomous.session_timeout_minutes",
			Value:       strconv.Itoa(c.config.Autonomous.SessionTimeoutMinutes),
			Type:        "int",
			Description: "Maximum session duration in minutes",
		},
		{
			Label:       "Idle Timeout (sec)",
			Key:         "autonomous.idle_timeout_seconds",
			Value:       strconv.Itoa(c.config.Autonomous.IdleTimeoutSeconds),
			Type:        "int",
			Description: "Stop after this many seconds without output",
		},
		{
			Label:       "Auto Commit",
			Key:         "autonomous.auto_commit",
			Value:       strconv.FormatBool(c.config.Autonomous.AutoCommit),
			Type:        "bool",
			Description: "Automatically commit changes after each session",
		},
		// Paths section
		{
			Label:       "App Spec File",
			Key:         "paths.app_spec_file",
			Value:       c.config.Paths.AppSpecFile,
			Type:        "string",
			Description: "Path to the application specification",
		},
		{
			Label:       "Database Path",
			Key:         "paths.database",
			Value:       c.config.Paths.Database,
			Type:        "string",
			Description: "Path to the SQLite database",
		},
		// UI section
		{
			Label:       "Show Progress",
			Key:         "ui.show_progress",
			Value:       strconv.FormatBool(c.config.UI.ShowProgress),
			Type:        "bool",
			Description: "Display progress bar during operations",
		},
	}
}

// Update handles input for the config screen.
func (c *ConfigScreen) Update(msg tea.Msg) (bool, ScreenType) {
	// Load config if not loaded
	if c.config == nil {
		if err := c.LoadConfig(); err != nil {
			c.statusMsg = fmt.Sprintf("Error loading config: %v", err)
		}
	}

	switch msg := msg.(type) {
	case tea.KeyMsg:
		if c.editing {
			return c.handleEditMode(msg)
		}
		return c.handleNavMode(msg)
	}

	return false, ScreenConfig
}

// handleNavMode handles navigation mode input.
func (c *ConfigScreen) handleNavMode(msg tea.KeyMsg) (bool, ScreenType) {
	switch msg.String() {
	case "esc", "q":
		if c.hasChanges {
			c.statusMsg = "Unsaved changes! Press 's' to save or 'Q' to discard"
			return false, ScreenConfig
		}
		return true, ScreenHome

	case "Q":
		// Force quit without saving
		return true, ScreenHome

	case "up", "k":
		if c.cursor > 0 {
			c.cursor--
		}

	case "down", "j":
		if c.cursor < len(c.fields)-1 {
			c.cursor++
		}

	case "enter", "e":
		c.startEditing()

	case " ":
		// Toggle boolean fields
		if c.cursor < len(c.fields) && c.fields[c.cursor].Type == "bool" {
			c.toggleBool()
		}

	case "s":
		if err := c.saveConfig(); err != nil {
			c.statusMsg = fmt.Sprintf("Error saving: %v", err)
		} else {
			c.statusMsg = "Configuration saved!"
			c.hasChanges = false
		}

	case "r":
		// Reset to defaults
		c.config = config.DefaultConfig()
		c.populateFields()
		c.hasChanges = true
		c.statusMsg = "Reset to defaults (press 's' to save)"
	}

	return false, ScreenConfig
}

// handleEditMode handles edit mode input.
func (c *ConfigScreen) handleEditMode(msg tea.KeyMsg) (bool, ScreenType) {
	switch msg.String() {
	case "esc":
		c.editing = false
		c.editBuffer = ""

	case "enter":
		c.finishEditing()

	case "backspace":
		if len(c.editBuffer) > 0 {
			c.editBuffer = c.editBuffer[:len(c.editBuffer)-1]
		}

	default:
		// Add character to buffer
		if len(msg.String()) == 1 {
			c.editBuffer += msg.String()
		}
	}

	return false, ScreenConfig
}

// startEditing begins editing the current field.
func (c *ConfigScreen) startEditing() {
	if c.cursor >= len(c.fields) {
		return
	}

	field := &c.fields[c.cursor]
	if field.Type == "bool" {
		// Toggle instead of edit
		c.toggleBool()
		return
	}

	c.editing = true
	c.editBuffer = field.Value
}

// finishEditing completes editing and validates the value.
func (c *ConfigScreen) finishEditing() {
	if c.cursor >= len(c.fields) {
		c.editing = false
		return
	}

	field := &c.fields[c.cursor]

	// Validate based on type
	switch field.Type {
	case "int":
		if _, err := strconv.Atoi(c.editBuffer); err != nil {
			c.statusMsg = "Invalid number"
			c.editing = false
			c.editBuffer = ""
			return
		}
	}

	field.Value = c.editBuffer
	c.hasChanges = true
	c.editing = false
	c.editBuffer = ""
	c.applyFieldToConfig(field)
}

// toggleBool toggles a boolean field.
func (c *ConfigScreen) toggleBool() {
	if c.cursor >= len(c.fields) {
		return
	}

	field := &c.fields[c.cursor]
	if field.Type != "bool" {
		return
	}

	if field.Value == "true" {
		field.Value = "false"
	} else {
		field.Value = "true"
	}
	c.hasChanges = true
	c.applyFieldToConfig(field)
}

// applyFieldToConfig applies a field value to the config struct.
func (c *ConfigScreen) applyFieldToConfig(field *ConfigField) {
	switch field.Key {
	case "models.default":
		c.config.Models.Default = field.Value
	case "models.autonomous":
		c.config.Models.Autonomous = field.Value
	case "autonomous.session_timeout_minutes":
		if v, err := strconv.Atoi(field.Value); err == nil {
			c.config.Autonomous.SessionTimeoutMinutes = v
		}
	case "autonomous.idle_timeout_seconds":
		if v, err := strconv.Atoi(field.Value); err == nil {
			c.config.Autonomous.IdleTimeoutSeconds = v
		}
	case "autonomous.auto_commit":
		c.config.Autonomous.AutoCommit = field.Value == "true"
	case "paths.app_spec_file":
		c.config.Paths.AppSpecFile = field.Value
	case "paths.database":
		c.config.Paths.Database = field.Value
	case "ui.show_progress":
		c.config.UI.ShowProgress = field.Value == "true"
	}
}

// saveConfig saves the configuration to file.
func (c *ConfigScreen) saveConfig() error {
	if err := c.config.Validate(); err != nil {
		return err
	}
	return config.Save(c.config, c.configPath)
}

// View renders the config screen.
func (c *ConfigScreen) View() string {
	var sb strings.Builder

	sb.WriteString(c.styles.Title.Render("Settings"))
	sb.WriteString("\n\n")

	if c.config == nil {
		sb.WriteString(c.styles.Error.Render("Failed to load configuration"))
		sb.WriteString("\n\n")
		sb.WriteString(c.styles.Muted.Render("[q] Back to menu"))
		return sb.String()
	}

	// Group fields by section
	sections := map[string][]int{
		"Models":     {0, 1},
		"Autonomous": {2, 3, 4},
		"Paths":      {5, 6},
		"UI":         {7},
	}
	sectionOrder := []string{"Models", "Autonomous", "Paths", "UI"}

	for _, section := range sectionOrder {
		indices := sections[section]
		sb.WriteString(c.styles.Subtitle.Render(section))
		sb.WriteString("\n")

		for _, i := range indices {
			if i >= len(c.fields) {
				continue
			}
			field := c.fields[i]
			c.renderField(&sb, i, field)
		}
		sb.WriteString("\n")
	}

	// Status bar
	if c.hasChanges {
		sb.WriteString(c.styles.Highlight.Render("* Unsaved changes"))
		sb.WriteString("\n")
	}

	if c.statusMsg != "" {
		sb.WriteString(c.styles.Muted.Render(c.statusMsg))
		sb.WriteString("\n")
	}

	sb.WriteString("\n")

	// Help
	if c.editing {
		sb.WriteString(c.styles.Highlight.Render("[Enter] Save"))
		sb.WriteString("  ")
		sb.WriteString(c.styles.Muted.Render("[Esc] Cancel"))
	} else {
		sb.WriteString(c.styles.Highlight.Render("[e] Edit"))
		sb.WriteString("  ")
		sb.WriteString(c.styles.Highlight.Render("[s] Save"))
		sb.WriteString("  ")
		sb.WriteString(c.styles.Highlight.Render("[r] Reset"))
		sb.WriteString("  ")
		sb.WriteString(c.styles.Muted.Render("[q] Back"))
	}

	return sb.String()
}

// renderField renders a single config field.
func (c *ConfigScreen) renderField(sb *strings.Builder, index int, field ConfigField) {
	prefix := "  "
	if index == c.cursor {
		prefix = "> "
	}

	var style = c.styles.MenuItem
	if index == c.cursor {
		style = c.styles.MenuItemSelected
	}

	// Format value display
	valueDisplay := field.Value
	if c.editing && index == c.cursor {
		valueDisplay = c.editBuffer + "_"
	} else if field.Type == "bool" {
		if field.Value == "true" {
			valueDisplay = "[✓]"
		} else {
			valueDisplay = "[ ]"
		}
	}

	line := fmt.Sprintf("%s%-25s %s", prefix, field.Label+":", valueDisplay)
	sb.WriteString(style.Render(line))
	sb.WriteString("\n")

	// Show description for selected field
	if index == c.cursor && field.Description != "" {
		sb.WriteString(c.styles.Muted.Render(fmt.Sprintf("    %s", field.Description)))
		sb.WriteString("\n")
	}
}

// SetConfigPath sets the path to the config file.
func (c *ConfigScreen) SetConfigPath(path string) {
	c.configPath = path
}

// GetConfig returns the current configuration.
func (c *ConfigScreen) GetConfig() *config.Config {
	return c.config
}

// HasChanges returns whether there are unsaved changes.
func (c *ConfigScreen) HasChanges() bool {
	return c.hasChanges
}
