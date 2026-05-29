# ADR-0001: Language and Distribution

**Date:** 2026-05-24
**Status:** Accepted

---

## Context

task-marshal is a CLI tool used by both developers and AI agents. It must:

- Start quickly (agents call it at session start)
- Run on macOS, Linux, and Windows
- Produce a distributable artifact that requires no runtime installation
- Have minimal startup overhead

The `.gitignore` in the repository already contains Cargo-generated patterns, confirming Rust was the intended language.

---

## Decision

task-marshal is implemented in **Rust**, compiled to a **single statically-linked binary** targeting each supported platform. No managed runtime is required.

---

## Rationale

- Rust compiles to native code with no runtime dependency
- Fast startup — important for agents that invoke it programmatically
- Single binary distribution simplifies installation (copy to PATH)
- Strong type system enforces correctness constraints at compile time
- `std::process::Command` provides safe, ergonomic subprocess execution
- Rich ecosystem for CLI (clap), config parsing (toml), JSON (serde_json), and error handling (thiserror)

---

## Alternatives Considered

| Alternative | Why rejected |
|---|---|
| Go | Would also produce a single binary; no strong reason to prefer it over Rust given the existing Cargo configuration in the repo |
| Python | Requires Python runtime; startup overhead; packaging a single binary requires PyInstaller or similar |
| Node.js / TypeScript | Requires Node.js runtime; startup overhead not suitable for agent bootstrap use |
| Shell script | Portable but not maintainable for complex logic; poor error handling; no type system |

---

## Consequences

**Positive:**
- Single binary; no runtime installation beyond copying to PATH
- Fast startup (<100ms expected for typical invocations)
- Memory safety guarantees from the Rust type system
- Compile-time verification of all error paths via `Result<T, E>`

**Negative / Tradeoffs:**
- Cannot link against `bd` or `gh` as libraries; all BEADS and GitHub integration must be via subprocess calls
- Cross-compilation required for Windows/Linux/macOS release artifacts
- Longer compile times compared to interpreted languages (acceptable for a CLI tool)

---

## References

- [constraints.md](../spec/constraints.md) — toolchain and dependency constraints
- User requirements §4 — non-goals (not a service; local dev tool)
