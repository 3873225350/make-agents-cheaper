# AGENTS.md

Guidance for planning files under `taskplan/`.

## Role

`taskplan/` is the project memory for long-running work.

Use it to record:

- roadmap state;
- subtask status;
- experiment milestones;
- why a decision was made;
- what remains blocked or untested.

## Status Format

Use consistent status markers:

```text
[ ] planned
[~] in progress or scaffolded
[x] done
```

For subtask files, keep a final `## Status` block with a short machine-readable word such as:

```text
planned
documented
fixture scaffolded
done
```

## Evolution Notes

Create an `evolution-YYYY-MM-DD-<topic>.md` note when:

- a new experiment suite is defined;
- a checkpoint is committed;
- raw trace capture changes;
- a result changes the interpretation;
- a product boundary changes.

Keep evolution notes factual. Do not invent results.

## Roadmap Updates

When a task is completed:

- update `roadmap.md`;
- update the relevant `subtasks/*.md`;
- mention validation commands if any ran;
- record whether files are tracked or intentionally ignored.

## Product Boundary

Planning must distinguish:

- audit/eval main evidence;
- skill usability/reproducibility artifact;
- future native agent framework work.
