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
fixed V2 dynamic-drift diagnostic supports the narrow prefix-cache claim
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

## V2 Prefix Diagnostic Snapshot

Experiment:

```text
runs/2026-05-12-claude-mimo-v2-diagnostic-r3/
```

Task:

```text
docs-token-accounting
```

Imported measured rows:

```text
baseline: 3
cache-friendly: 3
validation: 6/6 passed
```

Fixed dynamic-drift all-runs result:

| Slice | Baseline uncached | Cache-friendly uncached | Ratio | Baseline success | Cache-friendly success |
| --- | ---: | ---: | ---: | ---: | ---: |
| `dynamic-drift` | 30,082 | 7,817 | 0.260x | 3/3 | 3/3 |

Conclusion:

- This fixed diagnostic supports the narrow prefix-cache claim.
- The old 2026-05-11 V2 pilot remains useful regression-diagnosis evidence, not the headline result.
- Prefix caching reduces paid uncached input; it does not by itself optimize tool-output volume or guarantee fewer agent turns.
