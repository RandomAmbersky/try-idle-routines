## Why

The default binary only prints to stdout, which does not exercise terminal UI patterns needed for richer CLI experiences. Adding **ratatui** with a minimal full-screen hello establishes a TUI baseline the project can grow from.

## What Changes

- Add **ratatui** and **crossterm** (terminal backend) as Cargo dependencies for the root package.
- Replace the default binary entrypoint with a minimal TUI that renders a **Hello, world!** greeting and runs a small event loop until the user quits (e.g. **q** or **Ctrl+C**).
- **BREAKING**: Observable behavior changes from a single `println!` line on stdout to alternate-screen TUI output; automated checks that expected plain stdout text must be updated.

## Capabilities

### New Capabilities

- *(none — requirements live under the existing `rust-cli-hello` capability)*

### Modified Capabilities

- `rust-cli-hello`: Replace the “single line to stdout / no non-stdlib deps” requirements with a TUI-based greeting, allowed third-party deps (ratatui, crossterm), and explicit run/exit scenarios.

## Impact

- [Cargo.toml](Cargo.toml): new dependencies.
- [src/main.rs](src/main.rs): TUI initialization, render loop, cleanup.
- [openspec/specs/rust-cli-hello/spec.md](openspec/specs/rust-cli-hello/spec.md): updated via delta in this change.
- Contributors need a terminal that supports crossterm (typical modern terminal); `cargo run` is interactive.
