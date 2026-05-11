# Subtask 06: Analysis And Paper Integration

## Goal

Turn benchmark results into paper-facing tables, figures, and careful claims.

## Work Items

- Add result tables to the LaTeX paper.
- Add a JSONL-driven Markdown report generator for aggregate and per-task tables.
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

- [x] Paper tables can be regenerated from JSONL or summarized with exact paths.
- [x] Limitations mention dataset size, one model route, and quality failures.
- [x] Results distinguish all-runs from successful-only subsets.
- [ ] Final LaTeX tables are filled after live V2 pilot/full-matrix runs exist.

## Current Command

```bash
cargo run --quiet -- analysis-report \
  --baseline runs/<experiment>/baseline.jsonl \
  --candidate runs/<experiment>/cache-friendly.jsonl \
  --output runs/<experiment>/analysis-report.md
```

The generated report explicitly names Codex as the development assistant, Claude
Code as the studied harness, MiMo as the current backend route/model family, and
`make-agents-cheaper` audit/eval logs as the measurement layer.

## Status

```text
report generator ready; waiting for live V2 run data before final LaTeX tables
```
