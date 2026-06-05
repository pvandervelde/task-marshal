# Selection

**Module:** `src/selection.rs`
**Architectural layer:** Core business logic — pure computation, no I/O

---

## Overview

This module implements the deterministic task selection algorithm. It accepts a list of
pre-fetched `TaskSummary` candidates (from all sources) and a filter, and returns the single
best candidate.

**RDD responsibilities:**

- Knowing: priority ordering rules, dependency-blocked filtering, role filtering, sort key
  construction
- Doing: filter, sort, and select from a candidate list
- Delegates to: nothing — pure computation, no I/O, no randomness

**Never calls any `TaskSource` directly.** Receives pre-fetched summaries from the CLI Layer.

---

## Types

### `SelectionFilter`

The combined set of criteria applied when selecting or listing tasks.

```rust
pub struct SelectionFilter {
    /// If set, only tasks with this role (or no role) are eligible.
    /// Comparison is case-insensitive.
    pub role: Option<Role>,

    /// If set, only tasks from this source are considered.
    /// Used when `--source <key>` is provided.
    pub source_key: Option<SourceKey>,

    /// If true, include tasks in all states (done, blocked) in list output.
    /// If false (default), only unstarted and in_progress tasks are returned.
    pub include_all: bool,
}
```

**Default:** `SelectionFilter { role: None, source_key: None, include_all: false }`

---

### `SortKey`

The composite ordering key for deterministic selection.

```rust
pub struct SortKey {
    /// Primary sort: lower priority integer wins (P0 before P1 before P2)
    pub priority: Priority,

    /// Secondary sort: source position in the configured priority list (lower index wins)
    pub source_order: usize,

    /// Tertiary sort: NativeId sort key string (ascending lexicographic)
    /// For numeric NativeIds, zero-padded to 9 digits.
    pub native_id_sort_key: String,
}
```

**Ordering:** Ascending on all three fields — the task with the smallest `SortKey` wins.

`SortKey` derives `PartialOrd` and `Ord` for direct comparison.

---

### `ScoredCandidate`

A `TaskSummary` paired with its source priority index, used as input to `TaskSelector`.

```rust
pub struct ScoredCandidate {
    /// Position of this task's source in the configured priority list (0-indexed).
    /// Lower source_order wins when Priority is tied.
    pub source_order: usize,

    /// The task summary returned by the source adapter.
    pub summary: TaskSummary,
}
```

**Construction:** The CLI Layer builds `ScoredCandidate` by pairing each `TaskSummary` from a
source with that source's index in the priority list.

---

### `SelectionError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum SelectionError {
    /// No eligible task was found after applying all filters.
    #[error("no eligible task found")]
    NoTaskFound,
}
```

**Exit code:** 1 (distinct from operational errors which use exit code 2).

---

## `TaskSelector`

Pure selection engine. No state, no I/O.

```rust
pub struct TaskSelector;

impl TaskSelector {
    /// Selects the single highest-priority eligible task from the candidate list.
    ///
    /// # Arguments
    /// * `candidates` — all `TaskSummary` values from all consulted sources,
    ///   each paired with its source's priority order index
    /// * `filter` — role and source restrictions to apply
    ///
    /// # Returns
    /// The `TaskSummary` of the selected task on success.
    ///
    /// # Errors
    /// * `SelectionError::NoTaskFound` — no candidate passes all filters
    ///
    /// # Selection Algorithm (deterministic)
    /// 1. Filter by `include_all`: if false, exclude tasks with state `Done` or `Blocked`
    /// 2. Filter by `source_key`: if set, exclude candidates not from that source
    /// 3. Filter by `role`: exclude tasks whose role does not match the filter
    ///    (tasks with no role pass any role filter)
    /// 4. Sort remaining candidates by `SortKey` ascending:
    ///    a. Priority (lower integer wins)
    ///    b. source_order (lower index wins)
    ///    c. NativeId sort key (lexicographic ascending; numeric IDs zero-padded to 9 digits)
    /// 5. Return the first candidate, or `NoTaskFound` if the list is empty
    ///
    /// # Determinism Guarantee
    /// Given identical `candidates` and `filter`, this function always returns the same result.
    /// No randomness is involved.
    ///
    /// See docs/spec/interfaces/selection.md for the full contract.
    pub fn select(
        candidates: Vec<ScoredCandidate>,
        filter: &SelectionFilter,
    ) -> Result<TaskSummary, SelectionError>;
}
```

---

## Selection Rules Detail

### Eligibility

A candidate is eligible when ALL of the following hold:

1. **State filter:** `state` is `Unstarted` or `InProgress` (unless `include_all` is true)
2. **Source filter:** if `filter.source_key` is `Some(k)`, `candidate.summary.source_key == k`
3. **Role filter:**
   - If `filter.role` is `None` → passes
   - If `candidate.summary.role` is `None` → passes (unassigned tasks are always eligible)
   - If both are `Some` → compare case-insensitively; passes if equal

### Sort Order

Candidates that pass all filters are sorted by the composite `SortKey`:

```
SortKey {
    priority:           candidate.summary.priority,          // ascending; Priority(0) wins
    source_order:       candidate.source_order,              // ascending; 0 wins
    native_id_sort_key: candidate.summary.id.native_id.sort_key(),  // ascending; "0000..." wins
}
```

The task with the **smallest** `SortKey` is selected.

### Blocked Task Handling

The source adapter is responsible for setting `TaskState::Blocked` on tasks whose dependencies
are unresolved. `TaskSelector` treats `Blocked` identically to `Done` — both are excluded from
selection unless `include_all` is true.

---

## Test Requirements

| Scenario | Expected |
|---|---|
| Empty candidate list | `Err(SelectionError::NoTaskFound)` |
| Single eligible candidate | `Ok(that candidate's summary)` |
| All candidates are `Done` | `Err(SelectionError::NoTaskFound)` |
| P0 and P2 candidates | P0 candidate returned |
| Two P1 candidates, different source order | Candidate from lower source_order returned |
| Two P1 candidates, same source order, NativeIds `"TASK-020"` and `"TASK-010"` | `"TASK-010"` returned |
| GitHub numeric NativeIds `"20"` and `"9"` | `"9"` returned (sort keys: `"000000009"` < `"000000020"`) |
| Role filter `"coder"`, task with role `"Coder"` | Eligible (case-insensitive) |
| Role filter `"coder"`, task with no role | Eligible (unassigned passes any filter) |
| Role filter `"coder"`, task with role `"tester"` | Not eligible |
| Source filter `"local"`, only beads tasks | `Err(SelectionError::NoTaskFound)` |
| `include_all: true` | `Done` and `Blocked` candidates included |
| **Property:** same input, multiple calls | Always returns same `TaskId` |

---

## Notes

- `TaskSelector` never calls any `TaskSource`. It receives pre-fetched summaries.
- The `include_all` flag on `SelectionFilter` controls visibility of done/blocked tasks for
  the `list` command. The `next` command always uses `include_all: false`.
- `in_progress` tasks are eligible for `next`. An agent resuming a session receives the same
  `in_progress` task if it remains highest-priority. No deduplication or locking is implemented.
