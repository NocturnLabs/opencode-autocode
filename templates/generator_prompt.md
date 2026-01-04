# Task: Generate Comprehensive Project Specification

Create a production-grade project specification based on the user's idea. The output should be detailed enough to drive an autonomous coding agent through complete implementation.

## Critical Requirements

**DO NOT create minimal or MVP-style specs.** Generate specifications suitable for:

- A YC-backed startup's initial product
- A mature open-source project's v1.0 release
- Enterprise-grade software
- Local software with robust logging, clean architecture, and professional error handling

## Research First

Before generating the spec:

1. Use web search to research current best practices for this type of project
2. Study what similar applications offer (competitor analysis)
3. Verify library/framework recommendations using documentation tools
4. Identify edge cases and failure modes common in this domain

## Scope Requirements

The specification scope should **scale with the project's complexity**. Do NOT apply arbitrary minimums.

**Guiding Principles:**

- Include **at least {{MIN_FEATURES}} features** for a production-ready, "finished" product.
- Include **ALL database tables** required by the features (minimum {{MIN_DATABASE_TABLES}} if applicable).
- Include **ALL API endpoints** (minimum {{MIN_API_ENDPOINTS}} if applicable).
- Include **at least {{MIN_STEPS}} implementation phases** to logically break down the work.

**{{COMPLEXITY_GUIDANCE}}**

## Output Format

Output ONLY valid XML following this structure. Fill every section with substantial content—no placeholders.

```xml
<project_specification>
<project_name>Name of the Project</project_name>

  <overview>
    5-10 sentences covering the full scope, purpose, target users,
    and key differentiators.
  </overview>

<technology_stack>
<frontend>
<framework>Choice with brief justification</framework>
<styling>Styling approach</styling>
<state_management>State solution</state_management>
<routing>Router choice</routing>
<port>Frontend port (e.g. 3000)</port>
<testing>Test framework</testing>
</frontend>
<backend>
<runtime>Language/runtime</runtime>
<framework>Framework choice</framework>
<database>Database with justification</database>
<auth>Auth approach</auth>
<api_style>REST/GraphQL/tRPC</api_style>
<port>Backend port (e.g. 8080)</port>
<testing>Test framework</testing>
</backend>
</technology_stack>

  <prerequisites>
    <environment_setup>
      - Required environment variables
      - System dependencies
      - Port availability
    </environment_setup>
  </prerequisites>

<core_features>
<!-- Minimum {{MIN_FEATURES}} features -->
<feature priority="high|medium|low">
  <name>Feature Name</name>
  <description>Detailed description</description>
  <sub_features>
    - Sub-feature with implementation detail
    - Sub-feature with implementation detail
  </sub_features>
  <error_handling>Error handling approach</error_handling>
  <edge_cases>Edge case: [specific scenario]</edge_cases>
</feature>
</core_features>

<ui_layout>
  <main_structure>Overall layout and shell</main_structure>
  <sidebar>Navigation and organization</sidebar>
  <main_area>Primary interaction space</main_area>
  <modals_overlays>Required overlays and dialogs</modals_overlays>
</ui_layout>

<design_system>
  <color_palette>Specific hex codes and usage</color_palette>
  <typography>Font choices and hierarchy</typography>
  <components>Custom UI components needed</components>
  <animations>Micro-interactions and transitions</animations>
</design_system>

<key_interactions>
  <primary_flow>Step-by-step user journey</primary_flow>
  <secondary_flows>Alternative paths</secondary_flows>
  <error_states>How the system handles failures</error_states>
</key_interactions>

<database_schema>
<tables>
<!-- Minimum {{MIN_DATABASE_TABLES}} tables -->
<table_name>
 - column_name: type, constraints, purpose
 - foreign_keys: [relationships]
</table_name>
</tables>
</database_schema>

<api_endpoints_summary>
<!-- Minimum {{MIN_API_ENDPOINTS}} endpoints -->
<endpoint>
  <method>GET|POST|PUT|DELETE</method>
  <path>/api/v1/resource</path>
  <description>Description of the endpoint [auth: required|public]</description>
</endpoint>
</api_endpoints_summary>

<testing_strategy>
<unit_tests>Coverage targets and priority areas</unit_tests>
<e2e_tests>
 - Critical flows to cover
 - **MANDATORY**: Every core feature MUST have a scriptable E2E test
 - verification_command MUST invoke E2E tests
</e2e_tests>
<entry_point_verification>
 - The main entry point MUST be wired to call all handlers.
 - verification_command MUST check that the application RUNS.
</entry_point_verification>
</testing_strategy>

<implementation_steps>
<!-- Minimum {{MIN_STEPS}} phases -->
<step number="1">
<title>Phase Title</title>
<tasks>
 - Specific task with detail
 - Specific task with detail
</tasks>
<verification>How to confirm this phase is complete</verification>
</step>
</implementation_steps>

<success_criteria>
<functionality>Testable criteria</functionality>
<quality>Code quality and coverage metrics</quality>
</success_criteria>

<future_enhancements>Potential post-launch features</future_enhancements>
</project_specification>
```

## User's Idea

{{IDEA}}

{{TESTING_PREFERENCE}}

---

Generate the complete specification now. Output ONLY the XML—no preamble, no commentary.
ULTRATHINK
