# Evolution: 2026-05-09 Trace Import

## Plan Used

```text
taskplan/roadmap.md
taskplan/subtasks/10-trace-capture-pipeline.md
docs/trace-capture-pipeline.md
```

## Bounded Target

Close the normalization gap between raw `claude-trace` captures and
`make-agents-cheaper eval` JSONL rows.

## Completed Work

- Added `make-agents-cheaper trace-import`.
- The importer reads raw trace JSONL, selects the final model request/response
  pair, and emits or appends one normalized eval row.
- It supports common usage shapes:
  - Anthropic-compatible `usage`;
  - Claude Code `modelUsage`;
  - MiMo/OpenAI-compatible `usage`.
- It exports safe derived artifacts when `--artifacts-dir` is provided:

```text
requests/<run_id>.request.json
traces/<run_id>.response.json
layers/<run_id>.layers.json
tools/<run_id>.tools.json
```

- It marks `cache_accounting_observable=false` instead of inventing cache
  tokens when usage fields are missing.
- Updated README, trace-capture docs, roadmap, and Subtask 10.

## Validation

Validation passed:

```bash
cargo fmt --check
cargo test
git diff --check
```

The test suite includes a synthetic raw trace import test.

## Deferrals

The actual pilot still needs live `claude-trace` captures. The importer is ready
for those logs, but Subtask 04 remains incomplete until baseline and
cache-friendly measured runs exist.

## Next Handoff

After a traced run, call `trace-import` for each measured baseline and
cache-friendly run, then run `eval` and `task-report`.
