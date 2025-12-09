#!/bin/bash
# OpenCode Autonomous Agent Runner
# Runs OpenCode in a loop with session management
#
# The loop continues automatically as long as the agent writes "CONTINUE"
# to the .opencode-signal file at the end of each session.

set -e

PROJECT_DIR="${1:-.}"
MAX_ITERATIONS="${2:-unlimited}"
DELAY_BETWEEN_SESSIONS=3

cd "$PROJECT_DIR"

echo "═══════════════════════════════════════════════════"
echo "  OpenCode Autonomous Agent Runner"
echo "═══════════════════════════════════════════════════"
echo ""
echo "Project directory: $(pwd)"
echo "Max iterations: $MAX_ITERATIONS"
echo ""
echo "The agent will continue automatically between sessions."
echo "Press Ctrl+C to stop the loop."
echo ""

# Clean up any existing signal file
rm -f .opencode-signal

iteration=0

while true; do
    iteration=$((iteration + 1))
    
    # Check max iterations
    if [ "$MAX_ITERATIONS" != "unlimited" ] && [ $iteration -gt $MAX_ITERATIONS ]; then
        echo ""
        echo "Reached max iterations ($MAX_ITERATIONS)"
        break
    fi
    
    echo "═══════════════════════════════════════════════════"
    echo "  Session $iteration"
    echo "═══════════════════════════════════════════════════"
    echo ""
    
    # Remove signal file before session
    rm -f .opencode-signal
    
    # Check if this is first run
    if [ ! -f "feature_list.json" ]; then
        echo "First run - using /auto-init command"
        echo ""
        opencode /auto-init || true
    else
        # Count remaining tests
        remaining=$(grep -c '"passes": false' feature_list.json 2>/dev/null || echo "0")
        passing=$(grep -c '"passes": true' feature_list.json 2>/dev/null || echo "0")
        echo "Progress: $passing passing, $remaining remaining"
        echo ""
        
        # Check if all tests pass
        if [ "$remaining" = "0" ]; then
            echo ""
            echo "🎉 All tests passing! Project complete!"
            echo ""
            break
        fi
        
        echo "Continuing - using /auto-continue command"
        echo ""
        opencode /auto-continue || true
    fi
    
    echo ""
    
    # Check for continuation signal
    if [ -f ".opencode-signal" ]; then
        signal=$(cat .opencode-signal 2>/dev/null || echo "")
        if [ "$signal" = "CONTINUE" ]; then
            echo "✓ Received continuation signal"
            echo "Session $iteration complete."
            echo "Waiting ${DELAY_BETWEEN_SESSIONS}s before next session..."
            echo "(Press Ctrl+C to pause)"
            echo ""
            sleep $DELAY_BETWEEN_SESSIONS
            continue
        elif [ "$signal" = "COMPLETE" ]; then
            echo "✓ Received completion signal"
            echo ""
            echo "🎉 Project marked as complete by agent!"
            break
        fi
    fi
    
    # No signal file found - session may have ended unexpectedly
    echo "⚠ No continuation signal received."
    echo ""
    echo "The agent did not signal to continue."
    echo "This could mean:"
    echo "  - All work is complete"
    echo "  - An error occurred"
    echo "  - The agent needs manual intervention"
    echo ""
    echo "Options:"
    echo "  - Run 'opencode /auto-continue' manually to resume"
    echo "  - Run this script again to restart the loop"
    echo "  - Check opencode-progress.txt for status"
    echo ""
    break
done

echo ""
echo "═══════════════════════════════════════════════════"
echo "  Autonomous session ended"
echo "═══════════════════════════════════════════════════"
echo ""

if [ -f "feature_list.json" ]; then
    passing=$(grep -c '"passes": true' feature_list.json 2>/dev/null || echo "0")
    total=$(grep -c '"passes"' feature_list.json 2>/dev/null || echo "?")
    echo "Final status: $passing / $total tests passing"
fi

if [ -f "opencode-progress.txt" ]; then
    echo ""
    echo "Last progress update:"
    tail -20 opencode-progress.txt
fi

echo ""
echo "To continue: ./scripts/run-autonomous.sh"
echo "To enhance:  opencode /auto-enhance"
