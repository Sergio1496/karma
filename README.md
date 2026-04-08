<h1 align="center">Karma</h1>

<p align="center">
  <strong>Configure Claude Code with agents, model routing & token optimizers</strong>
</p>

<p align="center">
  <a href="#installation">Installation</a> &bull;
  <a href="#quick-start">Quick Start</a> &bull;
  <a href="#components">Components</a> &bull;
  <a href="#optimizers">Optimizers</a> &bull;
  <a href="#commands">Commands</a>
</p>

---

Karma is a Rust CLI that sets up Claude Code with intelligent model routing, 16 specialized sub-agents, reusable skills, MCP servers, security defaults, and a context isolation system that keeps your sessions clean.

```
$ karma
```

Launches an interactive TUI wizard that walks you through configuration:

```
 Model Preset    ──  Balanced / Performance / Economy
 Scope           ──  User (~/.claude/) or Project (.claude/)
 Preset          ──  Minimal / Recommended / Full / Custom
 Components      ──  Skills, Orchestrator, MCP, Permissions, Sub-Agents
 Skills          ──  branch-pr, issue-creation
 Optimizers      ──  Context Isolation, RTK, Code Review Graph
```

Every change is atomic (backup before write, rollback on error) and idempotent (safe to run multiple times).

---

## Installation

```bash
cargo install --git https://github.com/Sergio1496/karma
```

Or build from source:

```bash
git clone https://github.com/Sergio1496/karma
cd karma
cargo install --path .
```

## Quick Start

```bash
# Interactive TUI (recommended)
karma

# Or install recommended preset directly
karma install --preset recommended

# Analyze current project and get suggestions
karma analyze
```

## Components

### Model Routing (Orchestrator)

Injects a phase-to-model assignment table into `CLAUDE.md`. Claude reads it at session start and routes each `Agent` call to the right model.

| Preset | Opus | Sonnet | Haiku |
|--------|------|--------|-------|
| **Balanced** | Planning, architecture, code review, debugging | Implementation, specs, testing (default) | Archiving, docs, search |
| **Performance** | + verification, test writing | Implementation (default) | Archiving, docs, search |
| **Economy** | -- | Everything (default) | Archiving, docs, search |

### Sub-Agents (16)

Pre-configured agent definitions deployed to `~/.claude/agents/`:

**SDD Workflow** -- sdd-explore, sdd-propose, sdd-spec, sdd-design, sdd-tasks, sdd-apply, sdd-verify, sdd-archive

**General Purpose** -- code-review, debugger, test-writer, docs-writer, refactor, searcher, git-ops, planner

Each agent gets a model assignment based on the selected preset.

### Skills

Reusable workflow definitions installed to `~/.claude/skills/`:

- **branch-pr** -- PR creation with issue-first enforcement, branch naming conventions, conventional commits
- **issue-creation** -- Bug/feature templates with status labels and priority tracking

### MCP Servers

Configures [Context7](https://github.com/upstash/context7) for framework documentation injection. Deep merges into `.claude.json` without touching existing servers.

### Permissions

Security deny rules merged into `settings.json`:

```
Bash(rm -rf /)    Bash(rm -rf ~)    Bash(rm -rf .)
Read(.env)        Read(credentials*)  Read(*secret*)
```

Additive merge -- never removes existing permissions.

## Optimizers

### Context Isolation

Intercepts broad searches via a `PreToolUse` hook and forces delegation to disposable Explore subagents. The main session only receives compact summaries.

**How it works:**

```
Claude  -->  Grep(pattern="TODO", path="")     # broad, no path
  |
  v
PreToolUse hook  -->  karma hook-guard          # evaluates tool_input
  |
  v
DENY + additionalContext: "Use Explore subagent"
  |
  v
Claude  -->  Agent(subagent_type="Explore")     # isolated context
  |
  v
Main session receives 5-10 bullet summary       # context stays clean
```

**Heuristics:**

| Scenario | Decision |
|----------|----------|
| Grep/Glob without path, recursive glob | DENY |
| Grep/Glob with specific path | ALLOW |
| Grep/Glob without path, narrow glob (no `**`) | ALLOW |
| Glob with scoped pattern (`src/components/**`) | ALLOW |
| Read calls 1-3 per session | ALLOW (silent) |
| Read calls 4+ per session | ALLOW + nudge |

**Estimated savings: 73-92% of research tokens per session.**

### RTK (Rust Token Killer)

CLI proxy that compresses output from 80+ commands (git, npm, cargo, tsc, docker, kubectl...). Transparent hook-based rewriting: `git status` becomes `rtk git status` automatically.

**60-90% token savings per command.**

### Code Review Graph

Builds a knowledge graph of your codebase for structural code review. Auto-updates via `PostToolUse` hook after Write/Edit operations.

**~8x token reduction on code reviews.**

## Commands

```
karma                         # Interactive TUI
karma install [OPTIONS]       # Install components
karma install --preset full   # Install everything
karma install -c orchestrator # Install specific component
karma install -s branch-pr    # Install specific skill
karma analyze                 # Analyze project, suggest components
karma status                  # Show what's installed
karma sync                    # Re-fetch remote skills
karma restore --latest        # Rollback to last backup
```

### Install Options

| Flag | Description |
|------|-------------|
| `--preset` | `minimal` / `recommended` / `full` / `custom` |
| `--component, -c` | Individual component (repeatable) |
| `--skill, -s` | Individual skill (repeatable) |
| `--model-preset, -m` | `balanced` / `performance` / `economy` |
| `--scope` | `user` (default) / `project` |
| `--dry-run` | Preview changes without writing |
| `--yes, -y` | Skip confirmation |

## Architecture

```
src/
  cli/           # Clap command definitions
  components/    # 7 installable components (Component trait)
  config/        # Path resolution, types, skill catalog
  filemerge/     # Section injection, JSON deep merge, atomic writes
  pipeline/      # Prepare -> Apply -> Rollback orchestration
  backup/        # Snapshot & restore system
  analyzer/      # Project tech detection & suggestions
  remote/        # Skill fetcher with local cache
  tui/           # Ratatui interactive wizard
  state/         # Persistent installation state
assets/
  templates/     # Orchestrator, context isolation markdown
  defaults/      # Permissions, MCP configs, agent definitions
```

**Key design decisions:**

- **Additive merges** -- never destructive, always appends
- **Idempotent** -- safe to run multiple times
- **Atomic writes** -- tempfile + rename, rollback on failure
- **Embedded assets** -- compiled into binary via `rust-embed`
- **No runtime deps** -- single static binary

## Acknowledgements

Karma builds on ideas, tools, and skills from several projects and people:

- **[Gentle AI](https://github.com/Gentleman-Programming/gentle-ai)** by [Gentleman Programming](https://github.com/Gentleman-Programming) -- Remote skills catalog (branch-pr, issue-creation), SDD workflow methodology, and the sub-agent architecture that inspired Karma's agent system.
- **[RTK (Rust Token Killer)](https://github.com/Sergio1496/rtk)** -- CLI proxy for token-optimized command output, integrated as an optimizer hook.
- **[Code Review Graph](https://pypi.org/project/code-review-graph/)** -- Knowledge graph for structural code review, integrated as an MCP server optimizer.
- **[Context7](https://github.com/upstash/context7)** by [Upstash](https://github.com/upstash) -- MCP server for live framework documentation injection.
- **[Claude Code](https://docs.anthropic.com/en/docs/claude-code)** by [Anthropic](https://github.com/anthropics) -- The AI coding agent that Karma configures.

## License

MIT
