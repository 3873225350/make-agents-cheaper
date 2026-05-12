# Evolution 2026-05-12: Fixed V2 Prefix Diagnostic

## Trigger

The project needed to replace the old V2 mixed/negative README and paper snapshot with the fixed prefix-cache diagnostic before GitHub publication.

## Run

```text
runs/2026-05-12-claude-mimo-v2-diagnostic-r3/
```

Command shape:

```bash
cargo run --quiet -- run-pilot \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --task docs-token-accounting \
  --experiment-dir runs/2026-05-12-claude-mimo-v2-diagnostic-r3 \
  --slice dynamic-drift \
  --repeats 3 \
  --execute true
```

Direct Claude JSON was used. Raw Claude JSON and validation logs remain ignored under `runs/`; request-shape evidence is unavailable for this run.

## Result

| Slice | Baseline uncached | Cache-friendly uncached | Ratio | Baseline hit | Cache-friendly hit | Observed cost ratio | Success |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `dynamic-drift` | 30,082 | 7,817 | 0.260x | 91.66% | 97.67% | 0.704x | 3/3 vs 3/3 |

Validation passed for all measured rows.

## Decision

Use this as the current headline V2 evidence for the narrow prefix-cache claim. Keep the 2026-05-11 mixed/negative pilot as a regression-diagnosis artifact.

Claim boundary:

- prefix layout reduced paid uncached input;
- task success and validation did not regress;
- output tokens increased slightly, so tool-output optimization remains future work.
