# AGENTS.md

Guidance for local experiment runs under `runs/`.

## Scope

`runs/` contains local experiment outputs, raw traces, temporary fixtures, and validation artifacts.

Most files under this directory are intentionally ignored by git.

## Raw Trace Storage

Store `claude-trace` captures here:

```text
runs/<experiment>/raw/claude-trace/<run_id>.jsonl
runs/<experiment>/raw/claude-trace/<run_id>.html
```

These files are sensitive and must not be committed.

## Normalized Artifacts

Derived artifacts should use this layout:

```text
runs/<experiment>/requests/<run_id>.request.json
runs/<experiment>/traces/<run_id>.response.json
runs/<experiment>/layers/<run_id>.layers.json
runs/<experiment>/tools/<run_id>.tools.json
runs/<experiment>/baseline.jsonl
runs/<experiment>/cache-friendly.jsonl
runs/<experiment>/notes.md
```

The normalized JSONL rows are used by `make-agents-cheaper eval` and `task-report`.

## Run Discipline

For paired A/B:

- reset the fixture before each task;
- separate warm-up from measured calls;
- keep model/provider/MCP/hooks fixed;
- record dynamic drift actions;
- run task validation;
- never silently delete failed runs.

## Safety

Do not print or paste:

- API keys;
- authorization headers;
- full raw prompts;
- full system prompts;
- full tool outputs from private files;
- raw trace bodies.

Summarize sensitive material with hashes, counts, and redacted examples.
