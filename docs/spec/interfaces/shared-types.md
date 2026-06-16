# Shared Domain Types

**Module:** `src/lib.rs`
**Architectural layer:** Core domain (shared across all modules)

---

## Overview

These types represent the canonical domain concepts of `task-marshal`. They are used by every
module and must not import anything from source adapters or infrastructure.

**RDD responsibilities:**

- Knowing: the shape of a task, its lifecycle states, priority values, role labels, and dependencies
- Doing: nothing — these are pure data types

---

## Types

### `TaskState`

The lifecycle state of a task.

```rust
pub enum TaskState {
    Unstarted,
    InProgress,
    Blocked,
    Done,
}
```

**Variants:**

| Variant | String (serialised) | Eligible for `next`? |
|---|---|---|
| `Unstarted` | `"unstarted"` | Yes |
| `InProgress` | `"in_progress"` | Yes |
| `Blocked` | `"blocked"` | No |
| `Done` | `"done"` | No |

**Parsing:** Case-insensitive from string. Unknown values cause the task block to be treated as
malformed (warn + skip).

---

### `Priority`

Urgency integer. Lower number = higher priority.

```rust
pub struct Priority(pub u32);
```

**Convention:**

| Value | Label |
|---|---|
| 0 | P0 — critical |
| 1 | P1 — high |
| 2 | P2 — normal (default) |
| 3+ | P3+ — low |

**Default:** `Priority(2)` if unspecified in source.

**Ordering:** `Priority(0) < Priority(1) < Priority(2)` — lower integer wins selection.

**Parsing from `Pn` format:** `"P0"` → `Priority(0)`, `"P1"` → `Priority(1)`, etc. Invalid
values (e.g., `"P-1"`, `"Pfoo"`, empty) cause the task block to be treated as malformed.

---

### `Role`

An agent role label that restricts task eligibility.

```rust
pub struct Role(pub String);
```

**Equality:** Case-insensitive. `Role("Coder") == Role("coder")`.

**Usage:** A task with no role is eligible under any `--role` filter. A task with a role is
only returned when the filter matches (case-insensitively) or no role filter is applied.

---

### `Dependency`

A reference to another task (by `NativeId`) within the same source.

```rust
pub struct Dependency(pub NativeId);
```

**v1 constraint:** Dependencies are within-source only. Cross-source dependency is out of scope.

---

### `Task`

The full, self-contained content of a single task.

```rust
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
```

**Fields:**

| Field | Description |
|---|---|
| `id` | Globally unique `<source>:<native_id>` identifier |
| `title` | Short task title |
| `description` | Full description of the work |
| `acceptance_criteria` | Criteria to consider the task done |
| `state` | Current lifecycle state |
| `priority` | Urgency integer (lower = higher priority) |
| `role` | Optional agent role restriction |
| `dependencies` | NativeIds of tasks in the same source that must be `done` first |
| `context_notes` | Optional additional context (not required, may be empty) |

**Constraint:** All fields must be present in the rendered `TaskBlock`. No field may be omitted
from output.

---

### `TaskSummary`

Abbreviated task information used during selection and in `list` output.

```rust
pub struct TaskSummary {
    pub id: TaskId,
    pub source_key: SourceKey,
    pub priority: Priority,
    pub role: Option<Role>,
    pub title: String,
    pub state: TaskState,
}
```

**Relationship to `Task`:** `TaskSummary` is a projection — it omits description, acceptance
criteria, dependencies, and context notes. The `TaskSource::get()` method returns a full `Task`.

---

### `TaskBlock`

The plain-text formatted representation of a `Task`, suitable for writing to stdout.

```rust
pub struct TaskBlock(pub String);
```

**Constraints:**

- No ANSI escape codes
- Suitable for direct inclusion in an agent's context window
- Produced exclusively by `OutputFormatter::format_task()`

---

## Deriving Traits

All domain types derive at minimum:

- `Debug`, `Clone`
- `PartialEq`, `Eq` (where meaningful)

`TaskState` and `Priority` additionally derive `PartialOrd`, `Ord` for sorting.

`SourceKey` and `Priority` derive `Copy`.

---

## Usage Examples

```rust
// Checking eligibility
fn is_eligible(summary: &TaskSummary) -> bool {
    matches!(summary.state, TaskState::Unstarted | TaskState::InProgress)
}

// Role filter: unassigned tasks pass any filter
fn matches_role_filter(summary: &TaskSummary, filter: Option<&Role>) -> bool {
    match (filter, &summary.role) {
        (None, _) => true,
        (Some(_), None) => true,         // unassigned passes any filter
        (Some(f), Some(r)) => f == r,    // case-insensitive equality
    }
}
```

---

## Notes

- `Task.dependencies` stores `Dependency(NativeId)` values, which are within-source references.
  The source adapter is responsible for resolving these to determine `TaskState::Blocked`.
- `Task.id` is the `TaskId` (fully qualified `<source>:<native_id>`), while each `Dependency`
  holds only the `NativeId` (source is implicit from `Task.id.source_key`).
- The `TaskId`, `NativeId`, and `SourceKey` types are defined in [identity.md](identity.md).
