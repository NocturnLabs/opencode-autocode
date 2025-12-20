# Task: Select Best Implementation Approach

Analyze the provided implementation approaches and select the BEST one for this specific project based on context.

## Critical Instruction

**IGNORE the probability scores.** They indicate conventionality, not quality. A low-probability approach may be the perfect choice for this project's specific needs.

## Selection Criteria (Priority Order)

1. **Project Alignment**: Does this approach fit the project's architecture and goals?
2. **Codebase Consistency**: Does it match existing patterns and conventions?
3. **Technical Fit**: Is it appropriate for the technology stack?
4. **Feature Requirements**: Does it handle the specific feature requirements well?
5. **Maintainability**: Will this be easy to maintain and extend?
6. **Risk Assessment**: Given the feature's priority, is the risk level appropriate?

## Context

**Project Overview:**
{{PROJECT_OVERVIEW}}

**Technology Stack:**
{{TECH_STACK}}

**Existing Codebase Patterns:**
{{CODEBASE_PATTERNS}}

**Feature Being Implemented:**
{{FEATURE_DESCRIPTION}}

**Feature Priority:** {{FEATURE_PRIORITY}}
(high = prefer safer approaches, low = can take more creative risks)

**Risk Tolerance:** {{RISK_LEVEL}}
(conservative = prefer proven patterns, experimental = open to innovation)

## Approaches to Evaluate

{{APPROACHES_JSON}}

## Process

1. Analyze each approach against the selection criteria
2. Consider how well each approach fits THIS SPECIFIC PROJECT
3. Don't default to high-probability options—evaluate based on context
4. If a creative (low-probability) approach is a better fit, select it
5. Provide clear justification for the selection
6. Note any adaptations needed to fit the project

## Output Format

```json
{
  "selected_index": <0-9 index of selected approach>,
  "approach": "<the text of the selected approach>",
  "probability_override_reason": "<if selecting a low/medium probability option, explain why it's better for this context>",
  "justification": "<2-3 sentences explaining why this approach is the best fit>",
  "alignment_scores": {
    "project_alignment": <1-5>,
    "codebase_consistency": <1-5>,
    "technical_fit": <1-5>,
    "feature_requirements": <1-5>,
    "maintainability": <1-5>
  },
  "adaptations": ["<any modifications needed to fit project patterns>"],
  "implementation_notes": "<specific notes for the implementing agent>"
}
```

## Example Reasoning

If a project uses event-driven architecture throughout:

- Approach #7 (probability 0.15) uses events
- Approach #1 (probability 0.85) uses synchronous REST

Select approach #7 despite lower probability.
Reason: "Codebase uses event-driven patterns; this approach maintains architectural consistency."

Analyze the approaches and select the best one for this project.
