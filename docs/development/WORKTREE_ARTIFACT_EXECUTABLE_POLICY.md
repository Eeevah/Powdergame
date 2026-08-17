# Worktree, Artifact, and Executable Policy

Date: 2026-08-17

This document is the authority for Powdergame development worktrees, launchers,
executables, and evidence copies. It is an operating policy, not a Gate result.

## Canonical user application

Powdergame maintains one user-facing application binary. For the current active
feature worktree it is:

```text
C:\Users\mdkap\source\repos\Powdergame-g8b\target\release\powdergame-windows.exe
```

The ordinary user entry point is `run_powdergame.bat`. The repository launcher
builds that canonical binary and can select an existing application mode:

```bat
run_powdergame.bat
run_powdergame.bat pressure
run_powdergame.bat activity
run_powdergame.bat gallery
```

Movement, Density, Thermal, Pressure, Parallel Integrity, Activity/Sleep,
Gallery, Observatory, and future Scenario or Gate surfaces belong in the same
`powdergame-windows.exe` as a menu, CLI argument, or mode. They do not receive a
separate user application executable.

The retired `run_g5_demo.bat` was removed after `run_powdergame.bat pressure`
became its direct replacement. Do not add new Gate-specific `run_*.bat` files.
Ordinary launch uses `run_powdergame.bat`; automated scenario evidence uses
`run_experiment.bat`.

## Developer-only executables and build cache

`powdergame-benchmark.exe` is the developer-only exception because headless G8
performance measurement is a distinct responsibility. Do not copy it to other
folders or publish it as the user application.

Cargo-generated test executables under `target/**/deps` are internal build
cache. Do not treat, copy, archive, or preserve them as user executables.

Before implementing any additional executable, document all of the following:

- why the existing application cannot own the function;
- why both executables must coexist;
- the function and owner of the additional executable;
- its retention deadline;
- the condition that removes it.

## Evidence and archive policy

Checkpoint, evidence, and archive directories do not preserve executable copies
by default. Record the source SHA and executable SHA-256 instead. A frozen binary
is permitted only when an explicit evidence contract requires binary inclusion;
that exception must state its purpose and removal or retention condition.

The post-remediation Experiment Harness source/binary seal is an explicit
evidence exception. Each newly executed Harness Run, including scratch mode,
executes a create-new frozen copy kept only inside that unique Run directory;
the copy is not a user installation or canonical application binary. Candidate
mode additionally transports that same copy in the sibling Audit Bundle, while
scratch mode creates no Audit Bundle. Retain the frozen copy with its immutable
Run and remove it only when the entire Run is explicitly retired under an
artifact-retention decision; never publish or copy it as a user application.

## Closing a worktree

Clean up a finished worktree in this order:

1. verify the worktree is clean;
2. verify its branch is pushed and upstream-equal;
3. preserve only the artifacts required by an explicit evidence contract;
4. run `cargo clean` in that finished worktree;
5. remove the worktree;
6. confirm no obsolete release executable remains.

Never run this cleanup sequence on the active worktree. A task-specific
instruction that prohibits `cargo clean` or worktree removal takes precedence.

## Completion report

Every worktree or release-surface completion report records:

- the canonical application binary path;
- the number of user-facing application binary copies;
- developer-only executables retained;
- launchers created or removed;
- every executable-copy exception, its reason, and its removal/retention
  condition.
