## Context

The root crate is a minimal Rust binary ([src/main.rs](src/main.rs)) that currently prints `Hello, world!` to stdout. The product direction needs a **terminal UI** baseline; **ratatui** is the common Rust choice and pairs with **crossterm** for cross-platform input/output.

## Goals / Non-Goals

**Goals:**

- Add **ratatui** + **crossterm** to the root `Cargo.toml` with versions compatible on crates.io at implementation time.
- Implement a **minimal** TUI: draw `Hello, world!`, run an event loop, exit cleanly on user quit (**q**) and handle terminal restore on interrupt (**Ctrl+C**) where feasible.
- Keep the default binary as the single entrypoint (`src/main.rs`).

**Non-Goals:**

- Screens, routing, persistent state, widgets beyond a simple layout, configuration files, or mouse-heavy UX.
- Headless/CI rendering guarantees beyond `cargo check` (interactive `cargo run` is the primary manual check).

## Decisions

1. **ratatui with crossterm backend** — Standard stack; avoids platform-specific backends for this hello. Alternatives: **termion** (Unix-only), **inline** stdout-only — rejected for TUI goal.
2. **Alternate screen / raw mode handled by the stack** — Use ratatui/crossterm APIs (`Terminal::new`, `backend::CrosstermBackend`, `enable_raw_mode` / `EnterAlternateScreen` as per current ratatui examples) so cleanup runs on exit.
3. **Quit: primary path `q`; interrupt acceptable** — Simple UX for demos; Ctrl+C may yield a non-zero exit code depending on signal handling — the spec will treat interrupt as “terminate and restore terminal” without mandating exit code 0 for that path.

## Risks / Trade-offs

- **[Risk]** TUI breaks in dumb pipes or some CI logs → **Mitigation**: `cargo check` remains the non-interactive gate; document that `cargo run` is interactive.
- **[Risk]** Dependency version drift → **Mitigation**: Pin versions in `Cargo.toml` consistent with ratatui’s documented crossterm pairing.

## Migration Plan

- Land dependency + code + spec delta together. Rollback: revert commit and restore `println!` behavior.
- Any automation that asserted plain stdout `Hello, world!` must switch to `cargo check` or a dedicated test harness.

## Open Questions

- None for this hello scope.
