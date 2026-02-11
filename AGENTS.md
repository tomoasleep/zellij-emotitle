# AGENTS Guide for zellij-emotitle
This file is for agentic coding assistants working in this repository.
Follow these project-specific commands and conventions.

## Scope and Purpose
- Project: `zellij-emotitle`
- Language: Rust (edition 2021)
- Artifact: Zellij plugin (`.wasm`) for `wasm32-wasip1`
- Entry point: `src/main.rs`
- Core modules: `src/command.rs`, `src/state.rs`
- Behavior: add emoji suffixes to pane/tab titles and restore titles in temp mode

## Repository Layout
- `Cargo.toml`: crate config and dependencies
- `.cargo/config.toml`: default build target (`wasm32-wasip1`)
- `src/main.rs`: plugin lifecycle, event handling, command dispatch
- `src/command.rs`: pipe argument parsing and validation
- `src/state.rs`: runtime state and restore behavior
- `README.md`: build and usage examples

## Toolchain and Environment Notes
- Rust toolchain expected (`rustup`, `cargo`)
- Release artifact target: `wasm32-wasip1`
- Plain `cargo build` uses `wasm32-wasip1` because target is pinned
- Typical Apple Silicon host target: `aarch64-apple-darwin`

## Build Commands
- Install wasm target once: `rustup target add wasm32-wasip1`
- Debug build (default target): `cargo build`
- Release build for distribution: `cargo build --release --target wasm32-wasip1`
- Expected artifact: `target/wasm32-wasip1/release/zellij_emotitle.wasm`

## Format and Lint Commands
- Format all code: `cargo fmt --all`
- Format check only: `cargo fmt --all -- --check`
- Clippy gate: `cargo clippy --all-targets -- -D warnings`
- Host-target clippy (if needed): `cargo clippy --all-targets --target aarch64-apple-darwin -- -D warnings`

## Test Commands
Important context:
- Tests live in binary-crate modules (`src/command.rs`, `src/state.rs`)
- `cargo test` on default target tries to execute wasm and fails
- Host-target test build currently fails at link time due unresolved Zellij host symbols from `zellij_tile`
- Tests exist but are not directly executable in the current crate layout

Commands to know:
- All tests (expected failure on wasm): `cargo test`
- All tests on host (currently link failure): `cargo test --target aarch64-apple-darwin`
- Single test by exact name (same current limitation):
  - `cargo test --target aarch64-apple-darwin parse_pane_temp_command -- --exact`
- Single test by module path:
  - `cargo test --target aarch64-apple-darwin command::tests::parse_pane_temp_command -- --exact`

Practical validation strategy today:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --release --target wasm32-wasip1`

## Development Workflow
- Prefer TDD where feasible: Red -> Green -> Refactor
- Keep changes minimal and scoped to requested behavior
- Avoid broad refactors unless required
- Preserve pipe-argument behavior and title-format contract

## Code Style Guidelines

### Imports
- Group imports with blank lines by origin: std, internal modules, external crates
- Keep imports explicit; avoid wildcard imports
- Remove unused imports quickly

### Formatting
- Follow `rustfmt` defaults (no project rustfmt config)
- Use 4-space indentation
- Keep long chains readable by line breaks consistent with existing style
- Prefer trailing commas in multiline literals and matches

### Types and Data Modeling
- Use enums for closed sets (`Mode`, `Target`, `PaneRef`)
- Use structs for grouped payloads/state (`Command`, `Entry`, `EmotitleState`)
- Derive traits intentionally (`Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Default`)
- Use `Option<T>` for optional args and derived state
- Use `Result<T, String>` for user-facing parse/apply errors

### Naming Conventions
- Types/enums: `UpperCamelCase`
- Functions/methods/modules/variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Prefer descriptive names tied to domain behavior

### Error Handling
- Return actionable parse/apply error messages
- Prefer `ok_or_else` and `map_err` where clearer than manual branching
- Avoid `unwrap`/`expect` in production code
- `unwrap` is acceptable in tests for setup and expectations
- Include argument names and invalid values when useful

### Control Flow and Logic
- Prefer `match` for enum-driven branching (`Target`, `Event`)
- Keep event handlers deterministic and side-effect boundaries clear
- Preserve tab index semantics:
  - internal `tab_index` is zero-based
  - `rename_tab` API expects one-based index (`tab_index + 1`)
- Do not change temp/permanent restore semantics unless requested

### State Management
- `EmotitleState` is the single source of runtime plugin state
- Update manifests first, then derive focused entities and restores
- Preserve originally captured titles when updating entries
- Keep pane and tab logic symmetrical unless API behavior differs

### Comments and Documentation
- Prefer clear code over explanatory comments
- Add comments only for non-obvious constraints
- Keep README examples aligned with actual argument behavior

### Testing Practices
- Keep tests close to logic in `#[cfg(test)]` modules
- Cover both success and invalid-input branches
- Focus tests on parse validation and state transitions
- Use small local helpers in tests (`map`, `pane_info`, `tab_info`)

## Zellij Plugin-Specific Guardrails
- Keep plugin non-selectable with `set_selectable(false)` unless requirements change
- Keep permissions minimal and explicit
- Keep subscriptions for `PaneUpdate` and `TabUpdate`
- Preserve pipe name contract: `emotitle`
- Preserve pipe response contract: stdout `ok` or error via `cli_pipe_output`

## Change Safety Rules
- Do not change `.cargo/config.toml` target defaults unless explicitly requested
- Do not rename plugin pipe command or alter existing argument names casually
- Keep `target=tab` validation behavior (`pane_id` and `tab_index` cannot be combined)
- Avoid changing `title_with_emojis` output format contract without a request
- If changing restore behavior, verify both pane and tab temp-mode flows
- Prefer additive changes over broad rewrites in `src/main.rs`
- Maintain compatibility with README command examples
- Update this guide when build/test limitations or workflow assumptions change

## Rules Files Check
No Cursor or Copilot instruction files were found during analysis:
- `.cursorrules`: not found
- `.cursor/rules/`: not found
- `.github/copilot-instructions.md`: not found

If these files are added later, update this guide and treat them as higher-priority repository policy.
