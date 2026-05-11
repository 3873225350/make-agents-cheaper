# Evolution: 2026-05-09 Pilot Command Plan

## Plan Used

```text
taskplan/roadmap.md
taskplan/subtasks/04-pilot-run.md
docs/evaluation-protocol.md
```

## Bounded Target

Advance the pilot from a directory scaffold to a task-specific command plan that
can be executed once live Claude Code + MiMo tracing is available.

## Completed Work

- Added a CLI command:

```bash
make-agents-cheaper pilot-plan \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --task docs-token-accounting \
  --experiment-dir runs/<date>-claude-mimo-real-coding-v2-pilot \
  --slice dynamic-drift \
  --repeats 1
```

- The command reads the V2 manifest and prints:
  - experiment setup;
  - exact prompt-file creation;
  - baseline and cache-friendly warm-up calls;
  - baseline and cache-friendly measured calls;
  - dynamic-drift probes when selected;
  - validation log paths;
  - final `eval` and `task-report` commands.
- Updated experiment scaffolds to include `prompts/` and `drift/`.
- Updated README, evaluation protocol, roadmap, and Subtask 04.

## Validation

The command was implemented as a non-executing planner. It does not trigger
paid model calls.

Validation passed:

```bash
cargo fmt --check
cargo test
git diff --check
cargo run --quiet -- pilot-plan \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --task docs-token-accounting \
  --experiment-dir runs/2026-05-09-claude-mimo-real-coding-v2-pilot \
  --slice dynamic-drift \
  --repeats 1
```

## Deferrals

`claude` is installed locally, but `claude-trace` is not currently on `PATH`.
The actual pilot still requires live traced runs before Subtask 04 can be marked
complete.

## Next Handoff

Install or expose `claude-trace`, then run the printed plan for
`docs-token-accounting` and one implementation task. Normalize usage into
`baseline.jsonl` and `cache-friendly.jsonl`, then run `eval` and `task-report`.
