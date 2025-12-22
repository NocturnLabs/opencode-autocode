# Workflow Preferences

## Testing Strategy

- Unit tests for core business logic (90% coverage target)
- Integration tests for API endpoints using node:test
- E2E tests for critical flows using headless browsers
- Custom test runner for frontend unit tests
- Regression testing via feature_list.json verification commands

## Code Style

- TypeScript strict mode enabled
- No 'any' types allowed
- Descriptive variable names
- Consistent indentation (2 spaces)
- JSDoc comments for public APIs

## Development Process

- Feature-driven development based on feature_list.json
- Commit after each feature implementation with descriptive messages
- Run tests before marking features as passing
- Lint and type-check after code changes
- Use native Node.js tools for all operations

## File Organization

- src/ for TypeScript source code
- public/ for static assets
- data/ for SQLite storage
- tests/ for test suites
- dist/ for compiled output