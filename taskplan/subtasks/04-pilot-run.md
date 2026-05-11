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
direct-json V2 pilot complete; mixed/negative for primary savings claim
```

## Live Pilot Snapshot

Experiment:

```text
runs/2026-05-09-live-claude-mimo-pilot/
```

Observed result for `docs-token-accounting`, `dynamic-drift`, repeat 1:

| Metric | Baseline | Cache-friendly |
| --- | ---: | ---: |
| Validation | passed | passed |
| Task success | 1/1 | 1/1 |
| Cache hit rate | 98.46% | 98.86% |
| Uncached input | 2,196 | 1,271 |
| Observed cost | $0.100471 | $0.075144 |
| Total latency | 25,098 ms | 15,762 ms |

This is a live pilot smoke result, not a full-matrix claim. It used direct
Claude Code JSON output, which is now the current roadmap capture path.

## Ready-To-Run Inputs

- Task source: `docs/task-suites/real-coding-ablation-v2.manifest.json`
- Fixture: `runs/fixtures/real-coding-v2/`
- Experiment scaffold command:

```bash
cargo run --quiet -- init-experiment --dir runs/<date>-claude-mimo-real-coding-v2-pilot
```

The scaffold now creates `raw/claude-json/`, optional trace artifact directories,
`validation/`, `baseline.jsonl`, and
`cache-friendly.jsonl`.

Generate the task-specific pilot command plan:

```bash
cargo run --quiet -- pilot-plan \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --task docs-token-accounting \
  --experiment-dir runs/<date>-claude-mimo-real-coding-v2-pilot \
  --slice dynamic-drift \
  --repeats 1
```

The command prints prompt-file setup, warm-up calls, measured calls, drift probe
commands, validation log paths, and final `eval` / `task-report` /
`analysis-report` commands.

Trace-captured reruns are optional and only needed if request-shape artifacts are explicitly required.

## V2 Direct-JSON Pilot Snapshot

Experiment:

```text
runs/2026-05-11-claude-mimo-direct-json-v2-pilot/
```

Task:

```text
docs-token-accounting
```

Imported measured rows:

```text
baseline: 6
cache-friendly: 6
validation: 12/12 passed
```

Per-slice all-runs result:

| Slice | Baseline uncached | Cache-friendly uncached | Ratio | Baseline success | Cache-friendly success |
| --- | ---: | ---: | ---: | ---: | ---: |
| `control-steady` | 8,162 | 7,305 | 0.895x | 3/3 | 3/3 |
| `dynamic-drift` | 6,470 | 13,930 | 2.153x | 3/3 | 3/3 |
| aggregate | 14,632 | 21,235 | 1.451x | 6/6 | 6/6 |

Conclusion:

- This pilot does not support the primary savings claim.
- `control-steady` improved slightly, but `dynamic-drift` regressed because one cache-friendly measured run took 14 turns and dominated the totals.
- The result is useful publishable engineering evidence for the repo, but not a paper-facing savings result.
- Do not claim that moving dynamic state later reduced paid uncached input on this V2 pilot.
