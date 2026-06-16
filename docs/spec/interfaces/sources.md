# Sources

**Module:** `src/sources/mod.rs`, `src/sources/local.rs`, `src/sources/beads.rs`, `src/sources/github.rs`
**Architectural layer:** Abstraction (trait) + Infrastructure (adapters)

---

## Overview

This module defines:

1. The `TaskSource` trait — the central abstraction all business logic depends on
2. `SourceRegistry` — builds and looks up source adapter instances from config
3. `SubprocessRunner` — the abstraction over `std::process::Command` for testability
4. Three concrete source adapters: `LocalFileSource`, `BeadsSource`, `GithubSource`

**Dependency rules:**

- Business logic depends only on `TaskSource` (the trait), never on the concrete adapter types
- Concrete adapters are only constructed in the CLI Layer (via `SourceRegistry`)
- Adapters never depend on each other

---

## `TaskSource` Trait

The single abstraction that all source adapters implement.

```rust
pub trait TaskSource {
    /// Returns all eligible tasks from this source, filtered by `filter`.
    ///
    /// "Eligible" means:
    /// - State is `Unstarted` or `InProgress` (unless `filter.include_all` is true)
    /// - Dependencies are resolved (blocked tasks have state `Blocked`)
    /// - Role matches filter (if role filter is set)
    ///
    /// # Errors
    /// * `SourceError::Unavailable` — source system is unreachable; caller should skip + warn
    /// * `SourceError::Io` — underlying I/O failure
    /// * `SourceError::Parse` — source output could not be interpreted
    ///
    /// # Guarantees
    /// - Must not modify the source
    /// - Returns an empty `Vec` if no tasks match (not an error)
    fn list(&self, filter: &SelectionFilter) -> Result<Vec<TaskSummary>, SourceError>;

    /// Returns the full task content for the given native ID.
    ///
    /// # Errors
    /// * `SourceError::Unavailable` — source system is unreachable
    /// * `SourceError::NotFound` — no task with this `native_id` exists in this source
    /// * `SourceError::Io` — underlying I/O failure
    /// * `SourceError::Parse` — source output could not be interpreted
    ///
    /// # Guarantees
    /// - Must not modify the source
    fn get(&self, native_id: &NativeId) -> Result<Task, SourceError>;

    /// Marks the task as done and propagates to the source system.
    ///
    /// # Arguments
    /// * `native_id` — the task to complete
    /// * `comment` — optional completion annotation; `None` or `Some("")` means no annotation
    ///
    /// # Errors
    /// * `SourceError::Unavailable` — source system is unreachable
    /// * `SourceError::NotFound` — task does not exist in this source
    /// * `SourceError::Io` — underlying I/O failure preventing completion
    /// * `SourceError::Parse` — source response could not be interpreted
    ///
    /// # Atomicity
    /// The operation must be atomic: either the task is fully marked done, or no change occurs.
    /// Partial state (done in one place but not another) must not be left behind.
    ///
    /// # Idempotency
    /// Calling `complete` on an already-done task must succeed (no error).
    fn complete(&self, native_id: &NativeId, comment: Option<&str>) -> Result<(), SourceError>;
}
```

**Object safety:** `TaskSource` is object-safe. Use as `Box<dyn TaskSource>`.

---

## `SourceError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    /// The source system is unreachable (binary missing, network down, DB locked).
    /// Callers should log a warning and skip this source; do not exit with code 2.
    #[error("source unavailable: {message}")]
    Unavailable { message: String },

    /// The requested task does not exist in this source.
    #[error("task not found: {native_id}")]
    NotFound { native_id: NativeId },

    /// An underlying I/O failure occurred.
    #[error("I/O error: {message}")]
    Io { message: String },

    /// The source's output could not be parsed into domain types.
    #[error("parse error: {message}")]
    Parse { message: String },

    /// A multi-step operation partially succeeded: the primary action (e.g. close) completed
    /// but a secondary action (e.g. post a comment) failed.
    /// The task IS marked done; only the annotation is missing.
    #[error("partial completion: {message}")]
    PartialCompletion { message: String },
}
```

---

## `SourceRegistry`

Builds and stores the ordered list of source adapters from `Config`.

```rust
pub struct SourceRegistry {
    /// Source adapters in priority order, paired with their SourceKey.
    /// Order matches the deduped priority list from config.
    ordered_sources: Vec<(SourceKey, Box<dyn TaskSource>)>,
}

