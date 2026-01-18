// Package enhance provides enhancement discovery and implementation functionality.
// It enables continuous improvement of existing projects by discovering new features,
// best practices, and optimizations through AI-powered analysis.
package enhance

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/yum-inc/opencode-forger/internal/config"
	"github.com/yum-inc/opencode-forger/internal/opencode"
	"github.com/yum-inc/opencode-forger/internal/templates"
)

// Mode represents the enhancement mode type.
type Mode int

const (
	// ModeDiscover only discovers and proposes enhancements.
	ModeDiscover Mode = iota
	// ModeImplement discovers and implements approved enhancements.
	ModeImplement
)

// Enhancement represents a proposed enhancement.
type Enhancement struct {
	Name           string
	Description    string
	Difficulty     string // Easy, Medium, Hard
	Priority       string // High, Medium, Low
	Impact         string
	Implementation string
	Source         string
	Approved       bool
}

// Result represents the result of an enhancement session.
type Result struct {
	Enhancements []Enhancement
	SessionLog   string
	Duration     time.Duration
	Success      bool
	Error        string
}

// Enhancer handles enhancement discovery and implementation.
type Enhancer struct {
	client     *opencode.Client
	templates  *templates.Templates
	config     *config.Config
	mode       Mode
	projectDir string
}

// NewEnhancer creates a new enhancer instance.
func NewEnhancer(client *opencode.Client, tmpl *templates.Templates, cfg *config.Config) *Enhancer {
	return &Enhancer{
		client:     client,
		templates:  tmpl,
		config:     cfg,
		mode:       ModeDiscover,
		projectDir: ".",
	}
}

// SetMode sets the enhancement mode.
func (e *Enhancer) SetMode(mode Mode) {
	e.mode = mode
}

// SetProjectDir sets the project directory.
func (e *Enhancer) SetProjectDir(dir string) {
	e.projectDir = dir
}

// Discover runs enhancement discovery for the project.
// It analyzes the codebase and proposes improvements without implementing them.
func (e *Enhancer) Discover(handler opencode.OutputHandler) (*Result, error) {
	start := time.Now()
	result := &Result{
		Enhancements: []Enhancement{},
	}

	// Load the enhance prompt template
	prompt, err := e.buildDiscoveryPrompt()
	if err != nil {
		result.Error = err.Error()
		return result, err
	}

	// Set model for enhancement
	e.client.SetModel(e.config.Models.Autonomous)

	// Run OpenCode to discover enhancements
	if err := e.client.Run("enhance", prompt, handler); err != nil {
		result.Error = err.Error()
		result.Duration = time.Since(start)
		return result, err
	}

	// Parse the proposed enhancements from the output file
	enhancements, err := e.parseProposedEnhancements()
	if err != nil {
		// Not a fatal error - just means no enhancements file was created yet
		result.SessionLog = fmt.Sprintf("Discovery completed but no enhancements file found: %v", err)
	} else {
		result.Enhancements = enhancements
	}

	result.Duration = time.Since(start)
	result.Success = true
	return result, nil
}

// Implement runs enhancement implementation for approved enhancements.
func (e *Enhancer) Implement(enhancement Enhancement, handler opencode.OutputHandler) error {
	prompt := e.buildImplementationPrompt(enhancement)

	// Set model for implementation
	e.client.SetModel(e.config.Models.Autonomous)

	return e.client.Run("implement", prompt, handler)
}

// buildDiscoveryPrompt builds the prompt for enhancement discovery.
func (e *Enhancer) buildDiscoveryPrompt() (string, error) {
	// Try to load the template
	content, err := e.templates.Load("commands/auto-enhance.xml")
	if err != nil {
		// Use built-in prompt if template not found
		content = defaultDiscoveryPrompt
	}

	// Substitute variables
	vars := map[string]string{
		"APP_SPEC_PATH": e.config.Paths.AppSpecFile,
	}

	return e.templates.Substitute(content, vars), nil
}

// buildImplementationPrompt builds the prompt for implementing an enhancement.
func (e *Enhancer) buildImplementationPrompt(enhancement Enhancement) string {
	return fmt.Sprintf(`# Enhancement Implementation

## Enhancement: %s

%s

### Implementation Notes
%s

### Priority: %s
### Difficulty: %s

---

Please implement this enhancement for the current project.

1. Read the app spec at %s to understand the project
2. Implement the enhancement following project conventions
3. Add appropriate tests
4. Update documentation if needed
5. Verify the implementation works correctly

Do NOT make breaking changes to existing functionality.
`,
		enhancement.Name,
		enhancement.Description,
		enhancement.Implementation,
		enhancement.Priority,
		enhancement.Difficulty,
		e.config.Paths.AppSpecFile,
	)
}

// parseProposedEnhancements parses the proposed_enhancements.md file.
func (e *Enhancer) parseProposedEnhancements() ([]Enhancement, error) {
	filePath := filepath.Join(e.projectDir, "proposed_enhancements.md")
	data, err := os.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read proposed enhancements: %w", err)
	}

	return parseEnhancementsMarkdown(string(data))
}

