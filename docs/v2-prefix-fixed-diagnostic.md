# V2 Fixed Prefix Diagnostic

This snapshot is a safe, derived summary of the local experiment:

```text
runs/2026-05-12-claude-mimo-v2-diagnostic-r3/
```

Raw Claude JSON and validation logs remain ignored under `runs/`. The tracked data here is limited to aggregate counters and sanitized JSONL rows.

![V2 fixed prefix-cache diagnostic chart](figures/v2-prefix-fixed-diagnostic.svg)

| Slice | Condition | Records | Cache hit | Uncached input | Output tokens | Observed cost | Turns | Validation | Task success |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `dynamic-drift` | baseline | 3 | 91.66% | 30,082 | 2,054 | $0.366976 | 15 | 3/3 | 3/3 |
| `dynamic-drift` | cache-friendly | 3 | 97.67% | 7,817 | 2,224 | $0.258237 | 15 | 3/3 | 3/3 |

## Interpretation

- The prefix-cache intervention is working in this bounded V2 diagnostic: cache-friendly uncached input is 0.260x baseline.
- Task success and validation are preserved at 3/3 vs 3/3.
- Observed cost is lower at 0.704x baseline, but the drop is smaller than the uncached-input drop because output tokens increased slightly.
- This supports the narrow claim: moving dynamic harness state later can reduce paid uncached input when cache accounting is observable and behavior remains comparable.
- It does not claim that every token becomes cheaper or that tool behavior is optimized. Tool-output compaction and behavior budgets are a separate next layer.

## Relation To The Earlier V2 Regression

The earlier `2026-05-11-claude-mimo-direct-json-v2-pilot` remains documented as a regression case. It exposed two practical issues:

- one cache-friendly run took many more agent turns than its paired baseline;
- the ignored V2 fixture could inherit parent-repository Git status without fixture-local Git isolation.

After fixing the generated runner to use absolute prompt paths, `GIT_CEILING_DIRECTORIES`, and fixture-local Git initialization, the bounded 3-repeat diagnostic returned to the expected prefix-cache direction.

## Reproduce The Report From Safe Examples

```bash
cargo run --quiet -- analysis-report \
  --baseline examples/v2-fixed-diagnostic-baseline.jsonl \
  --candidate examples/v2-fixed-diagnostic-cache-friendly.jsonl
```
