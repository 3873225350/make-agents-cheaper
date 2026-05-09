# Evolution: 2026-05-09 Checkpoint

## What Changed

The project moved from exploratory cache-hit notes into a checkpointed two-repository structure:

```text
make-agents-cheaper:
  Rust audit/eval instrumentation and paper-facing experiment plan

make-agents-cheaper-skill:
  reusable skill packaging, starting with cheaper-skill-for-claude
```

## Boundary Decision

Main experimental evidence comes from `make-agents-cheaper` audit/eval logs. The skill layer is an artifact/reuse layer and will be tested later as a usability ablation.

The current object of study remains:

```text
development assistant: Codex
studied harness: Claude Code
backend route/model: MiMo, e.g. mimo-v2.5-pro
measurement layer: make-agents-cheaper audit/eval
```

## Validation

`cargo test` passed before checkpointing the main repository.

## Local Reference

`references/keep-codex-fast/` is a nested external git repository used as local reference material. It is ignored and not committed as source.
