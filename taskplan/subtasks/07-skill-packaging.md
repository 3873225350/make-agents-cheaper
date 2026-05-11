# Subtask 07: Package Reusable Skill Workflow

## Goal

Turn the runbook and evaluation flow into a reusable skill/audit workflow for existing agents.

This remains different from `cheapcode`:

```text
make-agents-cheaper:
  skill / audit / eval layer for existing agents

cheapcode:
  future native agent framework
```

## Work Items

- [x] Update `SKILL.md` with the standardized ablation workflow.
- [x] Add command examples for Claude Code and Codex.
- [x] Add checklist for prompt-cache-friendly config.
- [x] Add warning that skills are runbooks and audit/eval logs are the evidence source.
- [x] Add a Claude adapter reference that links `init-experiment`, `pilot-plan`, `matrix-plan`, `trace-import`, `eval`, `task-report`, and `analysis-report`.

## Acceptance Criteria

- [x] A user can run the benchmark from the skill instructions.
- [x] The workflow does not require patching agent source.
- [x] The workflow records logs and prevents overclaiming.
- [ ] The workflow has been tested by an independent skill-guided agent.

## Updated Files

```text
SKILL.md
../make-agents-cheaper-skill/README.md
../make-agents-cheaper-skill/cheaper-skill-for-claude/SKILL.md
../make-agents-cheaper-skill/cheaper-skill-for-claude/references/standardized-ablation-workflow.md
```

## Status

```text
workflow synchronized; usability ablation still pending
```
