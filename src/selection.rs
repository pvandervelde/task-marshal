// Task selection algorithm.
// See docs/spec/interfaces/selection.md for full contract documentation.

use thiserror::Error;

use crate::{Priority, Role, SourceKey, TaskSummary};

// ── SelectionFilter ───────────────────────────────────────────────────────────

/// The combined set of criteria applied when selecting or listing tasks.
///
/// See docs/spec/interfaces/selection.md
#[derive(Debug, Clone, Default)]
pub struct SelectionFilter {
    /// If set, only tasks with this role (or no role) are eligible.
    /// Comparison is case-insensitive.
    pub role: Option<Role>,

    /// If set, only tasks from this source are considered.
    /// Used when `--source <key>` is provided on the CLI.
    pub source_key: Option<SourceKey>,

    /// If true, include tasks in all states (Done, Blocked) in list output.
    /// If false (default), only Unstarted and InProgress tasks are returned.
    pub include_all: bool,
}

// ── SortKey ───────────────────────────────────────────────────────────────────

/// The composite ordering key for deterministic task selection.
///
/// Tasks are sorted ascending on all three fields. The task with the smallest
/// SortKey is selected.
///
/// See docs/spec/interfaces/selection.md
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SortKey {
    /// Primary: lower Priority integer wins (P0 before P1 before P2).
    pub priority: crate::Priority,

    /// Secondary: lower source_order wins (position in configured priority list).
    pub source_order: usize,

    /// Tertiary: lexicographic ascending.
    /// Numeric NativeIds are zero-padded to 9 digits for natural numeric ordering.
    pub native_id_sort_key: String,
}

// ── ScoredCandidate ───────────────────────────────────────────────────────────

/// A `TaskSummary` paired with its source priority index, as input to `TaskSelector`.
///
/// The CLI Layer constructs `ScoredCandidate` values by pairing each `TaskSummary`
/// from a source with that source's index in the priority list.
///
/// See docs/spec/interfaces/selection.md
#[derive(Debug, Clone)]
pub struct ScoredCandidate {
    /// 0-indexed position of this task's source in the configured priority list.
    /// Lower index wins when Priority is tied.
    pub source_order: usize,

    /// The task summary returned by the source adapter.
    pub summary: TaskSummary,
}

// ── SelectionError ────────────────────────────────────────────────────────────

/// Error returned when no eligible task is found.
///
/// Maps to exit code 1 (distinct from operational errors at exit code 2).
///
/// See docs/spec/interfaces/selection.md
#[derive(Debug, Error)]
pub enum SelectionError {
    /// No eligible task was found after applying all filters.
    #[error("no eligible task found")]
    NoTaskFound,
}

// ── TaskSelector ──────────────────────────────────────────────────────────────

/// Deterministic task selection engine.
///
/// Pure computation — no state, no I/O, no randomness.
/// Never calls any TaskSource directly. Receives pre-fetched summaries from the CLI Layer.
///
/// See docs/spec/interfaces/selection.md
pub struct TaskSelector;

impl TaskSelector {
    /// Selects the single highest-priority eligible task from the candidate list.
    ///
    /// # Arguments
    /// * `candidates` — all TaskSummary values from all consulted sources,
    ///   each paired with its source's priority order index
    /// * `filter` — role and source restrictions, and include_all flag
    ///
    /// # Returns
    /// The TaskSummary of the selected task.
    ///
    /// # Errors
    /// * `SelectionError::NoTaskFound` — no candidate passes all filters
    ///
    /// # Selection Algorithm (deterministic)
    /// 1. Filter by state: exclude Done/Blocked unless filter.include_all
    /// 2. Filter by source_key: exclude candidates not from filter.source_key (if set)
    /// 3. Filter by role: unassigned tasks pass any role filter; assigned tasks must match
    /// 4. Sort by SortKey ascending: (Priority, source_order, NativeId sort key)
    /// 5. Return first candidate, or NoTaskFound if empty
    ///
    /// # Determinism guarantee
    /// Given identical inputs, always returns the same result. No randomness.
    pub fn select(
        candidates: Vec<ScoredCandidate>,
        filter: &SelectionFilter,
    ) -> Result<TaskSummary, SelectionError> {
        unimplemented!("See docs/spec/interfaces/selection.md")
    }
}
