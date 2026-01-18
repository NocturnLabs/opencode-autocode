package enhance

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestParseEnhancementsMarkdown(t *testing.T) {
	markdown := `# Proposed Enhancements

## Enhancement 1: Add Dark Mode

- **Description**: Add dark mode toggle to settings
- **Difficulty**: Easy
- **Priority**: High
- **Impact**: Improves user experience
- **Implementation Notes**: Use CSS variables
- **Source**: User feedback

## Enhancement 2: Add Export Feature

- **Description**: Export data to CSV
- **Difficulty**: Medium
- **Priority**: Medium
- **Impact**: Better data portability
- **Implementation Notes**: Add export button
- **Source**: Feature request
`

	enhancements, err := parseEnhancementsMarkdown(markdown)
	assert.NoError(t, err)
	assert.Len(t, enhancements, 2)

	// Check first enhancement
	assert.Equal(t, "Add Dark Mode", enhancements[0].Name)
	assert.Equal(t, "Add dark mode toggle to settings", enhancements[0].Description)
	assert.Equal(t, "Easy", enhancements[0].Difficulty)
	assert.Equal(t, "High", enhancements[0].Priority)
	assert.Equal(t, "Improves user experience", enhancements[0].Impact)
	assert.Equal(t, "Use CSS variables", enhancements[0].Implementation)
	assert.Equal(t, "User feedback", enhancements[0].Source)

	// Check second enhancement
	assert.Equal(t, "Add Export Feature", enhancements[1].Name)
	assert.Equal(t, "Medium", enhancements[1].Difficulty)
}

func TestParseEnhancementsMarkdownEmpty(t *testing.T) {
	markdown := `# Proposed Enhancements

No enhancements proposed yet.
`

	enhancements, err := parseEnhancementsMarkdown(markdown)
	assert.NoError(t, err)
	assert.Len(t, enhancements, 0)
}

func TestParseEnhancementsMarkdownAlternateFormat(t *testing.T) {
	markdown := `# Enhancements

## Performance Optimization

- **Description**: Improve loading speed
- **Priority**: High

## Better Documentation

- **Description**: Add more examples
- **Priority**: Low
`

	enhancements, err := parseEnhancementsMarkdown(markdown)
	assert.NoError(t, err)
	assert.Len(t, enhancements, 2)
	assert.Equal(t, "Performance Optimization", enhancements[0].Name)
	assert.Equal(t, "Better Documentation", enhancements[1].Name)
}
