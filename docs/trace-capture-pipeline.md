# Trace Capture Pipeline

This document describes where `claude-trace` fits in the `make-agents-cheaper` evaluation pipeline.

## Role

`claude-trace` is a raw request/response capture layer. It wraps Claude Code with a Node preloader, intercepts `fetch` / HTTP calls, and writes request-response pairs to `.claude-trace/*.jsonl`.

In this project, use it as raw evidence for:

- actual `system`, `tools`, and `messages` order;
- `cache_control` breakpoint placement;
- streaming or non-streaming response usage fields;
- whether `--exclude-dynamic-system-prompt-sections` changes the request shape;
- whether baseline and candidate runs really kept model/provider/tool settings stable.

It is not the main eval format. The main eval format remains `baseline.jsonl` and `cache-friendly.jsonl` produced for `make-agents-cheaper eval`.

## Storage Layout

Raw captured logs stay under ignored experiment directories:

```text
runs/<experiment>/
  raw/
    claude-trace/
      <run_id>.jsonl
      <run_id>.html
  requests/
    <run_id>.request.json
  traces/
    <run_id>.response.json
  layers/
    <run_id>.layers.json
  tools/
    <run_id>.tools.json
  baseline.jsonl
  cache-friendly.jsonl
  notes.md
```

Do not commit raw `.claude-trace` logs. They may contain system prompts, tool outputs, file contents, local paths, and user text.

## Pipeline Position

For every measured Claude Code run:

```text
task reset
optional dynamic drift
claude-trace-wrapped Claude Code call
save raw claude-trace JSONL
extract request/body artifacts
run breakpoint/fingerprint/tool-schema checks
run task validation
append one normalized eval row
```

This creates two evidence layers:

```text
raw evidence:
  request/response pairs captured by claude-trace

normalized evidence:
  make-agents-cheaper JSONL rows and derived request/layer/tool artifacts
```

## Command Shape

Baseline example:

```bash
cd runs/fixtures/real-coding-v2
bash task-reset.sh docs-token-accounting

CLAUDE_TRACE_API_ENDPOINT="${CLAUDE_TRACE_API_ENDPOINT:-api.anthropic.com}" \
claude-trace --include-all-requests --run-with -p \
  --model mimo-v2.5-pro \
  --output-format json \
  --no-session-persistence \
  "$PROMPT"
```

Cache-friendly example:

```bash
cd runs/fixtures/real-coding-v2
bash task-reset.sh docs-token-accounting

CLAUDE_TRACE_API_ENDPOINT="${CLAUDE_TRACE_API_ENDPOINT:-api.anthropic.com}" \
claude-trace --include-all-requests --run-with -p \
  --model mimo-v2.5-pro \
  --output-format json \
  --no-session-persistence \
  --exclude-dynamic-system-prompt-sections \
  "$PROMPT"
```

After each run, move the generated `.claude-trace/log-*.jsonl` and `.html` into the experiment directory:

```bash
mkdir -p runs/<experiment>/raw/claude-trace
mv .claude-trace/log-*.jsonl runs/<experiment>/raw/claude-trace/<run_id>.jsonl
mv .claude-trace/log-*.html runs/<experiment>/raw/claude-trace/<run_id>.html
```

## Extraction Targets

A future `make-agents-cheaper trace-import` command should convert raw pairs into:

```text
requests/<run_id>.request.json
traces/<run_id>.response.json
layers/<run_id>.layers.json
tools/<run_id>.tools.json
baseline.jsonl or cache-friendly.jsonl
```

Minimum extraction rules:

- choose `/v1/messages` request-response pairs;
- keep the final pair for the run unless the run intentionally has multiple turns;
- parse SSE `body_raw` if response is streaming;
- extract request body `system`, `tools`, `messages`, and `model`;
- extract usage tokens from response body or reconstructed SSE;
- compute `uncached_input_tokens = input_tokens - cached_input_tokens` when safe;
- preserve `cache_creation_input_tokens` when present;
- mark `cache_accounting_observable=false` if usage fields are missing.

## Safety Rules

- Never commit raw trace files.
- Never paste full raw request bodies into reports.
- Sanitize local paths, API headers, auth material, file contents, and user text before sharing.
- Prefer hash/fingerprint summaries for prompt layers.
- Store paper-facing excerpts as minimal derived facts, not raw payload dumps.

## Why This Matters

Session JSONL logs are useful for token/session summaries, but raw API traces are better for proving request-shape claims. The cache-hit claim depends on the shape of the prompt prefix, so the pipeline needs access to the actual API request body whenever possible.
