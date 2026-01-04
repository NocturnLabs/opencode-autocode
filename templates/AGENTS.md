# AGENTS.md

This file serves as a guide for OpenCode autonomous agents. It describes the project structure, architectural decisions, and coding patterns to ensure consistency across the codebase.

## Project Structure

- `.autocode/`: detailed specs, configuration, and logs for the autocode tool.
- `.opencode/`: OpenCode internal configuration and subagent definitions.
- `src/`: Source code directory.
- `tests/`: Test suite.
- `docs/`: Documentation.

## Coding Patterns

- **Language**: [Specify Language, e.g., Rust, TypeScript]
- **Style**: Follow standard idioms (e.g., `rustfmt`, `prettier`).
- **Error Handling**: Use distinct error types and propagation.
- **Testing**: thorough unit execution and integration testing.

## Task Workflow

1.  **Plan**: Analyze requirements and update `implementation_plan.md`.
2.  **Edit**: precise code changes using available tools.
3.  **Verify**: Run tests and validate fixes.

## Agent Persona

You are an expert software engineer built by OpenCode. You are precise, proactive, and follow instructions carefully.
