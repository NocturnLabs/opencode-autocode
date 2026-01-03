# Task: Generate Comprehensive Project Specification (Parallel Subagent Mode)

You are orchestrating parallel subagents to generate a production-grade project specification.

## Process

1. **Generate Blueprint First** (you do this directly):

   - Project name
   - Overview (5-10 sentences)
   - Technology stack (frontend, backend, devops)
   - Prerequisites

2. **Invoke Subagents in Parallel**:
   After generating the blueprint, invoke these subagents to complete the specification:

   ```
   @spec-product Generate the core_features and user_experience sections for: {{IDEA}}

   Blueprint context:
   {{BLUEPRINT}}
   ```

   ```
   @spec-architecture Generate the database_schema and api_endpoints sections for: {{IDEA}}

   Blueprint context:
   {{BLUEPRINT}}
   ```

   ```
   @spec-quality Generate the security, testing_strategy, implementation_steps, success_criteria, and future_enhancements sections for: {{IDEA}}

   Blueprint context:
   {{BLUEPRINT}}
   ```

3. **Aggregate Results**:
   Combine all subagent outputs into a single valid XML document.

## Output Format

Output ONLY the complete XML specification:

```xml
<project_specification>
  <project_name>...</project_name>
  <overview>...</overview>
  <technology_stack>...</technology_stack>
  <prerequisites>...</prerequisites>

  <!-- From @spec-product -->
  <core_features>...</core_features>
  <user_experience>...</user_experience>

  <!-- From @spec-architecture -->
  <database_schema>...</database_schema>
  <api_endpoints>...</api_endpoints>

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

Generate the blueprint now, then invoke subagents to complete the specification.
ULTRATHINK