impl SourceRegistry {
    /// Builds the source registry from the parsed config.
    ///
    /// Sources are instantiated in priority order. Sources listed in priority but without
    /// a corresponding config section are silently skipped.
    ///
    /// # Errors
    /// * `ConfigError::Invalid` — a source config contains an invalid path
    ///   (e.g., path traversal detected for LocalFileSource)
    pub fn from_config(config: &Config) -> Result<Self, ConfigError>;

    /// Returns all source adapters in priority order, paired with their SourceKey.
    pub fn ordered_sources(&self) -> &[(SourceKey, Box<dyn TaskSource>)];

    /// Looks up a source adapter by SourceKey.
    ///
    /// Returns `None` if the key is not in the configured sources.
    pub fn find_source(&self, key: &SourceKey) -> Option<&dyn TaskSource>;

    /// Returns the source priority index (0-indexed) for the given key.
    ///
    /// Returns `None` if the key is not in the configured sources.
    pub fn source_order(&self, key: &SourceKey) -> Option<usize>;
}
```

---

## `SubprocessRunner` and `SubprocessOutput`

An abstraction over `std::process::Command` that enables test doubles for `BeadsSource` and
`GithubSource`.

```rust
/// The output of a subprocess invocation.
pub struct SubprocessOutput {
    /// The raw stdout bytes from the subprocess.
    pub stdout: Vec<u8>,
    /// The raw stderr bytes from the subprocess.
    pub stderr: Vec<u8>,
    /// The exit code of the subprocess (0 = success).
    pub exit_code: i32,
}

/// Abstraction over subprocess execution. Injected into BeadsSource and GithubSource.
///
/// Production implementation calls `std::process::Command` with NO shell interpolation.
/// Test implementation returns a configured (stdout, stderr, exit_code) triple.
pub trait SubprocessRunner {
    /// Runs an external program with the given arguments and environment variables.
    ///
    /// # Arguments
    /// * `program` — the binary to execute (e.g., `"bd"`, `"gh"`)
    /// * `args` — arguments passed as separate process arguments (no shell interpolation)
    /// * `env_vars` — additional environment variable key-value pairs to set
    ///
    /// # Returns
    /// `SubprocessOutput` if the process was spawned and exited (regardless of exit code).
    ///
    /// # Errors
    /// * `SourceError::Unavailable` — binary not found in PATH (`ENOENT`) or process
    ///   could not be spawned
    /// * `SourceError::Io` — I/O error capturing stdout/stderr
    ///
    /// # Security
    /// Implementations MUST pass arguments as separate process arguments.
    /// Shell string composition is FORBIDDEN. User-provided values (task IDs, comment text)
    /// are passed as argument values, never interpolated into a command string.
    fn run(
        &self,
        program: &str,
        args: &[&str],
        env_vars: &[(String, String)],
    ) -> Result<SubprocessOutput, SourceError>;
}
```

**Object safety:** `SubprocessRunner` is object-safe. Use as `Box<dyn SubprocessRunner>`.

---

## `RealSubprocessRunner`

Production implementation of `SubprocessRunner` using `std::process::Command`.

**File:** `src/sources/mod.rs`

```rust
pub struct RealSubprocessRunner;

impl SubprocessRunner for RealSubprocessRunner {
    fn run(
        &self,
        program: &str,
        args: &[&str],
        env_vars: &[(String, String)],
    ) -> Result<SubprocessOutput, SourceError>;
}
```

All arguments are passed as separate process arguments. No shell interpolation.
Pass `Box::new(RealSubprocessRunner)` to `BeadsSource::new` and `GithubSource::new` in `main.rs`.

---

## `LocalFileSource`

Reads and updates the local markdown task file.

**File:** `src/sources/local.rs`

```rust
pub struct LocalFileSource {
    /// Absolute, validated path to the task markdown file.
    path: PathBuf,
}

