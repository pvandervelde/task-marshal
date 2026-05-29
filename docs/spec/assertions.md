# Behavioral Assertions

Testable assertions for every behavioral requirement. Each assertion follows Given/When/Then format. These drive test coverage requirements for the interface designer and coder.

---

## `task-marshal next`

### A1 — Returns exactly one task when tasks are available

**Given:** At least one eligible (unblocked, state not `done`) task exists across configured sources
**When:** `task-marshal next` is invoked
**Then:** Exactly one `TaskBlock` is written to stdout
**And:** Process exits with code 0

---

### A2 — Exits non-zero when no task is found

**Given:** No eligible task exists in any configured source
**When:** `task-marshal next` is invoked
**Then:** A human-readable message is written to stderr
**And:** Nothing is written to stdout
**And:** Process exits with code 1

---

### A3 — Deterministic: same state always returns same task

**Given:** A fixed, known set of tasks in a fixed state
**When:** `task-marshal next` is invoked multiple times without any state change
**Then:** The same `TaskId` is returned on every invocation
**And:** No side effects occur between invocations

---

### A4 — Pure read: `next` does not modify any source

**Given:** A configured source with tasks
**When:** `task-marshal next` is invoked
**Then:** No task source is modified (file unchanged, no `bd update`, no GitHub API write)

---

### A5 — Role filter: returns only tasks matching the given role

**Given:** Tasks with roles `coder` and `tester` exist
**When:** `task-marshal next --role coder` is invoked
**Then:** Only tasks with role `coder` are eligible
**And:** Unassigned tasks (no role set) are also eligible

---

### A6 — Role filter: unassigned tasks are eligible under any role filter

**Given:** A task with no role assigned
**When:** `task-marshal next --role architect` is invoked
**Then:** The unassigned task is eligible for selection

---

### A7 — Source filter: `--source` bypasses priority order

**Given:** Sources configured in order `local`, `beads`, `github`; a task exists only in `github`
**When:** `task-marshal next --source github` is invoked
**Then:** The GitHub task is returned
**And:** `local` and `beads` are not consulted

---

### A7b — Source filter: no task in specified source exits 1 without fallback

**Given:** Sources configured with tasks in `local` only; `beads` source has no tasks
**When:** `task-marshal next --source beads` is invoked
**Then:** Process exits with code 1 with a "no task found" message
**And:** `local` is not consulted as a fallback

---

### A8 — Source priority order is respected

**Given:** An eligible task exists in both `local` (P1) and `beads` (P1), with `local` first in priority
**When:** `task-marshal next` is invoked
**Then:** The `local` task is returned (source order tiebreaker)

---

### A9 — Priority ordering: lower number wins

**Given:** Two eligible tasks: `local:TASK-001` at P2 and `local:TASK-002` at P0
**When:** `task-marshal next` is invoked
**Then:** `local:TASK-002` (P0) is returned

---

### A10 — Blocked tasks are excluded

**Given:** Task `local:TASK-010` depends on `TASK-009`, which is not `done`
**When:** `task-marshal next` is invoked
**Then:** `local:TASK-010` is not returned
**And:** If another unblocked task exists, that task is returned instead

---

### A10b — `in_progress` tasks are eligible for selection

**Given:** Task `local:TASK-005` has state `in_progress`; no higher-priority unstarted task exists
**When:** `task-marshal next` is invoked
**Then:** `local:TASK-005` is returned
**And:** Process exits with code 0

---

### A11 — Unavailable source is skipped gracefully

**Given:** `beads` is configured but `bd` is not in PATH
**When:** `task-marshal next` is invoked
**Then:** A warning is written to stderr identifying the unavailable source
**And:** Remaining sources are consulted normally
**And:** If a task exists in another source, it is returned (exit 0)
**And:** Process does not crash or exit with code 2

---

### A12 — All sources unavailable with no task found

**Given:** All configured sources are unavailable
**When:** `task-marshal next` is invoked
**Then:** A warning per unavailable source is written to stderr
**And:** A "no task found" message is written to stderr
**And:** Process exits with code 1

---

## `task-marshal done`

### A13 — Successful completion for `local` source

**Given:** Task `local:TASK-042` exists with state `unstarted`
**When:** `task-marshal done local:TASK-042` is invoked
**Then:** The task's state is updated to `done` in the local file
**And:** The file content is otherwise unchanged
**And:** A confirmation message is written to stdout
**And:** Process exits with code 0

---

### A14 — Successful completion for `beads` source

**Given:** Task `beads:bd-a1b2` exists in the BEADS database
**When:** `task-marshal done beads:bd-a1b2` is invoked
**Then:** `bd close bd-a1b2` is executed
**And:** BEADS reflects the task as closed
**And:** Process exits with code 0

---

### A15 — Successful completion for `github` source

