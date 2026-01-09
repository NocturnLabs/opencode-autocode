#!/bin/bash
set -e

# OpenCode Forger Environment Setup Script
# This script sets up and verifies the development environment

echo "=========================================="
echo "OpenCode Forger - Environment Setup"
echo "=========================================="

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $2 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $1"
    else
        echo -e "${RED}✗${NC} $1"
        exit 1
    fi
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

echo ""
echo "Step 1: Checking Prerequisites"
echo "-------------------------------"

# Check Rust toolchain
if command_exists rustc; then
    RUST_VERSION=$(rustc --version)
    print_status "Rust installed: $RUST_VERSION" 0
else
    print_status "Rust not found - installing via rustup" 1
    echo "Please run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "Then source ~/.cargo/env"
    exit 1
fi

# Check cargo
if command_exists cargo; then
    CARGO_VERSION=$(cargo --version)
    print_status "Cargo installed: $CARGO_VERSION" 0
else
    print_status "Cargo not found" 1
    exit 1
fi

# Check Git
if command_exists git; then
    GIT_VERSION=$(git --version)
    print_status "Git installed: $GIT_VERSION" 0
else
    echo -e "${YELLOW}!${NC} Git not found - some features may be limited"
fi

# Check OpenCode CLI (optional)
if command_exists opencode; then
    OC_VERSION=$(opencode --version 2>/dev/null || echo "unknown")
    print_status "OpenCode CLI installed: $OC_VERSION" 0
else
    echo -e "${YELLOW}!${NC} OpenCode CLI not found - install from https://github.com/sst/opencode"
fi

echo ""
echo "Step 2: Building Project"
echo "-------------------------"

# Build the project
if cargo build --release 2>&1; then
    print_status "Build successful" 0
else
    print_status "Build failed" 1
fi

# Check binary exists
if [ -f "./target/release/opencode-forger" ]; then
    print_status "Binary created: ./target/release/opencode-forger" 0
else
    print_status "Binary not found at expected location" 1
fi

echo ""
echo "Step 3: Running Tests"
echo "----------------------"

# Run cargo tests
if cargo test --quiet 2>&1; then
    print_status "All tests passed" 0
else
    echo -e "${YELLOW}!${NC} Some tests failed - review output above"
fi

echo ""
echo "Step 4: Verifying Installation"
echo "--------------------------------"

# Verify the binary works
if ./target/release/opencode-forger --help >/dev/null 2>&1; then
    print_status "CLI is functional" 0
else
    print_status "CLI failed to run" 1
fi

# Check templates directory
if [ -d "./templates" ]; then
    TEMPLATE_COUNT=$(ls -1 ./templates/*.json 2>/dev/null | wc -l)
    print_status "Templates found: $TEMPLATE_COUNT template files" 0
else
    echo -e "${YELLOW}!${NC} Templates directory not found"
fi

echo ""
echo "=========================================="
echo "Setup Complete!"
echo "=========================================="
echo ""
echo "Quick Start:"
echo "  # Scaffold a new project (Interactive TUI)"
echo "  ./target/release/opencode-forger --interactive"
echo ""
echo "  # Scaffold with default template"
echo "  ./target/release/opencode-forger --default --output /path/to/project"
echo ""
echo "  # Start autonomous development loop"
echo "  ./target/release/opencode-forger vibe --developer"
echo ""
echo "  # View available commands"
echo "  ./target/release/opencode-forger --help"
echo ""
echo "Useful Commands:"
echo "  make build      - Build release binary"
echo "  make test       - Run all tests"
echo "  make lint       - Run clippy linting"
echo "  make install    - Install to ~/.cargo/bin"
echo ""
echo "For more information, see README.md"
echo ""
