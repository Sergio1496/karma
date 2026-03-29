# RTK - Rust Token Killer

**Usage**: Token-optimized CLI proxy (60-90% savings on dev operations)

## Meta Commands (always use rtk directly)

```bash
rtk gain              # Show token savings analytics
rtk gain --history    # Show command usage history with savings
rtk discover          # Analyze Claude Code history for missed opportunities
rtk proxy <cmd>       # Execute raw command without filtering (for debugging)
rtk session           # Show RTK adoption across Claude Code sessions
rtk cc-economics      # Spending (ccusage) vs savings (rtk) analysis
rtk verify            # Verify hook integrity and run filter tests
```

## Installation Verification

```bash
rtk --version         # Should show: rtk X.Y.Z
rtk gain              # Should work (not "command not found")
which rtk             # Verify correct binary
```

## Hook-Based Usage

All commands below are automatically rewritten by the Claude Code hook.
Example: `git status` → `rtk git status` (transparent, 0 tokens overhead)

## Supported Commands

### File & Directory
- `ls` — Directory listing with compact output
- `tree` — Directory tree, token-optimized
- `read` — File reading with intelligent filtering
- `find` — File search with compact tree output
- `grep` — Compact grep, groups by file
- `wc` — Word/line/byte count, compact
- `diff` — Ultra-condensed diff (only changed lines)
- `cat/head/tail` → rewritten to `rtk read`

### Git & GitHub
- `git` — All git commands with compact output
- `gh` — GitHub CLI with token-optimized output
- `gt` — Graphite stacked PR commands

### JavaScript / TypeScript
- `npm` — Filtered output, strips boilerplate
- `pnpm` — Ultra-compact output
- `npx` — Intelligent routing (tsc, eslint, prisma → specialized)
- `tsc` — TypeScript compiler, grouped errors
- `vitest` — Compact test output
- `next` — Next.js build, compact
- `lint` — ESLint, grouped rule violations
- `prettier` — Format checker, compact
- `prisma` — No ASCII art, compact

### Testing
- `test` — Universal: show only failures
- `cargo test` — Rust test failures only
- `pytest` — Python test runner, compact
- `rspec` — Rails/Ruby test runner, compact
- `rake` — Minitest compact output
- `playwright` — E2E tests, compact
- `vitest` — Vitest compact output

### Rust
- `cargo` — All cargo commands, compact
- `err` — Show only errors/warnings from any command

### Python
- `pytest` — Test runner, compact
- `ruff` — Linter/formatter, compact
- `mypy` — Type checker, grouped errors
- `pip` — Package manager, compact (auto-detects uv)

### Go
- `go` — All go commands, compact
- `golangci-lint` — Linter, compact

### Ruby
- `rake` — Minitest, compact
- `rspec` — RSpec, compact
- `rubocop` — Linter, compact

### Flutter / Dart
- `flutter` — test/build/analyze/pub/doctor/clean/create

### .NET
- `dotnet` — build/test/restore/format, compact

### DevOps
- `docker` — Docker commands, compact
- `kubectl` — Kubernetes commands, compact
- `aws` — AWS CLI, force JSON, compress
- `curl` — Auto-JSON detection, schema output
- `wget` — Strips progress bars

### Data & Logs
- `json` — Compact values or schema-only with --schema
- `log` — Filter and deduplicate log output
- `env` — Environment variables, sensitive values masked
- `psql` — PostgreSQL, strip borders, compress tables
- `deps` — Summarize project dependencies
- `smart` — 2-line technical summary of any command
- `summary` — Heuristic summary of any command output

### Utilities
- `format` — Universal format checker (prettier, black, ruff)
- `proxy` — Execute without filtering but track usage
- `trust/untrust` — Manage project-local TOML filters
- `rewrite` — Show what RTK would rewrite a command to
- `hook` — Hook processors for LLM CLI tools

## Options

- `-v` / `-vv` / `-vvv` — Verbosity levels
- `-u` / `--ultra-compact` — ASCII icons, inline format (Level 2)
- `--skip-env` — Set SKIP_ENV_VALIDATION=1 for child processes
