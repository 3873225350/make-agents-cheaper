# Subtask 01: Preserve Micro Benchmark

## Goal

Keep the current exact-reply benchmark as a cheap mechanism sanity check.

It should answer:

```text
Can we observe prompt cache accounting?
Can we separate cold start from warm state?
Can dynamic drift reduce baseline cache hit?
Can cache-friendly assembly preserve high cache hit?
```

## Current Inputs

- `docs/task-suites/claude-cache-ablation-v1.md`
- `docs/paired-ablation-runbook.md`
- `runs/2026-05-09-claude-mimo-paired-drift/`
- `runs/2026-05-09-claude-mimo-task-suite-v1/`

## Work Items

- Keep micro-exact tasks small and deterministic.
- Use strict output contracts:

```text
Return only the exact string: <expected>
Do not explain.
```

- Record quality failures instead of hiding them.
- Keep retries separate from main results.

## Acceptance Criteria

- `make-agents-cheaper eval` can reproduce the summary from JSONL files.
- Warm-up calls are excluded from main results.
- Main notes explain any failed exact-reply task.

## Status

```text
done
```

## Completion Evidence

- `make-agents-cheaper eval` reproduces the multi-task micro-suite summary from `runs/2026-05-09-claude-mimo-task-suite-v1/`.
- `make-agents-cheaper task-report` prints per-task token usage from the same JSONL files.
- Warm-up, dynamic-drift, and anomaly notes are recorded under `runs/2026-05-09-claude-mimo-*`.
