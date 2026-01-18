package scaffold

import (
	"fmt"

	"github.com/yum-inc/opencode-forger/internal/opencode"
	"github.com/yum-inc/opencode-forger/internal/spec"
	"github.com/yum-inc/opencode-forger/internal/templates"
)

// Generator handles spec generation from ideas
type Generator struct {
	opencodeClient *opencode.Client
	templates      *templates.Templates
}

// NewGenerator creates a new spec generator
func NewGenerator(oc *opencode.Client, tmpl *templates.Templates) *Generator {
	return &Generator{
		opencodeClient: oc,
		templates:      tmpl,
	}
}

// GenerateSpec generates a spec from a project idea
func (g *Generator) GenerateSpec(idea string, model string) (*spec.AppSpec, error) {
	// Load generator prompt template
	prompt, err := g.templates.Load("generator_prompt.xml")
	if err != nil {
		return nil, fmt.Errorf("failed to load generator prompt: %w", err)
	}

	// Substitute variables
	vars := map[string]string{
		"PROJECT_IDEA": idea,
	}
	fullPrompt := templates.New().Substitute(prompt, vars)

	// Generate spec using OpenCode
	g.opencodeClient.SetModel(model)
	output, err := g.opencodeClient.RunSimple("generate", fullPrompt)
	if err != nil {
		return nil, fmt.Errorf("failed to generate spec: %w", err)
	}

	// Extract spec from output
	specText, err := spec.ExtractSpecFromOutput(output)
	if err != nil {
		return nil, err
	}

	// Parse spec
	appSpec, err := spec.FromText(specText)
	if err != nil {
		return nil, fmt.Errorf("failed to parse spec: %w", err)
	}

	return appSpec, nil
}

// RefineSpec refines an existing spec
func (g *Generator) RefineSpec(appSpec *spec.AppSpec, instructions string, model string) (*spec.AppSpec, error) {
	// Load refine prompt template
	prompt, err := g.templates.Load("refine_prompt.xml")
	if err != nil {
		return nil, fmt.Errorf("failed to load refine prompt: %w", err)
	}

	// Substitute variables
	vars := map[string]string{
		"SPEC_TEXT":    appSpec.ToSpecText(),
		"INSTRUCTIONS": instructions,
	}
	fullPrompt := templates.New().Substitute(prompt, vars)

	// Refine spec using OpenCode
	g.opencodeClient.SetModel(model)
	output, err := g.opencodeClient.RunSimple("refine", fullPrompt)
	if err != nil {
		return nil, fmt.Errorf("failed to refine spec: %w", err)
	}

	// Extract spec from output
	specText, err := spec.ExtractSpecFromOutput(output)
	if err != nil {
		return nil, err
	}

	// Parse spec
	newSpec, err := spec.FromText(specText)
	if err != nil {
		return nil, fmt.Errorf("failed to parse refined spec: %w", err)
	}

	return newSpec, nil
}

// FixMalformedSpec attempts to fix malformed XML in a spec
func (g *Generator) FixMalformedSpec(badSpec string, model string) (*spec.AppSpec, error) {
	// Load fix prompt template
	prompt, err := g.templates.Load("generator/fix_malformed_xml.xml")
	if err != nil {
		return nil, fmt.Errorf("failed to load fix prompt: %w", err)
	}

	// Substitute variables
	vars := map[string]string{
		"MALFORMED_XML": badSpec,
	}
	fullPrompt := templates.New().Substitute(prompt, vars)

	// Fix spec using OpenCode
	g.opencodeClient.SetModel(model)
	output, err := g.opencodeClient.RunSimple("fix", fullPrompt)
	if err != nil {
		return nil, fmt.Errorf("failed to fix spec: %w", err)
	}

	// Extract spec from output
	specText, err := spec.ExtractSpecFromOutput(output)
	if err != nil {
		return nil, err
	}

	// Parse spec
	appSpec, err := spec.FromText(specText)
	if err != nil {
		return nil, fmt.Errorf("failed to parse fixed spec: %w", err)
	}

	return appSpec, nil
}
