# Tech Stack

## Language/Runtime

- **Primary**: TypeScript 5.x for frontend, Node.js 22.x LTS for backend
- **Rationale**: Type safety, modern ES2022+ features, native modules

## Frameworks

- **Frontend**: None (Vanilla TypeScript + DOM API)
- **Backend**: None (Built-in node:http and node:sqlite)
- **Rationale**: Zero dependencies, maximum performance, security

## Styling

- Plain CSS3 with CSS Variables
- Dynamic theming support for dark/light modes
- High-contrast accessibility compliance

## Database

- SQLite via node:sqlite (built-in)
- Single-file persistence with enterprise reliability
- No external drivers required

## State Management

- Custom Observable Pattern (frontend)
- Reactive UI updates without libraries

## Routing

- Custom Hash-based Router
- Native window.location APIs

## Forms

- Native HTML5 Form API + Custom TS Handlers
- Standards-compliant validation

## Testing

- Custom Minimal Test Runner (frontend)
- Built-in node:test (backend)
- Native assertions and DOM mocks

## Deployment

- Alpine Linux Docker multi-stage build (<50MB)
- Standard shell scripts for CI/CD
- Native health/metrics endpoints