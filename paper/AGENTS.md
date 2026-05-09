# AGENTS.md

Guidance for paper work under `paper/`.

## Paper Object

The paper studies harness-level cache-aware prompting for coding-agent workflows.

Current experimental setting:

```text
Claude Code harness routed to MiMo-compatible models
```

Codex is the development assistant used to build the project. Do not present Codex as the studied harness unless a Codex-specific experiment is actually run.

## Claim Discipline

Every claim needs one of:

- experiment logs;
- eval JSONL;
- trace-derived request-shape evidence;
- an explicit statement that it is a hypothesis or design motivation.

Use conservative wording:

```text
can reduce observed uncached input
```

not:

```text
makes agents cheap
```

## Evidence Hierarchy

- Main evidence: `make-agents-cheaper` audit/eval logs.
- Mechanism evidence: raw `claude-trace` request/response captures converted into safe artifacts.
- Artifact evidence: `cheaper-skill-for-claude` usability ablation.
- Background/motivation: DeepSeek/MiMo/cache-hit discussion and trace observations.

## Safety

Never paste full raw prompts, system prompts, API headers, user files, or raw trace bodies into the paper.

Use:

- aggregate tables;
- hashes/fingerprints;
- small redacted examples;
- exact metric definitions.

## Writing Style

Preserve the user's direct research motivation. Do not inflate it into generic AI-paper language.

Prefer simple wording around the core idea:

```text
structure the prompt so stable components come first and dynamic components come later
```
