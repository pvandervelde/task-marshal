# Domain Vocabulary

This document defines the canonical terms used throughout all spec documents, ADRs, and ultimately the implementation. The interface designer must use these exact names (adapted to language conventions) when defining types.

---

## Core Task Concepts

### Task

A discrete unit of work to be performed by an agent or developer.

- **Identified by:** `TaskId`
- **Contains:** title, description, acceptance criteria, state, priority, optional role, optional list of dependencies
- **Returned by:** `next` (as `TaskBlock`) and `show` (as `TaskBlock`)
- **Constraint:** A task must be self-contained — the consumer must not need to read any other resource to act on it

### TaskId

The globally unique, source-encoded identifier for a task.

- **Format:** `<SourceKey>:<NativeId>`
- **Examples:** `local:TASK-042`, `beads:bd-a1b2`, `gh:187`
- **Property:** Opaque to the consumer — the consumer must not parse or interpret it, only pass it back to `done` or `show`
- **Property:** Self-routing — task-marshal can determine which source to contact purely from the TaskId, with no additional context

### NativeId

The identifier used by the source system for a task.

- **For `local`:** Task heading identifier (e.g., `TASK-042`)
- **For `beads`:** BEADS hash ID (e.g., `bd-a1b2`, may include hierarchy like `bd-a1b2.3`)
- **For `gh`:** GitHub issue number (e.g., `187`)
- **Constraint:** Unique within a source; must be stable (does not change after task creation)

### SourceKey

The source prefix component of a `TaskId`. Determines which adapter handles the task.

- **Valid values (v1):** `local`, `beads`, `gh`
- **Constraint:** Must match a key in the configured sources

### TaskBlock

The full, self-contained task content returned by `next` and `show`.

- **Contains:** TaskId, title, description, acceptance criteria, TaskState, Priority, optional Role, optional list of Dependency TaskIds, optional context notes
- **Output channel:** stdout
- **Constraint:** Must be plain text; no ANSI color codes; suitable for direct inclusion in an agent's context window

### TaskSummary

Abbreviated task information used in `list` output.

- **Contains:** TaskId, SourceKey, Priority, optional Role, title, TaskState
- **Output:** One row per task in a summary table
- **Does not include:** full description, acceptance criteria

### TaskState

The lifecycle state of a task.

| Value | Meaning |
|---|---|
| `unstarted` | Not yet begun; eligible for selection |
| `in_progress` | Currently being worked; eligible for selection |
| `blocked` | Has unresolved dependencies; not eligible for selection |
| `done` | Completed; not eligible for selection (unless `--all`) |

- **Constraint:** Only `unstarted` and `in_progress` tasks are eligible to be returned by `next`
- **Constraint:** A task is `blocked` if any of its dependencies are not in `done` state

### Priority

An integer representing urgency. Lower number = higher priority.

| Value | Convention |
|---|---|
| 0 | P0 — critical / blocking |
| 1 | P1 — high |
| 2 | P2 — normal |
| 3+ | P3+ — low |

- **Selection rule:** Tasks with lower priority integer are preferred over higher
- **Constraint:** Priority must be a non-negative integer; default is 2 if unspecified
- **Source alignment:** Matches BEADS priority convention (`bd create -p 0`)

### Role

An agent role label that restricts which agent type should work a task.

- **Examples:** `coder`, `tester`, `architect`, `reviewer`
- **Constraint:** Free-form string; case-insensitive comparison
- **Usage:** Task may specify a role; `next --role X` filters to matching tasks only
- **Unassigned tasks:** A task with no role is eligible for any role filter (it is returned regardless of `--role`)

### Dependency

A reference to another task (by NativeId, within the same source) that must reach `done` state before the depending task is eligible for selection.

- **v1 constraint:** Dependencies are within-source only. Cross-source dependency is out of scope.
- **Representation in local file:** List of NativeIds in the task's metadata block
- **Representation in BEADS:** Native BEADS dependency graph — resolved by `bd ready`
- **Representation in GitHub:** Not supported in v1; all GitHub issues are treated as unblocked

---

## Selection Concepts

### SelectionFilter

The combined set of criteria applied when selecting tasks.

- **Contains:** optional Role (from `--role`), optional SourceKey restriction (from `--source`)
- **Applied by:** TaskSelector during `next` and `list`

### SortKey

The composite value used to establish deterministic ordering among candidate tasks.

- **Primary:** Priority (ascending, lower = better)
- **Secondary:** Source order (index in the configured priority list)
- **Tertiary:** NativeId (lexicographic, ascending) — tiebreaker for stability across calls
- **Constraint:** Sort must be stable and deterministic; no randomness

---

## Configuration Concepts

### Config

The complete parsed configuration for task-marshal.

- **Source:** `task-marshal.toml` at the repo root, discovered by walking up from the current working directory
- **Contains:** SourcesConfig

### SourcesConfig

The configuration for all task sources.

- **Contains:** priority order (list of SourceKeys), per-source configurations

### LocalSourceConfig

Configuration for the local markdown source.

- **Contains:** path (default: `.llm/tasks.md`, relative to the directory containing the config file)

### BeadsSourceConfig

Configuration for the BEADS source.

- **Contains:** optional `beads_dir` (path to the `.beads/` directory; if absent, `bd` uses its own discovery via git root)
- **Note:** `bd` binary must be in `PATH`; no credential configuration — BEADS manages its own auth

### GithubSourceConfig

Configuration for the GitHub Issues source.

- **Contains:** repo (`owner/repo`), labels (list of label strings to filter issues)
- **Note:** `gh` binary must be in `PATH` and authenticated; no credential configuration

---

## Error Concepts

### SourceUnavailable

A source could not be contacted (binary not found, network error, database locked).

- **Behaviour:** task-marshal logs a warning to stderr, skips the source, continues with remaining sources
- **Does not cause:** process exit

### NoTaskFound

No actionable task was found across all consulted sources after all filters were applied.

- **Behaviour:** task-marshal prints a human-readable message to stderr, exits with code 1

### InvalidTaskId

A `TaskId` provided to `done` or `show` could not be parsed or does not match a known SourceKey.

- **Behaviour:** task-marshal prints an error to stderr, exits with code 2

### CompletionError

The source adapter failed to propagate a `done` state change to the source.

- **Behaviour:** task-marshal prints an error to stderr, exits with code 2
- **Constraint:** The task must not be left in a partially-updated state (atomic operation)

### ConfigError

The configuration file could not be found, read, or parsed.

- **Behaviour:** task-marshal prints an error to stderr, exits with code 2
- **Note:** A missing config file (no `task-marshal.toml` found in walk-up) is a `ConfigError`

---

## Operational Concepts

### Config Discovery

The process of locating `task-marshal.toml` by starting at the current working directory and walking up to the filesystem root. The first file found wins.

### Source Priority Order

The configured list of SourceKeys that defines the order in which sources are consulted during `next`. When `--source` is specified, priority order is bypassed.

### Atomic Completion

The guarantee that `done` either fully updates the source or fails cleanly, never leaving a task in an ambiguous intermediate state.

- **For `local`:** Write to a temporary file in the same directory, then atomically rename over the original
- **For `beads`:** Single `bd close` command; atomicity is guaranteed by BEADS
- **For `gh`:** Single `gh issue close` command; atomicity guaranteed by GitHub API

### TaskBlock Format

Plain text. No ANSI escape codes. Sections:

```
Task: <TaskId>
Title: <title>

Description:
<description>

Acceptance Criteria:
<criteria>

[Role: <role>]
[Priority: P<n>]
[Depends on: <id>, <id>]
```