impl LocalFileSource {
    /// Creates a new `LocalFileSource` for the given path.
    ///
    /// # Errors
    /// * `ConfigError::Invalid` — path is outside the project root (path traversal)
    pub fn new(config: &LocalSourceConfig) -> Result<Self, ConfigError>;
}

impl TaskSource for LocalFileSource { /* ... */ }
```

**`list()` behaviour:**

- Reads and parses the markdown file
- Skips malformed task blocks with a warning to stderr
- Returns only tasks matching the filter (state, role)
- Sets `TaskState::Blocked` for tasks whose dependencies (in the same file) are not `Done`
- Returns `SourceError::Unavailable` if the file does not exist
- Returns `SourceError::Io` if the file exists but cannot be read

**`get()` behaviour:**

- Parses the full `Task` for the given `NativeId`
- Returns `SourceError::NotFound` if no task with that ID exists in the file

**`complete()` behaviour:**

- Reads the file, finds the task block by `NativeId`
- Updates the `**State:**` line to `done` in memory
- If `comment` is non-empty, appends `**Completion Note:** <comment>` after the state line
- Writes the modified content to a temp file in the same directory
- Atomically renames the temp file over the original
- Returns `SourceError::Io` if the rename fails (original file is unchanged)
- Returns `SourceError::NotFound` if the task does not exist in the file

**Local file format:** Defined in `operations.md`. The parser must tolerate:

- Trailing whitespace on lines
- Mixed line endings (CRLF / LF)
- Missing optional fields (`Role`, `Depends`)

---

## `BeadsSource`

Wraps the `bd` CLI binary.

**File:** `src/sources/beads.rs`

```rust
pub struct BeadsSource {
    config: BeadsSourceConfig,
    runner: Box<dyn SubprocessRunner>,
}

impl BeadsSource {
    /// Creates a new `BeadsSource`.
    ///
    /// `runner` is injected for testability. In production, use `RealSubprocessRunner`.
    pub fn new(config: BeadsSourceConfig, runner: Box<dyn SubprocessRunner>) -> Self;
}

impl TaskSource for BeadsSource { /* ... */ }
```

**`list()` CLI invocation:**

```
bd ready --json
```

Optional env var: `BEADS_DIR=<beads_dir>` if `config.beads_dir` is set.

**`list()` behaviour:**

- Parses the JSON array of task objects from `bd ready --json`
- Maps BEADS priority field to `Priority`
- `bd ready` already filters out blocked tasks; returns `TaskState::Unstarted` or `InProgress`
- Returns `SourceError::Unavailable` if `bd` is not in PATH or if `bd` exits non-zero due
  to DB unavailability

**`get()` CLI invocation:**

```
bd show <native_id> --json
```

**`complete()` CLI invocation (without comment):**

```
bd close <native_id>
```

**`complete()` CLI invocation (with comment):**

```
bd close <native_id> --comment "<comment_text>"
```

**Error mapping:**

- Subprocess ENOENT → `SourceError::Unavailable`
- Subprocess non-zero exit on `list` or `get` → `SourceError::Unavailable`
- Subprocess non-zero exit on `complete` → `SourceError::Io`
- JSON parse failure → `SourceError::Parse`

---

## `GithubSource`

Wraps the `gh` CLI binary.

**File:** `src/sources/github.rs`

```rust
pub struct GithubSource {
    config: GithubSourceConfig,
    runner: Box<dyn SubprocessRunner>,
}

impl GithubSource {
    /// Creates a new `GithubSource`.
    ///
    /// `runner` is injected for testability. In production, use `RealSubprocessRunner`.
    pub fn new(config: GithubSourceConfig, runner: Box<dyn SubprocessRunner>) -> Self;
}

