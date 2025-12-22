#!/bin/bash

# ZenithKanban Development Environment Setup
# This script sets up and runs the development environment for ZenithKanban

set -e

echo "=== ZenithKanban Development Setup ==="

# Check Node.js version
if ! command -v node &> /dev/null; then
    echo "Error: Node.js is not installed. Please install Node.js 22.x or higher."
    exit 1
fi

NODE_VERSION=$(node --version | sed 's/v//' | cut -d. -f1)
if [ "$NODE_VERSION" -lt 22 ]; then
    echo "Error: Node.js 22.x or higher is required. Current version: $(node --version)"
    exit 1
fi

echo "✓ Node.js version: $(node --version)"

# Check TypeScript
if ! command -v tsc &> /dev/null; then
    echo "Error: TypeScript compiler is not installed. Please install TypeScript 5.x globally."
    exit 1
fi

echo "✓ TypeScript version: $(tsc --version)"

# Create necessary directories
echo "Creating directories..."
mkdir -p data
mkdir -p dist
mkdir -p public

# Compile TypeScript (assuming src/main.ts exists)
echo "Compiling TypeScript..."
tsc

# Set environment variables
export PORT=${PORT:-3000}
export DB_PATH=${DB_PATH:-./data/zenith.db}
export APP_ENV=${APP_ENV:-development}

echo "Environment variables:"
echo "  PORT: $PORT"
echo "  DB_PATH: $DB_PATH"
echo "  APP_ENV: $APP_ENV"

# Run database migrations (assuming they exist)
echo "Running database setup..."
node dist/migrate.js || echo "Migration script not found, skipping..."

# Start the server
echo "Starting ZenithKanban server..."
echo ""
echo "=== ZenithKanban is running ==="
echo "Access the application at: http://localhost:$PORT"
echo "API endpoints available at: http://localhost:$PORT/api/"
echo "Health check: http://localhost:$PORT/api/health"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

node dist/main.js