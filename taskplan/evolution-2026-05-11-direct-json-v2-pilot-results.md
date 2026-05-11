# evolution-2026-05-11-direct-json-v2-pilot-results.md

## Loop Type

- type: experiment

## Plan

- path: `taskplan/roadmap.md`
- milestone: run standardized paired A/B experiments
- bounded target: complete a clean direct-json V2 pilot for `docs-token-accounting` across `control-steady` and `dynamic-drift`.

## Experiment

```text
runs/2026-05-11-claude-mimo-direct-json-v2-pilot/
```

Harness and route:

```text
studied harness: Claude Code
model/route: mimo-v2.5-pro
capture: claude --output-format json
normalizer: claude-json-import
trace capture: skipped; claude-trace not required
```

## Completed

- Ran `docs-token-accounting/control-steady` with 3 repeats per condition.
- Ran `docs-token-accounting/dynamic-drift` with 3 successful retry repeats per condition.
- Used `--permission-mode bypassPermissions` on the ignored fixture.
- Imported only measured rows with `claude_status=0` and `validation_status=0`.
- Generated:
  - `baseline.jsonl`
  - `cache-friendly.jsonl`
  - `analysis-report.md`
  - `slice-summary.tsv`
  - `slice-deltas.tsv`

## Results

All imported measured rows passed validation.

| Slice | Baseline uncached | Cache-friendly uncached | Ratio | Baseline success | Cache-friendly success |
| --- | ---: | ---: | ---: | ---: | ---: |
| `control-steady` | 8,162 | 7,305 | 0.895x | 3/3 | 3/3 |
| `dynamic-drift` | 6,470 | 13,930 | 2.153x | 3/3 | 3/3 |
| aggregate | 14,632 | 21,235 | 1.451x | 6/6 | 6/6 |

Interpretation:

- The pilot does not support the primary savings claim.
- `control-steady` shows a small candidate uncached-input reduction.
- `dynamic-drift` regresses; one cache-friendly measured run took 14 turns and dominates the aggregate.
- Because direct JSON does not expose request-shape artifacts, this run cannot prove whether stable prefix preservation failed or whether task behavior variance dominated.

## Anomalies

- The first `dynamic-drift` attempt used a wrong prompt relative path from the fixture directory.
- Claude exited before task execution for those rows, so they remain in `notes/run-log.tsv` but were excluded from JSONL.
- The retry run IDs include `retry1` and all imported retry measured rows passed validation.

## Decision

- Do not claim V2 pilot savings.
- Do not launch the full V2 matrix blindly.
- Keep `claude-trace` optional and skipped unless request-shape evidence is explicitly needed.
- Next bounded pass should diagnose the dynamic-drift regression before spending a full 96 measured calls.

## Validation

Commands run:

```bash
cargo run --quiet -- eval \
  --baseline runs/2026-05-11-claude-mimo-direct-json-v2-pilot/baseline.jsonl \
  --candidate runs/2026-05-11-claude-mimo-direct-json-v2-pilot/cache-friendly.jsonl

cargo run --quiet -- task-report \
  --baseline runs/2026-05-11-claude-mimo-direct-json-v2-pilot/baseline.jsonl \
  --candidate runs/2026-05-11-claude-mimo-direct-json-v2-pilot/cache-friendly.jsonl

cargo run --quiet -- analysis-report \
  --baseline runs/2026-05-11-claude-mimo-direct-json-v2-pilot/baseline.jsonl \
  --candidate runs/2026-05-11-claude-mimo-direct-json-v2-pilot/cache-friendly.jsonl \
  --output runs/2026-05-11-claude-mimo-direct-json-v2-pilot/analysis-report.md

cargo test
```
