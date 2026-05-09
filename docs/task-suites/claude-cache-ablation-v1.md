# Claude Cache Ablation V1 Task Suite

This suite is a small "dataset" for cache-hit ablation. It is intentionally cheap to run and easy to validate.

Each task asks for a short deterministic answer so the measured difference mostly comes from prompt/harness structure, not generation length.

Observed note from the first run: one `paper-positioning` cache-friendly call ignored the weak exact-reply instruction and produced a longer answer. Future suites should use the stricter output contract from `docs/paired-ablation-runbook.md`.

## Tasks

| Task ID | Family | Expected Reply |
| --- | --- | --- |
| `docs-summary` | documentation reasoning | `dataset-v1-docs-ok` |
| `rust-cli` | code planning | `dataset-v1-rust-ok` |
| `eval-metrics` | evaluation design | `dataset-v1-eval-ok` |
| `paper-positioning` | research writing | `dataset-v1-paper-ok` |
| `agent-routing` | agent harness reasoning | `dataset-v1-agent-ok` |

## Prompts

### docs-summary

```text
You are evaluating a documentation task. Consider a README section about prompt-cache hit rate, stable prefixes, and dynamic harness state. Reply exactly: dataset-v1-docs-ok
```

### rust-cli

```text
You are evaluating a Rust CLI task. Consider adding a command that extracts Claude Code JSON usage into benchmark JSONL without printing secrets. Reply exactly: dataset-v1-rust-ok
```

### eval-metrics

```text
You are evaluating an experiment-metrics task. Consider cache hit rate, uncached input, observed cost, validation pass rate, and task success. Reply exactly: dataset-v1-eval-ok
```

### paper-positioning

```text
You are evaluating a paper-positioning task. Consider model-side efficiency, inference-side KV cache, and harness-level prompt cache hit engineering. Reply exactly: dataset-v1-paper-ok
```

### agent-routing

```text
You are evaluating an agent-routing task. Consider stable provider, stable model, stable tool schema, session continuity, and dynamic state drift. Reply exactly: dataset-v1-agent-ok
```

## Use

For each task:

1. warm baseline for this exact prompt shape;
2. warm cache-friendly for this exact prompt shape;
3. exclude warm-up from main result;
4. create a dynamic drift probe;
5. run measured baseline;
6. run measured cache-friendly;
7. remove drift probes after the experiment.

The first full run of this suite is stored under:

```text
runs/2026-05-09-claude-mimo-task-suite-v1/
```
