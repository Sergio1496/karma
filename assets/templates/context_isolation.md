# Context Isolation

Protect the main session from token bloat. Heavy research MUST run inside disposable subagents (the `Agent` tool) so only a compact summary returns to this context.

## When to delegate (ALWAYS use Agent tool)

- Searching across the codebase (Grep/Glob for unknown locations)
- Reading 3+ files you haven't seen yet in this session
- Investigating an error whose source file is unknown
- Analyzing build output, test results, or logs longer than ~50 lines
- Exploring unfamiliar modules or directories
- Any task where you'd need multiple rounds of search-then-read

## When NOT to delegate (use tools directly)

- You already know the exact file and approximate line
- Reading 1-2 specific, known files
- A single targeted grep for one symbol in a known directory
- Information already present in this conversation

## Subagent prompt rules

1. **One sentence goal** — state what you need answered
2. **Scope boundary** — specify directories or file patterns to search
3. **Output contract** — request a structured summary, never raw file dumps
4. **Example prompt**: "Search `src/` for all callers of `process_payment()`. Return: file path, line number, and a one-line description of each call site. Max 10 results."

## Summary format subagents MUST follow

- Direct answer to the question (1-3 sentences)
- Relevant file paths with line numbers
- Key findings as bullet points (max 10)
- NEVER paste full file contents — only the specific lines that matter

## Parallel investigation

When a task requires multiple independent searches, launch all subagents in a single response (parallel Agent calls). Do not serialize what can run concurrently.

## Threshold rule

> Before calling Grep, Glob, or Read on a file you haven't touched this session, ask: "Will this likely expand to 3+ files?" If yes → subagent. If no → direct call.
