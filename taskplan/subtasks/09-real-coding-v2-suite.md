# Subtask 09: Define Real-Coding V2 Suite

## Goal

Create a larger task suite that behaves more like a small dataset than a smoke test.

## Output

Primary task-suite definition:

```text
docs/task-suites/real-coding-ablation-v2.md
```

## Design Requirements

- Include documentation-only, CLI feature, parser bug fix, schema update, audit rule, JSONL summary, failing-test repair, and multi-turn tasks.
- Include both `control-steady` and `dynamic-drift` slices.
- Keep the studied harness explicit: Claude Code routed to MiMo.
- Keep the measurement layer explicit: `make-agents-cheaper` audit/eval.
- Treat skill-guided runs as auxiliary usability ablation, not primary evidence.

## Status

```text
dataset manifest added
```

## Local Fixture

The ignored local fixture was created at:

```text
runs/fixtures/real-coding-v2/
```

Base validation passed:

```bash
cd runs/fixtures/real-coding-v2
bash task-reset.sh base
bash task-validate.sh base
```

The task-specific validations are intentionally stricter than the baseline. They are expected to fail until an agent completes the corresponding task.

## Dataset Manifest

The tracked manifest fixes task prompts and runner metadata:

```text
docs/task-suites/real-coding-ablation-v2.manifest.json
```

It records:

- baseline and cache-friendly conditions;
- control-steady and dynamic-drift slices;
- required per-run JSONL fields;
- exact prompt turns for all eight tasks;
- validation command and expected files for every task.
