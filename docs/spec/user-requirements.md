# task-marshal — User Requirements

**Document ID:** TM-REQ-001
**Status:** Draft
**Audience:** Developer, AI agents consuming the CLI interface

---

## 1. Purpose

`task-marshal` is a command-line tool that provides a single, stable interface for retrieving and completing tasks, regardless of where those tasks originate. It acts as a broker between multiple task sources (local files, databases, remote services) and the consumers of those tasks — whether human developers or AI coding agents.

---

## 2. Problem Statement

AI coding agents (Copilot, Claude Code, CogWorks) currently retrieve their task context by reading source files directly. This is wasteful on two fronts: it consumes tokens ingesting content that deterministic logic could resolve, and it couples the agent's bootstrap behaviour to the format and location of task sources. As Copilot moves to per-token pricing, the cost of this pattern becomes material.

Additionally, as the number of task sources grows (local task files, BEADS, GitHub Issues), agents need a consistent way to receive and complete tasks without needing to understand each source's format or API.

---

## 3. Goals

- Provide a single CLI interface for retrieving the next actionable task from any configured source.
- Abstract source-specific logic entirely away from the consuming agent or developer.
- Allow tasks to be marked as done through the same interface, with the completion propagated back to the originating source.
- Reduce agent token consumption at bootstrap by returning only the information needed for a single task.
- Support deterministic, auditable task selection — the same state always produces the same result.

---

## 4. Non-Goals

- `task-marshal` is not a task creation tool. Tasks are authored in their respective sources.
- `task-marshal` is not a project management UI.
- `task-marshal` is not a dark factory service. It is a local development tool. Lessons learned may inform a future service equivalent, but that is out of scope here.
- `task-marshal` does not manage agent lifecycle or orchestration.

---

## 5. Users

| User | Description |
|---|---|
| AI coding agent | Calls `task-marshal next` at session start via a skill file; calls `task-marshal done` on completion. |
| Developer | Uses `task-marshal next` and `task-marshal list` to understand what work is queued; uses `task-marshal done` to close out tasks without touching source systems manually. |

---

## 6. Task Sources

`task-marshal` must support the following task sources:

### 6.1 Local task file (`local`)

A markdown file (`.llm/tasks.md` by default) containing structured task blocks. This is the primary source for locally-defined agent work.

### 6.2 BEADS database (`beads`)

The BEADS local database. Connection details are defined in configuration.

### 6.3 GitHub Issues (`github`)

Issues from a configured GitHub repository. Only issues matching a configured label filter are considered (e.g. `agent-task`). Requires the `gh` CLI to be authenticated in the environment.

### 6.4 Future sources

The architecture must allow additional sources to be added without changing the CLI interface.

---

## 7. Task IDs

Every task returned by `task-marshal` must carry a source-encoded ID. This ID is opaque to the consumer but must be sufficient for `task-marshal` to route completion back to the correct source without any additional context.

**Format:** `<source>:<identifier>`

Examples:

- `local:TASK-042`
- `beads:a3f9c2`
- `gh:187`

---

## 8. Commands

### 8.1 `task-marshal next`

Returns the single highest-priority actionable task from the configured sources, in priority order.

**Options:**

| Flag | Description |
|---|---|
| `--role <role>` | Filter to tasks assigned to a specific agent role (e.g. `coder`, `tester`, `architect`). |
| `--source <source>` | Restrict lookup to a specific source (`local`, `beads`, `github`). Bypasses priority order. |

**Output:** A self-contained task block including task ID, title, description, acceptance criteria, and any role or dependency context needed to complete the work. Nothing more.

**Selection rules (deterministic):**

1. Consult sources in configured priority order (unless `--source` is specified).
2. Within a source, filter to tasks whose dependencies are all in a completed state.
3. Apply role filter if `--role` is provided.
4. Select the highest-priority unblocked task by priority field, then by source order.
5. Return exactly one task. If no task is found, exit with a clear message and a non-zero exit code.

### 8.2 `task-marshal done <id>`

Marks the specified task as complete and propagates the completion back to its source.

**Options:**

| Flag | Description |
|---|---|
| `--comment <text>` | Attach a completion note. Written to the source where supported (GitHub issue comment, task file annotation). |

**Behaviour by source:**

- `local`: Updates the state marker of the task in the task file in-place.
- `beads`: Updates the task state via the BEADS API.
- `github`: Closes the issue and posts a comment if `--comment` is provided.

### 8.3 `task-marshal show <id>`

Displays the full task block for a given task ID. Useful for re-inspecting a task mid-session without calling `next` again.

### 8.4 `task-marshal list`

Lists all actionable tasks across all sources (or a filtered subset).

**Options:**

| Flag | Description |
|---|---|
| `--source <source>` | Restrict to a specific source. |
| `--role <role>` | Filter by role. |
| `--all` | Include blocked and completed tasks. |

Output is a summary table: ID, source, priority, role, title.

---

## 9. Configuration

`task-marshal` is configured via a TOML file. Default location: `.llm/task-marshal.toml`, resolved from the working directory upward (similar to `.git` discovery).

```toml
[sources]
priority = ["local", "beads", "github"]

[sources.local]
path = ".llm/tasks.md"

[sources.beads]
db_path = ".llm/beads.db"

[sources.github]
repo = "owner/repo"
labels = ["agent-task"]
```

All source sections are optional. Sources absent from configuration are ignored.

---

## 10. Skill File Integration

`task-marshal` must be usable from agent skill files with minimal instruction. The consuming skill file should require no knowledge of task sources, formats, or selection logic. A conforming skill file looks like:

```markdown
## Finding Your Task

Run `task-marshal next --role <your_role>` to receive your task.
Work the task. When acceptance criteria are met, run `task-marshal done <id>`.
Do not read task source files directly.
```

This is the complete agent interface. All other behaviour is encapsulated in the binary.

---

## 11. Behaviour Requirements

- **Deterministic selection.** Given the same source state, `task-marshal next` must always return the same task. There must be no randomness or LLM involvement in selection.
- **Atomic done.** `task-marshal done` must either fully propagate completion to the source or fail with a clear error. It must not leave a task in an ambiguous state.
- **Graceful source failure.** If a source is unavailable (e.g. no network for GitHub), `task-marshal` must skip that source, log a warning, and continue to the next source in priority order. It must not crash.
- **Non-zero exit on no task.** If `task-marshal next` finds no actionable task, it exits with a non-zero code and a human-readable message. This allows agent skill files to detect and handle the empty-queue case.
- **No side effects on `next`.** Calling `task-marshal next` must not modify any task source. It is a pure read operation.
- **Human-readable output by default.** All output is plain text suitable for both human reading and LLM context ingestion. No ANSI colour codes in task output (warnings and errors to stderr are exempt).

---

## 12. Out-of-Scope for v1

- Task creation
- Task editing
- Dependency graph visualisation
- Multi-task batch operations
- Dark factory / service mode
- Authentication management (delegated to `gh` CLI and environment variables)
