## Context

The repository is a multi-tool workspace (Beads UI, OpenSpec) with no Rust code today. For a minimal hello-world baseline, we adopt the standard Cargo layout at the repository root.

## Goals / Non-Goals

**Goals:**

- Provide a single default binary runnable with `cargo run` from the repo root.
- Use current stable Rust idioms (`edition = "2021"`) without extra crates.
- Keep verification simple: `cargo check` / `cargo run`.

**Non-Goals:**

- No library crate split, workspaces, FFI, async stack, CLI argument parsing, or CI integration in this change.
- No pinning toolchain via `rust-toolchain.toml` unless the team adopts that separately.

## Decisions

- **Root Cargo package**: One `[package]` in the repo root so `cargo` commands run without `-p`/`--manifest-path`. Rationale: matches the approved plan and keeps the scaffold obvious.
- **Package name**: `try_idle_routines` — valid Rust crate identifier aligned with the repository directory name `try-idle-routines`. Hyphens are invalid in Cargo `name`; underscores are canonical.
- **Binary entry**: `src/main.rs` only; default binary inferred by Cargo.

## Risks / Trade-offs

- **Rust not installed locally** → contributors cannot build; mitigation: document commands in [`CLAUDE.md`](../../../CLAUDE.md) after implementation.
