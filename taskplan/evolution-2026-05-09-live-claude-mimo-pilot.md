# evolution-2026-05-09-live-claude-mimo-pilot.md

## Loop Type

- type: execution

## Plan

- path: `taskplan/roadmap.md`
- milestone: Phase 4, live paired A/B pilot
- bounded target: run the smallest live Claude Code / MiMo pilot and identify the current blocker

## Review Window

- reviewed loops: roadmap, Subtask 04, Subtask 10, live pilot command plan
- status before run: pilot was ready-to-run but no live Claude/MiMo calls had been executed

## Completed

- Confirmed `claude` is installed: Claude Code `2.1.123`.
- Confirmed a live `mimo-v2.5-pro` JSON probe succeeds and exposes usage fields.
- Ran a one-task, one-repeat `dynamic-drift` pilot for `docs-token-accounting`.
- Captured direct Claude Code JSON output under:

```text
runs/2026-05-09-live-claude-mimo-pilot/raw/claude-json/
```

- Normalized measured rows into:

```text
runs/2026-05-09-live-claude-mimo-pilot/baseline.jsonl
runs/2026-05-09-live-claude-mimo-pilot/cache-friendly.jsonl
```

- Ran `eval`, `task-report`, and `analysis-report`.

## Result Snapshot

```text
task: docs-token-accounting
slice: dynamic-drift
repeat: 1
model: mimo-v2.5-pro
transport captured: claude_code_json
baseline validation: passed
cache-friendly validation: passed
baseline task success: 1/1
cache-friendly task success: 1/1
baseline cache hit rate: 98.46%
cache-friendly cache hit rate: 98.86%
baseline uncached input: 2,196
cache-friendly uncached input: 1,271
uncached input ratio: 0.579x
baseline observed cost: $0.100471
cache-friendly observed cost: $0.075144
observed cost ratio: 0.748x
baseline latency: 25,098 ms
cache-friendly latency: 15,762 ms
```

## Blocker Found

`claude-trace` is not installed in PATH:

```text
claude-trace: command not found
```

Therefore this run has direct Claude Code JSON usage evidence, but it does not
have raw request/response trace JSONL, request artifacts, layer artifacts, or
tool-schema artifacts. That blocks the stronger request-shape analysis promised
by Subtask 10.

## Failed or Deferred

- No `claude-trace` raw JSONL was produced.
- No request/layer/tool artifacts were produced.
- The run has only one measured repeat and one task, so it is a pilot smoke
  result, not a full-matrix claim.

## Decisions

- Treat the result as a live pilot smoke success with direct JSON usage.
- Do not mark the trace-capture pipeline fully validated until `claude-trace` is
  installed and a trace-captured rerun produces request/layer/tool artifacts.
- A future CLI importer for direct Claude JSON output would remove the manual
  normalization step used in this run.

## Analysis Checks

- regression risk: low; fixture was reset after each run
- drift risk: medium; dynamic drift was simulated, but request-shape artifacts are missing
- version safety: raw live outputs stay under ignored `runs/`
- plan adjustment: Phase 4 can move from ready-to-run to live direct-json pilot run; Phase 10 remains blocked on `claude-trace`

## Next Handoff

```text
Continue taskplan/roadmap.md. Next smallest useful slice: install or locate claude-trace, then rerun the same docs-token-accounting dynamic-drift pilot with raw request/response capture so trace-import can produce request/layer/tool artifacts. Alternatively, add a first-class claude-json-import command for direct Claude Code JSON outputs.
```
