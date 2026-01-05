# Task: Generate Comprehensive Project Specification (Parallel Subagent Mode)

You are orchestrating parallel subagents to generate a production-grade project specification.

## Phase 1: Generate Blueprint (You do this directly)

First, create the foundation:

- **Project name**: Descriptive, memorable name
- **Overview**: 5-10 sentences explaining the project
- **Technology stack**: Frontend, backend, database, devops choices
- **Prerequisites**: Required tools, accounts, and knowledge

## Phase 2: Invoke Subagents

After generating the blueprint, invoke these THREE subagents IN PARALLEL:

```
@spec-product Generate at least {{MIN_FEATURES}} core features and user experience sections for: {{IDEA}}

Use the blueprint above as context.
```

```
@spec-architecture Generate database schema with at least 8 tables and {{MIN_API_ENDPOINTS}} API endpoints for: {{IDEA}}

Use the blueprint above as context.
```

```
@spec-quality Generate security, testing strategy, implementation steps, success criteria, and future enhancements for: {{IDEA}}

Use the blueprint above as context.
```

## Phase 3: Aggregate and Validate

Once all subagents have returned their sections:

1. **Merge** all XML fragments into a single document
2. **Self-validate** the output:
   - Count features: must be >= {{MIN_FEATURES}}
   - Count endpoints: must be >= {{MIN_API_ENDPOINTS}}
   - Verify ALL closing tags are present
   - Ensure no truncated content

## Output Format

> [!CRITICAL]
> Output ONLY the complete, valid XML specification. Do NOT include any explanation or commentary outside the XML tags.

```xml
<project_specification>
  <project_name>...</project_name>
  <overview>...</overview>
  <technology_stack>...</technology_stack>
  <prerequisites>...</prerequisites>

  <!-- From @spec-product -->
  <core_features>
    <feature>
      <name>Feature Name</name>
      <description>...</description>
    </feature>
    <!-- MUST have {{MIN_FEATURES}}+ features -->
  </core_features>
  <user_experience>...</user_experience>

  <!-- From @spec-architecture -->
  <database_schema>...</database_schema>
  <api_endpoints>
    <!-- MUST have {{MIN_API_ENDPOINTS}}+ endpoints -->
  </api_endpoints>

  <!-- From @spec-quality -->
  <security>...</security>
  <testing_strategy>
    <unit_tests>...</unit_tests>
    <integration_tests>...</integration_tests>
    <e2e_tests>...</e2e_tests>
    <entry_point_verification>...</entry_point_verification>
  </testing_strategy>
  <implementation_steps>...</implementation_steps>
  <success_criteria>...</success_criteria>
  <future_enhancements>...</future_enhancements>
</project_specification>
```

## User's Idea

{{IDEA}}

{{TESTING_PREFERENCE}}

---

ULTRATHINK
