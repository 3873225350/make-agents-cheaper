# AGENTS.md

Project guidance for Codex and other coding agents working in this repository.

## Project Identity

`make-agents-cheaper` is the audit/eval and research-instrumentation repo for harness-level prompt-cache experiments.

Keep these roles separate:

- Codex: development assistant used to build the tooling and write docs.
- Claude Code: current studied coding-agent harness.
- MiMo: current backend model/provider route, for example `mimo-v2.5-pro`.
- `make-agents-cheaper`: Rust audit/eval instrumentation and paper-facing evidence.
- `make-agents-cheaper-skill`: separate skill-packaging repo for reusable runbooks.
- `cheapcode`: possible future native cheaper-agent harness, not the current experiment.

Do not describe current experiments as Codex-cache experiments unless Codex is actually the harness under test.

## Core Claim Discipline

The project tests a narrow claim:

```text
When dynamic harness state would otherwise disturb the early prompt prefix,
moving that dynamic state later preserves a longer reusable prefix,
increases prompt-cache hit rate,
and can reduce observed input cost without reducing task success.
```

Do not claim:

- universal savings;
- lower total input tokens unless measured;
- quality preservation without validation;
- skill-layer savings as main evidence;
- cost savings when cache accounting is not observable.

Preferred wording:

```text
reduces paid uncached input
```

not:

```text
uses less context
```

## Product Layers

Use the right layer for the task:

- Rust CLI: implement audit/eval commands, token accounting, fingerprinting, trace import, and reports.
- `docs/`: write protocol, task-suite, trace-capture, and metric specifications.
- `paper/`: write conservative paper-facing claims grounded in logs.
- `taskplan/`: manage roadmap, subtasks, and evolution notes.
- `runs/`: keep local raw experiment logs and fixtures; do not commit raw traces.

## Evaluation Rules

- Separate warm-up calls from measured calls.
- Use paired A/B comparisons: baseline warm-up, candidate warm-up, baseline measured, candidate measured.
- Keep provider, model, route, MCP, hooks, task prompt, and validation command fixed across paired runs.
- Record token usage per task: input, cached input, cache creation input, uncached input, and output.
- Check task success and validation before counting a cheaper run as a win.
- Preserve failed or anomalous runs in notes instead of silently replacing them.

## Trace Rules

`claude-trace` raw request logs belong under:

```text
runs/<experiment>/raw/claude-trace/
```

Raw traces are sensitive. They may contain system prompts, tool outputs, file contents, local paths, and user text.

Never commit raw `.jsonl` or `.html` trace files. Derive safe artifacts instead:

```text
requests/<run_id>.request.json
traces/<run_id>.response.json
layers/<run_id>.layers.json
tools/<run_id>.tools.json
baseline.jsonl
cache-friendly.jsonl
```

Paper-facing artifacts should use minimal facts, hashes, counters, and excerpts, not full raw payloads.

## Rust Development

- Prefer small, testable CLI additions.
- Keep parsing tolerant of missing fields.
- Do not print API keys, authorization headers, or full raw prompts.
- Use structured JSON parsing instead of ad hoc string matching when possible.
- Run `cargo test` after Rust changes.

## Commit Discipline

- Run `git status --short` before and after changes.
- Do not commit ignored raw experiment logs.
- Commit plan/docs/tooling changes separately when they represent different checkpoints.
- If an external reference repo is nested under `references/`, do not add it as ordinary source unless explicitly requested.
