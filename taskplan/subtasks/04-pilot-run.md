# Subtask 04: Run Pilot A/B

## Goal

Run the standardized paired A/B flow on 1-2 realistic coding tasks before scaling.

## Experiment Shape

For each selected task:

```text
baseline warm-up
cache-friendly warm-up
dynamic drift probe
baseline measured call
cache-friendly measured call
validation command
JSONL extraction
eval summary
```

## Required Logs

- raw Claude JSON traces;
- extracted baseline/cache-friendly JSONL;
- drift state;
- validation output;
- anomaly notes.

## Acceptance Criteria

- Both conditions complete the task or failure is clearly recorded.
- `make-agents-cheaper eval` runs on the extracted JSONL.
- Notes explain whether cache-friendly improved cache hit and whether task quality changed.

## Status

```text
planned
```
