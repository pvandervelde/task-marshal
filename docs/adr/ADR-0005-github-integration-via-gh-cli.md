# ADR-0005: GitHub Integration via `gh` CLI Subprocess

**Date:** 2026-05-24
**Status:** Accepted

---

## Context

task-marshal needs to list open GitHub Issues (filtered by label) and close issues on behalf of users and agents. This requires authenticated access to the GitHub API.

The requirements (§6.3) already mandate: "Requires the `gh` CLI to be authenticated in the environment."

A decision is needed on how to call the GitHub API.

---

## Decision

GitHub operations are performed by spawning the **`gh` CLI** as a subprocess using `std::process::Command`. task-marshal does not call the GitHub REST or GraphQL API directly. Arguments are passed as separate process arguments; no shell interpolation is used.

Relevant commands:

- `gh issue list --repo <owner/repo> --label <label> --json number,title,body,labels,state`
- `gh issue view <number> --json number,title,body,labels,state`
- `gh issue close <number>`
- `gh issue comment <number> --body "<text>"`

If `gh` is not in PATH, not authenticated, or the network is unavailable, `GithubSource` returns `SourceError::Unavailable`.

---

## Rationale

- The requirements already mandate `gh` authentication — no new dependency is introduced
- Auth is fully delegated to `gh`; task-marshal never handles GitHub tokens
- `gh` handles rate limiting, retries, and HTTP connection management
- No TLS or HTTP client library is needed in task-marshal's binary, keeping it small and free of SSL CVEs
- JSON output from `gh --json` is stable and well-documented

---

## Alternatives Considered

| Alternative | Why rejected |
|---|---|
| Direct GitHub REST API with token from `GITHUB_TOKEN` env var | Requires task-marshal to manage token (read env, pass as header); introduces HTTP client dependency; duplicates functionality `gh` already provides |
| GitHub GraphQL API | More efficient for bulk queries but adds complexity; not justified for a tool fetching at most dozens of issues |
| `octocrab` Rust crate | Adds a significant dependency (async, TLS); requires token management; overkill for subprocess-equivalent operations |

---

## Consequences

**Positive:**

- Zero credential management in task-marshal
- `gh` handles authentication flow, token refresh, and enterprise GitHub URLs
- `gh` is already a standard part of developer and agent environments that use GitHub

**Negative / Tradeoffs:**

- Runtime dependency on `gh` binary in PATH; graceful degradation (skip source) if absent
- Cannot distinguish "not authenticated" from "network unavailable" — both surface as `SourceUnavailable`
- If `gh` hangs (e.g., DNS timeout with no connection refused), task-marshal also hangs (v1 does not implement subprocess timeouts). **v2 extension point:** timeout can be added by spawning via `Command::spawn()` (non-blocking), then racing `child.wait()` against a timer in a separate thread. The `CommandRunner` trait abstraction (adopted for testing) is the natural injection point for this.
- GitHub Issues have no dependency model; all issues are treated as unblocked (v1 limitation)

---

## Security Notes

- Arguments are passed as separate `std::process::Command` arguments, never shell-interpolated
- See [security.md](../spec/security.md) — T1: command injection mitigation

---

## References

- [architecture.md](../spec/architecture.md) — `GithubSource` infrastructure section
- [responsibilities.md](../spec/responsibilities.md) — GithubSource card
- [security.md](../spec/security.md) — T1, T3
- [edge-cases.md](../spec/edge-cases.md) — EC-03, EC-04
- User requirements §6.3, §8.2 (github behaviour)
