# Command Shapes

Use these templates inside `make-agents-cheaper`.

## Direct JSON Default

Use this path for the current roadmap. It does not require `claude-trace`.

```bash
cd runs/fixtures/real-coding-v2
bash task-reset.sh <task-id>

claude -p \
  --model mimo-v2.5-pro \
  --output-format json \
  --no-session-persistence \
  --permission-mode bypassPermissions \
  <condition-flags> \
  "$PROMPT" \
  > runs/<experiment>/raw/claude-json/<run-id>.json \
  2> runs/<experiment>/raw/claude-json/<run-id>.stderr.txt
```

Normalize measured runs:

```bash
cargo run --quiet -- claude-json-import \
  --input runs/<experiment>/raw/claude-json/<run-id>.json \
  --run-id <run-id> \
  --task-id <task-id> \
  --condition <baseline|cache-friendly> \
  --slice <control-steady|dynamic-drift> \
  --repeat-id <n> \
  --phase measured \
  --output runs/<experiment>/<baseline|cache-friendly>.jsonl \
  --validation-path runs/<experiment>/validation/<run-id>.txt \
  --validation-passed <true|false> \
  --task-success <true|false>
```

Direct JSON rows should include `request_shape_observable=false`.

## Candidate Flag

Use this only for the cache-friendly condition:

```bash
--exclude-dynamic-system-prompt-sections
```

## Optional Trace Path

Use this only if the user explicitly asks for request-shape artifacts.

```bash
cargo run --quiet -- trace-import \
  --input runs/<experiment>/raw/claude-trace/<run-id>.jsonl \
  --run-id <run-id> \
  --task-id <task-id> \
  --condition <baseline|cache-friendly> \
  --slice <control-steady|dynamic-drift> \
  --repeat-id <n> \
  --phase measured \
  --output runs/<experiment>/<baseline|cache-friendly>.jsonl \
  --artifacts-dir runs/<experiment> \
  --validation-path runs/<experiment>/validation/<run-id>.txt \
  --validation-passed <true|false> \
  --task-success <true|false>
```

## Post-Run Analysis

```bash
cargo run --quiet -- eval \
  --baseline runs/<experiment>/baseline.jsonl \
  --candidate runs/<experiment>/cache-friendly.jsonl

cargo run --quiet -- task-report \
  --baseline runs/<experiment>/baseline.jsonl \
  --candidate runs/<experiment>/cache-friendly.jsonl

cargo run --quiet -- analysis-report \
  --baseline runs/<experiment>/baseline.jsonl \
  --candidate runs/<experiment>/cache-friendly.jsonl \
  --output runs/<experiment>/analysis-report.md
```
