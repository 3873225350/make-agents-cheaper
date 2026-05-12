# V2 Regression Diagnosis

This note explains why the 2026-05-11 V2 direct-JSON pilot was mixed/negative even though the earlier paired-drift run showed a strong cache-friendly win. It also records the fixed 2026-05-12 diagnostic that returned to the expected prefix-cache direction.

## Result Contrast

Earlier real paired-drift run:

| Experiment | Slice | Baseline uncached | Cache-friendly uncached | Ratio | Success |
| --- | --- | ---: | ---: | ---: | --- |
| `2026-05-09-claude-mimo-paired-drift` | dynamic-drift | 45,548 | 4,630 | 0.102x | 2/2 vs 2/2 |

Original V2 direct-JSON pilot:

| Experiment | Slice | Baseline uncached | Cache-friendly uncached | Ratio | Success |
| --- | --- | ---: | ---: | ---: | --- |
| `2026-05-11-claude-mimo-direct-json-v2-pilot` | control-steady | 8,162 | 7,305 | 0.895x | 3/3 vs 3/3 |
| `2026-05-11-claude-mimo-direct-json-v2-pilot` | dynamic-drift | 6,470 | 13,930 | 2.153x | 3/3 vs 3/3 |

The method did not uniformly fail. It still helped slightly in `control-steady`, but regressed in `dynamic-drift`.

## Primary Suspect

The V2 dynamic-drift aggregate is dominated by one behavioral outlier:

| Run | Condition | Turns | Input | Cached | Uncached | Output | Cost |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `docs-token-accounting-dynamic-drift-retry1-baseline-r1-measured` | baseline | 6 | 143,086 | 140,992 | 2,094 | 667 | $0.097641 |
| `docs-token-accounting-dynamic-drift-retry1-cache-friendly-r1-measured` | cache-friendly | 14 | 401,897 | 391,232 | 10,665 | 2,065 | $0.300566 |

This is not just a prompt-cache hit-rate problem. The cache-friendly run caused or coincided with a longer Claude Code trajectory. More agent turns repeated large cached context, increased output, and still raised paid uncached input.

## Changes From The Positive Run

1. The earlier positive run was a tight print-mode paired-drift test with exact-reply validation. V2 is a real coding edit that lets Claude Code inspect files, edit, and run tests.
2. V2 measures full harness behavior. This is good for realism, but it means a candidate can lose by changing the number of tool/model turns even if its prefix cache shape is mostly stable.
3. The first dynamic-drift attempt also exposed runner fragility: a relative prompt path failed when commands were executed from the fixture. Those failed calls were not imported as measured rows, but they show the protocol needed a more robust generated script.
4. The V2 fixture was not an isolated git repository. `git status` inside `runs/fixtures/real-coding-v2` could discover the parent `make-agents-cheaper` repository, so the dynamic state could include unrelated parent-worktree dirtiness rather than only fixture-local state.
5. Direct JSON records usage and cost, but not request/layer shape. Therefore the V2 run can diagnose token and behavior regressions, but cannot prove which exact prompt block moved without optional trace artifacts.

## Fixes Applied

- `run-pilot` now generates scripts with an absolute default `REPO_ROOT` instead of depending on the caller's current directory.
- The generated script now checks the prompt file before calling Claude, so path mistakes fail early and clearly.
- The generated script now sets `GIT_CEILING_DIRECTORIES` and initializes a fixture-local git repo when needed, so dynamic git status comes from the fixture rather than the parent repository.
- The generated script now excludes stable fixture-local noise directories (`.baseline/`, `.claude-trace/`, `target/`) from fixture git status.
- `analysis-report` now emits a behavioral-diagnostics table that flags candidate runs with many more turns, higher uncached input, or much higher output.
- `examples/` now uses sanitized real run rows, plus a V2 mixed example for reproducing the regression report.

## Bounded Diagnostic Reruns

After the runner fixes, a one-repeat dynamic-drift diagnostic first confirmed the expected direction:

```text
runs/2026-05-12-claude-mimo-v2-diagnostic-r1/
```

| Slice | Baseline uncached | Cache-friendly uncached | Ratio | Success | Turns |
| --- | ---: | ---: | ---: | --- | --- |
| dynamic-drift | 24,804 | 3,367 | 0.136x | 1/1 vs 1/1 | 5 vs 4 |

This single-repeat result is not yet a full paper claim, but it supports the diagnosis: once fixture git state is isolated from the parent repo and the prompt path is robust, the cache-friendly dynamic-drift case returns to the expected direction.

The follow-up three-repeat diagnostic is the current headline V2 evidence:

```text
runs/2026-05-12-claude-mimo-v2-diagnostic-r3/
```

| Slice | Baseline uncached | Cache-friendly uncached | Ratio | Baseline hit | Cache-friendly hit | Cost ratio | Success | Turns |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| dynamic-drift | 30,082 | 7,817 | 0.260x | 91.66% | 97.67% | 0.704x | 3/3 vs 3/3 | 15 vs 15 |

This result supports the narrow prefix-cache claim for the fixed V2 diagnostic: moving dynamic harness state later reduced paid uncached input while preserving validation and task success. It also shows the boundary of the claim. Output tokens increased slightly from 2,054 to 2,224, so tool-output behavior remains a separate optimization layer.

## Next Experiment

Do not rerun the full V2 matrix blindly. The next step should be a focused expansion around the fixed condition:

```bash
cargo run --quiet -- run-pilot \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --task docs-token-accounting \
  --experiment-dir runs/<date>-claude-mimo-v2-diagnostic \
  --slice dynamic-drift \
  --repeats 3
```

Then inspect `analysis-report.md`. A publishable expanded run should show:

- no prompt-path failures in `notes/run-log.tsv`;
- `git -C runs/fixtures/real-coding-v2 rev-parse --show-toplevel` resolves to the fixture itself;
- no candidate turn-count outlier like 14 turns vs 6 turns;
- candidate uncached input lower than baseline in all-runs accounting;
- validation and task success unchanged.

If the candidate still uses more turns or produces much more output, report that as tool/behavior cost rather than prefix failure. The prefix result and the future RTK-style tool-output layer should remain separate conditions.
