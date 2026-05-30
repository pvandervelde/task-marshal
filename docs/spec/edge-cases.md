# Edge Cases and Failure Modes

---

## Source Availability

### EC-01: Source binary not in PATH

**Scenario:** `beads` is configured, but `bd` is not found in `PATH`.
**Expected:** `SourceError::Unavailable` returned from `BeadsSource`; CLI Dispatcher logs a warning to stderr naming the source; proceeds to next source; does not exit with code 2.
**Test:** Unit test on `BeadsSource.list()` with mock subprocess returning `ENOENT`.

---

### EC-02: All configured sources unavailable

**Scenario:** `bd` not in PATH, `gh` not in PATH, and local task file does not exist.
**Expected:** One warning per source on stderr; exits with code 1 and "no task found" message (not code 2, as no error occurred — all sources simply had no tasks to offer).
**Note:** This is consistent with the graceful degradation requirement; unavailability is a warning, not an error.

---

### EC-03: GitHub not authenticated

**Scenario:** `gh` is in PATH but `gh auth status` would fail; `gh issue list` exits with non-zero.
**Expected:** Treated as `SourceError::Unavailable`; warning on stderr; source skipped.
**Distinction:** task-marshal cannot distinguish "not authenticated" from "network unavailable" — both result in `SourceUnavailable`.

---

### EC-04: Network unavailable for GitHub

**Scenario:** `gh issue list` hangs or fails due to network unavailability.
**Expected:** Subprocess timeout handling (v1 does not implement timeout; if `gh` hangs, task-marshal hangs). This is a v2 concern — subprocess timeout is out of scope for v1.
**Documented limitation:** If `gh` hangs (e.g., DNS timeout), task-marshal will also hang. Users should disable the `github` source if the network is unavailable.

---

### EC-05: BEADS database locked by another process

**Scenario:** `bd ready --json` exits with a non-zero code because another `bd` invocation holds the lock.
**Expected:** `SourceError::Unavailable`; warning on stderr; source skipped.

---

## Task File Issues

### EC-06: Empty local task file

**Scenario:** `.llm/tasks.md` exists but contains no task blocks.
**Expected:** `LocalFileSource.list()` returns `Ok(vec![])`. Not an error.

---

### EC-07: Malformed task block in local file

**Scenario:** A task block is missing required fields (e.g., no state marker, no title).
**Expected:** The malformed block is skipped with a warning to stderr. Other tasks in the file are still returned. task-marshal does not abort.
**Rationale:** Robustness — a partially-authored task should not block the entire file.

---

### EC-08: Local task file not readable (permissions)

**Scenario:** `.llm/tasks.md` exists but task-marshal lacks read permission.
**Expected:** `SourceError::Io` with descriptive message; warning on stderr; source skipped.

---

### EC-09: Local task file not writable for `done`

**Scenario:** `task-marshal done local:TASK-042` is invoked but the file is read-only.
**Expected:** `CompletionError` with descriptive message; exits with code 2; file is unchanged.

---

### EC-10: Local task file concurrently modified

**Scenario:** Two `task-marshal done` calls execute against the same local file at nearly the same time.
**Expected (v1 behaviour):** Last-writer-wins via atomic rename. No corruption — each writer reads the file, modifies it in memory, and writes a temp file. The final `rename()` wins. The other write's changes are lost.
**Documented limitation:** Concurrent `done` on the same local file may silently drop one completion in v1.

---

## Task Selection Edge Cases

### EC-11: All tasks blocked (circular dependency)

**Scenario:** Task A depends on Task B, Task B depends on Task A (cycle in the local file).
**Expected:** Both tasks appear blocked; no task is returned; exits with code 1 and "no task found" message.
**Note:** task-marshal does not detect or report cycles explicitly in v1. Tasks that appear blocked are simply skipped. A cycle results in no task found.

---

### EC-12: Multiple tasks with identical Priority and source order

**Scenario:** Tasks `local:TASK-010` and `local:TASK-020` both have P1 and are in the same source.
**Expected:** The task whose NativeId sorts lexicographically first (`TASK-010`) is selected. Deterministic.

---

### EC-13: Task referenced in `done` is already `done`

**Scenario:** `task-marshal done local:TASK-042` is called, but `TASK-042` is already `done`.
**Expected (local source):** Task is updated to `done` again (idempotent write). No error.
**Expected (BEADS source):** `bd close` behaviour on an already-closed task — passes through `bd`'s response; if `bd` reports success, task-marshal reports success.
**Expected (GitHub source):** `gh issue close` on an already-closed issue — passes through `gh`'s response.
**Rationale:** Idempotency is safer than rejecting double-done calls, which would require extra state inspection.

---

### EC-14: Role values are compared case-insensitively

**Scenario:** Task has `role: Coder`; user runs `task-marshal next --role coder`.
**Expected:** Task is eligible. Role comparison is case-insensitive.

---

## Configuration Edge Cases

### EC-15: Config file found in ancestor directory

**Scenario:** CWD is `/project/src/components`; config is at `/project/task-marshal.toml`.
**Expected:** Config is found and used correctly. All relative paths in config (e.g., `path = ".llm/tasks.md"`) are resolved relative to the config file's directory (`/project/`).
**Critical:** Paths in config are relative to the config file's directory (the repo root), not to CWD.

---

### EC-16: Source listed in priority but not in `[sources.*]`

**Scenario:** `priority = ["local", "beads"]` but no `[sources.beads]` section.
**Expected:** `beads` source is ignored (treated as not configured). Only `local` is used. No error.

---

### EC-17: Duplicate source key in priority list

**Scenario:** `priority = ["local", "local", "github"]`
**Expected:** `local` is consulted once (deduplicated). No crash.

---

## Output Edge Cases

### EC-18: Task title or description contains markdown

**Scenario:** Task title contains `**bold**` or `# Heading`.
**Expected:** Content is output as-is (plain text). task-marshal does not strip or escape markdown.

---

### EC-19: `--comment` text is empty string

**Scenario:** `task-marshal done local:TASK-042 --comment ""`
**Expected:** Treated as if `--comment` was not provided (no annotation written). An empty comment is not useful and should be a no-op.

---

## v1 Documented Limitations

| ID | Limitation |
|---|---|
| EC-04 | Subprocess timeout not implemented; `gh`/`bd` hang will hang task-marshal |
| EC-10 | Concurrent `done` on local file: last-writer-wins, no locking |
| EC-11 | Circular dependencies not detected; result is "no task found" |
| CA-03 | GitHub Issues have no dependency support in v1 |
| CA-04 | Cross-source dependencies not supported in v1 |
