# Changelog

All notable changes to this project will be documented in this file.

## [0.7.0] - 2026-01-05

### Added

- **Port Management**: Implemented dynamic port discovery for Go servers and Python-based port detection for JavaScript environments.
- **Security & Validation**: Introduced a new security module for command validation and enforced read-only database queries for safety.
- **Process Tracking**: Added CLI commands for tracking and managing background server processes.
- **Scaffolding**: Added `AGENTS.md` template to project scaffolding and updated `opencode.json` with provider configurations.
- **Database Concurrency**: Improved database reliability with enhanced concurrency handling.

### Changed

- **Core Architecture**: Centralized session initialization and extracted shared utilities to improve codebase maintainability.
- **Autonomous Runner**: Enhanced rebase safety with automated stashing and enabled configurable worktree database paths.
- **Generation Prompts**: Refined agent architecture generation prompts to improve the quality and relevance of generated specifications.

### Fixed

- **Parallel Execution**: Resolved issues with database test setup and streamlined parallel workspace initialization.

## [0.6.0] - 2026-01-04

### Added

- **Autonomous Runner & Supervisor**:
  - **Parallel Execution**: Introduced `--parallel N` mode using isolated Git worktrees for concurrent feature development.
  - **Supervisor State Machine**: Reinforced "one feature per session" isolation and moved feature marking responsibilities from the agent to the supervisor after verification.
  - **Enhanced Context**: Supervisor now injects mandatory Knowledge Base recall and feature-specific context into OpenCode sessions.
  - **Robustness**: Added smart "stuck detection" for broken verification commands and real-time session termination on completion signals.
  - **Graceful Shutdown**: Implemented Ctrl+C signal handling for clean exit.
- **Configuration & TUI**:
  - **Exhaustive Config Wiring**: Fully integrated `ui.spec_preview_lines`, `alternative_approaches`, `communication`, `ui.colored_output`, `ui.verbose`, and `ui.show_progress` fields into the application logic.
  - **TUI Expansion**: Added configuration sections for Communication, Features, Scaffolding, and MCP to the interactive TUI.
  - **Refined Selection**: Users can now specify testing framework preferences during interactive setup.
- **Scaffolding & AI Generation**:
  - **Dynamic Guidance**: Refactored generator prompts to use open-ended, complexity-based constraints instead of hardcoded ranges.
  - **Spec Generation Retry**: Implemented a 5-attempt retry mechanism for fixing malformed XML using `opencode/grok-code`.
  - **Template Centralization**: Relocated `{{INCLUDE}}` resolution to the `scaffold` module for better maintainability.
  - **Project Schema**: Added `<entry_point_verification>` tag and enhanced Go module scaffolding guidelines.

### Changed

- **Documentation**:
  - **Architecture Reference**: Integrated `ARCHITECTURE_DETAILED.md` with complete module mappings and call graphs.
  - **Intent-First Comments**: Reflowed internal code documentation to emphasize design decisions ("Why") over implementation details ("What").

### Fixed

- **Fixes & CI/CD**:
  - **Merge Management**: Resolved Git index lock issues and worktree cleanup failures in parallel mode.
  - **Webhook Optimization**: Fixed Discord notifications to update a single message ID, preventing notification spam.
  - **Release Workflow**: Fixed GITHUB_TOKEN permissions for automated binary releases.

## [0.5.0] - 2026-01-02

### Added

- **Configurable Fixer Model**: A dedicated model for fixing malformed XML and spec errors, configurable via `autocode.toml` and TUI (default: `opencode/grok-code`).
- **Doc: Template Token Costs**: New documentation (`docs/template_costs.md`) tracking token usage for all system prompts (init session: ~1,800 tokens).
- **Test: Token Counting**: Automated tests to monitor prompt size and prevent context bloat.

### Changed

- **Defaults**: `prefer_osgrep` now defaults to `true` for 100x faster searches.
- **Spec Generation**:
  - Added "Local Software" to Role Definition.
  - Replaced hardcoded feature minimums (e.g., "15+") with open-ended guidance scaling with project complexity.
- **Session Discipline**:
  - Renamed `auto-continue-v2` -> `auto-continue`.
  - Added strict "Stop after ONE feature" enforcement to prevent context drift.
  - **Mandatory** Knowledge Base recall at session start.
- **Scaffolding**: `init.sh` guidelines updated to strictly require "Install + Launch" in a single command.

## [0.4.5] - 2026-01-02

### Added

- **TUI Visual Improvements**: New theming module with 256-color palette, Unicode box drawing (╭╮╰╯), and styled symbols (✔✗→●).
- Consistent styling across autonomous runner, config TUI, and interactive setup.

## [0.4.4] - 2026-01-02

### Fixed

- **Self-Update**: Added explicit `bin_path_in_archive` configuration to correctly locate the binary inside the tar.gz archive.

## [0.4.3] - 2026-01-02

### Fixed

- **Self-Update**: Fixed archive extraction failure by enabling correct gzip decompression (`compression-flate2` instead of `compression-zip-deflate`).

## [0.4.2] - 2026-01-01

### Fixed

- **Spec Generation Limit**: Removed hardcoded ranges (e.g., "15-25") from prompt templates, allowing the AI to generate comprehensive specifications for complex projects without artificial caps.
- **Bun Enforcement**: Replaced all hardcoded `npm`/`npx` examples with `bun`/`bun x` across all command and project templates to ensure consistent toolchain usage.
- **Port Management**: Refined port-killing logic to strictly forbid killing processes not started by the agent. Added mandatory warnings and reinforced the "find free port" pattern for local development safety.
- **Security Allowlist**: Updated `.autocode/security-allowlist.json` to include common `bun` subcommands (`bun run`, `bun test`, etc.).

## [0.4.1] - 2026-01-01

### Fixed

- Minor stability improvements and test fixes.

## [0.4.0] - 2025-12-31

### Added

- **Persistent Agent Knowledge**: Agents can now store and recall facts (ports, keys, decisions) across sessions using `db knowledge` commands.
- **Supervisor Architecture**: New state machine for the autonomous loop that provides stricter control over agent lifecycle and feature isolation.
- **Discord Dashboard**: Webhooks now output a single, updating dashboard message with progress bars instead of a spammy log stream.
- **Interactive Scaffolding Options**: Users can now specify a "Testing Framework Preference" during the `auto interactive` flow.
- **Symlink Optimization**: Binary installation now uses a symlink for the `auto` alias, saving space.

### Changed

- **Default Models**:
  - Spec Generation: `opencode/glm-4.7-free`
  - Autonomous Coding: `opencode/minimax-m2.1-free`
- **Configuration**: `ui.verbose` now defaults to `true` for better visibility.
- **JS/TS Toolchain**: Enforced `bun` as the primary runtime and package manager for JavaScript projects.
- **Agent Guidance**: Updated templates (`auto-input-v2`, `javascript.md`) to reflect new best practices (single-feature focus, bun usage).

### Fixed

- **Version Compatibility**: Checkpointing system now correctly handles version mismatches.
- **Spec Generation**: Fixed argument handling in the generator module.

## [0.3.2] - 2025-11-15

### Added

- Initial release of the autonomous agent scaffolding tool.
