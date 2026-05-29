# Design Tradeoffs

---

## T1: Trait objects (`dyn TaskSource`) vs enum dispatch for source polymorphism

**Decision:** Trait objects (`Box<dyn TaskSource>`)

**Alternatives considered:**

| Approach | Pros | Cons |
|---|---|---|
| `Box<dyn TaskSource>` | Adding a new source requires zero changes to selection/CLI code; clean extension boundary; consistent with requirement §6.4 | Dynamic dispatch overhead (negligible for CLI); compiler cannot exhaustively check all variants |
| Enum of source variants | Exhaustive matching; compiler-checked; slightly faster | Every new source requires editing the enum and all match arms; forces changes across the codebase to add a source |
| Plugin/dynamic loading | True runtime extensibility | Far too complex for a local dev tool; binary distribution complicates |

**Rationale:** The requirement explicitly states "the architecture must allow additional sources to be added without changing the CLI interface." Trait objects satisfy this directly. The performance overhead is immaterial for a CLI tool that makes at most 3 subprocess calls per invocation.

---

## T2: `bd` CLI subprocess vs direct Dolt database access for BEADS

**Decision:** `bd` CLI subprocess

**Alternatives considered:**

| Approach | Pros | Cons |
|---|---|---|
| `bd` subprocess with `--json` flag | Stable interface; BEADS manages its own locking and transactions; no coupling to internal schema; auth and BEADS_DIR handled by `bd` | Runtime dependency on `bd` binary in PATH; subprocess overhead; JSON parsing required |
| Direct Dolt SQL access via Rust driver | Faster; no subprocess overhead; no PATH dependency | Couples to internal BEADS schema (may change); bypasses BEADS' concurrency controls (file locking, WAL); significant complexity; no stable Rust Dolt library exists |
| `.beads/issues.jsonl` file parsing | No subprocess dependency | The README explicitly states this is "an export for viewers and interchange, not the source of truth"; would miss non-issue tables, working-set state; stale between writes |

**Rationale:** BEADS is a CLI tool by design. Its README notes that `issues.jsonl` is not the source of truth. Direct database access would be fragile and bypass safety guarantees. The subprocess approach is the intended integration method.

See `ADR-0006`.

---

## T3: `gh` CLI subprocess vs direct GitHub API calls for GitHub source

**Decision:** `gh` CLI subprocess

**Alternatives considered:**

| Approach | Pros | Cons |
|---|---|---|
| `gh` subprocess | Auth delegated to `gh`; no token management; existing requirement already mandates `gh` auth (§6.3); `gh` handles rate limiting feedback | Runtime dependency on `gh` in PATH; subprocess overhead |
| Direct GitHub REST/GraphQL API with token from env var | No external binary dependency; more control over requests | Requires token management (env var, .netrc, etc.); rate limit handling; TLS/HTTP stack dependency; duplicates functionality `gh` already provides |

**Rationale:** The requirements already state "`gh` CLI must be authenticated in the environment." Using `gh` directly means no new auth burden is introduced.

See `ADR-0005`.

---

## T4: Synchronous I/O vs async/parallel source queries

**Decision:** Synchronous I/O for v1

**Alternatives considered:**

| Approach | Pros | Cons |
|---|---|---|
| Synchronous (`std::process::Command`, `std::fs`) | Simple; fast startup; no async runtime; trivial error handling | Sources queried sequentially; GitHub latency adds to total wall time |
| Parallel source queries (std threads) | Potentially faster wall time | Complexity increase; error propagation across threads; no change to `TaskSource` trait needed |
| Full async (tokio) | Idiomatic for many Rust network tools | Heavyweight runtime; startup overhead; no benefit for 3 subprocess calls; complexity without proportionate value |

**Rationale:** task-marshal is a local dev tool, not a service. Three sequential subprocess calls (each taking 50–500ms) results in at most ~1.5s total, which is acceptable for interactive use. Sources can be parallelised in v2 if benchmarks show it matters, without changing the `TaskSource` trait.

---

## T5: Local task file format — markdown with structured blocks vs TOML/JSON/YAML

**Decision:** Markdown with structured task blocks

**Alternatives considered:**

| Approach | Pros | Cons |
|---|---|---|
| Markdown with structured blocks | Human-readable; git-diffable; familiar to developers and AI agents; inline with existing `.llm/` conventions | Requires a custom parser; editing state requires in-place text manipulation |
| TOML task list | Structured; easy to parse; good Rust support | Less readable for humans; agents less likely to author tasks directly |
| SQLite file | Structured; queryable; ACID | Binary file; not git-diffable; heavyweight for a task list |
| JSON | Structured; easy to parse | Poor readability; not author-friendly |

**Rationale:** The requirements specify markdown (§6.1). It is the natural format for a tool targeting developers and AI agents that work in git repositories. In-place atomic update is the accepted tradeoff.

See `ADR-0009`.

---

## T6: Config walk-up discovery vs fixed path vs env var override

**Decision:** Walk-up discovery (no override in v1)

**Alternatives considered:**

| Approach | Pros | Cons |
|---|---|---|
| Walk-up from CWD (git-style) | Works from any project subdirectory; zero config for users; consistent with `.git` convention | May pick up an ancestor project's config unintentionally |
| Fixed absolute path or env var | Explicit and unambiguous | Requires users to set env vars or use absolute paths; breaks when switching between projects |
| `--config` flag | Explicit override available | Added complexity; not needed for typical usage |

**Rationale:** The requirements specify "similar to `.git` discovery." Walk-up is the right default. An override flag can be added in v2 if multi-project scenarios arise.

See `ADR-0004`.

---

## T7: Selection as pure function vs selection as source-aware process

**Decision:** `TaskSelector` is a pure function over pre-fetched summaries

**Alternatives considered:**

| Approach | Pros | Cons |
|---|---|---|
| Pure function: selector receives `Vec<TaskSummary>` already fetched | Fully testable without mocking sources; clean separation; determinism is trivially verifiable | All sources must be queried upfront even if the first source has a P0 task (minor inefficiency) |
| Lazy/streaming: selector queries sources one at a time, stopping at first hit | Potentially fewer subprocess calls if first source has a P0 task | Couples selector to sources; breaks testability; harder to reason about determinism |

**Rationale:** Correctness and testability outweigh the minor inefficiency of fetching all sources. Priority ordering across sources requires seeing all candidates anyway to guarantee the globally best task is returned.
