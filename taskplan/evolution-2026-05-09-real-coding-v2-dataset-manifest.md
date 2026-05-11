# Evolution: 2026-05-09 Real Coding V2 Dataset Manifest

## Plan Used

```text
taskplan/roadmap.md
taskplan/subtasks/09-real-coding-v2-suite.md
```

## Bounded Target

Move the V2 real-coding suite from a scaffold plus prose prompts toward a small
dataset that an automated runner can consume.

## Completed Work

- Added a tracked machine-readable manifest:

```text
docs/task-suites/real-coding-ablation-v2.manifest.json
```

- Recorded the fixed baseline and cache-friendly conditions.
- Recorded the `control-steady` and `dynamic-drift` slices.
- Recorded the required per-run JSONL fields.
- Recorded all eight task prompts, including the four-turn eval-polish task.
- Recorded validation commands and expected file scopes per task.
- Updated the V2 suite document and roadmap to point at the manifest.

## Validation

The manifest was parsed as JSON and inspected for the expected eight task IDs.
The Rust test suite still passes.

## Deferrals

The next roadmap item is execution, not more suite definition:

```text
Run standardized paired A/B experiments.
```

## Next Handoff

Start with `taskplan/subtasks/04-pilot-run.md`. Use the manifest as the task
source, choose 1-2 tasks, and produce a first paired warm-up/measured run layout
under `runs/<date>-<experiment-name>/`.
