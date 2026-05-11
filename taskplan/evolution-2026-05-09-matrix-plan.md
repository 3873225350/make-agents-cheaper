# Evolution: 2026-05-09 Matrix Plan

## Plan Used

```text
taskplan/roadmap.md
taskplan/subtasks/05-scale-evaluation.md
docs/task-suites/real-coding-ablation-v2.manifest.json
```

## Bounded Target

Move scale evaluation from a prose plan to a manifest-driven command plan.

## Completed Work

- Added `make-agents-cheaper matrix-plan`.
- The command reads the V2 manifest and prints:
  - selected task count;
  - condition count;
  - slice count;
  - repeats per task/condition/slice;
  - expected warm-up calls;
  - expected measured calls;
  - expected validation logs;
  - one `pilot-plan` command for every task/slice pair.
- Supports `--tasks task-a,task-b` for subset runs before the full matrix.
- Updated README, evaluation protocol, roadmap, and Subtask 05.

## Validation

Validation passed:

```bash
cargo fmt --check
cargo test
git diff --check
cargo run --quiet -- matrix-plan \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --experiment-dir runs/2026-05-09-claude-mimo-real-coding-v2-full \
  --repeats 3
```

## Deferrals

The matrix is ready to generate run plans, but live execution still depends on
actual traced Claude Code + MiMo runs.

## Next Handoff

After the pilot is clean, run `matrix-plan`, execute the generated task/slice
plans, import traces with `trace-import`, then run `eval` and `task-report`.
