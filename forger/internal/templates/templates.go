package templates

import (
	"embed"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
)

// templatesFS embeds template files from the templates directory.
// Note: This embeds from ../../templates relative to this package.
// The templates must be copied or symlinked to be embedded properly.
// For production, templates are loaded from the filesystem.
//
//go:embed templates.go
var templatesFS embed.FS

// Template represents a parsed template.
type Template struct {
	Path    string
	Content string
}

// Templates handles template operations.
// It loads templates from the filesystem with fallback to embedded templates.
type Templates struct {
	baseDir  string
	embedFS  fs.FS
	useEmbed bool
}

// New creates a new Templates instance.
// It attempts to find templates in the filesystem first, falling back to embedded.
func New() *Templates {
	// Try common template locations
	searchPaths := []string{
		"templates",
		"forger/templates",
		"../templates",
		"../../templates",
	}

	for _, path := range searchPaths {
		if info, err := os.Stat(path); err == nil && info.IsDir() {
			return &Templates{
				baseDir:  path,
				useEmbed: false,
			}
		}
	}

	// Fall back to embedded (though limited in this package)
	return &Templates{
		baseDir:  ".",
		embedFS:  templatesFS,
		useEmbed: true,
	}
}

// NewWithBaseDir creates a Templates instance with a specific base directory.
func NewWithBaseDir(baseDir string) *Templates {
	return &Templates{
		baseDir:  baseDir,
		useEmbed: false,
	}
}

// Load loads a template by path (relative to templates directory).
func (t *Templates) Load(path string) (string, error) {
	if t.useEmbed && t.embedFS != nil {
		data, err := fs.ReadFile(t.embedFS, path)
		if err == nil {
			return string(data), nil
		}
	}

	// Load from filesystem
	fullPath := filepath.Join(t.baseDir, path)
	data, err := os.ReadFile(fullPath)
	if err != nil {
		return "", fmt.Errorf("failed to load template %s: %w", path, err)
	}
	return string(data), nil
}

// ResolveIncludes resolves {{INCLUDE path}} directives in template content.
// It prevents circular dependencies by tracking visited paths.
func (t *Templates) ResolveIncludes(content string, visited map[string]bool) (string, error) {
	if visited == nil {
		visited = make(map[string]bool)
	}

	// Find all INCLUDE directives
	const includeStart = "{{INCLUDE "
	const includeEnd = "}}"

	var result strings.Builder
	pos := 0

	for {
		// Find next INCLUDE directive
		startIdx := strings.Index(content[pos:], includeStart)
		if startIdx == -1 {
			// No more includes, append remaining content
			result.WriteString(content[pos:])
			break
		}

		startIdx += pos

		// Find end of directive
		endIdx := strings.Index(content[startIdx:], includeEnd)
		if endIdx == -1 {
			return "", fmt.Errorf("unclosed INCLUDE directive at position %d", startIdx)
		}
		endIdx += startIdx + len(includeEnd)

		// Extract include path
		includePath := strings.TrimSpace(content[startIdx+len(includeStart) : endIdx-len(includeEnd)])

		// Check for circular dependencies
		if visited[includePath] {
			return "", fmt.Errorf("circular INCLUDE dependency detected for %s", includePath)
		}

		// Append content before INCLUDE
		result.WriteString(content[pos:startIdx])

		// Load included template
		includedContent, err := t.Load(includePath)
		if err != nil {
			return "", fmt.Errorf("failed to load INCLUDE %s: %w", includePath, err)
		}

		// Recursively resolve includes in the included content
		visited[includePath] = true
		resolvedInclude, err := t.ResolveIncludes(includedContent, visited)
		if err != nil {
			return "", err
		}
		delete(visited, includePath)

		// Append resolved include
		result.WriteString(resolvedInclude)

		// Move position forward
		pos = endIdx
	}

	return result.String(), nil
}

// Substitute replaces template variables with provided values.
// Variables use the format {{VARIABLE_NAME}}.
func (t *Templates) Substitute(template string, vars map[string]string) string {
	result := template
	for key, value := range vars {
		placeholder := fmt.Sprintf("{{%s}}", key)
		result = strings.ReplaceAll(result, placeholder, value)
	}
	return result
}

// LoadAndResolve loads a template and resolves all includes.
func (t *Templates) LoadAndResolve(path string) (string, error) {
	content, err := t.Load(path)
	if err != nil {
		return "", err
	}
	return t.ResolveIncludes(content, nil)
}

// LoadResolveAndSubstitute loads, resolves includes, and substitutes variables.
func (t *Templates) LoadResolveAndSubstitute(path string, vars map[string]string) (string, error) {
	resolved, err := t.LoadAndResolve(path)
	if err != nil {
		return "", err
	}
	return t.Substitute(resolved, vars), nil
}

// ListFiles returns a list of all template files in the base directory.
func (t *Templates) ListFiles() ([]string, error) {
	var files []string

	err := filepath.Walk(t.baseDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Skip directories
		if info.IsDir() {
			return nil
		}

		// Get relative path
		relPath, err := filepath.Rel(t.baseDir, path)
		if err != nil {
			return err
		}

		files = append(files, relPath)
		return nil
	})

	return files, err
}

// WriteTemplate writes a template to a file after resolving includes and substituting variables.
func (t *Templates) WriteTemplate(path string, outputPath string, vars map[string]string) error {
	content, err := t.LoadAndResolve(path)
	if err != nil {
		return err
	}

	if vars != nil {
		content = t.Substitute(content, vars)
	}

	// Create directory if needed
	dir := filepath.Dir(outputPath)
	if dir != "." {
		if err := os.MkdirAll(dir, 0755); err != nil {
			return fmt.Errorf("failed to create directory %s: %w", dir, err)
		}
	}

	return os.WriteFile(outputPath, []byte(content), 0644)
}

// Exists checks if a template file exists.
func (t *Templates) Exists(path string) bool {
	fullPath := filepath.Join(t.baseDir, path)
	_, err := os.Stat(fullPath)
	return err == nil
}

// GetBaseDir returns the current base directory for templates.
func (t *Templates) GetBaseDir() string {
	return t.baseDir
}
