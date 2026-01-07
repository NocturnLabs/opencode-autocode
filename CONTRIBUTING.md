# Contributing to OpenCode Forger

Thank you for your interest in contributing to `opencode-forger`! This document provides guidelines for contributing to this project.

## Development Setup

1.  **Install Rust:** Ensure you have the latest stable version of Rust installed (via [rustup](https://rustup.rs/)).
2.  **Clone the Repository:**
    ```bash
    git clone https://github.com/nocturnlabs/opencode-forger.git
    cd opencode-forger
    ```
3.  **Build the Project:**
    ```bash
    cargo build
    ```

## Coding Standards

- **Formatting:** We use `rustfmt` to maintain consistent code style. Run `cargo fmt` before committing.
- **Linting:** We use `clippy` for static analysis. Ensure your code is clean by running `make lint`.
- **Naming:** Follow standard Rust naming conventions (CamelCase for types, snake_case for functions/variables).

## Testing Procedures

We take testing seriously to ensure the reliability of autonomous coding tasks.

- **Unit Tests:** Run `cargo test` to execute the internal test suite.
- **Regression Tests:** Use the full regression suite to verify that your changes don't break existing autonomous logic:
  ```bash
  make regression
  ```
- **Manual Verification:** Test your changes against a real OpenCode environment if possible.

## Pull Request Process

1.  **Branching:** Create a new branch for your feature or bugfix: `git checkout -b feature/my-feature`.
2.  **Commit Messages:** Use descriptive commit messages. We follow [Conventional Commits](https://www.conventionalcommits.org/).
3.  **Verification:** Ensure all tests pass (`make test lint regression`).
4.  **Submit PR:** Open a Pull Request against the `main` branch. Provide a clear description of your changes and why they are necessary.

## Security

If you discover a security vulnerability, please do NOT open a public issue. Instead, contact the maintainers directly.

---

_Maintained by NocturnLabs_
