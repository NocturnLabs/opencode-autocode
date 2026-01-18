package spec

import (
	"encoding/xml"
	"fmt"
	"strings"
)

// AppSpec represents an application specification
type AppSpec struct {
	XMLName      xml.Name   `xml:"project_specification"`
	ProjectName  string     `xml:"project_name"`
	Overview     string     `xml:"overview"`
	Features     []Feature  `xml:"core_features>feature"`
	Technology   *TechStack `xml:"technology_stack,omitempty"`
	Database     *Database  `xml:"database,omitempty"`
	APIEndpoints []Endpoint `xml:"api_endpoints>endpoint,omitempty"`
}

// Feature represents a feature in spec
type Feature struct {
	XMLName  xml.Name `xml:"feature"`
	Name     string   `xml:"name"`
	Desc     string   `xml:"description"` // Renamed to avoid conflict
	Priority string   `xml:"priority,attr"`
}

// TechStack represents technology stack
type TechStack struct {
	Languages  string `xml:"languages"` // Changed to string for simpler parsing
	Frameworks string `xml:"frameworks"`
	Tools      string `xml:"tools,omitempty"`
}

// Database represents database configuration
type Database struct {
	Type   string `xml:"type"`
	Tables Tables `xml:"tables"`
}

// Tables wraps table names
type Tables struct {
	Names []string `xml:"-"`
}

// Endpoint represents an API endpoint
type Endpoint struct {
	Method      string `xml:"method"`
	Path        string `xml:"path"`
	Description string `xml:"description"`
}

// New creates a new empty AppSpec
func New(name string) *AppSpec {
	return &AppSpec{
		ProjectName: name,
		Features:    []Feature{},
	}
}

// ToSpecText converts AppSpec to XML-like text format
func (a *AppSpec) ToSpecText() string {
	var b strings.Builder

	b.WriteString("<project_specification>\n")
	b.WriteString(fmt.Sprintf("  <project_name>%s</project_name>\n\n", a.ProjectName))

	b.WriteString("  <overview>\n")
	b.WriteString(fmt.Sprintf("    %s\n", a.Overview))
	b.WriteString("  </overview>\n\n")

	if a.Technology != nil {
		b.WriteString("  <technology_stack>\n")
		if a.Technology.Languages != "" {
			b.WriteString(fmt.Sprintf("    <languages>%s</languages>\n", a.Technology.Languages))
		}
		if a.Technology.Frameworks != "" {
			b.WriteString(fmt.Sprintf("    <frameworks>%s</frameworks>\n", a.Technology.Frameworks))
		}
		if a.Technology.Tools != "" {
			b.WriteString(fmt.Sprintf("    <tools>%s</tools>\n", a.Technology.Tools))
		}
		b.WriteString("  </technology_stack>\n\n")
	}

	b.WriteString("  <core_features>\n")
	for _, feature := range a.Features {
		b.WriteString(fmt.Sprintf("    <feature priority=\"%s\">\n", feature.Priority))
		b.WriteString(fmt.Sprintf("      <name>%s</name>\n", feature.Name))
		b.WriteString(fmt.Sprintf("      <description>%s</description>\n", feature.Desc))
		b.WriteString("    </feature>\n")
	}
	b.WriteString("  </core_features>\n\n")

	if a.Database != nil {
		b.WriteString("  <database>\n")
		b.WriteString(fmt.Sprintf("    <type>%s</type>\n", a.Database.Type))
		b.WriteString("    <tables>\n")
		for _, table := range a.Database.Tables.Names {
			b.WriteString(fmt.Sprintf("      - %s\n", table))
		}
		b.WriteString("    </tables>\n")
		b.WriteString("  </database>\n\n")
	}

	if len(a.APIEndpoints) > 0 {
		b.WriteString("  <api_endpoints>\n")
		for _, ep := range a.APIEndpoints {
			b.WriteString("    <endpoint>\n")
			b.WriteString(fmt.Sprintf("      <method>%s</method>\n", ep.Method))
			b.WriteString(fmt.Sprintf("      <path>%s</path>\n", ep.Path))
			b.WriteString(fmt.Sprintf("      <description>%s</description>\n", ep.Description))
			b.WriteString("    </endpoint>\n")
		}
		b.WriteString("  </api_endpoints>\n\n")
	}

	b.WriteString("  <success_criteria>\n")
	b.WriteString("  </success_criteria>\n")

	b.WriteString("</project_specification>\n")

	return b.String()
}

// FromText parses AppSpec from XML-like text format
func FromText(text string) (*AppSpec, error) {
	var spec AppSpec
	if err := xml.Unmarshal([]byte(text), &spec); err != nil {
		return nil, fmt.Errorf("failed to parse spec: %w", err)
	}
	return &spec, nil
}

// ExtractSpecFromOutput extracts XML specification from OpenCode's output
func ExtractSpecFromOutput(output string) (string, error) {
	// Look for XML specification block
	if start := strings.Index(output, "<project_specification>"); start != -1 {
		if end := strings.Index(output, "</project_specification>"); end != -1 {
			spec := output[start : end+len("</project_specification>")]
			return spec, nil
		}
	}

	// Try to find it in markdown code blocks
	if start := strings.Index(output, "```xml"); start != -1 {
		if end := strings.Index(output[start:], "```"); end != -1 {
			block := output[start+6 : start+end]
			if strings.Contains(block, "<project_specification>") {
				return strings.TrimSpace(block), nil
			}
		}
	}

	// If we can't find XML, check if output contains spec fragments
	if strings.Contains(output, "<project_name>") && strings.Contains(output, "<overview>") {
		return "", fmt.Errorf(
			"could not extract complete specification. The AI response may be malformed. Please try again.",
		)
	}

	return "", fmt.Errorf(
		"no project specification found in OpenCode output. The AI may have encountered an error.\n\nPartial output:\n%s",
		truncate(output, 500),
	)
}

// truncate truncates string to max length
func truncate(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen]
}
