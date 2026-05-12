# Examples

These JSONL files are sanitized derivatives of real local Claude Code + MiMo runs. Raw traces and local paths are omitted; token/cost/success fields are preserved so users can try the analysis commands without running Claude Code.

```bash
cargo run --quiet -- eval \
  --baseline examples/baseline.jsonl \
  --candidate examples/cache-friendly.jsonl

cargo run --quiet -- task-report \
  --baseline examples/baseline.jsonl \
  --candidate examples/cache-friendly.jsonl

cargo run --quiet -- analysis-report \
  --baseline examples/baseline.jsonl \
  --candidate examples/cache-friendly.jsonl
```

The default pair comes from `2026-05-09-claude-mimo-paired-drift`, where the dynamic-drift slice improved.

To inspect the fixed V2 dynamic-drift diagnostic, run:

```bash
cargo run --quiet -- analysis-report \
  --baseline examples/v2-fixed-diagnostic-baseline.jsonl \
  --candidate examples/v2-fixed-diagnostic-cache-friendly.jsonl
```

To inspect the earlier mixed/negative V2 pilot that motivated the runner fixes, run:

```bash
cargo run --quiet -- analysis-report \
  --baseline examples/v2-mixed-baseline.jsonl \
  --candidate examples/v2-mixed-cache-friendly.jsonl
```

These examples are safe, derived artifacts. They are useful for reproducing reports, but paper claims should still cite the corresponding experiment notes and full local run directory.
