# Real Coding Ablation V1

This suite is the first realistic coding-task dataset for `make-agents-cheaper`.

It moves beyond exact-reply micro prompts. The goal is to test whether cache-friendly harness assembly still helps when the agent edits files, runs tests, and handles small implementation details.

## Fixture

Fixture path:

```text
runs/fixtures/real-coding-v1/
```

The fixture is intentionally ignored by the parent repository through `runs/`.

Baseline validation:

```bash
cd runs/fixtures/real-coding-v1
bash task-reset.sh base
bash task-validate.sh base
```

## Experimental Conditions

Run every selected task under two prompt/harness conditions:

```text
baseline
cache-friendly
```

For Claude Code + MiMo experiments, keep these fixed:

```text
model: mimo-v2.5-pro
output format: json
session persistence: disabled unless explicitly testing session persistence
```

The cache-friendly Claude Code condition should use the official prompt assembly flag:

```bash
--exclude-dynamic-system-prompt-sections
```

Do not patch Claude Code source. Do not inject dynamic hook text before the stable prefix.

## Drift Slices

Each task can be measured in two slices:

| Slice | Drift action | Purpose |
| --- | --- | --- |
| `control-steady` | no deliberate repo-state drift after warm-up | checks warm-cache fairness |
| `dynamic-drift` | create or update `.cache-drift-probe-<run>` before measured calls | tests dynamic harness-state sensitivity |

Dynamic drift command:

```bash
printf 'drift %s\n' "$(date +%s%N)" > .cache-drift-probe-<run-id>
```

Reset must remove drift probes:

```bash
bash task-reset.sh <task-id>
```

## Required Token Log Fields

Every measured model call must produce one JSONL row with:

```text
task_id
condition
slice
run_id
model
input_tokens
cached_input_tokens
cache_creation_input_tokens
uncached_input_tokens
output_tokens
task_success
validation_passed
cache_accounting_observable
trace_path
validation_path
anomaly
```

For direct MiMo OpenAI-compatible calls, map:

```text
input_tokens = usage.prompt_tokens
cached_input_tokens = usage.prompt_tokens_details.cached_tokens
output_tokens = usage.completion_tokens
reasoning_tokens = usage.completion_tokens_details.reasoning_tokens
```

For Claude Code JSON routed to MiMo, map:

```text
input_tokens = inputTokens + cacheReadInputTokens + cacheCreationInputTokens
cached_input_tokens = cacheReadInputTokens
cache_creation_input_tokens = cacheCreationInputTokens
output_tokens = outputTokens
```

If cached-token accounting is unavailable, set `cache_accounting_observable=false` and do not claim token-cost savings for that row.

## Task Matrix

| Task ID | Family | Turns | Reset | Validation |
| --- | --- | ---: | --- | --- |
| `docs-edit` | documentation edit | 1 | `bash task-reset.sh docs-edit` | `bash task-validate.sh docs-edit` |
| `rust-cli-flag` | small CLI feature | 1 | `bash task-reset.sh rust-cli-flag` | `bash task-validate.sh rust-cli-flag` |
| `bug-fix` | parser bug fix | 1 | `bash task-reset.sh bug-fix` | `bash task-validate.sh bug-fix` |
| `schema-report` | output/schema update | 1 | `bash task-reset.sh schema-report` | `bash task-validate.sh schema-report` |
| `multi-turn-refine` | iterative improvement | 3 | `bash task-reset.sh multi-turn-refine` | `bash task-validate.sh multi-turn-refine` |

## Task Prompts

### docs-edit

```text
You are editing the Rust fixture repository at runs/fixtures/real-coding-v1.

Task: update README.md with a short "Cache-Hit Benchmark Notes" section.

Requirements:
- Explain that token usage must record input, cached input, uncached input, and output tokens.
- Mention that cache-hit improvements should not be counted as wins if validation fails.
- Keep the change documentation-only.
- Do not change source code or tests.

After editing, run:
bash task-validate.sh docs-edit
```

Success rubric:

```text
README.md contains a cache-hit section.
No source code changes are required.
Validation passes.
```

### rust-cli-flag

```text
You are editing the Rust fixture repository at runs/fixtures/real-coding-v1.

Task: add a --json-summary option to the summarize command.

Expected behavior:
cargo run --quiet -- summarize --json-summary examples/basic.csv

It should print a single JSON object with at least:
- records
- input_tokens
- cached_input_tokens
- uncached_input_tokens
- output_tokens
- cache_hit_rate
- success_rate

Keep the existing text output working.

After editing, run:
bash task-validate.sh rust-cli-flag
```

Success rubric:

```text
Existing tests pass.
JSON summary mode exists.
Validation passes.
```

### bug-fix

```text
You are editing the Rust fixture repository at runs/fixtures/real-coding-v1.

Task: make the CSV parser ignore comment lines.

Comment lines begin with # after optional leading whitespace.

Example input:
# comment
alpha,75,25,10,true

The summarize command should treat that as one record.

After editing, run:
bash task-validate.sh bug-fix
```

Success rubric:

```text
Comment lines are ignored.
Normal CSV parsing still works.
Validation passes.
```

### schema-report

```text
You are editing the Rust fixture repository at runs/fixtures/real-coding-v1.

Task: extend the text summary with total_tokens.

Expected output must include:
total_tokens=<input_tokens + output_tokens>

Keep existing output fields unchanged.

After editing, run:
bash task-validate.sh schema-report
```

Success rubric:

```text
Text summary includes total_tokens.
Existing fields still print.
Validation passes.
```

### multi-turn-refine

Turn 1:

```text
You are editing the Rust fixture repository at runs/fixtures/real-coding-v1.

Task: improve the CLI help text so it clearly shows the summarize command and the expected CSV input path.

Run:
bash task-validate.sh base
```

Turn 2:

```text
Continue the same task.

Update README.md so it includes the exact example command:
cargo run --quiet -- summarize examples/basic.csv

Run:
bash task-validate.sh base
```

Turn 3:

```text
Finish the multi-turn refinement.

Make sure tests still pass and the README references examples/basic.csv.

Run:
bash task-validate.sh multi-turn-refine
```

Success rubric:

```text
Help text remains useful.
README includes the example fixture input.
Validation passes after turn 3.
```

## Pilot Recommendation

Start with:

```text
docs-edit
bug-fix
```

These two tasks are small but exercise different surfaces:

```text
docs-edit: low tool complexity, documentation-only
bug-fix: source edit + validation command
```

If both are clean, continue to the full five-task matrix.
