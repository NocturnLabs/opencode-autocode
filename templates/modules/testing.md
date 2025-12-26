# Testing Module

Read this module when you need to verify features with automated or interactive tests.

---

## E2E Testing (Playwright)

For web projects, every feature MUST have a Playwright E2E test.

### Setup

```bash
npm init playwright@latest
```

### Test Structure

```javascript
// tests/e2e/feature-name.spec.js
import { test, expect } from "@playwright/test";

test("feature description", async ({ page }) => {
  await page.goto("/");
  await page.click("#element-id");
  await expect(page.locator(".result")).toBeVisible();
});
```

### Running Tests

```bash
npx playwright test
npx playwright test --headed  # See browser
npx playwright test --debug   # Step through
```

---

## Verification Command

Each feature in the database should have a `verification_command`:

```sql
INSERT INTO features (category, description, passes, verification_command)
VALUES ('functional', 'User can login', 0, 'npx playwright test --grep "login"');
```

**verification_command MUST invoke E2E tests, NOT unit tests.**

---

## Interactive Verification (chrome-devtools)

In addition to automated tests, manually verify with chrome-devtools MCP:

1. `navigate_page` to the feature
2. `list_console_messages` - check for errors
3. `click` / `fill` - interact with elements
4. `take_screenshot` - visual verification

---

## Regression Testing

Before marking a new feature as passing:

1. Get count of passing features:

   ```bash
   opencode-autocode db query "SELECT COUNT(*) FROM features WHERE passes = 1"
   ```

2. Run full test suite to verify no regressions

3. If any regression detected:
   - Mark that feature as `passes = 0`
   - Fix regression BEFORE continuing
