# evolution-2026-05-11-v2-full-matrix-scaffold.md

## Loop Type

- type: execution
- skill: codex-loop

## Plan

- path: `taskplan/roadmap.md`
- milestone: run standardized paired A/B experiments
- bounded target: check running codex-loop status, prepare the V2 full matrix execution scaffold, and make direct Claude JSON normalization the current run-capture path.

## Review Window

- reviewed loops:
  - `taskplan/roadmap.md`
  - `taskplan/evolution-2026-05-11-mainline-v2-full-matrix.md`
  - `taskplan/evolution-2026-05-09-live-claude-mimo-pilot.md`
  - `taskplan/subtasks/05-scale-evaluation.md`
  - `taskplan/subtasks/10-trace-capture-pipeline.md`
- status:
  - no `codex-loop` daemon was running;
  - V2 matrix was ready-to-run but only as a plan;
  - `claude` was installed;
  - `claude-trace` was missing, but no longer needs to block the roadmap.

## Completed

- Checked `codex-loop` automation status:

```text
daemon_running: false
```

- Confirmed Claude Code is installed:

```text
claude 2.1.123
```

- Confirmed `claude-trace` is not available in PATH and later decided to skip it for the current roadmap.
- Initialized the full V2 experiment scaffold:

```text
runs/2026-05-11-claude-mimo-real-coding-v2-full/
```

- Generated the full matrix summary:

```text
runs/2026-05-11-claude-mimo-real-coding-v2-full/notes/full-matrix-plan.txt
```

- Generated 16 task/slice pilot plans under:

```text
runs/2026-05-11-claude-mimo-real-coding-v2-full/notes/
```

- Matrix size:

```text
tasks: 8
conditions: 2
slices: 2
repeats: 3
warm-up calls: 96
measured calls: 96
validation logs expected: 96
```

- Added a first-class `claude-json-import` command so direct Claude Code `--output-format json` results can be normalized into eval JSONL without manual copying.
- Documented the direct JSON fallback in README, evaluation protocol, trace-capture pipeline notes, roadmap, and Subtask 10.
- Attempted the first live direct-json full-matrix slice:

```text
task: docs-token-accounting
slice: control-steady
repeat: 1
```

- Preserved raw direct Claude JSON outputs under:

```text
runs/2026-05-11-claude-mimo-real-coding-v2-full/raw/claude-json/
```

- Normalized the two measured rows into:

```text
runs/2026-05-11-claude-mimo-real-coding-v2-full/baseline.jsonl
runs/2026-05-11-claude-mimo-real-coding-v2-full/cache-friendly.jsonl
```

- Generated:

```text
runs/2026-05-11-claude-mimo-real-coding-v2-full/analysis-report.md
```

- Ran a single permission diagnostic probe with:

```text
--permission-mode bypassPermissions
--max-budget-usd 0.6
```

- The probe edited the ignored fixture successfully and passed validation.
- Updated `pilot-plan` output so generated Claude commands include `--permission-mode bypassPermissions` for the ignored fixture.
- Regenerated the full matrix notes so the printed task/slice plans include the fixed permission mode.
- Packaged the recurring `claude-trace` recovery workflow as a project-local skill:

```text
.claude/skills/claude-trace-recovery/
```
- Repurposed that project-local skill so its default path is direct Claude JSON capture, not trace installation/debugging.
- Updated `pilot-plan` so generated commands use direct Claude JSON capture and `claude-json-import`, not `claude-trace`.

## Failed or Deferred

- Did not start a background `codex-loop` daemon; this was a manual codex-loop iteration in the current thread.
- Did not run the 96 measured live calls. Full-matrix live execution may incur real model cost and should proceed one bounded task/slice at a time.
- Did not produce raw request/layer/tool artifacts because trace capture is now optional and was skipped.
- The first `control-steady` live slice is not usable as a success result:
  - both warm-up calls hit `error_max_budget_usd` with `--max-budget-usd 0.2`;
  - both measured calls returned Claude Code success, but the model asked for write approval instead of editing `README.md`;
  - both measured validations failed and were recorded as `task_success=false`.
- The permission probe is diagnostic only. It confirms the command-shape fix, but it does not replace the failed measured rows.

## Decisions

