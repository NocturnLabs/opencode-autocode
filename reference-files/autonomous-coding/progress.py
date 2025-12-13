"""
Progress Tracking Utilities
===========================

Functions for tracking and displaying progress of the autonomous coding agent.
"""

import json
from pathlib import Path


def get_regression_status(project_dir: Path) -> str:
    """
    Get regression test status from results directory.

    Args:
        project_dir: The project directory

    Returns:
        Status string describing regression test state
    """
    regression_dir = project_dir.parent / "tests" / "regression" / "results"
    summary_file = regression_dir / "summary.txt"

    if summary_file.exists():
        try:
            content = summary_file.read_text()
            if "All tests completed successfully" in content:
                return "✅ Regression tests passing"
            elif "failed" in content.lower():
                return "❌ Regression tests failed"
            else:
                return "⚠️ Regression tests completed with issues"
        except:
            return "⚠️ Could not read regression results"
    else:
        return "⏳ No regression tests run yet"


def count_passing_tests(project_dir: Path) -> tuple[int, int]:
    """
    Count passing and total tests in feature_list.json.

    Args:
        project_dir: Directory containing feature_list.json

    Returns:
        (passing_count, total_count)
    """
    tests_file = project_dir / "feature_list.json"

    if not tests_file.exists():
        return 0, 0

    try:
        with open(tests_file, "r") as f:
            tests = json.load(f)

        total = len(tests)
        passing = sum(1 for test in tests if test.get("passes", False))

        return passing, total
    except (json.JSONDecodeError, IOError):
        return 0, 0


def print_session_header(session_num: int, is_initializer: bool) -> None:
    """Print a formatted header for the session."""
    session_type = "INITIALIZER" if is_initializer else "CODING AGENT"

    print("\n" + "=" * 70)
    print(f"  SESSION {session_num}: {session_type}")
    print("=" * 70)
    print()


def print_progress_summary(project_dir: Path) -> None:
    """Print a summary of current progress."""
    passing, total = count_passing_tests(project_dir)
    regression_status = get_regression_status(project_dir)

    if total > 0:
        percentage = (passing / total) * 100
        print(f"\nProgress: {passing}/{total} tests passing ({percentage:.1f}%)")
    else:
        print("\nProgress: feature_list.json not yet created")

    print(f"Regression Status: {regression_status}")
