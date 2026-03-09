# CLAUDE.md — ssynth-cli

## Build & Test Commands

```bash
npm run lint         # eslint strict (MUST pass clean)
npm run test         # unit tests (node:test)
```

Always run lint and tests before committing.

## Code Conventions

- **Language:** Plain JavaScript (ESM, `"type": "module"`), Node.js >=22.
- **No `var`** — use `const` (preferred) or `let`.
- **Errors:** `CliError` class in `lib/error.js` with typed exit codes.
- **Output:** Human-readable tables by default, `--json` for machine output.
- **Config:** JSON at `~/.config/ssynth/config.json`, chmod 0600.
- **HTTP:** Built-in `fetch` (no axios/node-fetch). `ApiClient` in `lib/client.js`.
- **WebSocket:** Built-in `WebSocket` (Node 22+).
- **No git commit amending** — always create new commits.
