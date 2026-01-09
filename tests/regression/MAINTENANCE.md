# Regression Testing Suite - Maintenance Guide

## Overview

The regression testing suite ensures that changes to the opencode-forger codebase don't introduce unintended side effects or break existing functionality. This guide covers maintenance procedures, baseline updates, and troubleshooting.

## Daily Operations

### Running Tests

```bash
# Quick run with default settings
./tests/regression/run_regression_tests.sh

# Run with custom configuration
./tests/regression/run_regression_tests.sh --config path/to/config.toml

# Run in verbose mode
./tests/regression/run_regression_tests.sh --verbose

# Stop on first failure
./tests/regression/run_regression_tests.sh --fail-fast
```

### Monitoring Results

After each run, check the `tests/regression/results/` directory for:

- `summary_report.md`: High-level overview
- `detailed_report.md`: Comprehensive test details
- `baseline_updates.md`: Recommendations for baseline updates

## Baseline Management

### When to Update Baselines

Update baselines when:

1. **Intentional functionality changes**: New features or bug fixes that change expected behavior
2. **Performance improvements**: Significant speed improvements that should be tracked
3. **New test cases**: Adding new regression tests
4. **Configuration changes**: Changes to test parameters that affect results

### How to Update Baselines

1. **Run tests to generate new results**:
   ```bash
   ./tests/regression/run_regression_tests.sh
   ```

2. **Review the baseline update recommendations** in `results/baseline_updates.md`

3. **Update baseline files** in `tests/regression/baselines/`:
   - Copy relevant data from `results/` to `baselines/`
   - Update timestamps and expected values
   - Ensure baseline files follow the naming convention: `{test_name}_baseline.json`

4. **Commit baseline changes** with descriptive commit messages

### Baseline File Format

```json
{
  "test_name": "example_test",
  "timestamp": "2024-12-12T00:00:00Z",
  "expected_output": {
    "is_valid": true,
    "has_project_name": true
  },
  "performance_metrics": {
    "execution_time_ms": 25,
    "memory_usage_kb": 512
  }
}
```

## Adding New Tests

### 1. Create Test Case Definition

Add a new JSON file to `tests/regression/test_cases/`:

```json
{
  "name": "new_feature_test",
  "category": "functional",
  "test_type": "custom_test_type",
  "config": {
    "parameter1": "value1",
    "parameter2": 42
  }
}
```

### 2. Implement Test Logic

Add the test implementation to the appropriate module:

- **Functional tests**: `src/functional_tests.rs`
- **Integration tests**: `src/integration_tests.rs`

### 3. Update Test Runner

Add the new test type to the `execute_test_case` function in `src/runner.rs`.

### 4. Create Initial Baseline

Run the tests and use the results to create an initial baseline file.

## Troubleshooting

### Common Issues

#### Tests Failing Due to Environmental Differences

**Symptoms**: Tests pass locally but fail in CI/CD
**Solution**:
- Check environment variables in CI/CD configuration
- Ensure test data paths are relative or properly configured
- Verify external dependencies are available

#### Performance Regressions

**Symptoms**: Tests marked as performance regressions
**Solutions**:
- Review recent code changes for performance impacts
- Check if baseline expectations are realistic
- Consider updating baselines if performance changes are acceptable

#### Baseline Comparison Failures

**Symptoms**: Tests pass but baseline comparison shows regressions
**Solutions**:
- Review what changed in the test output
- Determine if the change is expected or a regression
- Update baselines for expected changes

### Debugging Test Failures

1. **Run tests in verbose mode**:
   ```bash
   ./tests/regression/run_regression_tests.sh --verbose
   ```

2. **Check individual test results** in `results/` directory

3. **Run specific test types** by modifying the configuration

4. **Check logs and error messages** for detailed failure information

## Configuration Management

### Test Configuration Files

Located in `tests/regression/config/`:

- `default.toml`: Default test configuration
- Custom configs can be created for different environments

### Configuration Options

```toml
[execution]
test_timeout_seconds = 300
max_concurrent_tests = 4
fail_fast = false

[reporting]
detailed_reports = true
baseline_comparison = true
output_formats = ["json", "html", "summary"]
save_results = true
```

## Performance Monitoring

### Tracking Test Performance

- Monitor execution times in baseline files
- Set up alerts for significant performance regressions
- Review performance trends over time

### Optimization

- Run tests in parallel where possible
- Optimize slow tests
- Consider test categorization for different run frequencies

## Integration with Development Workflow

### Pre-commit Hooks

Consider adding regression tests to pre-commit hooks for critical changes.

### Branch Protection

Configure branch protection rules to require regression tests to pass.

### Release Process

- Run full regression test suite before releases
- Include regression test results in release notes
- Archive test results for historical analysis

## Metrics and Reporting

### Key Metrics to Track

- Test pass/fail rates
- Execution time trends
- Number of regressions detected
- Test coverage (when available)

### Automated Reporting

- Generate weekly/monthly regression test reports
- Send notifications for test failures
- Integrate with project dashboards

## Maintenance Schedule

### Daily
- Monitor CI/CD test results
- Review any failed tests

### Weekly
- Review baseline update recommendations
- Clean up old test results
- Update test configurations as needed

### Monthly
- Review test performance trends
- Audit test cases for relevance
- Update documentation

### Quarterly
- Major baseline updates
- Test suite refactoring
- Performance optimization reviews

## Support

For issues with the regression testing suite:

1. Check this documentation
2. Review recent changes to the test suite
3. Check CI/CD logs for additional context
4. Create an issue in the project repository with:
   - Test failure details
   - Environment information
   - Steps to reproduce