# rust-cli-hello Specification Delta

## MODIFIED Requirements

### Requirement: Default binary prints greeting

The system SHALL ship a default Rust binary in the repository root Cargo package that, when run interactively in a supported terminal, uses **ratatui** with a **crossterm** backend to render the visible text `Hello, world!` in the terminal UI and SHALL restore the terminal to a usable state on normal exit.

#### Scenario: Successful interactive run

- **WHEN** a developer runs the default binary via Cargo from the repository root (e.g., `cargo run`) in a supported terminal
- **THEN** the terminal UI SHALL visibly display the text `Hello, world!`

#### Scenario: Quit with q

- **WHEN** the user presses **q** while the TUI is running
- **THEN** the process SHALL exit with code 0 and the terminal SHALL be restored to a usable state

#### Scenario: Interrupt

- **WHEN** the user sends an interrupt signal (e.g., **Ctrl+C**) while the TUI is running
- **THEN** the process SHALL terminate and the terminal SHALL be restored to a usable state

### Requirement: Package layout

The repository root SHALL contain a valid Cargo manifest and a `src/main.rs` source file for the default binary that implements the TUI greeting behavior. The package MAY declare the third-party dependencies **ratatui** and **crossterm** required for that behavior.

#### Scenario: Manifest and source present

- **WHEN** the Cargo manifest and `src/main.rs` are inspected at the repository root
- **THEN** `cargo check` SHALL succeed
