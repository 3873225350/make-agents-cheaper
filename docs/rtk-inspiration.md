# RTK Inspiration For make-agents-cheaper

Source inspected: <https://github.com/rtk-ai/rtk>

## What RTK Does Differently

RTK is a runtime tool-output optimizer. It integrates with agents through hooks or plugins, rewrites commands such as `git status` or `cargo test` to `rtk git status` or `rtk cargo test`, and returns compact output to the LLM.

Key design choices worth borrowing:

1. **Transparent hook path.** Users keep typing normal commands; hooks rewrite them before execution.
2. **Single rewrite registry.** Agent-specific hooks are thin delegates. The Rust binary owns the command-pattern registry and output filters.
3. **Graceful degradation.** If RTK is missing, hook JSON is malformed, or rewrite fails, the command runs unmodified.
4. **Install-first product surface.** Homebrew, install script, `cargo install --git`, verification commands, and release binaries are all first-class.
5. **Evidence surface around command output.** RTK's savings are mostly about fewer tool-output tokens reaching the model, not prompt-cache hit rate.

## Why Prefix-Only Can Still Get More Expensive

Our current cache-friendly condition only changes Claude Code prompt assembly:

```bash
--exclude-dynamic-system-prompt-sections
```

That can preserve a longer stable prefix, but it does not control:

- how many Claude Code model/tool turns happen;
- how verbose tool outputs are;
- whether validation/test output is compact;
- whether a cache-friendly run takes a different behavioral path.

The original V2 negative row was exactly this failure mode. The candidate cache-friendly run succeeded, but it used 14 turns while the matched baseline used 6 turns. The extra turns repeated a large cached context and still added more paid uncached input.

After fixing prompt paths and fixture-local Git isolation, the bounded 3-repeat V2 diagnostic returned to the expected prefix-cache direction: uncached input fell from 30,082 to 7,817 tokens with task success preserved. That does not make the RTK-style layer unnecessary. It clarifies the layering:

- prefix layout reduces paid uncached input for repeated context;
- tool-output compaction reduces the volume of tool text fed back to the model;
- behavior budgets reduce extra agent turns.

RTK points to the next layer: after prefix caching, we also need tool-output and behavior-budget controls.

## Concrete Roadmap Borrowed From RTK

1. Add a Claude Code hook or workflow that rewrites noisy shell commands to compact equivalents:

```text
git status       -> make-agents-cheaper compact git status
cargo test       -> make-agents-cheaper compact cargo test
rg PATTERN DIR   -> make-agents-cheaper compact rg PATTERN DIR
```

2. Keep hooks as thin delegates:

```text
Claude hook -> make-agents-cheaper rewrite-command -> compact command
```

3. Keep failure non-blocking. If rewrite fails, run the original command and mark the run as unoptimized instead of blocking the task.

4. Extend reports with behavior budgets:

- turns to completion;
- output tokens;
- tool-output size where observable;
- candidate/baseline turn ratio;
- candidate/baseline uncached ratio.

5. Split paper claims:

- Prompt-cache result: dynamic state placement can reduce paid uncached input when behavior remains comparable.
- Runtime-output result: compact tool outputs reduce context fed back to the model.
- Combined agent-cost result: cache-friendly prefix plus compact tool-output and behavior budget.

## Immediate Experiment Design

Do not compare only `baseline` vs `cache-friendly`. Add a 2x2 design:

| Condition | Prompt-prefix policy | Tool-output policy |
| --- | --- | --- |
| baseline | ordinary Claude Code | raw tool outputs |
| prefix-only | `--exclude-dynamic-system-prompt-sections` | raw tool outputs |
| output-only | ordinary Claude Code | compact shell outputs |
| combined | cache-friendly prefix | compact shell outputs |

This design can tell whether V2 regressed because the prefix method failed, because agent behavior changed, or because tool-output/context expansion dominated the savings.
