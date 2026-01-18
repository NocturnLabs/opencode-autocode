package templates

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestSubstitute(t *testing.T) {
	tmpl := New()

	content := `Hello {{name}}, welcome to {{project}}.`
	vars := map[string]string{
		"name":    "User",
		"project": "Forger",
	}

	result := tmpl.Substitute(content, vars)
	assert.Equal(t, "Hello User, welcome to Forger.", result)
}

func TestSubstituteMultiple(t *testing.T) {
	tmpl := New()

	content := `{{first}}, {{second}}, {{third}}, {{second}}`
	vars := map[string]string{
		"first":  "A",
		"second": "B",
		"third":  "C",
	}

	result := tmpl.Substitute(content, vars)
	assert.Equal(t, "A, B, C, B", result)
}

func TestCircularDependency(t *testing.T) {
	tmpl := New()

	// Create a circular dependency scenario by manually managing the visited map
	content := `{{INCLUDE file1.xml}}`
	visited := map[string]bool{"file1.xml": true}

	// This should fail with circular dependency error because we manually mark it as visited
	_, err := tmpl.ResolveIncludes(content, visited)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "circular")
}
