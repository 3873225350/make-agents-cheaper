# Subtask 10: Add Trace Capture Pipeline

## Goal

Insert `claude-trace` into the evaluation workflow as a raw request/response capture layer.

## Why

The current audit/eval pipeline can compare token records, but raw request logs make the request-shape evidence stronger:

- actual `system` order;
- actual `tools` order;
- actual `messages` order;
- `cache_control` breakpoint placement;
- response usage fields;
- differences between baseline and cache-friendly request bodies.

## Placement

Use `claude-trace` during every measured Claude Code run, before normalization into `make-agents-cheaper` JSONL:

```text
Claude Code run
  -> claude-trace raw JSONL
  -> request/layer/tool artifacts
  -> make-agents-cheaper eval JSONL
  -> eval/task-report
```

## Storage

Raw trace files belong under ignored experiment directories:

```text
runs/<experiment>/raw/claude-trace/
```

Do not commit raw traces.

## Work Items

- Document the capture pipeline.
- Add `.claude-trace/` to `.gitignore`.
- Later implement a `trace-import` command that converts raw `claude-trace` JSONL into the normalized eval schema.
- Add a pilot run that captures both baseline and cache-friendly raw traces.

## Status

```text
documented
```
