# OpenCode Forger

A Rust CLI that scaffolds autonomous coding projects for [OpenCode](https://github.com/sst/opencode) and runs them to completion.

## Quick Start

```bash
# Run the setup script to build and test
./init.sh

# Start the interactive project scaffolding TUI
./target/release/opencode-forger init --interactive

# Or scaffold with default template
./target/release/opencode-forger init --default --output /path/to/project

# Start autonomous development loop
./target/release/opencode-forger vibe --developer
```

## Project Structure

```
opencode-forger/
├── src/                # Rust source code
│   ├── autonomous/     # "Vibe Loop" engine and sub-modules
│   ├── cli/            # CLI command handlers
│   ├── common/         # Shared utilities (logging, errors)
│   ├── db/             # Database models and repositories
│   ├── services/       # Core services (generator, scaffold)
│   ├── tui/            # Interactive terminal UI
│   ├── config/         # Configuration loading
│   └── main.rs         # Entry point
├── templates/          # Project templates for progressive discovery
├── tests/              # Integration and regression tests
├── feature_list.json   # Comprehensive test cases for all features
├── init.sh             # Environment setup script
├── Makefile            # Build and test automation
└── README.md           # This file

```

## Features

- **Zero-Config Scaffolding**: Interactive TUI for building project specifications
- **SQLite Persistence**: Robust progress tracking with persistent state
- **Autonomous Vibe Loop**: Automated session management with intelligent continuation
- **Parallel Execution**: Concurrent feature implementation using git worktrees
- **Dual-Model Architecture**: Reasoning model plans while coding subagent implements
- **Progressive Discovery**: Modular template system reduces context window usage
- **Auto-Commit**: Automatically commits completed features to Git

## CLI Reference

- `init`: Initialize environment and project config.
- `vibe`: Start the autonomous development loop.
- `db`: Manage the SQLite database (list, show, reset features).
- `reset`: Reset project state (database, workspaces).
- `templates`: List and manage available templates.
- `example`: Generate example project specifications.


## Development

```bash
# Build the project
make build

# Run tests
make test

# Run linting
make lint

# Install to ~/.cargo/bin
make install
```

## Testing

All features are documented in `feature_list.json` with comprehensive test cases. Run the setup script to verify all tests:

```bash
./init.sh
```

## Documentation

- [Architecture Overview](ARCHITECTURE.md) - High-level module responsibilities
- [Development Guide](docs/DEVELOPMENT.md) - Developer onboarding
- [Contributing](CONTRIBUTING.md) - Code standards and PR process
