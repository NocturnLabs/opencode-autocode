#!/bin/bash
# Regression Test Runner Script
# This script runs the regression test suite for opencode-autocode

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
REGRESSION_DIR="$PROJECT_ROOT/tests/regression"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to print usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Run regression tests for opencode-autocode"
    echo ""
    echo "Options:"
    echo "  -c, --config FILE    Use custom config file (default: config/default.toml)"
    echo "  -v, --verbose        Enable verbose output"
    echo "  -h, --help          Show this help message"
    echo "  --no-baseline       Skip baseline comparison"
    echo "  --fail-fast         Stop on first failure"
    echo ""
}

# Default values
CONFIG_FILE="$REGRESSION_DIR/config/default.toml"
VERBOSE=false
BASELINE_COMPARISON=true
FAIL_FAST=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--config)
            CONFIG_FILE="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --no-baseline)
            BASELINE_COMPARISON=false
            shift
            ;;
        --fail-fast)
            FAIL_FAST=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            print_status $RED "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Check if config file exists
if [[ ! -f "$CONFIG_FILE" ]]; then
    print_status $RED "Config file not found: $CONFIG_FILE"
    exit 1
fi

print_status $BLUE "Starting Regression Tests"
print_status $BLUE "========================"

# Change to project root
cd "$PROJECT_ROOT"

# Build the project first
print_status $YELLOW "Building project..."
if ! cargo build --release; then
    print_status $RED "Failed to build project"
    exit 1
fi

# Run unit tests first
print_status $YELLOW "Running unit tests..."
if ! cargo test; then
    print_status $RED "Unit tests failed"
    exit 1
fi

# Build the regression test runner
print_status $YELLOW "Building regression test runner..."
cd "$REGRESSION_DIR"
if ! cargo build --release; then
    print_status $RED "Failed to build regression test runner"
    exit 1
fi

# Run regression tests
print_status $YELLOW "Running regression tests..."
export REGRESSION_CONFIG_FILE="$CONFIG_FILE"
export REGRESSION_BASELINE_COMPARISON="$BASELINE_COMPARISON"
export REGRESSION_FAIL_FAST="$FAIL_FAST"
export REGRESSION_VERBOSE="$VERBOSE"

if "../../target/release/opencode-regression-tests"; then
    print_status $GREEN "Regression tests completed successfully"
else
    print_status $RED "Regression tests failed"
    exit 1
fi

# Display results
RESULTS_DIR="$REGRESSION_DIR/results"
if [[ -f "$RESULTS_DIR/summary.txt" ]]; then
    echo ""
    print_status $BLUE "Test Summary:"
    cat "$RESULTS_DIR/summary.txt"
fi

print_status $GREEN "All tests completed successfully!"