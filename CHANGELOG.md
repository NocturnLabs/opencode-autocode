# Changelog

All notable changes to this project will be documented in this file.

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
