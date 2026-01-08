# OpenCode Forger

A Rust CLI that scaffolds autonomous coding projects for [OpenCode](https://github.com/sst/opencode) and runs them to completion.

## Quick Start

```bash
# Run the setup script to build and test
./init.sh

# Start the interactive project scaffolding TUI
./target/release/opencode-forger --interactive

# Or scaffold with default template
./target/release/opencode-forger --default --output /path/to/project

# Start autonomous development loop
./target/release/opencode-forger vibe --developer
```

## Project Structure

```
opencode-forger/
├── src/              # Rust source code
├── templates/        # Project templates for progressive discovery
├── tests/            # Integration and regression tests
├── feature_list.json # Comprehensive test cases for all features
├── init.sh           # Environment setup script
├── Makefile          # Build and test automation
└── README.md         # This file
```

## Features

- **Zero-Config Scaffolding**: Interactive TUI for building project specifications
- **SQLite Persistence**: Robust progress tracking with persistent state
- **Autonomous Vibe Loop**: Automated session management with intelligent continuation
- **Dual-Model Architecture**: Reasoning model plans while coding subagent implements
- **Progressive Discovery**: Modular template system reduces context window usage
- **Auto-Commit**: Automatically commits completed features to Git

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
