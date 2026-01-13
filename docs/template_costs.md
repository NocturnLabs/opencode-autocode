# Template Token Costs

This document tracks the approximate token usage for the core templates used by `opencode-forger`.
These estimates uses a heuristic of ~4 characters per token. Actual usage varies by tokenizer.

> **Note:** You can run `cargo test token_count -- --nocapture` to generate fresh estimates.

## Workflow Costs (Context Window Impact)

These are the "taxes" incurred on your context window when running these specific commands.

| Command / Context      | Template              | ~Tokens    | Impact                                       |
| ---------------------- | --------------------- | ---------- | -------------------------------------------- |
| **Project Generation** | `generator_prompt.xml` | **~2,200** | One-time cost at start of project            |
| **Session Loop**       | `auto-continue.xml`    | **~1,700** | **Per-turn cost** during autonomous sessions |
| **Initialization**     | `auto-init.xml`        | **~1,800** | One-time cost at session start               |
| **Enhancement**        | `auto-enhance.xml`     | **~1,100** | One-time cost during refinement phase        |

## Component Breakdown

### Core Modules

These modules are included into the command templates above via `{{INCLUDE}}`.

| Module                  | ~Tokens | Description                     |
| ----------------------- | ------- | ------------------------------- |
| `core/database.xml`      | ~475    | KB operations, feature tracking |
| `core/mcp_guide.xml`     | ~290    | Tool usage instructions         |
| `core/identity.xml`      | ~165    | System role and constraints     |
| `core/security.xml`      | ~160    | File access rules               |
| `core/signaling.xml`     | ~85     | Session control signals         |

### Project Templates

Base templates used when scaffolding new projects.

| Template            | ~Tokens | Description                    |
| ------------------- | ------- | ------------------------------ |
| `api-rest`          | ~800    | Python/FastAPI with PostgreSQL |
| `web-app-fullstack` | ~800    | React + Node/Express           |
| `cli-tool`          | ~600    | Rust CLI                       |

## Optimization Targets

The `auto-continue.xml` template is the most critical for optimization as it is re-sent to the LLM on every turn of an autonomous loop.
Current breakdown of `auto-continue.xml` (~1,675 tokens):

- **base content**: ~600 tokens
- **core/database**: ~475 tokens
- **core/mcp_guide**: ~290 tokens
- **other modules**: ~310 tokens
