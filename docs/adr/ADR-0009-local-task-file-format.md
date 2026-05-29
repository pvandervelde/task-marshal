# ADR-0009: Local Task File Format

**Date:** 2026-05-24
**Status:** Accepted

---

## Context

The `local` source reads tasks from a markdown file (`.llm/tasks.md` by default). The requirements specify markdown (§6.1) but do not define the exact format. A format decision is needed that satisfies:

- Human-readable and author-friendly for both developers and AI agents
- Machine-parseable by task-marshal
- Git-diffable (plain text)
- Supports: task ID, title, description, acceptance criteria, priority, role, state, dependencies
- Supports in-place state updates (task-marshal `done` updates a task without rewriting the entire file structure)

---

## Decision

The local task file uses **markdown with structured task blocks**. Each task is delimited by a level-2 heading containing the `NativeId` and title.

**Task block format:**

```markdown
## TASK-042: Short descriptive title

**Priority:** P1
**Role:** coder
**State:** unstarted
**Depends:** TASK-040, TASK-041

### Description

Full description of the task.

### Acceptance Criteria

- [ ] Criterion one is satisfied
- [ ] Criterion two is satisfied
```

**Field rules:**

| Field | Required | Format | Default |
|---|---|---|---|
| Heading (`## ID: title`) | Yes | `## <NativeId>: <title>` | — |
| `**Priority:**` | No | `P0`, `P1`, `P2`, `P3` | P2 |
| `**Role:**` | No | Free-form string | (unassigned) |
| `**State:**` | Yes | `unstarted`, `in_progress`, `blocked`, `done` | — |
| `**Depends:**` | No | Comma-separated NativeIds; whitespace around commas is ignored | (none) |
| `### Description` | Yes | Markdown prose | — |
| `### Acceptance Criteria` | Yes | Markdown list | — |

**Parser notes:**
- **Priority:** The parser maps `Pn` to integer `n` (`P0`→0, `P1`→1, etc.). Values above P3 are accepted. An invalid value (e.g., `P-1`, `Pfoo`, empty string) causes the entire task block to be treated as malformed and skipped with a stderr warning.
- **Depends:** Comma-separated NativeIds; whitespace around commas is ignored by the parser.

**NativeId convention:** `TASK-NNN` (three or more digits). The parser accepts any non-whitespace, non-colon string as a NativeId.

**In-place update by `done`:**
The `**State:**` line is located and its value is replaced with `done`. Written via atomic rename: a temp file is created **in the same directory as the target task file** (not the system temp directory), then renamed over the original. Same-directory placement is required — cross-filesystem renames fail with `EXDEV`. Use `tempfile::Builder::new().tempfile_in(parent_dir)` and call `.persist(target_path)` to rename atomically.

**Comment annotation by `done --comment`:**
A `**Completion Note:**` line is inserted immediately after the updated `**State:**` line.

```markdown
**State:** done
**Completion Note:** Fixed in commit abc123
```

**Parser tolerance:**

- Trailing whitespace on lines is ignored
- Both `\r\n` and `\n` line endings are accepted
- A malformed task block (missing required `### Description` or `### Acceptance Criteria`) is skipped with a stderr warning; other tasks are unaffected
- Content between task blocks (before the first `##` heading) is preserved unchanged on write

---

## Rationale

- Markdown is the natural format for `.llm/` directories and is already used for agent instructions
- Level-2 headings as task delimiters are visually clear and produce meaningful git diffs
- Inline bold metadata (`**Key:** Value`) is easy for humans to read and AI agents to author
- The format is minimal — no YAML frontmatter, no custom syntax
- In-place update via `**State:**` line replacement is straightforward text manipulation

---

## Alternatives Considered

| Alternative | Why rejected |
|---|---|
| YAML frontmatter per task | Complex to parse with varying frontmatter blocks; YAML indentation errors are common |
| TOML list of task objects | Not human-readable inline with the task content; requires all tasks in one TOML file |
| JSON | Poor readability; not author-friendly for AI agents writing tasks |
| SQLite per project | Binary; not git-diffable; heavyweight |
| Separate file per task | Too many files; harder to reorder and review; more filesystem operations |

---

## Consequences

**Positive:**

- Human-readable and author-friendly — agents and developers can read and edit the file directly
- Git-diffable — state changes show as minimal one-line diffs
- No binary format — no special tools needed to inspect the task file

**Negative / Tradeoffs:**

- Requires a custom markdown parser (not a general markdown renderer — only task block structure is parsed)
- In-place update is more complex than replacing the entire file (must preserve surrounding content exactly)
- Malformed blocks are silently skipped (with warning) rather than causing a hard error — requires careful authoring

---

## References

- [responsibilities.md](../spec/responsibilities.md) — LocalFileSource card
- [operations.md](../spec/operations.md) — Local Task File Format section
- [edge-cases.md](../spec/edge-cases.md) — EC-06, EC-07, EC-08, EC-09, EC-10
- [assertions.md](../spec/assertions.md) — A13, A17, A18
- [security.md](../spec/security.md) — T2: path traversal mitigation
- User requirements §6.1, §8.2 (local behaviour)
