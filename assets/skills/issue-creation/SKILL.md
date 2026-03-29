---
name: issue-creation
description: >
  Issue creation workflow following an issue-first enforcement system.
  Trigger: When creating a GitHub issue, reporting a bug, or requesting a feature.
license: Apache-2.0
metadata:
  version: "1.0"
---

# Issue Creation Skill

## When to Use

Load this skill whenever you need to:
- Report a bug
- Request a new feature or enhancement
- Open a GitHub issue on the project repository

## Critical Rules

1. **Use issue templates** when available (bug report, feature request).
2. **Search for duplicates first** — check existing issues before creating a new one.
3. **Issues should be approved before work begins** — wait for maintainer review before opening a PR.
4. **Questions go to Discussions** — use GitHub Discussions for questions and general conversation.

## Workflow

```
1. Search existing issues → confirm it's not a duplicate
   gh issue list --state open --search "your keywords"

2. Choose the correct template:
   - Bug   → bug report template
   - Feat  → feature request template

3. Submit the issue

4. Wait — a maintainer reviews and approves (or closes)

5. Only AFTER approval → open a PR referencing this issue
```

---

## Bug Report

### Required Fields

| Field | Description |
|-------|-------------|
| Bug Description | Clear description of what the bug is |
| Steps to Reproduce | Numbered steps to reproduce the behavior |
| Expected Behavior | What should happen |
| Actual Behavior | What actually happens |
| Version | Output of `--version` command |
| Operating System | macOS / Linux distro / Windows / WSL |
| Affected Area | Which part of the project is affected |

### Example CLI Command

```bash
gh issue create \
  --template bug_report.yml \
  --title "fix(scope): short bug description"
```

---

## Feature Request

### Required Fields

| Field | Description |
|-------|-------------|
| Problem Statement | Describe the problem this feature solves |
| Proposed Solution | Specific description of the feature |
| Alternatives Considered | (optional) Other approaches you thought about |
| Additional Context | (optional) Screenshots, config files, etc. |

### Example CLI Command

```bash
gh issue create \
  --template feature_request.yml \
  --title "feat(scope): short feature description"
```

---

## Label System

### Status Labels

| Label | Description | Who Applies |
|-------|-------------|-------------|
| `status:needs-review` | Newly opened, awaiting review | Auto / Author |
| `status:approved` | Approved — work can begin | Maintainer |
| `status:in-progress` | Being actively worked on | Contributor |
| `status:blocked` | Blocked by another issue | Maintainer / Contributor |
| `status:wont-fix` | Out of scope | Maintainer |

### Type Labels

| Label | Description |
|-------|-------------|
| `bug` | Defect report |
| `enhancement` | Feature or improvement request |
| `type:bug` | Bug fix (used on PRs) |
| `type:feature` | New feature (used on PRs) |
| `type:docs` | Documentation only (used on PRs) |
| `type:refactor` | Refactoring (used on PRs) |
| `type:chore` | Build, CI, tooling (used on PRs) |
| `type:breaking-change` | Breaking change (used on PRs) |

### Priority Labels

| Label | Description |
|-------|-------------|
| `priority:critical` | Blocking issues, security vulnerabilities |
| `priority:high` | Important, affects many users |
| `priority:medium` | Normal priority |
| `priority:low` | Nice to have |

---

## Approval Workflow

```
Issue submitted
      │
      ▼
status:needs-review
      │
      ▼
Maintainer reviews
      │
  ┌───┴────────────────┐
  │                    │
  ▼                    ▼
status:approved    Closed
(work can begin)   (invalid / duplicate / wont-fix)
      │
      ▼
Contributor starts work
      │
      ▼
status:in-progress
      │
      ▼
PR opened with `Closes #<N>`
```

---

## Decision Tree

```
Do you have a question or idea to discuss?
├── YES → GitHub Discussions (NOT issues)
└── NO  → Is it a defect?
          ├── YES → Bug Report template
          └── NO  → Feature Request template
                    │
                    ▼
          Does a similar issue already exist?
          ├── YES → Comment on existing issue instead
          └── NO  → Submit new issue → wait for approval
```

---

## Commands

### Search for Existing Issues

```bash
# Search open issues
gh issue list --state open --search "your keywords"

# Search all issues including closed
gh issue list --state all --search "your keywords"
```

### Create a Bug Report

```bash
gh issue create \
  --template bug_report.yml \
  --title "fix(<scope>): <short description>"
```

### Create a Feature Request

```bash
gh issue create \
  --template feature_request.yml \
  --title "feat(<scope>): <short description>"
```

### Check Issue Status

```bash
gh issue view <number>
```
