# ssynth

Command-line client for [SuperSynth](https://supersynth.ai), a cloud FPGA synthesis platform.

Upload HDL source, run synthesis across multiple seeds in parallel, and download optimized bitstreams — all from your terminal.

## Install

```bash
cargo install --git https://github.com/ACS-Org/ssynth-cli
```

Requires Rust 1.70+.

## Authenticate

Get an API key from the SuperSynth web UI (**Settings > API Keys**), then:

```bash
ssynth login --api-key sk_live_YOUR_KEY
```

Credentials are stored in `~/.config/ssynth/config.toml` (mode 0600).

You can also set `SSYNTH_API_KEY` as an environment variable.

## Quick start

```bash
# Create a project
ssynth project create --slug my-fpga --name "My FPGA Project"

# Submit a synthesis job
ssynth job submit ./src \
  --project <PROJECT_ID> \
  --target ice40:hx8k:ct256 \
  --top top_module

# Watch logs in real time
ssynth job logs <JOB_ID> --follow

# Download the bitstream
ssynth artifact download <JOB_ID>
```

## hwbuild.yml

Put a `hwbuild.yml` in your project root to avoid repeating CLI flags:

```yaml
top_module: blinky
target:
  family: ice40
  device: hx8k
  package: ct256
constraints:
  - pins.pcf
seeds: 4
pick: best_timing
priority: standard
parallelism: 2
steps:
  - synth
  - pnr
  - bitstream
max_runtime: 2h
max_memory: 8GB
```

Then submit with just:

```bash
ssynth job submit . --project <PROJECT_ID>
```

CLI flags override `hwbuild.yml` values.

## Commands

### Jobs

```
ssynth job submit <PATH>       Submit a synthesis job
  --project <ID>               Project ID (or env SSYNTH_PROJECT)
  --target <SPEC>              Target (e.g. ice40:hx8k:ct256)
  --top <MODULE>               Top module name
  --constraints <FILE>...      Constraint files
  --seeds <N>                  Number of seeds to search
  --pick <STRATEGY>            best_timing | best_area
  --priority <LEVEL>           interactive | standard | batch
  --parallelism <N>            Parallel seed count
  --steps <LIST>               Pipeline steps (comma-separated)
  --max-runtime <DURATION>     e.g. 2h, 30m, 1h30m
  --max-memory <SIZE>          e.g. 16GB, 4096MB
  --wait                       Block until complete, streaming logs

ssynth job status <JOB_ID>     Show job status and run details
  --watch                      Refresh every 5s

ssynth job list                List jobs
  --status <STATUS>            Filter by status
  --project <ID>               Filter by project
  --limit <N>                  Max results

ssynth job logs <JOB_ID>       View job logs
  --follow                     Stream via WebSocket
  --offset <N>                 Starting line
  --limit <N>                  Max lines

ssynth job cancel <JOB_ID>     Cancel a running job

ssynth job retry <JOB_ID>      Retry failed seeds
  --scope failed|all           Retry scope (default: failed)

ssynth job clone <JOB_ID>      Clone a job with overrides
  --seeds, --parallelism, --priority, --pick, --target
```

### Artifacts

```
ssynth artifact list <JOB_ID>          List artifacts for a job
ssynth artifact download <JOB_ID>      Download artifacts
  --output-dir <DIR>                   Output directory (default: .)
```

### Projects

```
ssynth project list                    List all projects
ssynth project create                  Create a project
  --slug <SLUG> --name <NAME>
  --target <ID>                        Default target
ssynth project get <ID>                Get project details
ssynth project update <ID>             Update a project
  --name <NAME>
  --retention-days <N>
ssynth project delete <ID>             Delete a project
```

### API Keys

```
ssynth api-key create --name <NAME>    Create a new API key
  --expires-at <ISO8601>               Optional expiration
ssynth api-key list                    List API keys
ssynth api-key revoke <ID>             Revoke an API key
```

### Other

```
ssynth targets                         List available FPGA targets
ssynth usage                           Show credit balance and usage
ssynth config show                     Show current configuration
ssynth login                           Authenticate (prompts for key)
  --api-key <KEY>                      Provide key directly
```

## Global flags

| Flag | Env var | Description |
|------|---------|-------------|
| `--json` | | Output as JSON instead of tables |
| `--api-url <URL>` | `SSYNTH_API_URL` | Override API endpoint |

## .ssynthignore

Source uploads respect a `.ssynthignore` file (gitignore syntax). `.git/` and `build/` are always excluded.

Example:

```
*.log
tmp/
simulation/
```

## Typical workflow

```
1. ssynth project create --slug blinky --name "Blinky LED"
2. edit HDL source + hwbuild.yml
3. ssynth job submit . --project <ID> --wait
4. ssynth artifact download <JOB_ID>
5. flash bitstream to board
```

## Documentation

Full documentation: [docs.supersynth.ai](https://docs.supersynth.ai)

## License

GPL-3.0-only. See [LICENSE](LICENSE).
