# Regression Testing Suite

This directory contains the regression testing suite for the opencode-autocode project. The regression tests ensure that changes to the codebase don't break existing functionality.

## Directory Structure

```
tests/regression/
├── config/                 # Configuration files
│   ├── default.toml       # Default regression test configuration
│   └── mod.rs             # Configuration module
├── fixtures/              # Test fixtures and sample data
├── baselines/             # Baseline results for comparison
├── test_cases/            # Test case definitions
├── results/               # Test execution results (generated)
├── src/                   # Regression test source code
│   ├── lib.rs            # Main library file
│   └── runner.rs         # Test runner implementation
├── Cargo.toml            # Cargo configuration for regression tests
└── run_regression_tests.sh # Test execution script
```

## Running Regression Tests

### Quick Start

```bash
# Run all regression tests with default configuration
./tests/regression/run_regression_tests.sh
```

### Custom Configuration

```bash
# Use a custom configuration file
./tests/regression/run_regression_tests.sh --config path/to/config.toml

# Enable verbose output
./tests/regression/run_regression_tests.sh --verbose

# Skip baseline comparison
./tests/regression/run_regression_tests.sh --no-baseline

# Stop on first failure
./tests/regression/run_regression_tests.sh --fail-fast
```

## Configuration

The regression tests are configured via TOML files in the `config/` directory. The default configuration is in `config/default.toml`.

### Configuration Options

- `execution.test_timeout_seconds`: Timeout for individual tests (default: 300)
- `execution.max_concurrent_tests`: Maximum concurrent tests (default: 4)
- `execution.fail_fast`: Stop on first failure (default: false)
- `reporting.detailed_reports`: Generate detailed reports (default: true)
- `reporting.baseline_comparison`: Compare with baselines (default: true)
- `reporting.output_formats`: Report formats (default: ["json", "html", "summary"])

## Test Cases

Test cases are defined as JSON files in the `test_cases/` directory. Each test case specifies:

- `name`: Test name
- `category`: Test category (functional, integration, performance)
- `test_type`: Type of test to run
- `config`: Test-specific configuration

### Example Test Case

```json
{
  "name": "basic_generator_test",
  "category": "functional",
  "test_type": "generator_prompt",
  "config": {
    "input": "A simple todo app",
    "expected_contains": ["project_specification", "technology_stack"]
  }
}
```

## Baselines

Baseline results are stored in the `baselines/` directory as JSON files. These represent the expected behavior of the system and are used for regression detection.

## Results

Test results are stored in the `results/` directory after execution. Results include:

- `results.json`: Detailed test results
- `summary.txt`: Human-readable summary
- `report.html`: HTML report (if configured)

## Adding New Tests

1. Create a new JSON file in `test_cases/` with your test case definition
2. Implement the test logic in the appropriate module in `src/`
3. Run the tests to generate initial baseline results
4. Review and commit the baseline if the test passes

## Test Categories

- **Functional**: Test individual components and functions
- **Integration**: Test component interactions and workflows
- **Performance**: Test execution time and resource usage
- **Security**: Test security-related functionality

## Continuous Integration

The regression tests can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions step
- name: Run Regression Tests
  run: ./tests/regression/run_regression_tests.sh --fail-fast
```

## Maintenance

- Update baselines when intentional changes are made to functionality
- Review failed tests to determine if they're due to bugs or expected changes
- Regularly update test cases to cover new functionality
- Monitor test execution time and optimize slow tests