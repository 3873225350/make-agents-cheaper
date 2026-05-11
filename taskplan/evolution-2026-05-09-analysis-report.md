# evolution-2026-05-09-analysis-report.md

## Loop Type

- type: execution

## Plan

- path: `taskplan/roadmap.md`
- milestone: Phase 6, analysis and paper-facing tables
- bounded target: add a JSONL-driven report generator that turns baseline/candidate records into paper-facing Markdown tables and conservative interpretation guardrails

## Review Window

- reviewed loops: roadmap, Subtask 06, current CLI `eval`/`task-report` paths
- status: previous phases have manifest, pilot plan, trace import, and matrix plan ready; live V2 Claude/MiMo runs are still deferred

## Completed

- Added `analysis-report` CLI command with `--baseline`, `--candidate`, and optional `--output`.
- Reused existing `RunRecord` and `RunStats` parsing so the report comes from the same JSONL schema as `eval` and `task-report`.
- Generated aggregate all-runs and successful-only tables.
- Generated per-task all-runs tables.
- Added quality, cache-accounting, and savings gates so the paper cannot silently turn failed or unobservable rows into a positive claim.
- Updated README, evaluation protocol, roadmap, and Subtask 06 with the new command and current status.

## Failed or Deferred

- Final LaTeX result tables are still deferred until real V2 pilot/full-matrix data exists.
- No live Claude Code/MiMo experiment was executed in this pass.

## Decisions

- Keep all-runs accounting as the primary evidence surface.
- Treat successful-only rows as diagnostic only.
- Keep the report in Markdown first so it can be regenerated from local JSONL before being copied into paper tables.

## Analysis Checks

- regression risk: low; this is a new command path that reuses existing parsers
- drift risk: medium; future trace schemas may add fields, but current report only depends on stable eval fields
- version safety: no destructive git operations; existing unrelated dirty files left untouched
- plan adjustment: Phase 6 moves from planned to report-generator-ready, but not complete

## Next Handoff

```text
Continue taskplan/roadmap.md. Next smallest useful slice: run the new analysis-report smoke on existing JSONL, then either fill a placeholder paper table from a real local run if available or prepare the Phase 7 skill usability ablation without claiming live V2 results.
```
