#!/bin/bash
# OpenCode Autonomous Agent Runner
# Runs OpenCode in batch mode with automatic session continuation
#
# Uses 'opencode run --command' which executes a command and exits,
# rather than starting the interactive TUI.

set -e

# Config loading function - reads from autocode.toml with fallback to defaults
read_config() {
    local key="$1"
    local default="$2"
    local config_file="${CONFIG_FILE:-autocode.toml}"
    
    if [ -f "$config_file" ]; then
        # Simple TOML parsing - extract value for key
        local value=$(grep -E "^${key}\s*=" "$config_file" 2>/dev/null | head -1 | sed 's/.*=\s*//' | tr -d '"' | tr -d "'")
        if [ -n "$value" ]; then
            # Expand $HOME if present
            echo "${value/\$HOME/$HOME}"
            return
        fi
    fi
    echo "$default"
}

# Load configuration values
PROJECT_DIR="${1:-.}"
MAX_ITERATIONS="${2:-unlimited}"

# Determine platform-appropriate default log directory
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    DEFAULT_LOG_DIR="$APPDATA/opencode/log"
else
    DEFAULT_LOG_DIR="$HOME/.local/share/opencode/log"
fi

# Read config values (with defaults)
DELAY_BETWEEN_SESSIONS=$(read_config "delay_between_sessions" "5")
LOG_DIR=$(read_config "log_dir" "$DEFAULT_LOG_DIR")
AUTONOMOUS_MODEL=$(read_config "autonomous" "opencode/grok-code")
LOG_LEVEL=$(read_config "log_level" "DEBUG")

LOG_FILE="$LOG_DIR/session-$(date '+%Y%m%d-%H%M%S').log"

cd "$PROJECT_DIR"

echo "═══════════════════════════════════════════════════"
echo "  OpenCode Autonomous Agent Runner"
echo "═══════════════════════════════════════════════════"
echo ""
echo "Project directory: $(pwd)"
echo "Max iterations: $MAX_ITERATIONS"
echo "Model: $AUTONOMOUS_MODEL"
echo "Delay between sessions: ${DELAY_BETWEEN_SESSIONS}s"
echo ""
echo "Sessions will run in batch mode and continue automatically."
echo "Press Ctrl+C to stop."
echo ""

iteration=0
SESSION_ID=""

while true; do
    iteration=$((iteration + 1))
    
    # Check max iterations
    if [ "$MAX_ITERATIONS" != "unlimited" ] && [ $iteration -gt "$MAX_ITERATIONS" ]; then
        echo ""
        echo "Reached max iterations ($MAX_ITERATIONS)"
        break
    fi
    
    echo ""
    echo "═══════════════════════════════════════════════════"
    echo "  Session $iteration - $(date '+%H:%M:%S')"
    echo "═══════════════════════════════════════════════════"
    echo ""
    
    # Check if feature_list.json exists to determine command
    if [ ! -f "feature_list.json" ]; then
        echo "→ First run: auto-init"
        COMMAND="auto-init"
    else
        # Count remaining tests
        remaining=$(grep -c '"passes": false' feature_list.json 2>/dev/null || echo "0")
        passing=$(grep -c '"passes": true' feature_list.json 2>/dev/null || echo "0")
        
        echo "→ Progress: $passing passing, $remaining remaining"
        
        # Check if all tests pass
        if [ "$remaining" = "0" ] && [ "$passing" != "0" ]; then
            echo "" 
            echo "🎉 All tests passing! Project complete!"
            break
        fi
        
        COMMAND="auto-continue"
    fi
    
    echo "→ Running: opencode run --command /$COMMAND"
    echo ""
    
    # Build the opencode run command with config values
    # Model format: provider/model, command without leading slash
    OPENCODE_CMD="opencode run --command $COMMAND --model $AUTONOMOUS_MODEL --log-level $LOG_LEVEL"
    
    # Continue session if we have one
    if [ -n "$SESSION_ID" ]; then
        OPENCODE_CMD="$OPENCODE_CMD --session $SESSION_ID"
        echo "→ Continuing session: $SESSION_ID"
    fi
    
    # Run opencode in batch mode - capture exit code
    EXIT_CODE=0
    $OPENCODE_CMD || EXIT_CODE=$?
    
    echo ""
    echo "→ OpenCode exited with code: $EXIT_CODE"
    
    # If exit code is non-zero, stop
    if [ "$EXIT_CODE" != "0" ]; then
        echo ""
        echo "⚠ OpenCode exited with error."
        echo "Check logs and run manually: opencode run --command /$COMMAND"
        break
    fi
    
    # Check for explicit stop signal
    if [ -f ".opencode-stop" ]; then
        echo ""
        echo "Stop signal detected (.opencode-stop file exists)"
        rm -f .opencode-stop
        break
    fi
    
    # Success - continue to next session
    echo "→ Session complete, continuing..."
    echo "→ Next session in ${DELAY_BETWEEN_SESSIONS}s (Ctrl+C to stop)"
    sleep $DELAY_BETWEEN_SESSIONS
done

echo ""
echo "═══════════════════════════════════════════════════"
echo "  Runner stopped"
echo "═══════════════════════════════════════════════════"
echo ""

if [ -f "feature_list.json" ]; then
    passing=$(grep -c '"passes": true' feature_list.json 2>/dev/null || echo "0")
    total=$(grep -c '"passes"' feature_list.json 2>/dev/null || echo "?")
    echo "Status: $passing / $total tests passing"
fi

echo ""
echo "To resume: ./scripts/run-autonomous.sh"
echo "To stop:   touch .opencode-stop"
