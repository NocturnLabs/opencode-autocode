# Testing Module

Read this module when you need to verify features with automated or interactive tests.

---

## E2E Testing (Playwright)

For web projects, every feature MUST have a Playwright E2E test.

### Setup

```bash
bun create playwright
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
bun x playwright test
bun x playwright test --headed  # See browser
bun x playwright test --debug   # Step through
```

---

## Verification Command

Each feature in the database should have a `verification_command`:

```sql
INSERT INTO features (category, description, passes, verification_command)
VALUES ('functional', 'User can login', 0, 'bun x playwright test --grep "login"');
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

Before marking a new feature as passing, you MUST verify that no existing functionality was broken.

```bash
# Run automated regression check
opencode-autocode db check
```

If any regression is detected:

1. Identify the failing feature
2. Fix the regression BEFORE continuing
3. DO NOT mark the current feature as passing until all regressions are fixed.
