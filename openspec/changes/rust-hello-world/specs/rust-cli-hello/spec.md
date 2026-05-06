## ADDED Requirements

### Requirement: Default binary prints greeting

The system SHALL ship a default Rust binary in the repository root Cargo package that, when executed successfully, writes the line `Hello, world!` followed by a platform-appropriate newline to standard output.

#### Scenario: Successful run

- **WHEN** a developer runs the default binary via Cargo from the repository root (e.g., `cargo run`)
- **THEN** standard output SHALL contain exactly the line `Hello, world!` (with trailing newline per platform convention) and the process SHALL exit with code 0.

### Requirement: Package layout

The repository root SHALL contain a valid Cargo manifest and a `src/main.rs` source file for the default binary that implements the greeting behavior.

#### Scenario: Manifest and source present

- **WHEN** the Cargo manifest and `src/main.rs` are inspected at the repository root
- **THEN** `cargo check` SHALL succeed without pulling non-stdlib dependencies for this package.
