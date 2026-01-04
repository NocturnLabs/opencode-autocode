# Task: Repair Malformed Project Specification

You previously attempted to generate a project specification, but it contained XML errors or was invalid.

## Errors Detected

{{ERRORS}}

## Original Request

{{IDEA}}

Please regenerate the **complete** project specification in valid XML.

- **DENSITY REQUIREMENT**: You MUST provide at least **{{MIN_FEATURES}}** core_features and **{{MIN_API_ENDPOINTS}}** API endpoints. This is a strict requirement for a comprehensive specification.
- Ensure all tags are properly closed.

- Ensure all tags are properly closed.
- Ensure special characters like `&`, `<`, `>` are escaped (e.g., `&amp;`, `&lt;`, `&gt;`).
- Do not include markdown code blocks around the XML if possible, or ensure the code block is closed.

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
    <!-- At least {{MIN_FEATURES}} features -->
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
