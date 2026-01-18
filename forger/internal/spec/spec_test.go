package spec

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNewAppSpec(t *testing.T) {
	spec := New("Test Project")

	assert.Equal(t, "Test Project", spec.ProjectName)
	assert.Empty(t, spec.Features)
}

func TestToSpecText(t *testing.T) {
	spec := &AppSpec{
		ProjectName: "Test Project",
		Overview:    "A test project",
		Features: []Feature{
			{
				Name:     "Feature 1",
				Desc:     "A feature",
				Priority: "high",
			},
		},
		Technology: &TechStack{
			Languages:  "Go",
			Frameworks: "Bubble Tea",
		},
	}

	text := spec.ToSpecText()
	assert.Contains(t, text, "<project_name>Test Project</project_name>")
	assert.Contains(t, text, "Feature 1")
	assert.Contains(t, text, "Go")
}

func TestFromText(t *testing.T) {
	xmlText := `<project_specification>
  <project_name>Test</project_name>
  <overview>Test overview</overview>
  <core_features>
    <feature priority="high">
      <name>Feature</name>
      <description>Description</description>
    </feature>
  </core_features>
</project_specification>`

	spec, err := FromText(xmlText)
	assert.NoError(t, err)
	assert.Equal(t, "Test", spec.ProjectName)
	assert.Equal(t, 1, len(spec.Features))
}

func TestExtractSpecFromOutput(t *testing.T) {
	output := `
Some preamble text...

<project_specification>
<project_name>Test Project</project_name>
<overview>A test</overview>
</project_specification>

Some trailing text...
`

	spec, err := ExtractSpecFromOutput(output)
	assert.NoError(t, err)
	assert.Contains(t, spec, "<project_specification>")
	assert.Contains(t, spec, "Test Project")
}

func TestExtractSpecNoMatch(t *testing.T) {
	output := "This is just random text without any spec"
	_, err := ExtractSpecFromOutput(output)
	assert.Error(t, err)
}

func TestTruncate(t *testing.T) {
	s := strings.Repeat("a", 1000)
	result := truncate(s, 500)
	assert.Equal(t, 500, len(result))
}
