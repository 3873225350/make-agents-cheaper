# evolution-2026-05-11-direct-json-capture-decision.md

## Loop Type

- type: decision

## Plan

- path: `taskplan/roadmap.md`
- milestone: run standardized paired A/B experiments
- bounded target: remove `claude-trace` as a blocker and make direct Claude JSON capture the current experiment path.

## Decision

Do not install or require `claude-trace` for the current V2 matrix.

Use:

```text
claude --output-format json
-> raw/claude-json/<run_id>.json
-> claude-json-import
-> baseline.jsonl / cache-friendly.jsonl
```

Keep `trace-import` as an optional higher-evidence path only if request-shape artifacts are explicitly requested later.

## Completed

- Updated `pilot-plan` so generated run commands use direct Claude JSON capture.
- Regenerated the V2 full-matrix plan files under:

```text
runs/2026-05-11-claude-mimo-real-coding-v2-full/notes/
```

- Updated the experiment scaffold to include:

```text
raw/claude-json/
```

- Updated documentation and planning notes so `claude-trace` is not listed as a current blocker.
- Updated the project-local workflow skill at:

```text
.claude/skills/claude-trace-recovery/
```

so its default is direct JSON capture, not trace recovery.

## Evidence Consequence

Direct JSON supports:

- input tokens;
- cached input tokens;
- cache creation input tokens;
- output tokens;
- observed cost;
- latency;
- validation result;
- task success.

Direct JSON does not support:

- exact raw API request body;
- system/tool/message ordering;
- request-layer fingerprints;
- tool-schema artifacts.

So the V2 matrix can test the main cost/quality claim, but request-shape claims must remain qualified unless optional trace artifacts are captured later.

## Next Handoff

```text
Continue taskplan/roadmap.md.

Next bounded run:
1. Use the regenerated direct-json pilot plan.
2. Rerun only docs-token-accounting/control-steady after resetting the fixture.
3. Use --permission-mode bypassPermissions for the ignored fixture.
4. Do not use --max-budget-usd 0.2.
5. Normalize measured rows with claude-json-import.
6. Preserve prior failed rows; record any retry separately.
7. Do not install or debug claude-trace unless the user explicitly asks for request-shape artifacts.
```
