---
name: claude-trace-recovery
description: Use the make-agents-cheaper direct Claude JSON capture workflow. Use when claude-trace is missing, should be skipped, raw request capture fails, or Codex must normalize Claude Code --output-format json with claude-json-import.
---

# Claude Trace Recovery

## Purpose

Keep `make-agents-cheaper` experiments moving without requiring `claude-trace`, while preserving evidence discipline.

The default path is direct Claude JSON plus `claude-json-import`. Optional trace capture is only for future request-shape analysis.

## Decision Flow

1. Use the direct JSON path by default:

```text
claude --output-format json
-> claude-json-import
-> baseline.jsonl or cache-friendly.jsonl
```

2. Mark evidence strength:

- direct JSON: usage, cost, latency, validation, and task success;
- optional trace path: usage plus request-shape evidence;
- no cache accounting: exploratory evidence only.

3. Do not install or debug `claude-trace` unless the user explicitly asks for request-shape artifacts.

## Required Rules

- Never delete or overwrite failed or anomalous runs.
- Keep warm-up calls out of measured results.
- Store direct Claude JSON fallback files under `runs/<experiment>/raw/claude-json/`.
- Store optional trace files under `runs/<experiment>/raw/claude-trace/` only when explicitly captured.
- Do not paste raw requests, system prompts, auth headers, or full tool outputs into reports.
- When using direct JSON fallback, record that request/layer/tool artifacts are unavailable.
- For ignored fixture experiments, use noninteractive permission mode only when the run is intentionally sandboxed to the fixture.

## Command Details

Read [references/command-shapes.md](references/command-shapes.md) when preparing or repairing a concrete run command.

Use the project CLI commands:

- `claude-json-import` for direct Claude Code JSON;
- `trace-import` only for optional raw `claude-trace` JSONL;
- `eval`, `task-report`, and `analysis-report` after normalization.

## Handoff Note

After every recovery pass, write or update an evolution note under `taskplan/` with:

- that direct JSON was used;
- raw artifact locations;
- validation result;
- that request-shape evidence is unavailable unless optional trace artifacts exist;
- the next smallest safe run.
