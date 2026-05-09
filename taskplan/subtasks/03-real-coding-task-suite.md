# Subtask 03: Define Real Coding Task Suite

## Goal

Create the first realistic coding-task dataset.

Recommended file:

```text
docs/task-suites/real-coding-ablation-v1.md
```

## Task Families

| Task ID | Family | Example | Validation |
| --- | --- | --- | --- |
| `docs-edit` | documentation edit | Add a cache-hit warning section | grep + diff review |
| `rust-cli-flag` | small CLI feature | Add `--json-summary` | `cargo test --locked` |
| `bug-fix` | failing test repair | Fix parser edge case | `cargo test --locked` |
| `schema-report` | benchmark schema | Add field extraction to summary | tests + sample command |
| `multi-turn-refine` | iterative improvement | improve help text over 3 turns | tests each turn |

## Work Items

- Write task prompts.
- Define expected file changes.
- Define validation command per task.
- Define reset command per task.
- Specify whether the task is single-turn or multi-turn.

## Acceptance Criteria

- Every task has a prompt, reset command, validation command, and success rubric.
- The suite distinguishes `control-steady` from `dynamic-drift`.
- The suite avoids tasks that require network access.

## Status

```text
done
```

## Completion Evidence

- Task suite written at `docs/task-suites/real-coding-ablation-v1.md`.
- Five task families are specified with prompt, reset command, validation command, and success rubric.
- The suite separates `control-steady` and `dynamic-drift`.
- The suite records token usage fields for Claude Code JSON and direct MiMo OpenAI-compatible usage.
