// Package scaffold provides project scaffolding functionality.
// It creates project directories, config files, and initializes the forger structure.
package scaffold

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/yum-inc/opencode-forger/internal/spec"
	"github.com/yum-inc/opencode-forger/internal/templates"
)

// Scaffold handles project scaffolding
type Scaffold struct {
	templates *templates.Templates
}

// NewScaffold creates a new scaffolder
func NewScaffold(tmpl *templates.Templates) *Scaffold {
	return &Scaffold{
		templates: tmpl,
	}
}

// ScaffoldFromSpec scaffolds a project from a spec
func (s *Scaffold) ScaffoldFromSpec(appSpec *spec.AppSpec, projectDir string) error {
	// Create project directory
	if err := os.MkdirAll(projectDir, 0755); err != nil {
		return fmt.Errorf("failed to create project directory: %w", err)
	}

	// Create .forger directory
	forgerDir := filepath.Join(projectDir, ".forger")
	if err := os.MkdirAll(forgerDir, 0755); err != nil {
		return fmt.Errorf("failed to create .forger directory: %w", err)
	}

	// Create .opencode directory
	opencodeDir := filepath.Join(projectDir, ".opencode")
	if err := os.MkdirAll(opencodeDir, 0755); err != nil {
		return fmt.Errorf("failed to create .opencode directory: %w", err)
	}

	// Write app_spec.md
	specPath := filepath.Join(forgerDir, "app_spec.md")
	if err := os.WriteFile(specPath, []byte(appSpec.ToSpecText()), 0644); err != nil {
		return fmt.Errorf("failed to write app_spec.md: %w", err)
	}

	// Write forger.toml
	tomlPath := filepath.Join(projectDir, "forger.toml")
	tomlContent := s.generateForgerToml(appSpec)
	if err := os.WriteFile(tomlPath, []byte(tomlContent), 0644); err != nil {
		return fmt.Errorf("failed to write forger.toml: %w", err)
	}

	// Write opencode.json
	opencodePath := filepath.Join(projectDir, "opencode.json")
	opencodeContent := s.generateOpencodeJson(appSpec)
	if err := os.WriteFile(opencodePath, []byte(opencodeContent), 0644); err != nil {
		return fmt.Errorf("failed to write opencode.json: %w", err)
	}

	// Write AGENTS.md
	agentsPath := filepath.Join(projectDir, "AGENTS.md")
	if err := s.templates.WriteTemplate("AGENTS.md", agentsPath, nil); err != nil {
		return fmt.Errorf("failed to write AGENTS.md: %w", err)
	}

	return nil
}

// generateForgerToml generates forger.toml content
func (s *Scaffold) generateForgerToml(appSpec *spec.AppSpec) string {
	return fmt.Sprintf(`# forger.toml - OpenCode Forger configuration

[models]
default = "opencode/glm-4.7-free"
autonomous = "opencode/minimax-m2.1-free"

[autonomous]
session_timeout_minutes = 15
idle_timeout_seconds = 600
auto_commit = true

[paths]
app_spec_file = ".forger/app_spec.md"
database = ".forger/progress.db"

[ui]
show_progress = true
`)
}

// generateOpencodeJson generates opencode.json content
func (s *Scaffold) generateOpencodeJson(appSpec *spec.AppSpec) string {
	return fmt.Sprintf(`{
  "model": "opencode/glm-4.7-free",
  "project_name": "%s",
  "description": "%s"
}`, appSpec.ProjectName, appSpec.Overview)
}

// LoadAndWrite loads a template, resolves includes, and writes to file
func (s *Scaffold) LoadAndWrite(templatePath string, outputPath string, vars map[string]string) error {
	content, err := s.templates.LoadAndResolve(templatePath)
	if err != nil {
		return err
	}

	if vars != nil {
		content = s.templates.Substitute(content, vars)
	}

	// Create directory if needed
	dir := filepath.Dir(outputPath)
	if dir != "." {
		if err := os.MkdirAll(dir, 0755); err != nil {
			return err
		}
	}

	return os.WriteFile(outputPath, []byte(content), 0644)
}
