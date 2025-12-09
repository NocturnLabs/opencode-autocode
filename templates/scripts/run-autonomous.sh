#!/bin/bash
# OpenCode Autonomous Agent Runner
# Runs OpenCode in a loop with session management

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
    
    # Check if this is first run
    if [ ! -f "feature_list.json" ]; then
        echo "First run - using /auto-init command"
        echo ""
        opencode /auto-init
    else
        # Count remaining tests
        remaining=$(grep -c '"passes": false' feature_list.json 2>/dev/null || echo "?")
        passing=$(grep -c '"passes": true' feature_list.json 2>/dev/null || echo "0")
        echo "Progress: $passing passing, $remaining remaining"
        echo ""
        
        # Check if all tests pass
        if [ "$remaining" = "0" ]; then
            echo "🎉 All tests passing! Project complete!"
            break
        fi
        
        echo "Continuing - using /auto-continue command"
        echo ""
        opencode /auto-continue
    fi
    
    echo ""
    echo "Session $iteration complete."
    echo "Waiting ${DELAY_BETWEEN_SESSIONS}s before next session..."
    echo "(Press Ctrl+C to pause)"
    echo ""
    sleep $DELAY_BETWEEN_SESSIONS
done

echo ""
echo "═══════════════════════════════════════════════════"
echo "  Autonomous session complete"
echo "═══════════════════════════════════════════════════"
echo ""

if [ -f "feature_list.json" ]; then
    passing=$(grep -c '"passes": true' feature_list.json 2>/dev/null || echo "0")
    total=$(grep -c '"passes"' feature_list.json 2>/dev/null || echo "?")
    echo "Final status: $passing / $total tests passing"
fi

echo ""
echo "To continue later, run this script again."
echo "To run enhancements discovery: opencode /auto-enhance"
