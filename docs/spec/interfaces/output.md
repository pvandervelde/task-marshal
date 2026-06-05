# Output Formatting

**Module:** `src/output.rs`
**Architectural layer:** Infrastructure (CLI layer) — pure formatting, no I/O

---

## Overview

This module defines the `OutputFormatter` (pure rendering of domain types to strings) and
`CompletionError` (the CLI-layer error type for `done` command failures).

**RDD responsibilities:**

- Knowing: `TaskBlock` format (plain text, no ANSI), `TaskSummary` table format for `list`
- Doing: format `Task` → `TaskBlock`; format `Vec<TaskSummary>` → table; format confirmations,
  warnings, and errors
- Delegates to: nothing — pure formatting, no I/O

---

## `OutputFormatter`

All methods are pure functions. No state, no I/O.

```rust
pub struct OutputFormatter;

impl OutputFormatter {
    /// Formats a full `Task` into a `TaskBlock` for stdout.
    ///
    /// # Output format
    /// Plain text. No ANSI escape codes. Suitable for direct LLM context ingestion.
    ///
    /// The output includes:
    /// - TaskId (source:native_id)
    /// - Title
    /// - State, Priority, Role (if set)
    /// - Dependencies (if any)
    /// - Description (full text)
    /// - Acceptance Criteria (full text)
    /// - Context notes (if present)
    ///
    /// See docs/spec/interfaces/output.md for the exact format layout.
    pub fn format_task(task: &Task) -> TaskBlock;

    /// Formats a list of `TaskSummary` values into a summary table for stdout.
    ///
    /// Columns: ID, State, Priority, Role, Title
    /// Rows sorted by the order provided (caller is responsible for ordering).
    ///
    /// # Empty list
    /// Returns a message like "No tasks found." rather than an empty table.
    ///
    /// No ANSI escape codes.
    pub fn format_summary_list(summaries: &[TaskSummary]) -> String;

    /// Formats a completion confirmation message for stdout.
    ///
    /// Example: "Done: local:TASK-042"
    ///
    /// No ANSI escape codes.
    pub fn format_completion_confirmation(task_id: &TaskId) -> String;

    /// Formats a source-unavailable warning for stderr.
    ///
    /// Example: "Warning: source 'beads' unavailable: bd not found in PATH"
    ///
    /// May include ANSI color codes (yellow/orange for warnings).
    pub fn format_source_warning(source_key: &SourceKey, message: &str) -> String;

    /// Formats an error message for stderr.
    ///
    /// May include ANSI color codes (red for errors).
    pub fn format_error(message: &str) -> String;
}
```

---

## `TaskBlock` Output Format

The format produced by `OutputFormatter::format_task()`. This is the full, self-contained
task content that consumers (agents, developers) receive.

```
Task: <source_key>:<native_id>
Title: <title>
State: <state>
Priority: P<n>
Role: <role>          (omitted if no role)
Depends: <id1>, <id2> (omitted if no dependencies)

Description
-----------
<description text>

Acceptance Criteria
-------------------
<acceptance_criteria text>

Context
-------
<context_notes text>  (omitted if no context notes)
```

**Constraints:**

- No ANSI escape codes anywhere in this output
- All fields present in `Task` are rendered; no field is silently dropped
- The format is stable — agents rely on it for task parsing

---

## `CompletionError`

The CLI-layer error type for `done` command failures. Wraps source-level failures with
additional context (the full `TaskId` being completed).

```rust
#[derive(Debug, thiserror::Error)]
pub enum CompletionError {
    /// The target task was not found in its source.
    #[error("task '{task_id}' not found")]
    TaskNotFound { task_id: TaskId },

    /// The source for this task was unavailable.
    #[error("source for task '{task_id}' unavailable: {message}")]
    SourceUnavailable { task_id: TaskId, message: String },

    /// An I/O error prevented the completion from being persisted.
    #[error("I/O error completing task '{task_id}': {message}")]
    Io { task_id: TaskId, message: String },

    /// The close operation succeeded but the comment could not be attached.
    /// The task IS marked done; only the annotation is missing.
    #[error("task '{task_id}' marked done but comment failed: {message}")]
    PartialCompletion { task_id: TaskId, message: String },
}
```

**Mapping from `SourceError`:**

| `SourceError` variant | `CompletionError` variant |
|---|---|
| `SourceError::Unavailable` | `CompletionError::SourceUnavailable` |
| `SourceError::NotFound` | `CompletionError::TaskNotFound` |
| `SourceError::Io` | `CompletionError::Io` |
| `SourceError::Parse` | `CompletionError::Io` (unexpected parse failure) |

**Exit code:** 2 for all variants.

---

## Test Requirements

| Scenario | Expected |
|---|---|
| `format_task()` output contains no ANSI codes | No `\x1b[` sequences in output |
| `format_task()` with no role | Role line omitted |
| `format_task()` with no dependencies | Depends line omitted |
| `format_task()` with no context notes | Context section omitted |
| `format_summary_list([])` | Returns non-empty "no tasks" message (not blank) |
| `format_summary_list([...])` | All summaries present; columns aligned |
| `format_completion_confirmation()` | Contains the full `TaskId` string |
| `format_source_warning()` | Contains source key name in message |
