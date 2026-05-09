# Subtask 00: Checkpoint Current Work

## Goal

Create a clean checkpoint before larger experiments change files, logs, and paper text.

## Why This Comes First

The project now has two active repositories:

```text
make-agents-cheaper:
  audit/eval tool and paper-facing experiment code

make-agents-cheaper-skill:
  reusable skill packaging, currently with cheaper-skill-for-claude
```

Before scaling the evaluation suite, keep these boundaries clear so later results can be traced back to a stable code and documentation state.

## Work Items

- Review dirty files in `make-agents-cheaper`.
- Separate intended experiment/tooling changes from unrelated local edits.
- Commit the new `make-agents-cheaper-skill` repository.
- Optionally commit a roadmap/documentation checkpoint in `make-agents-cheaper`.
- Record any uncommitted files that are intentionally left alone.

## Acceptance Criteria

- `git status --short` is reviewed for both repositories.
- Commit boundaries are clear.
- No unrelated user changes are reverted or swept into an experiment commit.

## Status

```text
done
```

## Checkpoint Notes

- `make-agents-cheaper-skill` was committed as a separate local repository.
- `make-agents-cheaper` checkpoint includes audit/eval code, docs, paper scaffold, and task planning files.
- `references/keep-codex-fast/` is an external nested git repository kept as a local reference and ignored instead of committed as ordinary source.
- Validation command: `cargo test`.
