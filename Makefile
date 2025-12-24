# OpenCode Autocode Makefile
# Replaces shell scripts with make targets

.PHONY: all build test regression clean autonomous install help

# Default target
all: test build

# Build the release binary
build:
	cargo build --release

# Run all cargo tests
test:
	cargo test

# Run clippy and format check
lint:
	cargo fmt --check
	cargo clippy -- -D warnings

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
	./target/release/opencode-autocode autonomous

# Install the binary to ~/.cargo/bin
install: build
	cargo install --path .

# Clean build artifacts
clean:
	cargo clean

# Show available targets
help:
	@echo "Available targets:"
	@echo "  make build      - Build release binary"
	@echo "  make test       - Run cargo tests"
	@echo "  make lint       - Run clippy linting"
	@echo "  make regression - Run full regression suite"
	@echo "  make autonomous - Run autonomous agent loop"
	@echo "  make install    - Install to ~/.cargo/bin"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make help       - Show this help"
