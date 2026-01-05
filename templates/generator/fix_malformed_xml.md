# Task: Repair Malformed Project Specification

You previously attempted to generate a project specification, but it contained XML errors or was invalid.

## Errors Detected

{{ERRORS}}

## Original Request

{{IDEA}}

{{PARTIAL_OUTPUT}}

## Repair Requirements

1. **MAINTAIN DEPTH**: Do not simplify or truncate the content to fix the error. The goal is a **COMPREHENSIVE, PRODUCTION-READY** specification.
2. **RESTORE ALL SECTIONS**: Ensure Features (20+), Database (8+ tables), and Endpoints (20+) are all present and detailed.
3. **FIX SYNTAX**:
   - Ensure all tags are properly closed.
   - Ensure special characters like `&`, `<`, `>` are escaped (e.g., `&amp;`, `&lt;`, `&gt;`).
   - If the previous output was truncated, complete the missing sections logically based on the {{IDEA}}.
4. **FORMAT**: Output ONLY the repaired XML specification. Do not include markdown code blocks around the XML.

Output ONLY the repaired XML specification using this exact structure:

```xml
<project_specification>
  <project_name>Project Name</project_name>
  <overview>Detailed summary...</overview>

  <technology_stack>
    <!-- List specific technologies -->
    <tech>Language/Framework</tech>
  </technology_stack>

  <core_features>
    <feature>
      <name>Feature Name</name>
      <description>Description...</description>
    </feature>
  </core_features>

  <database_schema>
    <!-- Database tables/models -->
  </database_schema>

  <api_endpoints>
    <endpoint>
      <method>GET|POST|PUT|DELETE</method>
      <path>/api/v1/resource</path>
      <description>Description...</description>
    </endpoint>
  </api_endpoints>

  <success_criteria>
    <functionality>...</functionality>
    <quality>...</quality>
  </success_criteria>
</project_specification>
```
