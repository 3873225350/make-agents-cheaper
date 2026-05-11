# Subtask 08: Skill Usability Ablation

## Goal

Evaluate the skill layer as a reproducibility and usability artifact, not as the main cache-hit evidence.

## Key Distinction

Main experiments use:

```text
make-agents-cheaper audit/eval
```

The skill ablation tests:

```text
whether cheaper-skill-for-claude can guide an agent to run the same protocol correctly
```

The skill itself does not make the model cheaper. It helps another agent apply the harness-level method consistently.

## Questions

- Can an agent using the skill generate the correct Claude Code baseline command?
- Can it generate the correct cache-friendly candidate command with `--exclude-dynamic-system-prompt-sections`?
- Does it preserve warm-up vs measured separation?
- Does it record token usage fields correctly?
- Does it keep Codex, Claude Code, MiMo, audit/eval, and skill roles separate?
- Does it avoid claiming universal savings from one cold run?

## Suggested Test Prompt

```text
Use cheaper-skill-for-claude to design a paired A/B experiment for Claude Code routed to mimo-v2.5-pro. Include warm-up, measured calls, dynamic drift, token usage fields, and validation checks.
```

## Metrics

- protocol completeness;
- role separation correctness;
- command correctness;
- token accounting correctness;
- overclaim avoidance;
- number of manual corrections needed.

## Current Rubric

The executable rubric is now documented in:

```text
../make-agents-cheaper-skill/cheaper-skill-for-claude/references/standardized-ablation-workflow.md
```

Use it to score whether a skill-guided agent reconstructs:

- baseline and cache-friendly command forms;
- warm-up vs measured separation;
- trace import and analysis commands;
- token accounting fields;
- role separation between Codex, Claude Code, MiMo, audit/eval, and the skill layer.

## Local Dry Run

A local dry-run check is recorded at:

```text
taskplan/skill-usability-dry-run-2026-05-09.md
```

It verifies that the skill workflow and `pilot-plan` output agree on baseline
versus cache-friendly command forms, warm-up versus measured separation,
dynamic drift, trace import, validation logs, and analysis commands. This is not
an independent skill-guided agent result.

## Acceptance Criteria

- The skill-guided agent reconstructs the run protocol without mixing up Codex as the studied harness.
- The generated commands are executable after filling in prompts and paths.
- Token fields match the documented accounting rules.
- The result is reported as artifact usability, not cache-hit proof.
- [x] A local dry-run checklist verifies the runbook against `pilot-plan`.
- [ ] An independent skill-guided agent reconstructs the protocol and is scored for manual corrections.

## Status

```text
local dry-run checked; independent skill-guided run still pending
```
