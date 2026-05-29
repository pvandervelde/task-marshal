# Testing Strategy

---

## Layers and Approaches

### 1. Unit Tests — Business Logic

Target: `TaskSelector`, `TaskIdParser`, `ConfigLoader`, `OutputFormatter`

These are pure or near-pure functions. All tests use in-memory data; no I/O.

| Component | Key test cases |
|---|---|
| `TaskSelector` | No candidates → `None`; single candidate → returns it; priority ordering (P0 before P1 before P2); source order tiebreaker; NativeId tiebreaker; blocked task excluded; role filter — match, no-match, unassigned; combined filters |
| `TaskSelector` (property) | Given fixed input, output is always identical (determinism property) |
| `TaskIdParser` | Valid `local:TASK-042` → `(local, TASK-042)`; valid `beads:bd-a1b2` → correct tuple; valid `gh:187` → correct tuple; missing colon → `InvalidTaskId`; unknown source key → `InvalidTaskId`; empty native id → `InvalidTaskId` |
| `ConfigLoader` | Valid TOML → `Config`; missing required field → `ConfigError`; invalid TOML → `ConfigError`; walk-up finds file in ancestor dir; walk-up finds no file → `ConfigError`; optional source sections absent → source not included |
| `OutputFormatter` | `TaskBlock` output has no ANSI codes; `TaskBlock` contains all required fields; summary table format; empty list output |

---

### 2. Integration Tests — Source Adapters

Target: `LocalFileSource`, `BeadsSource`, `GithubSource`

Each adapter is tested in isolation against a controlled environment.

#### `LocalFileSource`

- Use a temp directory with a fixture `tasks.md` file
- Test `list()`: returns all unblocked tasks; filters done tasks; filters blocked tasks (unresolved deps)
- Test `get()`: returns correct task by NativeId; returns `NotFound` for unknown ID
- Test `complete()`: updates task state to `done` in-place; leaves surrounding content unchanged; atomic (temp file + rename, verified by intercepting rename failure)
- Test `complete()` with comment: appends annotation to task block
- Edge: empty task file; malformed task block; task file not found → `SourceError::Unavailable`

#### `BeadsSource`

- Use a mock subprocess runner (test double for `std::process::Command` execution)
- The mock returns controlled JSON for `bd ready --json`, `bd show --json`, or simulates missing binary
- Test `list()`: parses BEADS JSON correctly; maps BEADS priority to `Priority`; `bd` not in PATH → `SourceUnavailable`
- Test `get()`: correct `bd show <id> --json` invocation; parses full task
- Test `complete()`: correct `bd close <id>` invocation; correct `bd close <id> --comment` invocation; non-zero exit from `bd` → `CompletionError`
- Test `BEADS_DIR` env var injection when `beads_dir` is configured

#### `GithubSource`

- Use a mock subprocess runner
- Test `list()`: correct `gh issue list --repo ... --label ... --json` invocation; parses GitHub JSON; `gh` not in PATH → `SourceUnavailable`
- Test `get()`: correct `gh issue view <N> --json` invocation
- Test `complete()` without comment: invokes `gh issue close <N>` only
- Test `complete()` with comment: invokes `gh issue close <N>` and `gh issue comment <N> --body "..."` in order; if close succeeds but comment fails → `CompletionError` (with note about partial state)
- Test `gh` non-zero exit → `SourceUnavailable` or `CompletionError`

---

### 3. CLI Acceptance Tests

Target: the compiled `task-marshal` binary invoked end-to-end

- Use a temp directory with a fixture `.llm/task-marshal.toml` and `.llm/tasks.md`
- Use mock `bd` and `gh` binaries (shell scripts or compiled test stubs) placed in a temporary PATH

| Scenario | Expected outcome |
|---|---|
| `next` — task available | Exit 0, TaskBlock on stdout, no ANSI in stdout |
| `next` — no tasks | Exit 1, message on stderr, nothing on stdout |
| `next --role coder` — matching task | Exit 0, correct task returned |
| `next --role coder` — no matching task | Exit 1 |
| `next --source local` — task in local only | Exit 0, only local task considered |
| `next` — source unavailable | Exit 0 (if another source has a task), warning on stderr |
| `done <valid-id>` | Exit 0, confirmation on stdout |
| `done <invalid-id-format>` | Exit 2, error on stderr |
| `done <unknown-id>` | Exit 2, error on stderr |
| `show <valid-id>` | Exit 0, TaskBlock on stdout |
| `show <unknown-id>` | Exit 2, error on stderr |
| `list` | Exit 0, summary table on stdout |
| `list --all` | Exit 0, includes done/blocked tasks |
| No config file found | Exit 2, error on stderr |

---

### 4. Property-Based Tests

Target: `TaskSelector` determinism and sort stability

Using a property-based testing crate (e.g., `proptest`):

- **Determinism:** For any randomly generated `Vec<TaskSummary>` and `SelectionFilter`, `select(candidates, filter)` returns the same value on repeated calls
- **Sort stability:** The selected task always has the lowest Priority value among eligible candidates; among equal-Priority tasks, the one from the source with lowest priority-order index is selected; among equal Priority and source order, the lexicographically smallest NativeId is selected

---

## Test Infrastructure

### Mock Subprocess Runner

An abstraction over `std::process::Command` execution, injected into `BeadsSource` and `GithubSource` during tests. In production, calls the real binary; in tests, returns a configured `(stdout, stderr, exit_code)` triple.

This requires the subprocess execution to be abstract enough for injection. The interface designer should define a `CommandRunner` trait (or similar) that `BeadsSource` and `GithubSource` depend on.

### Test Fixture Task File

A `tasks.md` file in `tests/fixtures/` with a representative set of tasks: unstarted, in_progress, blocked, done, various priorities, various roles, with and without dependencies.

---

## Coverage Goals

| Layer | Target |
|---|---|
| `TaskSelector` | 100% branch coverage |
| `TaskIdParser` | 100% branch coverage |
| `ConfigLoader` | All error paths covered |
| `LocalFileSource` | All `list`, `get`, `complete` paths |
| `BeadsSource` | All command variants and error paths |
| `GithubSource` | All command variants and error paths |
| CLI acceptance | All four commands × success + key failure paths |
