# Task: Repair Malformed Project Specification

You previously attempted to generate a project specification, but it contained XML errors or was invalid.

## Errors Detected

{{ERRORS}}

## Original Request

{{IDEA}}

## Requirement

Please regenerate the **complete** project specification in valid XML.

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
    <!-- At least 3 features -->
  </core_features>

  <database_schema>
    <!-- Database tables/models -->
  </database_schema>

  <api_endpoints>
    <!-- API definitions if applicable -->
  </api_endpoints>

  <success_criteria>
    <!-- Measurable goals -->
  </success_criteria>
</project_specification>
```
