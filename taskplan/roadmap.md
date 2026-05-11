# Long-Term Roadmap

This roadmap turns the current cache-hit mechanism experiments into a larger, more realistic evaluation program.

Current status:

```text
micro-exact mechanism tests: done
paired dynamic-drift ablation: done
multi-task micro suite v1: done
real coding fixture: done
real coding-task suite v1: done
Claude skill adapter scaffold: done
checkpoint before next experiment: done
larger real coding-task suite v2: dataset manifest ready
trace capture pipeline: optional only; not required for current roadmap
direct Claude JSON capture: current default path with claude-json-import
claude-trace recovery workflow: repurposed as .claude/skills/claude-trace-recovery/ direct-json workflow
real coding-task pilot: direct-json V2 pilot complete; mixed/negative for savings claim
full task matrix: ready-to-run matrix plan
full task matrix scaffold: generated under runs/2026-05-11-claude-mimo-real-coding-v2-full/
first full-matrix live slice: attempted; blocked by noninteractive write permission and low budget cap
permission diagnostic: bypassPermissions works on ignored fixture; generated pilot plans refreshed
analysis/paper tables: report generator includes per-slice all-runs table
skill packaging: standardized workflow synchronized
skill usability ablation: local dry-run checked
```

## Core Claim To Test

The project should not claim that cache-friendly prompt assembly is always cheaper.

The claim to test is narrower and stronger:

```text
When dynamic harness state would otherwise disturb the early prompt prefix,
moving that dynamic state later preserves a longer reusable prefix,
increases prompt-cache hit rate,
and can reduce observed input cost without reducing task success.
```

## Roadmap Overview

| Phase | Task | File | Status |
| --- | --- | --- | --- |
| 0 | Checkpoint current repos before more experiments | `subtasks/00-checkpoint.md` | [x] done |
| 1 | Preserve the micro benchmark as a sanity check | `subtasks/01-micro-benchmark.md` | [x] done |
| 2 | Build a real coding-task fixture repo | `subtasks/02-fixture-repo.md` | [x] done |
| 3 | Define realistic task-suite v1 | `subtasks/03-real-coding-task-suite.md` | [x] done |
| 3a | Define larger real-coding task-suite v2 | `subtasks/09-real-coding-v2-suite.md` | [x] defined |
| 3b | Add raw trace capture pipeline | `subtasks/10-trace-capture-pipeline.md` | [~] direct-json default implemented; trace optional |
| 4 | Run paired A/B pilot on 1-2 tasks | `subtasks/04-pilot-run.md` | [x] direct-json V2 pilot complete; mixed/negative savings result |
| 5 | Scale to a full task matrix | `subtasks/05-scale-evaluation.md` | [~] scaffold generated; pilot says refine before scaling |
| 6 | Add analysis and paper-facing tables | `subtasks/06-analysis-and-paper.md` | [~] report generator ready with per-slice table |
| 7 | Package reusable skill/audit workflow | `subtasks/07-skill-packaging.md` | [~] workflow synchronized |
| 8 | Test skill usability as an auxiliary ablation | `subtasks/08-skill-usability-ablation.md` | [~] local dry-run checked |

## Non-Negotiable Evaluation Rules

- Separate warm-up from measured calls.
- Record raw traces, extracted JSONL, validation result, and anomaly notes.
- Record per-task token usage: input, cache read, cache creation, uncached input, and output tokens.
- Keep `control-steady` and `dynamic-drift` slices separate.
- Do not count a cheaper run as a win if task success regresses.
- Do not silently replace failed runs with retries.
- Use `--no-session-persistence` for standalone benchmark runs unless the experiment intentionally studies persisted Claude Code behavior.
- Keep drift probes temporary and never commit them.

## Near-Term Execution Order

1. [x] Checkpoint the current work.
   - Commit or clearly stage boundaries for `make-agents-cheaper`.
   - Commit the newly created `make-agents-cheaper-skill` repo.
   - Record that existing dirty files not related to the next experiment were left untouched.
2. [~] Build a larger real-coding evaluation suite.
   - Keep v1 as the smoke/fixture suite.
   - [x] Define a v2 suite with larger tasks, more realistic file edits, failing-test repair, multi-turn planning, and dynamic drift.
   - [x] Implement the ignored local fixture scaffold under `runs/fixtures/real-coding-v2/`.
   - [x] Make the suite strong enough to act like a small dataset, not just a demo.
     - Added `docs/task-suites/real-coding-ablation-v2.manifest.json` as the runner-facing task manifest.
