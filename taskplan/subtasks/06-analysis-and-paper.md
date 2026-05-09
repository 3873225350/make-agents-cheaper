# Subtask 06: Analysis And Paper Integration

## Goal

Turn benchmark results into paper-facing tables, figures, and careful claims.

## Work Items

- Add result tables to the LaTeX paper.
- Separate mechanism evidence from real-task evidence.
- Add a cold-start vs warm-state explanation.
- Add a quality-regression section.
- Add limitations for model/provider specificity.

## Claim Discipline

Allowed:

```text
Cache-friendly prompt assembly improves cache-hit robustness under dynamic harness drift in our evaluated setting.
```

Not allowed:

```text
This always makes agents cheaper.
```

## Acceptance Criteria

- Paper tables can be regenerated from JSONL or summarized with exact paths.
- Limitations mention simple dataset size, one model route, and quality failures.
- Results distinguish all-runs from successful-only slices.

## Status

```text
planned
```
