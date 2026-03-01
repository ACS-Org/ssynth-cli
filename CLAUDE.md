# CLAUDE.md — ssynth-cli

## Build & Test Commands

```bash
cargo clippy --release --all-targets -- -D warnings   # lint (MUST pass clean)
cargo test --release                                    # tests
cargo fmt                                               # format
```

Always run clippy and tests before committing.

## Code Conventions

- **Lints:** `unsafe_code = "deny"`, clippy pedantic warn. Same policy as supersynth.
- **No `#[allow(...)]`** — fix the underlying issue instead.
- **No git commit amending** — always create new commits.
- **Errors:** `CliError` (thiserror) for typed errors, `anyhow::Result` for internal chains.
- **Output:** Human-readable tables by default, `--json` for machine output.
