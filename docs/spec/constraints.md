# Implementation Constraints

Rules that must be enforced in all implementation code. These are non-negotiable.

---

## Language and Toolchain

- **Language:** Rust (stable channel)
- **Edition:** 2021
- **Binary target:** Single executable (`task-marshal`)
- **No `unsafe` code** outside of FFI boundaries (none anticipated in v1)

---

## Error Handling

- All fallible operations must return `Result<T, E>` — never `panic` or `unwrap` on expected failure paths
- `unwrap()` and `expect()` are forbidden in non-test code, **with one exception:** `main()` may use `?` propagation or an explicit top-level match
- Error types must use `thiserror` for derivation
- All error types must be discriminated enums (not stringly-typed)
- Expected errors (source unavailable, no task, invalid id) are `Result` values, not exceptions
- Unexpected errors (programming bugs) may use `unreachable!()` with a message explaining the invariant

**Error type assignments:**

| Error | Type |
|---|---|
| Config not found or invalid TOML | `ConfigError` |
| Source unreachable (binary missing, network down) | `SourceError::Unavailable` |
| Task not found in source | `SourceError::NotFound` |
| Underlying I/O failure in source | `SourceError::Io` |
| Source output could not be parsed | `SourceError::Parse` |
| No eligible task after selection | `SelectionError::NoTaskFound` |
| Completion propagation failed | `CompletionError` |
| TaskId format invalid | `ParseError::InvalidTaskId` |

---

## Exit Codes

| Code | Meaning |
|---|---|
| 0 | Command succeeded; task was found (for `next`) or operation completed (for `done`, `show`, `list`) |
| 1 | No eligible task found (`next` only) |
| 2 | Operational error (config missing, task not found, completion failed, invalid argument) |

**Rule:** The CLI Dispatcher is the only component that converts errors to exit codes. All other components return `Result`.

---

## Output Channels

- **stdout:** Task content only — `TaskBlock`, summary table, completion confirmation
- **stderr:** Warnings (e.g., skipped unavailable source), errors, all diagnostic messages
- **Rule:** No task content may ever be written to stderr; no error or warning may ever be written to stdout

---

## Output Format

- No ANSI escape codes in stdout output (task blocks, summaries, confirmations)
- ANSI color in stderr output is permitted (errors, warnings)
- All stdout output must be valid plain text suitable for direct LLM context ingestion

---

## Subprocess Safety

- All external commands (`bd`, `gh`) must be invoked using `std::process::Command` with arguments passed as **separate process arguments**, never via shell string interpolation
- Shell-string composition of commands (e.g., `format!("gh issue close {id}")` passed to `sh -c`) is **forbidden**
- User-provided values (task IDs, comment text) must never be interpolated into shell strings
- Subprocess stdout is captured; subprocess stderr is forwarded to task-marshal's stderr or discarded (not forwarded to stdout)

---

## File I/O

- Writes to the local task file must use **atomic rename**: write to a temp file in the same directory as the target, then `rename()` over the original
- Reading the task file does not require locking (read-only)
- File paths from config must be validated as canonical before use (no path traversal)
- The local task file parser must be tolerant of minor formatting variations (trailing whitespace, mixed line endings)

---

## Concurrency

- task-marshal is a single-invocation CLI tool; it does not use async I/O
- All I/O is synchronous (`std::fs`, `std::process::Command`)
- No `tokio`, `async-std`, or similar runtime is used in v1
- Rationale: simplicity, fast startup, no concurrent operations required

---

## Dependency Constraints

Required crates:

- `clap` (v4+) — CLI argument parsing
- `toml` — TOML config parsing
- `serde` + `serde_json` — JSON deserialization for `bd` and `gh` output
- `thiserror` — error type derivation
- `tempfile` — safe temp file creation for atomic writes

Discouraged (requires justification):

- Any async runtime (tokio, async-std)
- Any crate that vendors a full TLS or HTTP stack (use `gh`/`bd` CLI instead)
- `anyhow` in library code (use typed errors; `anyhow` may be used only in `main()` as a last resort)

---

## Testing

- Business logic (`TaskSelector`, `TaskIdParser`, `ConfigLoader`) must have unit tests for all meaningful cases
- Source adapters must have integration tests using a controlled environment (test fixture files for `local`; mock subprocess runner for `beads` and `github`)
- Property-based tests must verify `TaskSelector` determinism: given the same input, output is always the same
- CLI acceptance tests must cover all four commands with both success and failure paths
- Test doubles must implement the `TaskSource` trait (not mock concrete types)

See [testing.md](testing.md) for full test strategy.

---

## Module Naming

- Module names must use **business domain vocabulary**, not architectural layer names
- Forbidden module names: `ports`, `adapters`, `core`, `domain`, `infrastructure`, `application`
- Acceptable module names: `selection`, `sources`, `config`, `output`, `identity` (or similar business-meaningful terms)

---

## Configuration

- Config must not contain credentials of any kind
- Secret handling is delegated entirely to `gh` CLI (GitHub) and `bd` CLI (BEADS)
- `BEADS_DIR` environment variable override is passed to `bd` subprocess only if `beads_dir` is set in config
- No other environment variables are read or written by task-marshal (v1)
