# ADR-0008: Deterministic Selection Algorithm

**Date:** 2026-05-24
**Status:** Accepted

---

## Context

The requirements (§8.1, §11) mandate that `task-marshal next` must be **deterministic**: given the same source state, it must always return the same task. There must be no randomness or LLM involvement in selection.

Sources return tasks in varying and potentially non-deterministic orders (file ordering, database query result ordering, API pagination order). A total ordering must be imposed.

Additionally, the selection must handle:
- Multiple sources with a configured priority order
- Tasks with explicit priority values (P0–P3+)
- An optional role filter
- Blocked tasks (dependencies not yet `done`)

---

## Decision

The `TaskSelector` implements the following deterministic algorithm:

**Input:** `Vec<TaskSummary>` (candidates from all sources, each annotated with its source's priority-order index), `SelectionFilter`

**Algorithm:**

1. **Filter by state:** Discard any task whose `TaskState` is `done` or `blocked`. Retain `unstarted` and `in_progress` only.
2. **Filter by dependency:** Discard any task that has at least one dependency not in `done` state (resolved within its source's returned data).
3. **Filter by role:** If `SelectionFilter.role` is set, discard tasks whose role is set to a different value. Tasks with no role assigned pass this filter regardless of the role filter value.
4. **Sort (stable) by `SortKey`:**
   - Primary: `Priority` ascending (P0 < P1 < P2 < P3)
   - Secondary: Source priority-order index ascending (0 = first in config)
   - Tertiary: `NativeId` ascending (lexicographic)
5. **Select:** Return the first element of the sorted list, or `None` if the list is empty.

**Key properties:**
- No randomness at any step
- All ordering criteria are deterministic given the same input
- The stable sort on `NativeId` as tiebreaker ensures two tasks with identical Priority and source order are always resolved consistently

---

## Rationale

- A three-level sort key (Priority → source order → NativeId) guarantees a total ordering over any set of candidates
- NativeId as the final tiebreaker is stable because NativeIds are immutable after task creation
- Delegating dependency filtering to the source (`bd ready` for BEADS) is correct because the source has the authoritative dependency graph; task-marshal must not re-implement it
- For the local file source, dependency state is determined inline (within the same file)

---

## Dependency Filtering Per Source

| Source | Dependency handling |
|---|---|
| `local` | task-marshal reads all tasks, checks `**Depends:**` fields, and filters out tasks with any dependency not in `done` state |
| `beads` | `bd ready` already returns only tasks with no open blockers; task-marshal trusts this output |
| `github` | No dependency support in v1; all issues treated as unblocked |

---

## Alternatives Considered

| Alternative | Why rejected |
|---|---|
| FIFO (first task created wins) | Creation order is not preserved across sources; not truly deterministic when tasks are added/removed |
| Random selection | Explicitly forbidden by requirements |
| LLM-ranked selection | Explicitly forbidden by requirements |
| Lazy/streaming (stop at first source with a task) | Does not guarantee the globally best task is returned (a P0 in `github` would be missed if `local` has any P2 task) |
| Timestamp-based ordering | Timestamps may not be available for all sources and may not be monotonic across sources |

---

## Consequences

**Positive:**
- Fully deterministic — the same source state always produces the same result
- Auditable — the algorithm is simple enough to reason about manually
- Cross-source priority: a P0 in any source beats a P1 in any source

**Negative / Tradeoffs:**
- All sources must be queried before any selection can be made (cannot short-circuit at first result)
- NativeId lexicographic ordering means task naming conventions affect selection order at equal priority — users should be aware
- GitHub dependency tracking is not supported in v1 (all GitHub issues are treated as unblocked regardless of labels or linked issues)

---

## References

- [vocabulary.md](../spec/vocabulary.md) — `Priority`, `SortKey`, `TaskState`, `Dependency`
- [architecture.md](../spec/architecture.md) — `TaskSelector` business logic
- [assertions.md](../spec/assertions.md) — A3, A8, A9, A10
- [tradeoffs.md](../spec/tradeoffs.md) — T7: pure function vs source-aware selection
- [assumptions.md](../spec/assumptions.md) — CA-03, CA-04
- User requirements §8.1, §11
