# Task: Generate Comprehensive Project Specification

Create a production-grade project specification based on the user's idea. The output should be detailed enough to drive an autonomous coding agent through complete implementation.

## Critical Requirements

**DO NOT create minimal or MVP-style specs.** Generate specifications suitable for:

- A YC-backed startup's initial product
- A mature open-source project's v1.0 release
- Enterprise-grade software

## Research First

Before generating the spec:

1. Use web search to research current best practices for this type of project
2. Study what similar applications offer (competitor analysis)
3. Verify library/framework recommendations using documentation tools
4. Identify edge cases and failure modes common in this domain

## Scope Requirements

The specification MUST include:

| Category             | Minimum | Description                                             |
| -------------------- | ------- | ------------------------------------------------------- |
| Core Features        | 15+     | Each with 5-10 sub-features, error handling, edge cases |
| Database Tables      | 10+     | Properly normalized with relationships and indexes      |
| API Endpoints        | 30+     | Organized by resource, with auth requirements noted     |
| Implementation Steps | 8+      | Each with clear deliverables                            |

## Content Guidelines

**Features should cover:**

- Primary user workflows
- Admin/management interfaces
- Search, filtering, and sorting
- Notifications and alerts
- User preferences and settings
- Export/import capabilities
- Analytics and monitoring hooks
- Error states and recovery
- Accessibility considerations
- Mobile/responsive behavior

**Technical sections should address:**

- Authentication and authorization
- Input validation and sanitization
- Rate limiting and abuse prevention
- Data encryption (at rest and in transit)
- Testing strategy (unit, integration, e2e)
- CI/CD and deployment considerations

**For JavaScript/TypeScript projects:**

- ES6 module system (import/export) requirements
- Module bundling approach (Vite, esbuild, webpack)
- Entry point file structure and dependency tree

## Output Format

**CRITICAL XML ESCAPING RULES:**
The output MUST be valid XML. Escape these special characters:

- `&` → `&amp;` (e.g., URLs with query params: `?foo=1&amp;bar=2`)
- `<` → `&lt;` (when showing code or comparisons)
- `>` → `&gt;` (when showing code or comparisons)
- `"` → `&quot;` (inside attribute values)
- `'` → `&apos;` (inside attribute values)

**Common mistakes to avoid:**

- URLs with `&` in query strings (MUST escape as `&amp;`)
- Code examples showing `<` or `>` operators
- Raw HTML examples inside the spec

Output ONLY the specification in this XML structure. Fill every section with substantial content—no placeholders or sparse sections.