impl TaskSource for GithubSource { /* ... */ }
```

**`list()` CLI invocation:**

```
gh issue list --repo <repo> --label <label1> --label <label2> ... --json number,title,state,labels,assignees --state open
```

Note: `--label` is repeated once per label (AND filtering by GitHub).
If `config.labels` is empty, no `--label` arguments are passed.

**`get()` CLI invocation:**

```
gh issue view <native_id> --repo <repo> --json number,title,body,state,labels
```

**`complete()` CLI invocation (without comment):**

```
gh issue close <native_id> --repo <repo>
```

**`complete()` CLI invocation (with comment):**

```
gh issue close <native_id> --repo <repo>
gh issue comment <native_id> --repo <repo> --body "<comment_text>"
```

Both commands are executed in sequence. If `close` succeeds but `comment` fails, return
`SourceError::PartialCompletion` with a message noting the partial state.

**v1 constraint:** GitHub issues have no dependency tracking. All issues returned by `list`
are treated as `TaskState::Unstarted` (or `InProgress` if the GitHub issue is assigned).

**Error mapping:**

- Subprocess ENOENT → `SourceError::Unavailable`
- Non-zero exit from `list` or `get` (e.g., auth failure) → `SourceError::Unavailable`
- Non-zero exit from `complete` → `SourceError::Io`
- `close` success + `comment` failure → `SourceError::PartialCompletion`
- JSON parse failure → `SourceError::Parse`

---

## JSON Field Mappings

### BEADS JSON → Domain Types

| BEADS field | Domain type | Notes |
|---|---|---|
| `id` | `NativeId` | Used as-is |
| `title` | `Task.title` | |
| `description` | `Task.description` | |
| `acceptance_criteria` | `Task.acceptance_criteria` | May be absent; use empty string |
| `priority` | `Priority` | Integer; default 2 if absent |
| `role` | `Role` | Optional; null if absent |
| `state` | `TaskState` | Map BEADS state strings to enum |

### GitHub JSON → Domain Types

| GitHub field | Domain type | Notes |
|---|---|---|
| `number` | `NativeId` | Converted to string |
| `title` | `Task.title` | |
| `body` | `Task.description` | Full issue body |
| `state` | `TaskState` | `"OPEN"` → `Unstarted`; `"CLOSED"` → `Done` |
| `labels` | `Role` | **v1: no role mapping.** GitHub labels are not mapped to roles; `role` is always `None` for GitHub tasks in v1. |

---

## Test Requirements

### `LocalFileSource`

- `list()` on fixture file with mix of states: returns only eligible tasks
- `list()` with blocked tasks (unresolved deps): returns `Blocked` state
- `list()` on non-existent file: `SourceError::Unavailable`
- `list()` on malformed task block: skips block, returns remaining
- `get()` returns correct `Task` for known NativeId
- `get()` returns `SourceError::NotFound` for unknown NativeId
- `complete()` updates state to `done`, file otherwise unchanged
- `complete()` with comment appends annotation
- `complete()` is atomic (temp file + rename)
- `complete()` on read-only file: `SourceError::Io`, file unchanged

### `BeadsSource`

- `list()` parses `bd ready --json` output correctly
- `list()` passes `BEADS_DIR` env var when configured
- `list()` when `bd` not in PATH: `SourceError::Unavailable`
- `get()` invokes correct `bd show <id> --json` command
- `complete()` without comment: correct `bd close <id>` invocation
- `complete()` with comment: correct `bd close <id> --comment <text>` invocation
- `complete()` non-zero exit from `bd`: `SourceError::Io`

### `GithubSource`

- `list()` constructs correct `gh issue list` invocation
- `list()` with no labels: no `--label` args
- `list()` when `gh` not in PATH: `SourceError::Unavailable`
- `get()` invokes correct `gh issue view <N>` command
- `complete()` without comment: only `gh issue close <N>` invoked
- `complete()` with comment: `gh issue close <N>` then `gh issue comment <N> --body <text>`
- `complete()` close succeeds, comment fails: `SourceError::Io` (partial state noted)
