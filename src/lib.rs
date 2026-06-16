// task-marshal library root
// See docs/spec/interfaces/shared-types.md for type documentation.

use std::hash::{Hash, Hasher};

pub mod config;
pub mod identity;
pub mod output;
pub mod selection;
pub mod sources;

// ── Core domain types ────────────────────────────────────────────────────────
// Re-exported from identity for convenience; defined there because they are
// tightly coupled to parsing logic.
pub use identity::{NativeId, SourceKey, TaskId};

// ── TaskState ─────────────────────────────────────────────────────────────────

/// The lifecycle state of a task.
///
/// See docs/spec/interfaces/shared-types.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskState {
    /// Not yet begun; eligible for selection.
    Unstarted,
    /// Currently being worked; eligible for selection.
    InProgress,
    /// Has unresolved dependencies; not eligible for selection.
    Blocked,
    /// Completed; not eligible for selection unless `include_all` is set.
    Done,
}

// ── Priority ──────────────────────────────────────────────────────────────────

/// Urgency integer. Lower number = higher priority. Default is 2 (P2/normal).
///
/// See docs/spec/interfaces/shared-types.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(pub u32);

impl Default for Priority {
    fn default() -> Self {
        Priority(2)
    }
}

// ── Role ──────────────────────────────────────────────────────────────────────

/// An agent role label. Equality is case-insensitive.
///
/// See docs/spec/interfaces/shared-types.md
#[derive(Debug, Clone)]
pub struct Role(pub String);

impl PartialEq for Role {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Eq for Role {}

// Hash must match the case-insensitive PartialEq: normalise to lowercase before hashing.
impl Hash for Role {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state);
    }
}

// ── Dependency ────────────────────────────────────────────────────────────────

/// A reference to another task (by NativeId) within the same source.
///
/// v1 constraint: within-source only.
///
/// See docs/spec/interfaces/shared-types.md
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency(pub NativeId);

// ── Task ──────────────────────────────────────────────────────────────────────

/// Full, self-contained content of a single task.
///
/// Returned by `TaskSource::get()`. Formatted into a `TaskBlock` by `OutputFormatter`.
///
/// See docs/spec/interfaces/shared-types.md
#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: String,
    pub state: TaskState,
    pub priority: Priority,
    pub role: Option<Role>,
    pub dependencies: Vec<Dependency>,
    pub context_notes: Option<String>,
}

// ── TaskSummary ───────────────────────────────────────────────────────────────

/// Abbreviated task information used in `list` output and during selection.
///
/// A projection of `Task` — omits description, acceptance criteria, and context notes.
///
/// See docs/spec/interfaces/shared-types.md
#[derive(Debug, Clone)]
pub struct TaskSummary {
    pub id: TaskId,
    pub source_key: SourceKey,
    pub priority: Priority,
    pub role: Option<Role>,
    pub title: String,
    pub state: TaskState,
}

// ── TaskBlock ─────────────────────────────────────────────────────────────────

/// Plain-text formatted representation of a Task, ready for writing to stdout.
///
/// No ANSI escape codes. Suitable for direct LLM context ingestion.
/// Produced exclusively by `OutputFormatter::format_task()`.
///
/// See docs/spec/interfaces/shared-types.md
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskBlock(pub String);
