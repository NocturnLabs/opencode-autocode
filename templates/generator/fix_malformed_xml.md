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

Output ONLY the repaired XML specification:

```xml
<project_specification>
  ...
</project_specification>
```
