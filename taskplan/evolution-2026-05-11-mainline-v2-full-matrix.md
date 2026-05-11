# evolution-2026-05-11-mainline-v2-full-matrix.md

## Loop Type

- type: analysis
- skill: codex-loop

## Plan

- path: `taskplan/roadmap.md`
- milestone: run standardized paired A/B experiments
- bounded target: keep the project focused on the V2 full matrix and verify whether moving dynamic harness state later protects the stable prompt prefix.

## Decision

Do not expand the project scope right now.

The hook-stack reports in `references/claude-code-hooks-github-repos.md` and `references/make_agent_cheap_plan.md` are useful as later idea sources, but the current priority is not Read dedup, RTK, Caveman, StatusLine automation, output compression, or security-hook integration.

The current priority is:

```text
dynamic-drift / control-steady V2 full matrix
-> stable-prefix evidence
-> cached input increases
-> uncached paid input decreases
-> task success and validation do not regress
```

## Mainline Claim To Protect

Use the existing narrow claim:

```text
When dynamic harness state would otherwise disturb the early prompt prefix,
moving that dynamic state later preserves a longer reusable prefix,
increases prompt-cache hit rate,
and can reduce observed input cost without reducing task success.
```

Do not claim universal savings, lower total tokens, or hook-stack savings unless a separate measured ablation later supports those claims.

## Current Anchor

The roadmap already says the V2 task matrix is ready to run:

- `taskplan/roadmap.md`: phase 3, run standardized paired A/B experiments.
- `docs/task-suites/real-coding-ablation-v2.manifest.json`: task manifest with `baseline` and `cache-friendly` conditions.
- slices to keep separate: `control-steady` and `dynamic-drift`.
- required measured repeats: 3.
- current capture decision: use direct Claude JSON with `claude-json-import`; do not block on `claude-trace`.

## Execution Checklist

1. Generate the full matrix plan.

```bash
cargo run --quiet -- matrix-plan \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --experiment-dir runs/2026-05-11-claude-mimo-real-coding-v2-full \
  --repeats 3
```

2. Run the printed plan without changing fixed variables.

Keep fixed:

- agent: `claude_code`
- model: `mimo-v2.5-pro`
- repository snapshot / fixture reset behavior
- task prompt
- validation command
- provider or router path
- MCP, hooks, and tool schema

3. For each task and slice, preserve the run order.

```text
baseline warm-up
cache-friendly warm-up
baseline measured
cache-friendly measured
validation for both measured runs
JSONL extraction
```

Warm-up records must not be counted as measured evidence.

4. Keep `control-steady` and `dynamic-drift` results separate.

Interpretation target:

- `control-steady`: both conditions may cache well after warm-up; this prevents overclaiming.
- `dynamic-drift`: baseline should be vulnerable to early dynamic state; cache-friendly should preserve more stable prefix.

5. Capture or normalize evidence.

```text
direct Claude Code JSON usage
-> claude-json-import
-> baseline.jsonl and cache-friendly.jsonl
-> mark request-shape evidence as unavailable
```

6. Run the analysis commands.

```bash
cargo run --quiet -- eval \
  --baseline runs/2026-05-11-claude-mimo-real-coding-v2-full/baseline.jsonl \
  --candidate runs/2026-05-11-claude-mimo-real-coding-v2-full/cache-friendly.jsonl

cargo run --quiet -- task-report \
  --baseline runs/2026-05-11-claude-mimo-real-coding-v2-full/baseline.jsonl \
  --candidate runs/2026-05-11-claude-mimo-real-coding-v2-full/cache-friendly.jsonl

cargo run --quiet -- analysis-report \
  --baseline runs/2026-05-11-claude-mimo-real-coding-v2-full/baseline.jsonl \
  --candidate runs/2026-05-11-claude-mimo-real-coding-v2-full/cache-friendly.jsonl \
  --output runs/2026-05-11-claude-mimo-real-coding-v2-full/analysis-report.md
```

## Stop Conditions

Do not report the main claim as supported unless all are true:

- cache accounting is observable;
- candidate cached input increases;
- candidate uncached paid input decreases;
- validation pass rate does not regress;
- task success does not regress;
- warm-up and measured records remain separate.

If cache accounting is not observable, report only prefix stability, trace shape, and latency as exploratory evidence.

## Deferred On Purpose

These are not next actions:

- Read-once hook ablation;
- diff compression ablation;
- RTK or shell-output compression;
- Caveman output compression;
- StatusLine automatic compact;
- hook security stack;
- Qdrant / HyDE memory layer;
- new native `cheapcode` harness design.

They can become later slices only after the V2 full matrix has produced a clean mainline result or a clearly documented failure.

## Next Handoff

```text
Use codex-loop manual mode. Anchor on taskplan/evolution-2026-05-11-mainline-v2-full-matrix.md and taskplan/roadmap.md.

Execute one bounded slice only:
1. Generate or refresh the V2 full matrix command plan.
2. Use the direct Claude JSON capture path; do not install or require claude-trace.
3. Normalize measured rows with claude-json-import.
4. Record that request-shape evidence is unavailable unless optional trace artifacts exist.
5. Do not add new hook/read-dedup/compression experiments.
6. End by writing the next evolution note with completed runs, failures, and the next smallest handoff.
```