- Keep the mainline focus on V2 `control-steady` and `dynamic-drift`.
- Do not add hook-stack, Read dedup, RTK, Caveman, StatusLine, Qdrant, or native `cheapcode` work in this phase.
- Use `claude-json-import` as the direct-json fallback when trace capture is unavailable.
- Treat direct-json rows as valid usage/cost/validation evidence but weaker request-shape evidence.
- Before continuing live matrix execution, use the regenerated plans with explicit noninteractive permission mode and avoid the too-low per-call budget cap.
- Do not install or require `claude-trace` for the current V2 matrix.
- Use `.claude/skills/claude-trace-recovery/` to keep future agents on the direct-json workflow unless request-shape artifacts are explicitly requested.

## Validation

Commands run:

```bash
cargo run --quiet -- matrix-plan \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --experiment-dir runs/2026-05-11-claude-mimo-real-coding-v2-full \
  --repeats 3

cargo run --quiet -- init-experiment \
  --dir runs/2026-05-11-claude-mimo-real-coding-v2-full

cargo fmt --check
cargo test
cargo run --quiet -- --help
cargo run --quiet -- claude-json-import \
  --input runs/2026-05-09-live-claude-mimo-pilot/raw/claude-json/docs-token-accounting-dynamic-drift-cache-friendly-r1-measured.json \
  --run-id docs-token-accounting-dynamic-drift-cache-friendly-r1-measured \
  --task-id docs-token-accounting \
  --condition cache-friendly \
  --slice dynamic-drift \
  --repeat-id 1 \
  --phase measured \
  --validation-path runs/2026-05-09-live-claude-mimo-pilot/validation/docs-token-accounting-dynamic-drift-cache-friendly-r1-measured.txt \
  --validation-passed true

cargo run --quiet -- claude-json-import \
  --input runs/2026-05-11-claude-mimo-real-coding-v2-full/raw/claude-json/docs-token-accounting-control-steady-baseline-r1-measured.json \
  --run-id docs-token-accounting-control-steady-baseline-r1-measured \
  --task-id docs-token-accounting \
  --condition baseline \
  --slice control-steady \
  --repeat-id 1 \
  --phase measured \
  --output runs/2026-05-11-claude-mimo-real-coding-v2-full/baseline.jsonl \
  --validation-path runs/2026-05-11-claude-mimo-real-coding-v2-full/validation/docs-token-accounting-control-steady-baseline-r1-measured.txt \
  --validation-passed false \
  --task-success false

cargo run --quiet -- eval \
  --baseline runs/2026-05-11-claude-mimo-real-coding-v2-full/baseline.jsonl \
  --candidate runs/2026-05-11-claude-mimo-real-coding-v2-full/cache-friendly.jsonl

cargo run --quiet -- pilot-plan \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --task docs-token-accounting \
  --experiment-dir runs/demo \
  --slice control-steady \
  --repeats 1

python /home/clashuser/.codex/skills/.system/skill-creator/scripts/quick_validate.py \
  .claude/skills/claude-trace-recovery
```

Result:

```text
cargo fmt --check: passed
cargo test: passed, 12 tests
claude-json-import smoke test: printed normalized eval row
first live control-steady r1: recorded as failed validation in JSONL
permission probe with bypassPermissions: validation passed
pilot-plan smoke: printed --permission-mode bypassPermissions
claude-trace-recovery skill: valid
```

Failure snapshot:

```text
baseline validation: 0/1
cache-friendly validation: 0/1
baseline uncached input: 2,480
cache-friendly uncached input: 5,198
baseline observed cost: $0.139987
cache-friendly observed cost: $0.127449
interpretation: not a cache-friendly win because task success failed
```

## Analysis Checks

- regression risk: low; code path is additive and covered by tests.
- drift risk update: generated plans now use direct JSON by default; request-shape evidence remains unavailable unless optional traces are captured later.
- live-run risk: high until noninteractive permission mode is fixed; otherwise Claude may ask for write approval instead of editing files.
- version safety: generated experiment files are under ignored `runs/`.
- plan adjustment: full matrix scaffold now exists; live execution should proceed with direct-json normalization without waiting for `claude-trace`.

## Next Handoff

```text
Continue taskplan/roadmap.md using codex-loop manual mode.

Next bounded execution slice:
1. Do not scale the matrix yet.
2. Use the regenerated pilot plans with `--permission-mode bypassPermissions` for ignored fixture runs.
3. Raise or remove `--max-budget-usd 0.2`; it was too low for warm-up.
4. Rerun only docs-token-accounting/control-steady after resetting the fixture.
5. Preserve the failed rows already recorded; do not silently delete them.
6. If the rerun passes validation, record it as a separate diagnostic retry or repeat 2, not as a replacement for the failed run.

Do not add hook/read-dedup/compression experiments.
```