// parseEnhancementsMarkdown parses enhancement entries from markdown content.
func parseEnhancementsMarkdown(content string) ([]Enhancement, error) {
	var enhancements []Enhancement
	var current *Enhancement

	lines := strings.Split(content, "\n")
	for _, line := range lines {
		line = strings.TrimSpace(line)

		// New enhancement header
		if strings.HasPrefix(line, "## Enhancement") || strings.HasPrefix(line, "## ") {
			if current != nil {
				enhancements = append(enhancements, *current)
			}
			name := strings.TrimPrefix(line, "## Enhancement")
			name = strings.TrimPrefix(name, "## ")
			name = strings.TrimPrefix(name, ":")
			name = strings.TrimSpace(name)
			// Remove leading numbers like "1: " or "2: "
			if len(name) > 2 && name[1] == ':' {
				name = strings.TrimSpace(name[2:])
			}
			current = &Enhancement{Name: name}
			continue
		}

		if current == nil {
			continue
		}

		// Parse fields
		if strings.HasPrefix(line, "- **Description**:") {
			current.Description = strings.TrimSpace(strings.TrimPrefix(line, "- **Description**:"))
		} else if strings.HasPrefix(line, "- **Difficulty**:") {
			current.Difficulty = strings.TrimSpace(strings.TrimPrefix(line, "- **Difficulty**:"))
		} else if strings.HasPrefix(line, "- **Priority**:") {
			current.Priority = strings.TrimSpace(strings.TrimPrefix(line, "- **Priority**:"))
		} else if strings.HasPrefix(line, "- **Impact**:") {
			current.Impact = strings.TrimSpace(strings.TrimPrefix(line, "- **Impact**:"))
		} else if strings.HasPrefix(line, "- **Implementation Notes**:") {
			current.Implementation = strings.TrimSpace(strings.TrimPrefix(line, "- **Implementation Notes**:"))
		} else if strings.HasPrefix(line, "- **Source**:") {
			current.Source = strings.TrimSpace(strings.TrimPrefix(line, "- **Source**:"))
		}
	}

	// Don't forget the last one
	if current != nil {
		enhancements = append(enhancements, *current)
	}

	return enhancements, nil
}

// SaveProposedEnhancements saves enhancements to the proposed_enhancements.md file.
func (e *Enhancer) SaveProposedEnhancements(enhancements []Enhancement) error {
	var sb strings.Builder

	sb.WriteString("# Proposed Enhancements\n\n")
	sb.WriteString(fmt.Sprintf("Generated: %s\n\n", time.Now().Format(time.RFC3339)))

	for i, enh := range enhancements {
		sb.WriteString(fmt.Sprintf("## Enhancement %d: %s\n\n", i+1, enh.Name))
		sb.WriteString(fmt.Sprintf("- **Description**: %s\n", enh.Description))
		sb.WriteString(fmt.Sprintf("- **Difficulty**: %s\n", enh.Difficulty))
		sb.WriteString(fmt.Sprintf("- **Priority**: %s\n", enh.Priority))
		sb.WriteString(fmt.Sprintf("- **Impact**: %s\n", enh.Impact))
		sb.WriteString(fmt.Sprintf("- **Implementation Notes**: %s\n", enh.Implementation))
		sb.WriteString(fmt.Sprintf("- **Source**: %s\n", enh.Source))
		sb.WriteString("\n")
	}

	filePath := filepath.Join(e.projectDir, "proposed_enhancements.md")
	return os.WriteFile(filePath, []byte(sb.String()), 0644)
}

// GetProposedEnhancements loads and returns the current proposed enhancements.
func (e *Enhancer) GetProposedEnhancements() ([]Enhancement, error) {
	return e.parseProposedEnhancements()
}

// ApproveEnhancement marks an enhancement as approved for implementation.
func (e *Enhancer) ApproveEnhancement(index int) error {
	enhancements, err := e.GetProposedEnhancements()
	if err != nil {
		return err
	}

	if index < 0 || index >= len(enhancements) {
		return fmt.Errorf("invalid enhancement index: %d", index)
	}

	enhancements[index].Approved = true
	return e.SaveProposedEnhancements(enhancements)
}

// defaultDiscoveryPrompt is the built-in enhancement discovery prompt.
const defaultDiscoveryPrompt = `# Enhancement Discovery Session

Research and propose enhancements for the current project based on popular patterns, best practices, and community recommendations.

### STEP 1: UNDERSTAND THE PROJECT

Read {{APP_SPEC_PATH}} to understand:
- What the project does
- What technology stack is used
- Current features and goals

### STEP 2: RESEARCH ENHANCEMENTS

Consider these areas:
- Performance optimizations
- Code quality improvements
- New features users might want
- Security enhancements
- Documentation improvements
- Testing coverage

### STEP 3: PROPOSE ENHANCEMENTS

Create proposed_enhancements.md with:

` + "```markdown" + `
# Proposed Enhancements

## Enhancement 1: [Name]

- **Description**: What this enhancement adds
- **Difficulty**: Easy / Medium / Hard
- **Priority**: High / Medium / Low
- **Impact**: What value it provides
- **Implementation Notes**: Brief approach
- **Source**: Where you found this recommendation
` + "```" + `

Order enhancements by priority (highest impact, lowest effort first).

### STEP 4: DO NOT AUTO-IMPLEMENT

This command is for DISCOVERY only. Research, document, and propose additions.
Wait for user approval before implementing.

Begin by reading {{APP_SPEC_PATH}} to understand the project.
`
