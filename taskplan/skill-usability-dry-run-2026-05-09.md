# Skill Usability Dry Run, 2026-05-09

## Scope

This is a local dry run of the `cheaper-skill-for-claude` workflow against the
main `make-agents-cheaper` command surface. It is not an independent
skill-guided agent result and is not evidence that the cache-friendly condition
is cheaper.

## Test Prompt

```text
Use cheaper-skill-for-claude to design a paired A/B experiment for Claude Code
routed to mimo-v2.5-pro. Include warm-up, measured calls, dynamic drift, token
usage fields, and validation checks.
```

## Artifact Under Test

```text
../make-agents-cheaper-skill/cheaper-skill-for-claude/SKILL.md
../make-agents-cheaper-skill/cheaper-skill-for-claude/references/standardized-ablation-workflow.md
```

## Smoke Command

```bash
cargo run --quiet -- pilot-plan \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --task docs-token-accounting \
  --experiment-dir runs/2026-05-09-skill-usability-dry-run \
  --slice dynamic-drift \
  --repeats 1
```

## Observed Reconstruction

The generated plan includes:

- `init-experiment` setup.
- Baseline warm-up without `--exclude-dynamic-system-prompt-sections`.
- Cache-friendly warm-up with `--exclude-dynamic-system-prompt-sections`.
- Baseline measured run without the flag.
- Cache-friendly measured run with the flag.
- Dynamic drift probes before measured runs.
- Validation logging to `validation/<run_id>.txt`.
- `trace-import` with `--slice`, `--repeat-id`, `--phase measured`,
  `--validation-passed`, and `--task-success`.
- Final `eval`, `task-report`, and `analysis-report` commands.

## Rubric Result

| Criterion | Local Dry-Run Result | Notes |
| --- | --- | --- |
| Protocol completeness | pass | Covers setup, warm-up, measured runs, dynamic drift, validation, trace import, and analysis. |
| Role separation | pass | Skill reference names Codex as operator, Claude Code as studied harness, MiMo as route/model, and `make-agents-cheaper` as evidence layer. |
| Command correctness | pass | Baseline omits the Claude flag; candidate includes `--exclude-dynamic-system-prompt-sections`. |
| Token accounting | partial-pass | The workflow preserves token fields through `trace-import`, but live traces are still required to test actual field extraction. |
| Overclaim avoidance | pass | The skill and report gate forbid universal claims, cold single-run claims, and savings claims without observable cache accounting. |
| Manual corrections | not measured | This was a local dry run, not an independent agent output. Runtime placeholders still require real paths, run IDs, and validation booleans. |

## Outcome

The skill runbook is internally consistent with the executable main-repo
protocol. Phase 8 can now move from rubric-only to local dry-run checked.

The independent usability ablation is still pending: a separate skill-guided
agent should be asked to reconstruct the protocol, then scored for corrections
needed before commands are executable.
