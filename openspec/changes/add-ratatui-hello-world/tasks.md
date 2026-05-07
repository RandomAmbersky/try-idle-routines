## 1. Dependencies

- [x] 1.1 Add `ratatui` and `crossterm` to root [Cargo.toml](Cargo.toml) with compatible versions (per crates.io / ratatui docs at implementation time).

## 2. Default binary (TUI hello)

- [x] 2.1 Replace [src/main.rs](src/main.rs) with a minimal ratatui app: initialize terminal with crossterm backend, render `Hello, world!`, event loop until **q**; ensure terminal restore on exit and reasonable behavior on **Ctrl+C** (per [design.md](design.md)).
- [x] 2.2 Run `cargo check` from the repository root; confirm success.
- [x] 2.3 Manually run `cargo run` in a supported terminal; confirm visible `Hello, world!`, exit **0** on **q**, and usable terminal after **Ctrl+C**.

## 3. Contributor docs (optional but recommended)

- [x] 3.1 Update [CLAUDE.md](CLAUDE.md) **Build & Test** note that `cargo run` is interactive TUI (keep `cargo check` as non-interactive gate).
