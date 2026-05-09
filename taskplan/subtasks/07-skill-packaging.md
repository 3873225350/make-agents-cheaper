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

- Update `SKILL.md` with the standardized ablation workflow.
- Add command examples for Claude Code and Codex.
- Add checklist for prompt-cache-friendly config.
- Add warning about hooks as measurement/guardrails, not primary rewriting.

## Acceptance Criteria

- A user can run the benchmark from the skill instructions.
- The workflow does not require patching agent source.
- The workflow records logs and prevents overclaiming.

## Status

```text
planned
```
