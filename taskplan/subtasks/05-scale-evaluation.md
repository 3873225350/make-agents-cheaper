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
- The full matrix can be generated from the manifest without hand-copying task prompts.
- Expected warm-up, measured, and validation counts are printed before running.

## Status

```text
full-matrix scaffold generated; fixed V2 dynamic-drift diagnostic is positive for the narrow prefix-cache claim
```

## Live Slice Attempt

Experiment:

```text
runs/2026-05-11-claude-mimo-real-coding-v2-full/
```

Attempted:

```text
task: docs-token-accounting
slice: control-steady
repeat: 1
capture: direct Claude Code JSON
```

Outcome:

- warm-up calls hit `error_max_budget_usd` with `--max-budget-usd 0.2`;
- measured calls returned Claude Code success but asked for write approval instead of editing files;
- both validations failed;
- measured rows were preserved in `baseline.jsonl` and `cache-friendly.jsonl`.

Next scale attempt should first fix the noninteractive permission mode and budget cap, then rerun one bounded task/slice before scaling.

Permission diagnostic:

- `--permission-mode bypassPermissions` succeeded on the ignored fixture.
- `pilot-plan` now prints that flag in generated Claude commands.
- The generated plan files under `runs/2026-05-11-claude-mimo-real-coding-v2-full/notes/` were refreshed with the permission fix.

## Bounded Direct-JSON Pilot Before Scaling

Experiment:

```text
runs/2026-05-11-claude-mimo-direct-json-v2-pilot/
```

Scope:

```text
task: docs-token-accounting
slices: control-steady, dynamic-drift
repeats: 3 per condition/slice
capture: direct Claude Code JSON
```

Outcome:

- 6 baseline and 6 cache-friendly measured rows were imported.
- All imported measured rows passed validation.
- `control-steady` showed a small candidate uncached-input reduction: 7,305 vs 8,162 (0.895x).
- `dynamic-drift` regressed: 13,930 candidate uncached input vs 6,470 baseline (2.153x).
- Aggregate candidate uncached input was higher: 21,235 vs 14,632 (1.451x).

Decision:

- Do not launch the full 96-measured-call matrix until this negative dynamic-drift signal is understood.
- Next bounded pass should diagnose whether the `dynamic-drift` regression comes from task-behavior variance, the `--exclude-dynamic-system-prompt-sections` flag, drift-probe shape, or cache warm-up mismatch.
- Keep the failed prompt-path dynamic attempt in the run log as an anomaly; it was excluded from JSONL because Claude exited before task execution.

## Matrix Planner

Generate the full V2 matrix plan:

```bash
cargo run --quiet -- matrix-plan \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --experiment-dir runs/<date>-claude-mimo-real-coding-v2-full \
  --repeats 3
```

For a smaller subset before scaling:

```bash
cargo run --quiet -- matrix-plan \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --experiment-dir runs/<date>-claude-mimo-real-coding-v2-subset \
  --tasks docs-token-accounting,config-warning-rule \
  --repeats 1
```

The command prints total warm-up calls, measured calls, expected validation logs,
and the `pilot-plan` commands for each task/slice pair.
