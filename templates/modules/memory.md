# Agent Memory System

You have access to a **Persistent Knowledge Base** to store facts across sessions. Use this to remember dynamic information like ports, paths, or decisions.

## When to Use

- **Ports**: "Started dev server on 3001" -> Save `dev_port=3001`
- **Paths**: "Config located at /etc/foo" -> Save `config_path=/etc/foo`
- **Decisions**: "Using PostgreSQL instead of MySQL" -> Save `db_type=postgres`

## Commands

### Save Knowledge

```bash
auto db knowledge set <KEY> <VALUE> --category <CATEGORY> --description "<DESC>"
```

Example:

```bash
auto db knowledge set dev_port 3001 --category network --description "Frontend server port"
```

### Recall Knowledge

```bash
auto db knowledge get <KEY>
```

Example:

```bash
# Get the port
auto db knowledge get dev_port
# Output: dev_port=3001
```

### List All Knowledge

```bash
auto db knowledge list
```

Use this at the start of a session to "remember" the project state.

---

## Server PID Tracking

When starting servers, track their PIDs for safe cleanup:

```bash
# Start server and save PID
bun run dev --port 8000 &
SERVER_PID=$!
auto db knowledge set server_port_8000_pid $SERVER_PID --category servers --description "Dev server"

# Later: safe cleanup (only kills YOUR server)
TRACKED_PID=$(auto db knowledge get server_port_8000_pid 2>/dev/null | grep -oP '(?<=value: )\d+')
[ -n "$TRACKED_PID" ] && kill "$TRACKED_PID" && auto db knowledge delete server_port_8000_pid
```

> [!CAUTION]
> Never use `pkill` or `killall` to kill servers. This can kill unrelated processes.
