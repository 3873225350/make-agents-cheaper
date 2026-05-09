# Subtask 02: Build Real Coding Fixture Repo

## Goal

Create a small fixture repository for realistic coding-agent tasks, without letting benchmark tasks mutate the main `make-agents-cheaper` codebase.

Recommended location:

```text
runs/fixtures/real-coding-v1/
```

This directory should remain ignored by git.

## Fixture Requirements

- Small Rust CLI project.
- Deterministic tests.
- Simple README and docs.
- A few intentionally small extension points.
- One optional failing-test branch or reset script.

## Suggested Fixture Shape

```text
runs/fixtures/real-coding-v1/
  Cargo.toml
  src/main.rs
  tests/cli.rs
  README.md
  task-reset.sh
  task-validate.sh
```

## Work Items

- Create the fixture project.
- Add a known-good initial state.
- Add reset scripts so every task starts clean.
- Add validation commands.
- Record fixture setup in the experiment manifest.

## Acceptance Criteria

- `cargo test --locked` passes in the fixture.
- Each task can reset the fixture before running.
- No fixture artifacts are committed accidentally.

## Status

```text
done
```

## Completion Evidence

- Fixture created at `runs/fixtures/real-coding-v1/`.
- Baseline reset command exists: `bash task-reset.sh base`.
- Baseline validation command exists: `bash task-validate.sh base`.
- `cargo test --locked` passes in the fixture.