**Given:** Issue `gh:187` is open in the configured GitHub repository
**When:** `task-marshal done gh:187` is invoked
**Then:** `gh issue close 187` is executed
**And:** The issue is closed on GitHub
**And:** Process exits with code 0

---

### A16 — Completion with `--comment` attaches note

**Given:** Task `gh:187` in GitHub source
**When:** `task-marshal done gh:187 --comment "Implemented via PR #42"` is invoked
**Then:** `gh issue close 187` and `gh issue comment 187 --body "..."` are both executed
**And:** The comment appears on the GitHub issue
**And:** Process exits with code 0

---

### A17 — `done` with comment for `local` source annotates the task

**Given:** Task `local:TASK-042` in the local file
**When:** `task-marshal done local:TASK-042 --comment "Done, see commit abc123"` is invoked
**Then:** The task state is updated to `done`
**And:** The comment text is appended as an annotation in the task block
**And:** Atomic write is used (no partial file state)

---

### A18 — Atomic completion: failure does not leave partial state

**Given:** Task `local:TASK-042` exists
**When:** `task-marshal done local:TASK-042` is invoked and an I/O error occurs mid-write
**Then:** The original task file is unchanged (temp file is not renamed over original)
**And:** An error is written to stderr
**And:** Process exits with code 2

---

### A19 — Invalid TaskId returns error

**Given:** No task exists with id `local:DOES-NOT-EXIST`
**When:** `task-marshal done local:DOES-NOT-EXIST` is invoked
**Then:** An error message is written to stderr
**And:** Process exits with code 2

---

### A20 — Malformed TaskId format returns error

**Given:** User provides `task-marshal done TASK-042` (no source prefix)
**When:** The command is invoked
**Then:** An error message describing the expected format is written to stderr
**And:** Process exits with code 2

---

## `task-marshal show`

### A21 — Returns full task block for valid ID

**Given:** Task `beads:bd-a1b2` exists in the BEADS database
**When:** `task-marshal show beads:bd-a1b2` is invoked
**Then:** The full `TaskBlock` for that task is written to stdout
**And:** Process exits with code 0

---

### A22 — Returns error for unknown ID

**Given:** No task with id `gh:999` exists
**When:** `task-marshal show gh:999` is invoked
**Then:** An error message is written to stderr
**And:** Process exits with code 2

---

### A23 — `show` does not modify any source

**Given:** Task `local:TASK-001` exists
**When:** `task-marshal show local:TASK-001` is invoked
**Then:** The task file is unchanged
**And:** No source is modified

---

## `task-marshal list`

### A24 — Lists all actionable tasks across all sources

**Given:** Eligible tasks in `local` and `beads` sources
**When:** `task-marshal list` is invoked
**Then:** A summary table containing all eligible tasks from all sources is written to stdout
**And:** Process exits with code 0
**And:** Blocked and done tasks are excluded

---

### A25 — `--all` includes blocked and done tasks

**Given:** A mix of `unstarted`, `blocked`, and `done` tasks
**When:** `task-marshal list --all` is invoked
**Then:** All tasks appear in the output regardless of state

---

### A26 — `--source` filters to one source

**Given:** Tasks in both `local` and `github`
**When:** `task-marshal list --source local` is invoked
**Then:** Only `local` tasks appear in the output

---

### A27 — `--role` filters to matching tasks

**Given:** Tasks with roles `coder` and `tester`; unassigned tasks
**When:** `task-marshal list --role tester` is invoked
**Then:** Only `tester` tasks and unassigned tasks appear

---

## Output Format

### A28 — Task output has no ANSI color codes

**Given:** Any invocation of `next`, `show`, or `list`
**When:** The command succeeds and writes task content to stdout
**Then:** The stdout output contains no ANSI escape sequences

---

### A29 — Warnings go to stderr, not stdout

**Given:** A source is unavailable
**When:** `task-marshal next` is invoked
**Then:** The warning message appears on stderr only
**And:** stdout contains only task content (or is empty if no task found)

---

## Configuration

### A30 — Config is discovered by walking up from CWD

**Given:** Working directory is `/project/src/module`; config exists at `/project/task-marshal.toml`
**When:** `task-marshal next` is invoked from `/project/src/module`
**Then:** The config at `/project/task-marshal.toml` is used

---

### A31 — No config file is a hard error

**Given:** No `task-marshal.toml` found anywhere in the directory tree
**When:** Any `task-marshal` command is invoked
**Then:** An error message is written to stderr
**And:** Process exits with code 2

---

### A32 — `list` with no matching tasks exits 0

**Given:** No tasks match the applied filters (or no tasks exist in any source)
**When:** `task-marshal list` (or `task-marshal list --role X`) is invoked
**Then:** A "no tasks found" message or empty table is written to stdout
**And:** Process exits with code 0
**Note:** Unlike `next`, `list` is informational — an empty result is a successful query, not an error condition.
