# Phenomena Analysis Log

This note records what we have observed so far and how to interpret it.

## 1. Is The Claude Experiment Based On Mimo?

Current experiments use:

```text
agent harness: Claude Code
model: mimo-v2.5-pro
command surface: claude --print --output-format json
```

So yes, the measured runs are based on Claude Code driving `mimo-v2.5-pro`.

But the mechanism is not "Mimo-only." The mechanism is:

```text
agent harness prompt assembly
+ provider-side prompt cache accounting
+ stable prefix reuse
```

`mimo-v2.5-pro` matters because it is the model used in the current route, and Claude Code JSON exposes usage fields for it:

```text
modelUsage.mimo-v2.5-pro.inputTokens
modelUsage.mimo-v2.5-pro.cacheReadInputTokens
modelUsage.mimo-v2.5-pro.cacheCreationInputTokens
modelUsage.mimo-v2.5-pro.outputTokens
modelUsage.mimo-v2.5-pro.costUSD
```

The broader claim should be written as harness-level:

```text
When an agent framework exposes or supports stable prompt assembly, moving dynamic harness state later can improve prompt cache reuse.
```

Do not write the claim as:

```text
Mimo itself makes this method work.
```

The current evidence is:

```text
Claude Code + mimo-v2.5-pro + observable cache accounting
```

Future evidence should test other models and providers.

## 2. Is The Current Task Dataset Too Simple?

Yes. The current task suite is intentionally simple.

It is a mechanism test, not a full coding-agent benchmark.

Why it is useful:

- It isolates prompt-cache behavior from complex task success.
- It keeps generation length short, so cost differences mostly come from input cache behavior.
- It makes exact validation cheap and reproducible.
- It helped reveal the core phenomenon: baseline cache hit can collapse under dynamic drift while cache-friendly prompt assembly remains high.

Why it is not enough:

- It uses exact-reply tasks rather than real code edits.
- It does not test tool use, file edits, compile/test loops, or multi-turn recovery.
- It does not measure real developer usefulness.
- One task already showed weak-output-contract failure: the model produced a verbose answer instead of the expected string.

So the current dataset can support:

```text
mechanism claim
```

but not yet:

```text
full coding-agent productivity claim
```

Next datasets should add:

| Dataset | Task type | Validation |
| --- | --- | --- |
| `micro-exact` | exact-reply mechanism tasks | exact string |
| `docs-edit` | modify one Markdown section | diff + grep |
| `rust-cli-small` | add one CLI option or parser behavior | `cargo test --locked` |
| `bug-fix` | repair one failing test | project tests |
| `multi-file-feature` | code + docs + tests | test suite + review |
| `long-session` | 4-6 turn iterative task | same validation each turn |

The right paper wording is:

```text
We first use controlled micro-tasks to isolate prompt-cache effects, then evaluate whether the effect persists on realistic coding tasks.
```

## 3. How Do We Reorder Prompt Without Changing Claude Source?

In the current experiments, we do not edit Claude Code source.

We use an official Claude Code CLI switch:

```bash
--exclude-dynamic-system-prompt-sections
```

This flag moves per-machine or dynamic sections such as cwd, environment, memory paths, and git status out of the early system prompt and into the first user message.

The important detail:

```text
Claude Code does the actual movement internally.
```

We are not intercepting a request and rewriting it ourselves. We are asking Claude Code to use a cache-friendlier prompt assembly mode that it already exposes.

So the chain is:

```text
our benchmark/wrapper command
  -> passes --exclude-dynamic-system-prompt-sections
  -> Claude Code internal prompt assembler changes section placement
  -> provider sees a more stable early prefix
  -> prompt cache can reuse more tokens under dynamic drift
```

Without the flag, Claude Code may assemble something like:

```text
system prompt:
  stable instructions
  tool definitions
  cwd / env / memory paths / git status

user message:
  latest user request
```

With the flag, the intended assembly is closer to:

```text
system prompt:
  stable instructions
  tool definitions

first user message:
  cwd / env / memory paths / git status
  latest user request
```

The dynamic information is still sent to the model. The method does not hide context. It changes where dynamic context appears in the request.

That means the early prefix becomes more stable:

```text
ordinary:
  [stable system][tools][dynamic cwd/git/env][user]

cache-friendly:
  [stable system][tools][stable policy][dynamic cwd/git/env later][user]
```

Prompt cache matching is prefix-sensitive. If the early dynamic section changes, the reusable prefix becomes short. If the dynamic section moves later, more of the stable prefix remains reusable.

So the current method is:

```text
official CLI option
+ wrapper/runbook discipline
+ benchmark logging
```

not:

```text
Claude source patch
```

and not primarily:

```text
hook-based rewriting
```

The precise wording for the project is:

```text
We use agent-exposed prompt assembly controls to make dynamic harness state appear later in the request, preserving a longer stable prefix for provider-side prompt caching.
```

## 4. Are Hooks The Mechanism?

Not in the current implementation.

Hooks can be useful for:

- recording metadata;
- enforcing benchmark commands;
- writing experiment logs;
- checking whether drift probes were cleaned up;
- preventing accidental tool/MCP changes during a run.

But hooks are risky as a prompt-cache optimization if they inject changing text into the early prompt. A dynamic hook can itself become a source of prefix drift.

For this project, hooks should be treated as:

```text
measurement and guardrail layer
```

not the primary prompt-reordering layer.

The primary layer should be:

```text
agent-supported prompt assembly controls
wrapper commands
skills / runbooks
stable configuration
```

For a future native agent such as `cheapcode`, we can implement this directly in the prompt assembler:

```text
stable system policy first
stable tool schema second
stable repo rules third
dynamic cwd/git/env later
latest user request last
```

## 5. Observed Phenomena So Far

### Cold Start

Changing prompt shape creates a cold prefix.

Observed pattern:

```text
first run after changing prompt structure: low cache read, high cost
second run with same structure: high cache read, low cost
```

### Warm-State Control

When no dynamic drift is introduced and both conditions are warm, both baseline and cache-friendly can hit cache well.

This prevents overclaiming:

```text
cache-friendly is not always cheaper
```

### Dynamic Drift

When git status changes, baseline can lose most of its early-prefix cache reuse.

Observed paired drift run:

```text
dynamic drift only:
  baseline cache hit: 15.24%
  cache-friendly cache hit: 91.39%
```

This supports the mechanism:

```text
moving dynamic harness state later protects the stable prefix
```

### Multi-Task V1

Across five small prompt families:

```text
baseline cache hit: 60.93%
cache-friendly cache hit: 92.77%
```

But there was a quality failure:

```text
paper-positioning cache-friendly main run produced a verbose answer
```

So the correct interpretation is:

```text
The cache-hit mechanism generalizes across several prompt families, but task quality must remain part of the evaluation.
```

## 6. Logging Rule

Every experiment should record:

- command line;
- model and provider route;
- prompt shape;
- whether warm-up was excluded;
- drift state;
- raw trace path;
- extracted JSONL metrics;
- validation result;
- anomaly notes.

Never claim savings from a run unless:

```text
cache accounting is observable
task success does not regress
warm-up and measured calls are separated
```
