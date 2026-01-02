# JavaScript/TypeScript Module

Read this module when working on web projects with JavaScript or TypeScript.

## Toolchain: Use Bun

**Always use `bun` instead of npm, pnpm, or yarn.**

- Install deps: `bun install`
- Run scripts: `bun run dev`
- Tests: `bun test`
- Init: `bun init`

---

## Port Conflict Prevention

Before starting any servers or running tests, verify required ports are free:

```bash
# Check if default ports are in use (ss is more reliable than lsof)
ss -tlnH "sport = :8000" | grep -q . && echo "Port 8000 in use" || echo "Port 8000 free"
ss -tlnH "sport = :3000" | grep -q . && echo "Port 3000 in use" || echo "Port 3000 free"
```

**If ports are occupied:**

- **NEVER kill a process unless you are 100% sure it belongs to this project and was started by you.**
- Search for a free port instead (8001, 8002, etc.)
- Update `playwright.config.ts` or other configs:
  ```bash
  sed -i 's/localhost:[0-9]*/localhost:NEW_PORT/g' playwright.config.ts
  ```

---

## ES6 Module Import Verification

Before marking a feature as passing, verify imports are correct:

1. **Check for import errors** in browser console (`ReferenceError`, `SyntaxError`)

2. **For each file you created or modified:**

   - Verify all `import` statements point to correct relative paths
   - Verify all imported names match `export` names in source files
   - Check that ES6 module files use `.js` extensions in imports (for browser)

3. **Quick import validation:**

   ```bash
   # List all exports in the project
   grep -rn "export class\|export function\|export const" src/**/*.js src/**/*.ts 2>/dev/null | head -20

   # List all imports in the project
   grep -rn "^import " src/**/*.js src/**/*.ts 2>/dev/null | head -20
   ```

4. **Common issues to check:**
   - Missing imports (class used but not imported)
   - Wrong paths (`./service.js` vs `./services/service.js`)
   - Named vs default export mismatch (`import { X }` vs `import X`)

---

## init.sh Template for Web Projects

```bash
#!/bin/bash
# init.sh with port conflict prevention

DEFAULT_PORT=8000
PORT=$DEFAULT_PORT

# Find an available port (using ss which is more reliable than lsof)
find_free_port() {
    local port=$1
    while ss -tlnH "sport = :$port" | grep -q .; do
        echo "Port $port is in use, trying $((port + 1))..."
        port=$((port + 1))
    done
    echo $port
}

PORT=$(find_free_port $DEFAULT_PORT)
echo "Starting server on port $PORT"
export PORT

# Start the server (adjust based on tech stack)
bun run dev --port $PORT &
# Or: bun start --port $PORT
# Or: python3 -m http.server $PORT

echo "Server running at http://localhost:$PORT"
```
