# Real Coding Ablation V2

This suite is the larger real-coding dataset for the Claude Code + MiMo cache-hit study.

V1 is a smoke suite. V2 should be strong enough to support paper-facing claims about realistic coding-agent workflows.

## Scope

Current object of study:

```text
development assistant: Codex
studied harness: Claude Code
backend route/model: MiMo, e.g. mimo-v2.5-pro
measurement layer: make-agents-cheaper audit/eval
```

The skill layer is not used as the primary evidence source. Skill-guided runs belong to the auxiliary usability ablation.

## Fixture

Planned fixture path:

```text
runs/fixtures/real-coding-v2/
```

The fixture remains ignored by git because `runs/` is an experiment/log area.

The fixture should be resettable:

```bash
cd runs/fixtures/real-coding-v2
bash task-reset.sh <task-id>
bash task-validate.sh <task-id>
```

## Conditions

Every task is run under the same paired conditions:

| Condition | Claude Code flags |
| --- | --- |
| `baseline` | no cache-friendly dynamic-system flag |
| `cache-friendly` | `--exclude-dynamic-system-prompt-sections` |

Keep fixed:

- model: `mimo-v2.5-pro`
- output format: JSON when measuring
- session persistence: disabled unless testing session persistence
- provider/route
- MCP servers and hooks
- validation command

## Run Shape

For each task, slice, and repeat:

```text
baseline warm-up
cache-friendly warm-up
baseline measured
cache-friendly measured
validation for both measured runs
JSONL extraction
task-report and eval summary
```

Do not compare a warm baseline to a cold candidate.

## Drift Slices

| Slice | Drift action | Purpose |
| --- | --- | --- |
| `control-steady` | no deliberate repo-state drift after warm-up | checks fair warm-cache behavior |
| `dynamic-drift` | mutate ignored local state before measured calls | tests whether dynamic state damages early prefix reuse |

Suggested drift actions:

```bash
printf 'drift %s\n' "$(date +%s%N)" > .cache-drift-probe
git status --short > .local-git-status-snapshot
mkdir -p .local-memory && printf '%s\n' "$PWD" > .local-memory/path.txt
```

Reset must remove drift files.

## Task Matrix

Minimum measured repeats:

```text
3 repeats per task / condition / slice
```

| Task ID | Family | Turns | Files Expected | Validation |
| --- | --- | ---: | --- | --- |
| `docs-token-accounting` | documentation edit | 1 | README/docs | `bash task-validate.sh docs-token-accounting` |
| `cli-jsonl-report` | CLI feature | 1 | src + tests | `bash task-validate.sh cli-jsonl-report` |
| `parser-comment-whitespace` | parser bug fix | 1 | src + tests | `bash task-validate.sh parser-comment-whitespace` |
| `schema-cache-creation` | schema update | 1 | src + docs + tests | `bash task-validate.sh schema-cache-creation` |
| `config-warning-rule` | audit rule | 1 | src + tests | `bash task-validate.sh config-warning-rule` |
| `experiment-log-summary` | multi-file feature | 1 | src + tests + example data | `bash task-validate.sh experiment-log-summary` |
| `failing-test-repair` | failing-test repair | 1 | src only | `bash task-validate.sh failing-test-repair` |
| `multi-turn-eval-polish` | iterative improvement | 4 | docs + src + tests | `bash task-validate.sh multi-turn-eval-polish` |

## Task Prompts

### docs-token-accounting

```text
You are editing the fixture repository at runs/fixtures/real-coding-v2.

Task: add a "Token Accounting" section to README.md.

Requirements:
- Explain input, cached input, cache creation input, uncached input, and output tokens.
- State that the main target is lower uncached input, not lower total input.
- Mention that validation failure invalidates a cost win.
- Keep the change documentation-only.

Run:
bash task-validate.sh docs-token-accounting
```

### cli-jsonl-report

```text
You are editing the fixture repository at runs/fixtures/real-coding-v2.

Task: add a report-jsonl command that reads a JSONL run log and prints one compact table row per task.

Expected command:
cargo run --quiet -- report-jsonl examples/runs.jsonl

Required output fields:
- task_id
- condition
- input_tokens
- cached_input_tokens
- uncached_input_tokens
- output_tokens
- validation_passed

Keep existing commands working.

Run:
bash task-validate.sh cli-jsonl-report
```

### parser-comment-whitespace

```text
You are editing the fixture repository at runs/fixtures/real-coding-v2.

Task: make the CSV parser ignore blank lines and comment lines.

Comment lines begin with # after optional leading whitespace.

Run:
bash task-validate.sh parser-comment-whitespace
```

### schema-cache-creation

```text
You are editing the fixture repository at runs/fixtures/real-coding-v2.

Task: add cache_creation_input_tokens to the run summary schema.

Requirements:
- Parse the field from JSONL when present.
- Treat missing cache_creation_input_tokens as 0.
- Include it in text output.
- Update README.md with the new field.

Run:
bash task-validate.sh schema-cache-creation
```

### config-warning-rule

```text
You are editing the fixture repository at runs/fixtures/real-coding-v2.

Task: add an audit warning when the config changes model between baseline and candidate.

Requirements:
- Detect mismatched model names in a small comparison input.
- Print a warning containing "model drift".
- Add or update a test.

Run:
bash task-validate.sh config-warning-rule
```

### experiment-log-summary

```text
You are editing the fixture repository at runs/fixtures/real-coding-v2.

Task: add an experiment-summary command that aggregates a JSONL log by condition.

Required aggregate fields:
- runs
- validation_passed
- total_input_tokens
- total_cached_input_tokens
- total_uncached_input_tokens
- total_output_tokens
- mean_cache_hit_rate

Run:
bash task-validate.sh experiment-log-summary
```

### failing-test-repair

```text
You are editing the fixture repository at runs/fixtures/real-coding-v2.

Task: one test is failing because uncached_input_tokens is computed incorrectly when cached tokens exceed total input due to malformed data.

Expected behavior:
- never underflow
- clamp uncached_input_tokens to 0 for malformed rows
- keep a warning count for malformed rows

Run:
bash task-validate.sh failing-test-repair
```

### multi-turn-eval-polish

Turn 1:

```text
You are editing the fixture repository at runs/fixtures/real-coding-v2.

Task: inspect the current CLI and propose the smallest implementation plan for adding a cache-hit evaluation summary.

Do not edit files yet. Reply with the plan only.
```

Turn 2:

```text
Implement the smallest useful version of the plan.

Run:
bash task-validate.sh multi-turn-eval-polish
```

Turn 3:

```text
Improve the README so a user can run the new evaluation summary command with examples/runs.jsonl.

Run:
bash task-validate.sh multi-turn-eval-polish
```

Turn 4:

```text
Review your own changes for unrelated edits, then run:
bash task-validate.sh multi-turn-eval-polish
```

## Required Per-Run Fields

Every measured call must be extractable into JSONL with:

```text
task_id
condition
slice
repeat_id
phase
agent
model
input_tokens
cached_input_tokens
cache_creation_input_tokens
uncached_input_tokens
output_tokens
validation_passed
task_success
cache_accounting_observable
trace_path
validation_path
anomaly
```

## Acceptance Criteria

The suite is ready for measurement when:

- every task has reset and validation commands;
- every task has a fixed prompt;
- every task can run under both baseline and cache-friendly conditions;
- dynamic drift files are cleaned by reset;
- validation catches at least one meaningful failure per implementation task;
- logs can be consumed by `make-agents-cheaper eval` and `task-report`.
