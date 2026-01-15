# OpenCode Forger Makefile
# Replaces shell scripts with make targets

.PHONY: all build build-rust build-go test regression clean autonomous install help

# Default target
all: test build

# Build all binaries (Rust + Go TUI)
build: build-rust build-go

# Build the Rust release binary
build-rust:
	cargo build --release

# Build the Go TUI binary
build-go:
	@echo "Building Go TUI..."
	cd tui-go && go build -o ../target/release/opencode-forger-tui ./cmd/opencode-forger-tui

# Run all cargo tests
test:
	cargo test

# Run clippy and format check
lint: lint-rust lint-go

# Lint Rust code
lint-rust:
	cargo fmt --check
	cargo clippy -- -D warnings

# Lint Go code
lint-go:
	cd tui-go && go fmt ./...
	cd tui-go && go vet ./...

# Create a GitHub release (v0.1.0)
release:
	gh release create v0.9.0 --title "v0.9.0 - Autonomous Refactor & Web UI" --generate-notes

# Build and run regression tests (replaces tests/regression/run_regression_tests.sh)
regression: build
	@echo "═══════════════════════════════════════════════════"
	@echo "  Running Regression Tests"
	@echo "═══════════════════════════════════════════════════"
	cargo test
	cd tests/regression && cargo build --release
	cd tests/regression && ../../target/release/opencode-regression-tests

# Run the autonomous agent (convenience target)
autonomous: build
	./target/release/opencode-forger autonomous

# Install both binaries
install: build
	cargo install --path .
	cp target/release/opencode-forger-tui ~/.cargo/bin/ 2>/dev/null || true

# Clean build artifacts
clean:
	cargo clean
	cd tui-go && go clean

# Show available targets
help:
	@echo "Available targets:"
	@echo "  make build      - Build all binaries (Rust + Go)"
	@echo "  make build-rust - Build Rust binary only"
	@echo "  make build-go   - Build Go TUI binary only"
	@echo "  make test       - Run cargo tests"
	@echo "  make lint       - Run linting for Rust and Go"
	@echo "  make lint-rust  - Run Rust linting only"
	@echo "  make lint-go    - Run Go linting only"
	@echo "  make regression - Run full regression suite"
	@echo "  make autonomous - Run autonomous agent loop"
	@echo "  make install    - Install to ~/.cargo/bin"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make help       - Show this help"