3. [~] Run standardized paired A/B experiments.
   - [x] Prepare the ready-to-run pilot scaffold and manifest inputs.
   - [x] Generate task-specific pilot command plans from the manifest.
   - [x] Baseline warm-up for one live direct-JSON pilot.
   - [x] Cache-friendly warm-up for one live direct-JSON pilot.
   - [x] Baseline measured for one live direct-JSON pilot.
   - [x] Cache-friendly measured for one live direct-JSON pilot.
   - [x] Control-steady and dynamic-drift slices for one bounded V2 direct-json pilot.
     - Experiment: `runs/2026-05-11-claude-mimo-direct-json-v2-pilot/`.
     - Imported measured rows: 6 baseline and 6 cache-friendly.
     - All imported measured rows passed validation.
     - `control-steady`: candidate uncached input 7,305 vs baseline 8,162 (0.895x).
     - `dynamic-drift`: candidate uncached input 13,930 vs baseline 6,470 (2.153x).
     - Aggregate: candidate uncached input 21,235 vs baseline 14,632 (1.451x).
     - Conclusion: this pilot does not support the primary savings claim; treat it as mixed/negative evidence and diagnose before scaling.
   - [x] Per-run token usage and validation logs for the live direct-JSON pilot.
   - [x] Implement `trace-import` for normalizing raw trace logs into audit/eval rows.
   - [x] Generate full task-matrix command plans from the manifest.
   - [~] First full-matrix live direct-json slice.
     - `docs-token-accounting/control-steady/r1` attempted.
     - Failed validation because Claude requested write approval in noninteractive mode.
     - Warm-up calls also hit `--max-budget-usd 0.2`.
     - Diagnostic probe with `--permission-mode bypassPermissions` passed validation; regenerated plans now include that flag.
   - [x] Use direct Claude Code JSON as the current run-capture path.
     - Do not install or require `claude-trace` for the V2 matrix.
     - Direct Claude Code JSON is normalized with `claude-json-import`; this supports usage/cost rows but not request-shape evidence.
     - Recovery workflow is packaged at `.claude/skills/claude-trace-recovery/`.
4. [~] Update paper-facing analysis.
   - [x] Add JSONL-driven `analysis-report` generator for aggregate, per-slice, and per-task Markdown tables.
   - [x] Clearly state that Codex is the development assistant.
   - [x] State that the current studied harness is Claude Code.
   - [x] State that the current backend route is MiMo, such as `mimo-v2.5-pro`.
   - [x] State that evidence comes from audit/eval logs, not from the skill layer.
   - [ ] Fill final LaTeX result tables after live V2 pilot/full-matrix runs exist.
5. [ ] Test the skill layer as auxiliary evidence.
   - [x] Synchronize the Claude skill adapter with the executable main-repo protocol.
   - [x] Add a scoring rubric for skill usability/reproducibility.
   - [x] Run a local dry-run checklist against `pilot-plan` and the Claude skill runbook.
   - Use `cheaper-skill-for-claude` to have an agent reconstruct the run protocol.
   - Check whether it correctly keeps Codex, Claude Code, MiMo, audit/eval, and skill roles separate.
   - Treat this as a usability/reproducibility ablation, not as the main cache-hit result.

## Product Roles

Do not mix the products in reports:

```text
make-agents-cheaper:
  Rust audit/eval instrumentation; main source of experimental evidence

make-agents-cheaper-skill:
  reusable skill/runbook packaging; helps agents apply the method

cheaper-skill-for-claude:
  first Claude Code adapter inside the skill repository

cheapcode:
  possible future native cheaper-agent harness
```

Current experiment object:

```text
development assistant: Codex
studied harness: Claude Code
backend route/model: MiMo, e.g. mimo-v2.5-pro
measurement layer: make-agents-cheaper audit/eval
reuse layer: cheaper-skill-for-claude
```

## Auxiliary Ablations

The main A/B experiments should use the audit/eval layer. Additional ablations can test whether the workflow transfers into reusable skill form:

- Skill reconstruction: can an agent using `cheaper-skill-for-claude` generate the correct baseline/candidate commands?
- Role separation: does the agent avoid saying the experiment studies Codex when the harness is Claude Code?
- Logging discipline: does the agent remember warm-up separation and token usage fields?
- Overclaim control: does the agent avoid claiming savings from a cold single run?
- Portability check: which parts remain Claude-specific and which parts could later become `cheaper-skill-for-codex`?

## Evidence Levels

| Evidence | What It Supports | What It Does Not Support |
| --- | --- | --- |
| micro-exact | mechanism visibility and cache accounting | real coding productivity |
| paired drift | dynamic-state prefix drift mechanism | broad generalization |
| multi-task micro | prompt-family robustness | tool-use robustness |
| real coding fixture | coding-task usefulness | all repositories |
| long-session test | session stability | all models/providers |

## Logging Location

Experiment logs stay under:

```text
runs/<date>-<experiment-name>/
```

Planning files stay under:

```text
taskplan/
```

Task-suite definitions stay under:

```text
docs/task-suites/
```
