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
defined
```
