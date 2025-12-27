# Example: Verification commands by project type

# Rust projects:

cargo test test_feature_name
cargo test --test integration_tests

# Web projects (Playwright):

npx playwright test --grep "feature description"
npx playwright test tests/e2e/login.spec.ts

# Node.js projects:

npm test -- --grep "feature"
npx vitest run --grep "feature"

# Python projects:

pytest -k "test_feature"
python -m pytest tests/test_module.py::test_feature

# Rules:

# - NEVER use 'cargo build' or 'npm run dev' as verification

# - Each command should test ONE specific behavior

# - Commands must exit 0 on success, non-zero on failure
