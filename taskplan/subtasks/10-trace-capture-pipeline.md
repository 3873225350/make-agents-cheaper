# Subtask 10: Add Capture Pipeline

## Goal

Use a reliable capture path for Claude Code / MiMo evaluation rows.

## Why

The current audit/eval pipeline needs reproducible usage, cost, latency, validation, and task-success records. Direct Claude Code JSON is sufficient for the current roadmap. Raw request logs are optional and only needed for stronger request-shape evidence:

- actual `system` order;
- actual `tools` order;
- actual `messages` order;
- `cache_control` breakpoint placement;
- response usage fields;
- differences between baseline and cache-friendly request bodies.

## Placement

Use direct Claude JSON during every measured Claude Code run, before normalization into `make-agents-cheaper` JSONL:

```text
Claude Code run
  -> raw/claude-json/<run_id>.json
  -> claude-json-import
  -> make-agents-cheaper eval JSONL
  -> eval/task-report
```

## Storage

Raw direct JSON files belong under ignored experiment directories:

```text
runs/<experiment>/raw/claude-json/
```

Do not commit raw outputs.

## Work Items

- [x] Document the capture pipeline.
- [x] Add `.claude-trace/` to `.gitignore`.
- [x] Implement a `trace-import` command that converts raw `claude-trace` JSONL into the normalized eval schema.
- [x] Implement a `claude-json-import` fallback for direct Claude Code `--output-format json` results.
- [x] Package the recovery workflow as a project skill at `.claude/skills/claude-trace-recovery/`.
- [x] Make direct Claude JSON the default current capture path.
- [~] Add a successful pilot run using direct Claude JSON for both baseline and cache-friendly.
  - Live Claude/MiMo direct JSON pilot ran at `runs/2026-05-09-live-claude-mimo-pilot/`.
  - Full-matrix `control-steady` first slice failed validation before permission mode was fixed.

## Current Decision

Do not install or require `claude-trace` for the V2 matrix. Direct Claude Code JSON output is enough for usage/cost/validation eval, but not for request-shape evidence such as system ordering, tool ordering, message ordering, or breakpoint placement.

Project-local recovery workflow:

```text
.claude/skills/claude-trace-recovery/
```

Use this skill whenever an agent is tempted to block on `claude-trace`; it should use direct JSON capture unless the user explicitly asks for request-shape artifacts.

Fallback:

```bash
cargo run --quiet -- claude-json-import \
  --input runs/<experiment>/raw/claude-json/<run_id>.json \
  --run-id <run_id> \
  --task-id <task-id> \
  --condition <baseline|cache-friendly> \
  --slice <control-steady|dynamic-drift> \
  --repeat-id <n> \
  --phase measured \
  --output runs/<experiment>/<baseline|cache-friendly>.jsonl \
  --validation-path runs/<experiment>/validation/<run_id>.txt \
  --validation-passed <true|false> \
  --task-success <true|false>
```

## Status

```text
direct-json capture implemented and current; trace importer retained as optional higher-evidence path
```
