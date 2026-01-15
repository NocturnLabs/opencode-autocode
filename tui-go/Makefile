# Makefile for the Go TUI client

# Variables
VERSION ?= $(shell git describe --tags --always --dirty 2>/dev/null || echo "dev")
BINARY_NAME := opencode-forger-tui
BUILD_DIR := ../target
GO_LDFLAGS := -ldflags "-X main.Version=$(VERSION)"

.PHONY: all build clean test fmt lint install

all: build

# Build the TUI binary
build:
	@echo "Building $(BINARY_NAME)..."
	go build $(GO_LDFLAGS) -o $(BUILD_DIR)/$(BINARY_NAME) ./cmd/opencode-forger-tui

# Build for release (with optimizations)
release:
	@echo "Building $(BINARY_NAME) for release..."
	CGO_ENABLED=0 go build $(GO_LDFLAGS) -trimpath -o $(BUILD_DIR)/$(BINARY_NAME) ./cmd/opencode-forger-tui

# Clean build artifacts
clean:
	rm -f $(BUILD_DIR)/$(BINARY_NAME)
	go clean

# Run tests
test:
	go test -v ./...

# Format code
fmt:
	go fmt ./...
	gofumpt -w . 2>/dev/null || true

# Lint code
lint:
	go vet ./...
	golangci-lint run 2>/dev/null || go vet ./...

# Install to GOPATH/bin
install:
	go install $(GO_LDFLAGS) ./cmd/opencode-forger-tui

# Cross-compile for multiple platforms
cross-compile:
	@echo "Building for Linux amd64..."
	GOOS=linux GOARCH=amd64 go build $(GO_LDFLAGS) -o $(BUILD_DIR)/$(BINARY_NAME)-linux-amd64 ./cmd/opencode-forger-tui
	@echo "Building for Linux arm64..."
	GOOS=linux GOARCH=arm64 go build $(GO_LDFLAGS) -o $(BUILD_DIR)/$(BINARY_NAME)-linux-arm64 ./cmd/opencode-forger-tui
	@echo "Building for macOS amd64..."
	GOOS=darwin GOARCH=amd64 go build $(GO_LDFLAGS) -o $(BUILD_DIR)/$(BINARY_NAME)-darwin-amd64 ./cmd/opencode-forger-tui
	@echo "Building for macOS arm64..."
	GOOS=darwin GOARCH=arm64 go build $(GO_LDFLAGS) -o $(BUILD_DIR)/$(BINARY_NAME)-darwin-arm64 ./cmd/opencode-forger-tui
	@echo "Building for Windows amd64..."
	GOOS=windows GOARCH=amd64 go build $(GO_LDFLAGS) -o $(BUILD_DIR)/$(BINARY_NAME)-windows-amd64.exe ./cmd/opencode-forger-tui

# Update dependencies
deps:
	go mod tidy
	go mod download
