# Evolution: 2026-05-09 Pilot Scaffold Ready

## Plan Used

```text
taskplan/roadmap.md
taskplan/subtasks/04-pilot-run.md
docs/trace-capture-pipeline.md
```

## Bounded Target

Prepare the standardized pilot run scaffold without pretending that live Claude
Code + MiMo measurements have already been executed.

## Completed Work

- Updated `init-experiment` so new experiment directories include:

```text
raw/claude-trace/
traces/
requests/
layers/
tools/
validation/
notes/
baseline.jsonl
cache-friendly.jsonl
```

- Updated the experiment README template so the minimum JSONL row includes:

```text
condition
slice
repeat_id
phase
cache_creation_input_tokens
uncached_input_tokens
cache_accounting_observable
validation_path
anomaly
```

- Marked Subtask 04 as `ready-to-run scaffold`, not complete.

## Validation

`cargo test` should exercise the updated scaffold creation test.

## Deferrals

The actual pilot still requires live Claude Code + MiMo calls. It should only be
marked complete after raw traces, validation logs, normalized JSONL rows, and
`make-agents-cheaper eval` output exist under a real `runs/<experiment>/`
directory.

## Next Handoff

Use the V2 manifest to choose `docs-token-accounting` and one implementation
task, then run the baseline/cache-friendly warm-up and measured calls with
`claude-trace` enabled.
