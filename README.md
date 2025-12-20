# OpenCode Autocode 🚀

**OpenCode Autocode** is a powerful Rust-based CLI tool designed to scaffold and manage autonomous coding plugins for [OpenCode](https://github.com/nocturnlabs/opencode). It mimics the functionality of the Claude Autonomous Repo/MCP, providing a robust cognitive scaffolding for AI agents to work within your local projects.

## Key Features

- **Autonomous Coding Loop**: Automated feature implementation with built-in retry and research protocols.
- **TUI Spec Editor**: Interactive terminal interface to build and refine project specifications (`app_spec.md`).
- **Regression Testing**: First-class support for verifying agent performance against baselines.
- **MCP Integration**: Enhanced reasoning using Model Context Protocol tools (`osgrep`, `chat-history`, `perplexica`, `sequential-thinking`).
- **Verbalized Sampling**: Creative approach exploration using probability-based sampling for unconventional solutions.
- **Security First**: Strict command allowlisting to keep autonomous agents safe.

## Installation

### From Source

```bash
git clone https://github.com/nocturnlabs/opencode-autocode.git
cd opencode-autocode
make install
```

This will install the `opencode-autocode` binary to your `~/.cargo/bin`.

## Quick Start

1.  **Initialize a Project**:

    ```bash
    opencode-autocode --interactive
    ```

    This launches the TUI to help you build your `app_spec.md`.

2.  **Scaffold with Defaults**:

    ```bash
    opencode-autocode --default --output ./my-project
    ```

3.  **Run Autonomous Loop**:
    ```bash
    opencode-autocode autonomous
    ```

## Usage

### Commands

| Command            | Description                                  |
| ------------------ | -------------------------------------------- |
| `edit`             | Launch the interactive TUI spec editor       |
| `autonomous`       | Start the autonomous coding loop             |
| `regression-check` | Verify features against the regression suite |
| `templates`        | Manage project scaffolding templates         |

### Options

- `-i, --interactive`: Use the TUI to generate project specs.
- `-d, --default`: Use the default template for instant scaffolding.
- `-s, --spec <FILE>`: Provide a custom markdown spec for scaffolding.
- `--dry-run`: Preview file creation without writing to disk.

## Configuration

The project is governed by `autocode.toml`. You can customize models, paths, and agent behavior:

```toml
[models]
autonomous = "opencode/grok-code"
reasoning = "opencode/grok-code"

[mcp]
priority_order = ["osgrep", "chat-history", "deepwiki", "perplexica", "sequential-thinking"]
```

## Security

`opencode-autocode` uses a security allowlist located in `scripts/security-allowlist.json`. Commands not in the allowlist or matching blocked patterns (like `rm -rf /`) will be rejected by the agent.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on our development workflow.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

_Developed by NocturnLabs_
