# AGENTS.md

Guidance for documentation work under `docs/`.

## Documentation Role

Docs are the canonical runbooks and method specifications for experiments.

Prefer exact, operational language:

- where logs are stored;
- which command produced an artifact;
- which fields are required;
- which claims are supported;
- which caveats block stronger claims.

## Required Boundaries

Always keep these distinctions visible when relevant:

- development assistant: Codex;
- studied harness: Claude Code;
- backend route/model: MiMo, for example `mimo-v2.5-pro`;
- measurement layer: `make-agents-cheaper` audit/eval;
- reuse layer: `cheaper-skill-for-claude`.

## Trace And Log Policy

Docs may describe raw trace formats, but must not include full raw request payloads.

Use paths like:

```text
runs/<experiment>/raw/claude-trace/<run_id>.jsonl
```

For examples, prefer toy JSON or redacted snippets.

## Claims

Do not convert a mechanism note into a result claim. If a result has not been measured, use:

```text
planned
expected to test
should record
```

not:

```text
proves
improves
saves
```

## Cross-References

When changing one protocol document, check nearby docs for consistency:

- `docs/evaluation-protocol.md`
- `docs/evaluation-metrics.md`
- `docs/trace-capture-pipeline.md`
- `docs/task-suites/*.md`
