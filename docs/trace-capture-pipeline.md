# Direct JSON Capture Pipeline

This document describes the current evidence-capture path for the `make-agents-cheaper` evaluation pipeline.

## Current Decision

The roadmap no longer depends on `claude-trace`.

Use direct Claude Code JSON as the default capture layer:

```text
Claude Code --output-format json
  -> raw/claude-json/<run_id>.json
  -> claude-json-import
  -> baseline.jsonl or cache-friendly.jsonl
  -> eval / task-report / analysis-report
```

This supports the current mainline experiment because Claude Code JSON exposes usage, cost, latency, validation, and task-success evidence. It does not expose raw API request shape, so do not use direct JSON rows to prove exact `system`, `tools`, or `messages` ordering.

## Storage Layout

Raw direct JSON logs stay under ignored experiment directories:

```text
runs/<experiment>/
  raw/
    claude-json/
      <run_id>.json
      <run_id>.stderr.txt
  validation/
    <run_id>.txt
  baseline.jsonl
  cache-friendly.jsonl
  notes.md
```

Optional trace artifacts, if someone explicitly captures them later, may still use:

```text
runs/<experiment>/raw/claude-trace/
runs/<experiment>/requests/
runs/<experiment>/traces/
runs/<experiment>/layers/
runs/<experiment>/tools/
```

Do not commit raw logs. They may contain system prompts, tool outputs, file contents, local paths, and user text.

## Command Shape

Baseline example:

```bash
cd runs/fixtures/real-coding-v2
bash task-reset.sh docs-token-accounting

claude -p \
  --model mimo-v2.5-pro \
  --output-format json \
  --no-session-persistence \
  --permission-mode bypassPermissions \
  "$PROMPT" \
  > runs/<experiment>/raw/claude-json/<run_id>.json \
  2> runs/<experiment>/raw/claude-json/<run_id>.stderr.txt
```

Cache-friendly example:

```bash
cd runs/fixtures/real-coding-v2
bash task-reset.sh docs-token-accounting

claude -p \
  --model mimo-v2.5-pro \
  --output-format json \
  --no-session-persistence \
  --permission-mode bypassPermissions \
  --exclude-dynamic-system-prompt-sections \
  "$PROMPT" \
  > runs/<experiment>/raw/claude-json/<run_id>.json \
  2> runs/<experiment>/raw/claude-json/<run_id>.stderr.txt
```

`--permission-mode bypassPermissions` is for ignored fixture experiments only. Do not generalize it to ordinary repo editing without an explicit safety decision.

## Normalization

Normalize measured baseline runs:

```bash
cargo run --quiet -- claude-json-import \
  --input runs/<experiment>/raw/claude-json/<run_id>.json \
  --run-id <run_id> \
  --task-id docs-token-accounting \
  --condition baseline \
  --slice control-steady \
  --repeat-id 1 \
  --phase measured \
  --output runs/<experiment>/baseline.jsonl \
  --validation-path runs/<experiment>/validation/<run_id>.txt \
  --validation-passed <true|false> \
  --task-success <true|false>
```

Use `--condition cache-friendly` and `--output runs/<experiment>/cache-friendly.jsonl` for candidate runs.

Minimum extraction rules:

- extract usage from Claude Code `modelUsage` when present;
- preserve `cacheReadInputTokens`, `cacheCreationInputTokens`, `inputTokens`, `outputTokens`, and `costUSD`;
- compute `uncached_input_tokens = input_tokens - cached_input_tokens` safely;
- preserve latency fields such as `duration_ms` and `duration_api_ms`;
- set `request_shape_observable=false`;
- mark `cache_accounting_observable=false` if usage fields are missing.

## Optional Trace Path

`trace-import` remains available as an optional higher-evidence path if raw request capture exists. It is not required for the current roadmap run.

Optional trace rows can support request-shape checks:

```bash
cargo run --quiet -- trace-import \
  --input runs/<experiment>/raw/claude-trace/<run_id>.jsonl \
  --run-id <run_id> \
  --task-id docs-token-accounting \
  --condition baseline \
  --slice dynamic-drift \
  --repeat-id 1 \
  --phase measured \
  --output runs/<experiment>/baseline.jsonl \
  --artifacts-dir runs/<experiment> \
  --validation-path runs/<experiment>/validation/<run_id>.txt \
  --validation-passed true
```

## Safety Rules

- Never commit raw JSON or trace files.
- Never paste full raw request bodies into reports.
- Sanitize local paths, API headers, auth material, file contents, and user text before sharing.
- Prefer hash/fingerprint summaries for prompt layers when optional request artifacts exist.
- Record failed or anomalous direct JSON runs instead of replacing them.

## Why This Matters

The primary experiment needs reliable usage/cost/validation rows first. Direct Claude JSON is enough for that. Request-shape evidence is stronger, but it is no longer a blocker for executing the V2 matrix.
