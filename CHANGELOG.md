# Changelog

All notable changes to this project will be documented in this file.

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
