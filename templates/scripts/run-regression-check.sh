#!/bin/bash
# run-regression-check.sh
# Runs all features marked as "passes": true in feature_list.json
# Returns exit code 0 if all pass, 1 if any fail
#
# Usage: ./run-regression-check.sh [feature_list.json]
#        ./run-regression-check.sh --help
#
# This script is designed to be used by the autonomous agent during
# regression checkpoints, or manually invoked for debugging.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default feature list path
FEATURE_LIST="${1:-feature_list.json}"

if [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
    echo "Usage: $0 [feature_list.json]"
    echo ""
    echo "Runs regression checks on all passing features in feature_list.json."
    echo "Features with verification_command will be executed automatically."
    echo "Features without verification_command will be listed for manual check."
    echo ""
    echo "Exit codes:"
    echo "  0 - All automated tests passed"
    echo "  1 - One or more tests failed"
    echo "  2 - feature_list.json not found"
    exit 0
fi

if [[ ! -f "$FEATURE_LIST" ]]; then
    echo -e "${RED}ERROR: $FEATURE_LIST not found${NC}"
    exit 2
fi

echo "========================================"
echo "REGRESSION CHECK"
echo "========================================"
echo "Feature list: $FEATURE_LIST"
echo ""

# Count features
TOTAL_FEATURES=$(jq 'length' "$FEATURE_LIST")
PASSING_FEATURES=$(jq '[.[] | select(.passes == true)] | length' "$FEATURE_LIST")
FAILING_FEATURES=$((TOTAL_FEATURES - PASSING_FEATURES))

echo "Total features: $TOTAL_FEATURES"
echo "Passing: $PASSING_FEATURES"
echo "Failing: $FAILING_FEATURES"
echo ""

if [[ "$PASSING_FEATURES" -eq 0 ]]; then
    echo -e "${YELLOW}No passing features to check.${NC}"
    exit 0
fi

# Track results
PASSED=0
FAILED=0
MANUAL_CHECK=0
REGRESSIONS=""

echo "----------------------------------------"
echo "Running regression checks..."
echo "----------------------------------------"
echo ""

# Extract passing features with verification_command
while IFS= read -r feature; do
    description=$(echo "$feature" | jq -r '.description')
    verification_cmd=$(echo "$feature" | jq -r '.verification_command // empty')
    
    if [[ -n "$verification_cmd" ]]; then
        echo -n "Testing: $description... "
        
        # Run the verification command
        if eval "$verification_cmd" > /dev/null 2>&1; then
            echo -e "${GREEN}PASS${NC}"
            ((PASSED++))
        else
            echo -e "${RED}FAIL${NC}"
            ((FAILED++))
            REGRESSIONS="$REGRESSIONS\n  - $description"
        fi
    else
        echo -e "${YELLOW}MANUAL${NC}: $description"
        ((MANUAL_CHECK++))
    fi
done < <(jq -c '.[] | select(.passes == true)' "$FEATURE_LIST")

echo ""
echo "========================================"
echo "REGRESSION CHECK SUMMARY"
echo "========================================"
echo -e "Automated tests passed: ${GREEN}$PASSED${NC}"
echo -e "Automated tests failed: ${RED}$FAILED${NC}"
echo -e "Manual verification needed: ${YELLOW}$MANUAL_CHECK${NC}"
echo ""

if [[ "$FAILED" -gt 0 ]]; then
    echo -e "${RED}REGRESSIONS DETECTED:${NC}$REGRESSIONS"
    echo ""
    echo "Action required: Fix regressions before continuing."
    exit 1
else
    echo -e "${GREEN}All automated regression tests passed!${NC}"
    if [[ "$MANUAL_CHECK" -gt 0 ]]; then
        echo "Reminder: $MANUAL_CHECK features require manual verification."
    fi
    exit 0
fi
