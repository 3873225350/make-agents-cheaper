# Subtask 05: Scale Evaluation

## Goal

Run the full real-coding task matrix after the pilot is clean.

## Matrix

Minimum:

```text
5 task families
2 conditions
control-steady and dynamic-drift slices
3 measured runs per task/condition/slice
```

Recommended:

```text
include at least one multi-turn task
include at least one failing-test repair
include at least one docs-only task
```

## Metrics

- cache hit rate;
- cached input tokens per task;
- uncached input;
- output tokens per task;
- observed cost;
- total latency;
- validation pass rate;
- task success;
- number of turns;
- anomaly count.

## Acceptance Criteria

- Results are reproducible from JSONL.
- Warm-up calls are excluded.
- Failed or anomalous runs remain in the main record unless explicitly marked as diagnostic retry.

## Status

```text
planned
```