```xml
<project_specification>
<project_name>Name of the Project</project_name>

  <overview>
    5-10 sentences covering the full scope, purpose, target users,
    and key differentiators of this application.
  </overview>

<technology_stack>
<frontend>
<framework>Choice with brief justification</framework>
<styling>Styling approach</styling>
<state_management>State solution</state_management>
<routing>Router choice</routing>
<forms>Form handling</forms>
<testing>Test framework</testing>
</frontend>
<backend>
<runtime>Language/runtime</runtime>
<framework>Framework choice</framework>
<database>Database with justification</database>
<cache>Caching layer</cache>
<auth>Auth approach</auth>
<api_style>REST/GraphQL/tRPC</api_style>
<testing>Test framework</testing>
</backend>
<devops>
<containerization>Docker approach</containerization>
<ci_cd>Pipeline approach</ci_cd>
<monitoring>Observability tools</monitoring>
</devops>
</technology_stack>

  <prerequisites>
    <environment_setup>
      - All required environment variables
      - System dependencies
      - Directory structure conventions
      - Dev vs production differences
    </environment_setup>
    <external_services>
      - Required third-party APIs
      - Optional integrations
    </external_services>
  </prerequisites>

<core_features>
<!-- 15+ feature blocks -->
<feature_name>
 - Sub-feature with implementation detail
 - Sub-feature with implementation detail
 - Sub-feature with implementation detail
 - Error handling approach
 - Edge case: [specific scenario]
</feature_name>
</core_features>

<user_experience>
<user_flows>
 - Primary journey: step by step
 - Secondary journeys
 - Admin journey
</user_flows>
<accessibility>
 - WCAG compliance target
 - Screen reader considerations
 - Keyboard navigation
</accessibility>
<responsive_design>
 - Mobile breakpoints
 - Touch interactions
 - Desktop optimizations
</responsive_design>
</user_experience>

<database_schema>
<tables>
<!-- 10+ tables -->
<table_name>
 - column_name: type, constraints, purpose
 - column_name: type, constraints, purpose
 - indexes: [index definitions]
 - foreign_keys: [relationships]
</table_name>
</tables>
<relationships>
 - Table A → Table B (one-to-many)
 - Table C ↔ Table D (many-to-many via join table)
</relationships>
</database_schema>

<api_endpoints>
<!-- 30+ endpoints by category -->
<resource_category>
 - METHOD /path — description [auth: required|public]
 - METHOD /path — description [auth: required|public]
</resource_category>
</api_endpoints>

<security>
<authentication>
 - Strategy (JWT, sessions, OAuth)
 - Token/session management
 - Password requirements
</authentication>
<authorization>
 - Role definitions
 - Permission model
 - Resource-level access
</authorization>
<data_protection>
 - Encryption approach
 - PII handling
 - Data retention policy
</data_protection>
<input_validation>
 - Sanitization rules
 - Rate limiting thresholds
</input_validation>
</security>

<testing_strategy>
<unit_tests>
 - Coverage targets
 - Priority areas
</unit_tests>
<integration_tests>
 - API testing approach
 - Database testing
</integration_tests>
<e2e_tests>
 - Critical flows to cover
 - Browser/device matrix
 - **MANDATORY**: Every core feature MUST have a scriptable E2E test (Playwright/Cypress)
 - verification_command in the database MUST invoke E2E tests, NOT unit tests
</e2e_tests>
<interactive_verification>
 - For web projects: Agent MUST use chrome-devtools MCP for manual verification
 - Check browser console for errors after each feature implementation
 - This is in ADDITION to automated E2E tests
</interactive_verification>
<module_verification>
 - All ES6 imports resolve correctly (no ReferenceErrors)
 - Named exports match import statements
 - Relative paths use correct file extensions (.js for browser)
 - No circular dependency issues
</module_verification>
<entry_point_verification>
 - For CLI/Server projects: The main entry point (cmd/main.go, index.js, main.rs) MUST be wired to call all implemented handlers/commands.
 - verification_command MUST include a check that the application RUNS, not just that individual components pass unit tests.
 - Example for Go: `go run ./cmd/server & sleep 2; curl -sf http://localhost:8080/health; kill $!`
 - If the entry point is a placeholder stub, the feature is NOT complete.
</entry_point_verification>
</testing_strategy>

<implementation_steps>
<!-- 8+ phases -->
<step number="1">
<title>Phase Title</title>
<estimated_effort>Rough time estimate</estimated_effort>
<tasks>
 - Specific task with detail
 - Specific task with detail
 - Specific task with detail
</tasks>
<deliverables>
 - What works after this phase
</deliverables>
<verification>
 - How to confirm this phase is complete
</verification>
</step>
</implementation_steps>

<success_criteria>
<functionality>
 - Specific, testable criterion
 - Specific, testable criterion
 - Specific, testable criterion
</functionality>
<performance>
 - Measurable metric with target
 - Measurable metric with target
</performance>
<quality>
 - Code coverage target
 - Linting/formatting requirements
</quality>
</success_criteria>

<future_enhancements>
 - Phase 2 features (post-launch)
 - Scalability improvements
 - Potential integrations
</future_enhancements>
</project_specification>
```

## User's Idea

{{IDEA}}

{{TESTING_PREFERENCE}}

---

Generate the complete specification now. Output ONLY the XML—no preamble, no commentary.
ULTRATHINK
