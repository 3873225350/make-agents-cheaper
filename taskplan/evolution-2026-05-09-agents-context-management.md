# Evolution: 2026-05-09 Agents Context Management

## What Changed

Added project-specific `AGENTS.md` files to make the growing project easier for agents to navigate.

Tracked guidance files:

```text
AGENTS.md
docs/AGENTS.md
paper/AGENTS.md
taskplan/AGENTS.md
runs/AGENTS.md
```

## Why

The project now has several layers:

- Rust audit/eval tooling;
- trace-capture pipeline;
- real-coding fixtures and logs;
- paper draft;
- roadmap and evolution notes;
- separate skill packaging repository.

Without local context rules, agents can easily mix up Codex, Claude Code, MiMo, audit/eval, skill packaging, and raw traces.

## Key Rules Added

- Keep Codex as development assistant, Claude Code as studied harness, and MiMo as backend route/model.
- Treat `make-agents-cheaper` audit/eval logs as main evidence.
- Treat `cheaper-skill-for-claude` as reuse/usability artifact, not primary result evidence.
- Store raw `claude-trace` logs under `runs/<experiment>/raw/claude-trace/`.
- Never commit raw traces or paste full raw request bodies.
- Update roadmap, subtasks, and evolution notes when milestones change.

## Validation

`cargo test` passed after adding the guidance files.
