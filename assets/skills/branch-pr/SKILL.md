---
name: branch-pr
description: >
  PR creation workflow following an issue-first enforcement system.
  Trigger: When creating a pull request, opening a PR, or preparing changes for review.
license: Apache-2.0
metadata:
  version: "2.0"
---

# Branch & PR Skill

## When to Use

Load this skill whenever you need to:
- Create a branch for a new fix or feature
- Open a pull request
- Prepare changes for review

## Critical Rules

1. **Every PR MUST link an approved issue** — `Closes/Fixes/Resolves #<N>` in the PR body.
2. **Exactly one `type:*` label** — apply exactly ONE type label to the PR.
3. **All automated checks must pass** before merge.
4. **No force-push to main/master** — protected branch.

## Workflow

```
1. Confirm the issue exists and is approved
   gh issue view <N>

2. Create a branch from main using the naming convention below

3. Implement changes following specs and design

4. Run tests locally

5. Commit using Conventional Commits format

6. Open a PR referencing the issue
   → Add exactly ONE type:* label
   → Fill in the PR body using the template

7. All automated checks must pass before merge
```

---

## Branch Naming

Branch names **must** match this pattern:

```
^(feat|fix|chore|docs|style|refactor|perf|test|build|ci|revert)\/[a-z0-9._-]+$
```

| Type | Example |
|------|---------|
| `feat/` | `feat/user-login` |
| `fix/` | `fix/duplicate-insert` |
| `docs/` | `docs/api-reference-update` |
| `refactor/` | `refactor/extract-query-sanitizer` |
| `chore/` | `chore/bump-dependency-v2` |
| `test/` | `test/add-pipeline-coverage` |
| `ci/` | `ci/add-e2e-job` |
| `revert/` | `revert/undo-breaking-change` |

**Rules:**
- All lowercase
- Use hyphens, dots, or underscores as separators (no spaces, no uppercase)
- Description must be short and descriptive

---

## PR Body Format

```markdown
## Linked Issue

Closes #<N>

## PR Type

- [ ] `type:bug` — Bug fix
- [ ] `type:feature` — New feature
- [ ] `type:docs` — Documentation only
- [ ] `type:refactor` — Code refactoring
- [ ] `type:chore` — Build, CI, or tooling changes
- [ ] `type:breaking-change` — Breaking change

## Summary

<!-- Clear description of what this PR does and why. -->

## Changes

| File / Area | What Changed |
|-------------|-------------|
| `path/to/file` | Brief description |

## Test Plan

- [ ] Tests pass
- [ ] Manually tested locally

## Checklist

- [ ] PR is linked to an issue
- [ ] I have added the appropriate `type:*` label
- [ ] Tests pass
- [ ] Documentation updated if necessary
- [ ] Commits follow Conventional Commits format
```

---

## Conventional Commits

Commit messages **must** match this pattern:

```
^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([a-z0-9\._-]+\))?!?: .+
```

### Format

```
<type>(<optional-scope>)!: <description>

[optional body]

[optional footer]
```

### Allowed Types

| Type | Purpose | PR Label |
|------|---------|----------|
| `feat` | New feature | `type:feature` |
| `fix` | Bug fix | `type:bug` |
| `docs` | Documentation only | `type:docs` |
| `refactor` | Code change (no behavior change) | `type:refactor` |
| `chore` | Maintenance, dependencies, tooling | `type:chore` |
| `style` | Formatting, linting (no logic change) | `type:chore` |
| `perf` | Performance improvement | `type:feature` |
| `test` | Adding or updating tests | `type:chore` |
| `build` | Build system or external deps | `type:chore` |
| `ci` | CI configuration | `type:chore` |
| `revert` | Reverts a previous commit | matches reverted type |

### Breaking Changes

Add `!` after the type/scope:

```
feat(cli)!: rename --config flag to --config-file

BREAKING CHANGE: the --config flag has been renamed to --config-file.
```

### Examples

```
feat(tui): add progress bar to installation steps
fix(auth): correct token refresh on expiry
docs: update contributing guide
chore(deps): bump dependency to v2
refactor(pipeline): extract step executor
test(api): add coverage for edge cases
ci: split unit and e2e test jobs
feat(cli)!: change default config path
```

---

## Commands

### Setup

```bash
# Confirm issue exists
gh issue view <N>

# Create branch
git checkout main && git pull
git checkout -b fix/<short-description>
```

### Open a PR

```bash
gh pr create \
  --title "fix(scope): short description" \
  --body "$(cat <<'EOF'
## Linked Issue

Closes #<N>

## PR Type

- [x] `type:bug` — Bug fix

## Summary

Short description of the fix.

## Test Plan

- [x] Tests pass
- [x] Manually tested locally

## Checklist

- [x] PR is linked to an issue
- [x] Appropriate `type:*` label added
- [x] Tests pass
- [x] Commits follow Conventional Commits format
EOF
)"
```

### Check PR Status

```bash
gh pr checks <PR-number>
gh pr view <PR-number>
```

### Add a Label

```bash
gh pr edit <PR-number> --add-label "type:bug"
```
