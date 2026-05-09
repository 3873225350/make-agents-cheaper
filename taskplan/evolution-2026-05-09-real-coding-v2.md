# Evolution: 2026-05-09 Real Coding V2

## What Changed

The larger real-coding task suite was defined and a local ignored fixture scaffold was created.

Tracked definition:

```text
docs/task-suites/real-coding-ablation-v2.md
```

Ignored local fixture:

```text
runs/fixtures/real-coding-v2/
```

## Why It Matters

V1 is a smoke suite. V2 adds a broader task mix:

- documentation-only token accounting;
- JSONL reporting;
- parser bug fix;
- schema update;
- audit warning rule;
- experiment-log aggregation;
- failing-test repair;
- multi-turn evaluation polish.

This makes the next pilot closer to a small dataset rather than a single demo.

## Validation

Base fixture validation passed:

```bash
bash task-reset.sh base
bash task-validate.sh base
```

Task-specific validation commands are expected to fail before the agent performs the requested task.
