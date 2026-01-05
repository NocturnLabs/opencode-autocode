# Go Module

Read this module when working on Go projects (HTTP services, CLI tools).

---

## Common Commands

```bash
# Initialize
go mod init <module-name>
go mod tidy

# Build
go build ./...
go build -o ./bin/app ./cmd/server

# Run
go run ./cmd/server

# Test
go test ./... -v
go test -race ./...
go test -cover ./...

# Lint
go vet ./...
golangci-lint run
```

---

## Project Structure

A standard Go project layout:

```
project-root/
├── cmd/
│   └── server/       # Entry point: main.go HERE
│       └── main.go
├── internal/         # Core logic packages
│   ├── handler/
│   ├── service/
│   └── config/
├── pkg/              # Public, reusable packages
├── go.mod
└── init.sh
```

> [!IMPORTANT]
> The `cmd/<app>/main.go` is the **entry point**. All handlers, services, and routes MUST be wired here. If `main.go` is a placeholder, the application is NOT functional.

---

## Entry Point Wiring

A functional `main.go` MUST:

1.  Load configuration.
2.  Initialize dependencies (database, services, handlers).
3.  Register HTTP routes/CLI commands.
4.  Start the server or execute the command.

**Example (`cmd/server/main.go`):**

```go
package main

import (
	"log"
	"net/http"

	"project/internal/config"
	"project/internal/handler"
)

func main() {
	cfg := config.Load()
	h := handler.New(cfg)

	http.HandleFunc("/health", h.Health)
	http.HandleFunc("/api/v1/data", h.HandleData)

	log.Printf("Starting server on :%s", cfg.Port)
	log.Fatal(http.ListenAndServe(":"+cfg.Port, nil))
}
```

---

## Verification Commands

For Go HTTP services, `verification_command` MUST include an integration check, not just unit tests.

**Bad (unit test only):**

```bash
go test -v ./internal/handler
```

**Good (integration check):**

```bash
# Start server in background, test endpoints, then shut down
go run ./cmd/server & PID=$!; sleep 2; \
curl -sf http://localhost:8080/health && echo "Health OK"; \
kill $PID
```

Or, if using a test harness:

```bash
go test -v ./... -tags=integration
```

---

## init.sh Template for Go

```bash
#!/bin/bash
set -e

echo "--- Installing dependencies ---"
go mod tidy

echo "--- Building ---"
go build -o ./bin/server ./cmd/server

echo "--- Running tests ---"
go test ./... -v

echo "--- Starting server ---"
DEFAULT_PORT=8080
PORT=$DEFAULT_PORT

# Find an available port
while ! python3 -c "import socket; s = socket.socket(socket.AF_INET, socket.SOCK_STREAM); s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1); s.bind(('0.0.0.0', $PORT))" &>/dev/null; do
  PORT=$((PORT + 1))
done

./bin/server --port $PORT &
PID=$!
sleep 2

# Basic smoke test
if curl -sf http://localhost:$PORT/health > /dev/null; then
  echo "✓ Server health check passed on port $PORT"
else
  echo "✗ Server health check failed on port $PORT"
  kill $PID 2>/dev/null
  exit 1
fi

kill $PID
echo "--- Init complete ---"
```
