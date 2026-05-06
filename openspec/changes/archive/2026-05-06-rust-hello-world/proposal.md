## Why

This repository had no Rust tooling or entrypoint, which makes it harder to experiment with or extend the project in Rust. Adding a minimal binary scaffold establishes a standard Cargo layout and a verifiable baseline (`cargo run`).

## What Changes

- Add a root-level Cargo package with a default binary that prints `Hello, world!`.
- Document basic build/run commands for contributors (in project agent instructions where appropriate).

## Capabilities

### New Capabilities

- `rust-cli-hello`: Minimal Rust binary in the repository root; runnable via Cargo; prints a fixed greeting to stdout.

### Modified Capabilities

- *(none — no existing specs in `openspec/specs/`)*

## Impact

- New files: `Cargo.toml`, `src/main.rs`, and Rust-related `.gitignore` entries if missing.
- Developers need a Rust toolchain locally to build and run the binary; no new runtime service or external API.
