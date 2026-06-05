// Output formatting and completion error type.
// See docs/spec/interfaces/output.md for full contract documentation.

use thiserror::Error;

use crate::identity::SourceKey;
use crate::{Task, TaskBlock, TaskId, TaskSummary};

// ── OutputFormatter ───────────────────────────────────────────────────────────

/// Pure rendering of domain types to strings. No state, no I/O.
///
/// stdout output (format_task, format_summary_list, format_completion_confirmation):
///   No ANSI escape codes. Suitable for direct LLM context ingestion.
///
/// stderr output (format_source_warning, format_error):
///   May include ANSI color codes for readability.
///
/// See docs/spec/interfaces/output.md
pub struct OutputFormatter;

impl OutputFormatter {
    /// Formats a full `Task` into a `TaskBlock` for stdout.
    ///
    /// Plain text, no ANSI escape codes. Includes all task fields.
    /// Omits optional fields (Role, Depends, Context) when absent.
    ///
    /// Format layout: see docs/spec/interfaces/output.md — "TaskBlock Output Format"
    pub fn format_task(task: &Task) -> TaskBlock {
        unimplemented!("See docs/spec/interfaces/output.md")
    }

    /// Formats a list of `TaskSummary` values into a summary table for stdout.
    ///
    /// Columns: ID, State, Priority, Role, Title.
    /// Rows are in the order provided (caller is responsible for ordering).
    ///
    /// Returns a "No tasks found." message if the slice is empty.
    /// No ANSI escape codes.
    pub fn format_summary_list(summaries: &[TaskSummary]) -> String {
        unimplemented!("See docs/spec/interfaces/output.md")
    }

    /// Formats a completion confirmation message for stdout.
    ///
    /// Example: "Done: local:TASK-042"
    /// No ANSI escape codes.
    pub fn format_completion_confirmation(task_id: &TaskId) -> String {
        unimplemented!("See docs/spec/interfaces/output.md")
    }

    /// Formats a source-unavailable warning for stderr.
    ///
    /// Example: "Warning: source 'beads' unavailable: bd not found in PATH"
    /// May include ANSI color codes.
    pub fn format_source_warning(source_key: &SourceKey, message: &str) -> String {
        unimplemented!("See docs/spec/interfaces/output.md")
    }

    /// Formats an error message for stderr.
    ///
    /// May include ANSI color codes.
    pub fn format_error(message: &str) -> String {
        unimplemented!("See docs/spec/interfaces/output.md")
    }
}

// ── CompletionError ───────────────────────────────────────────────────────────

/// CLI-layer error type for `done` command failures.
///
/// Wraps source-level failures with the full `TaskId` being completed.
/// All variants map to exit code 2.
///
/// Mapping from SourceError:
///   SourceError::Unavailable → CompletionError::SourceUnavailable
///   SourceError::NotFound    → CompletionError::TaskNotFound
///   SourceError::Io          → CompletionError::Io
///   SourceError::Parse       → CompletionError::Io
///
/// See docs/spec/interfaces/output.md
#[derive(Debug, Error)]
pub enum CompletionError {
    /// The target task was not found in its source.
    #[error("task '{task_id}' not found")]
    TaskNotFound { task_id: TaskId },

    /// The source system for this task was unavailable.
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
