# evolution-2026-05-09-real-coding-v1.md

## Loop Type

- type: execution

## Plan

- path: `taskplan/roadmap.md`
- milestone: move from micro mechanism tests to real coding-task benchmark setup
- bounded target: close Phase 1, create the fixture repo, and define real-coding task-suite v1

## Review Window

- reviewed loops: current roadmap plus subtasks 01-07
- status: Phase 1 evidence exists; Phase 2 and Phase 3 were executable in this pass

## Completed

- Verified the existing micro-suite JSONL with `make-agents-cheaper eval`.
- Verified per-task token reporting with `make-agents-cheaper task-report`.
- Created the ignored fixture repo at `runs/fixtures/real-coding-v1/`.
- Added deterministic Rust tests, reset script, and validation script for the fixture.
- Wrote `docs/task-suites/real-coding-ablation-v1.md`.
- Marked roadmap phases 1-3 as done.

## Failed or Deferred

- The real Claude Code / MiMo pilot was not run in this pass.
- The fixture lives under ignored `runs/`, so its files are not part of the parent repository commit unless intentionally moved or archived later.

## Decisions

- Keep the fixture under `runs/fixtures/` to avoid benchmark mutations touching the main project.
- Use `docs-edit` and `bug-fix` as the recommended pilot pair.
- Continue recording token usage first; cost remains secondary.

## Analysis Checks

- regression risk: low for committed project files; fixture is ignored and isolated
- drift risk: pilot must still separate `control-steady` from `dynamic-drift`
- version safety: current suite assumes Claude Code routed to `mimo-v2.5-pro`
- plan adjustment: next phase is paired A/B pilot, not full matrix

## Next Handoff

```text
Continue from taskplan/roadmap.md. Run the Phase 4 paired A/B pilot for docs-edit and bug-fix using docs/task-suites/real-coding-ablation-v1.md. Record raw traces, validation logs, JSONL, per-task token usage, and anomaly notes.
```
