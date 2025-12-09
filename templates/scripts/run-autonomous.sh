#!/bin/bash
# OpenCode Autonomous Agent Runner
# Runs OpenCode in a loop with automatic session continuation
#
# This script assumes that if OpenCode exits successfully (exit code 0),
# the session completed and should continue to the next iteration.

set -e

PROJECT_DIR="${1:-.}"
MAX_ITERATIONS="${2:-unlimited}"
DELAY_BETWEEN_SESSIONS=5

cd "$PROJECT_DIR"

echo "═══════════════════════════════════════════════════"
echo "  OpenCode Autonomous Agent Runner"
echo "═══════════════════════════════════════════════════"
echo ""
echo "Project directory: $(pwd)"
echo "Max iterations: $MAX_ITERATIONS"
echo ""
echo "Sessions will continue automatically."
echo "Press Ctrl+C to stop."
echo ""

iteration=0

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
        echo "→ First run: /auto-init"
        COMMAND="/auto-init"
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
        
        COMMAND="/auto-continue"
    fi
    
    echo "→ Running: opencode $COMMAND"
    echo ""
    
    # Run opencode - capture exit code
    EXIT_CODE=0
    opencode "$COMMAND" || EXIT_CODE=$?
    
    echo ""
    echo "→ OpenCode exited with code: $EXIT_CODE"
    
    # If exit code is non-zero, stop
    if [ "$EXIT_CODE" != "0" ]; then
        echo ""
        echo "⚠ OpenCode exited with error."
        echo "Check logs and run manually: opencode $COMMAND"
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
