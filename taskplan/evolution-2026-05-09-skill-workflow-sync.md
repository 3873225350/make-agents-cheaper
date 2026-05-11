# evolution-2026-05-09-skill-workflow-sync.md

## Loop Type

- type: execution

## Plan

- path: `taskplan/roadmap.md`
- milestone: Phase 7 skill packaging, plus Phase 8 usability-ablation preparation
- bounded target: synchronize reusable skill instructions with the executable main-repo ablation protocol

## Review Window

- reviewed loops: latest analysis-report note, roadmap Phase 7 and Phase 8, root `SKILL.md`, Claude adapter `SKILL.md`
- status: audit/eval CLI now has manifest planning, trace import, matrix planning, and analysis report generation; live Claude/MiMo V2 runs remain deferred

## Completed

- Updated root `SKILL.md` with a standardized A/B workflow based on `init-experiment`, `pilot-plan`, `matrix-plan`, `trace-import`, `eval`, `task-report`, and `analysis-report`.
- Updated `cheaper-skill-for-claude/SKILL.md` so the Claude adapter points back to the main repo's executable protocol.
- Added `cheaper-skill-for-claude/references/standardized-ablation-workflow.md` with command forms, trace import, analysis commands, claim gates, and a skill-usability rubric.
- Updated the skill packaging repo README to clarify the standardized workflow and artifact/evidence split.
- Updated Subtasks 07 and 08 plus the roadmap status.

## Failed or Deferred

- Did not run an independent skill-guided agent ablation yet.
- Did not execute live Claude Code/MiMo V2 benchmark runs.

## Decisions

- Keep the skill layer as a runbook and reproducibility artifact.
- Keep `make-agents-cheaper` JSONL, trace-derived artifacts, validation logs, and reports as the main evidence source.
- Treat the new rubric as preparation for Phase 8, not as a completed usability result.

## Analysis Checks

- regression risk: low; changes are documentation/runbook updates
- drift risk: medium; the Claude CLI flag and trace shapes should be rechecked before live runs
- version safety: no destructive git operations; existing dirty files left untouched
- plan adjustment: Phase 7 moves to workflow-synchronized and Phase 8 moves to rubric-ready

## Next Handoff

```text
Continue taskplan/roadmap.md. Next smallest useful slice: run a dry skill-usability ablation by prompting an agent or local checklist against cheaper-skill-for-claude, then record whether it reconstructs the baseline/candidate commands, warm-up separation, token fields, and role separation without claiming cache-hit proof.
```
