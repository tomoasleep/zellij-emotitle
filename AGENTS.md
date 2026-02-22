# AGENTS Guide for zellij-emotitle
This file is for agentic coding assistants working in this repository.
Follow these project-specific commands and conventions.

## Scope
- Project: `zellij-emotitle`
- Language: Rust (`edition = 2021`)
- Artifact: Zellij plugin wasm (`wasm32-wasip1`)
- Entry point: `src/main.rs`
- Core modules: `src/command.rs`, `src/state.rs`
- Behavior: apply emoji suffixes to pane/tab titles and restore titles in temp mode

## Repository Layout
- `Cargo.toml`: crate metadata and Rust dependencies
- `.cargo/config.toml`: pinned default target (`wasm32-wasip1`)
- `src/main.rs`: plugin lifecycle, event dispatch, pipe handling
- `src/command.rs`: pipe argument parsing and validation
- `src/state.rs`: runtime state and restore logic
- `README.md`: build and usage contract
- `e2e/pane.test.ts`: E2E tests for pane target
- `e2e/tab.test.ts`: E2E tests for tab target
- `e2e/info.test.ts`: E2E tests for info command
- `e2e/test-helpers.ts`: E2E fixtures, config/cache setup, zellij actions

## Toolchain and Runtime Notes
- Rust toolchain is required (`rustup`, `cargo`)
- Build target is pinned to `wasm32-wasip1`, so plain `cargo build` outputs wasm artifacts
- Typical host target on Apple Silicon is `aarch64-apple-darwin`
- Runtime integration is through `zellij_tile` host APIs (not available in normal host test linking)

## Build Commands
- Install target once: `rustup target add wasm32-wasip1`
- Debug build (pinned target): `cargo build`
- Release wasm build: `cargo build --release --target wasm32-wasip1`
- Expected output: `target/wasm32-wasip1/release/zellij_emotitle.wasm`

## Format and Lint Commands
- Format all Rust: `cargo fmt --all`
- Format check only: `cargo fmt --all -- --check`
- Clippy gate (default target): `cargo clippy --all-targets -- -D warnings`
- Optional host clippy: `cargo clippy --all-targets --target aarch64-apple-darwin -- -D warnings`

## Test Commands
Rust unit tests cannot be executed directly (wasm target + `zellij_tile` host symbols limitation).

E2E test command reference (in `e2e/`):
- Run all E2E tests: `bun test`
- Run a single test file: `bun test pane.test.ts`
- Run one E2E case by name regex:
  - `bun test pane.test.ts --test-name-pattern "should apply emojis to focused pane"`
  - `bun test tab.test.ts --test-name-pattern "should apply emojis to the tab"`
- Debug E2E tests: `DEBUG=1 bun test pane.test.ts`
- Build WASM before E2E: `cargo build --release --target wasm32-wasip1`
- WASM path for E2E: `../target/wasm32-wasip1/release/zellij_emotitle.wasm` (relative to e2e/)

Practical validation strategy for code changes today:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --release --target wasm32-wasip1`
- Run targeted `bun test ... --test-name-pattern ...` when behavior-level confidence is needed

## Development Workflow
- Prefer TDD where practical: Red -> Green -> Refactor
- Keep edits minimal and scoped to requested behavior
- Avoid broad refactors unless required by the task
- Preserve existing pipe argument contract and title-format contract

## Rust Code Style Guidelines

### Imports
- Group imports with blank lines by origin: std, crate/internal, external
- Prefer explicit imports over wildcard imports
- Remove unused imports immediately

### Formatting
- Follow `rustfmt` defaults (no repo-specific rustfmt config)
- Use 4-space indentation
- Keep long chains and match arms readable with line breaks
- Prefer trailing commas in multiline literals and match arms

### Types and Data Modeling
- Use enums for closed domain sets (`Mode`, `Target`, `PaneRef`)
- Use structs for grouped state and payloads (`Command`, `EmotitleState`)
- Derive traits intentionally (`Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Default`)
- Use `Option<T>` for optional inputs, `Result<T, String>` for parse/apply failures

### Naming Conventions
- Types/enums/traits: `UpperCamelCase`
- Functions/methods/modules/variables: `snake_case`
- Constants/statics: `SCREAMING_SNAKE_CASE`

### Error Handling
- Return actionable, user-readable error strings
- Include argument names and invalid values where possible
- Prefer `ok_or_else`, `map_err`, and expression-oriented error paths
- Avoid `unwrap`/`expect` in production code (acceptable in tests)

### Control Flow and State Management
- Prefer `match` for enum-driven branching (`Target`, `Event`)
- Treat `EmotitleState` as the single source of runtime truth
- Preserve tab index convention: internal is zero-based, `rename_tab` requires one-based
- Keep pane and tab logic symmetrical unless API behavior requires divergence

### Testing Practices
- Keep tests close to logic in `#[cfg(test)]` modules
- Cover success and invalid-input branches for parsing and state transitions
- Add regression tests when changing restoration or focus behavior

### Documentation
- Prefer clear code over explanatory comments
- Keep `README.md` examples aligned with real argument behavior

## TypeScript/E2E Test Guidelines

### Test Structure
- Use `describe` blocks for grouping related tests
- Use `test.todo` for pending tests, `test.failing` for known failing tests
- Set appropriate timeouts (e.g., `}, 60000)` for complex scenarios)

### Test Helpers
- Use `launchZellijSession()` from `test-helpers.ts` for session setup
- Use `runPipe()` for emotitle pipe commands
- Use `zellijAction()` for zellij actions like `new-tab`, `new-pane`
- Use `getInfo()` to get current pane/tab state via `info` command

### Async and Debugging
- Use `await sleep(ms)` for timing-dependent assertions
- Allow sufficient time for zellij state propagation (100-500ms typical)
- Set `DEBUG=1` environment variable for verbose output

## Zellij Plugin Guardrails
- Keep plugin non-selectable via `set_selectable(false)` unless requirements change
- Keep permissions minimal and explicit
- Keep subscriptions for `PaneUpdate`, `TabUpdate`, and `Timer`
- Preserve pipe name contract: `emotitle`
- Preserve pipe response contract: stdout `ok` or error via `cli_pipe_output`

## Change Safety Rules
- Do not change `.cargo/config.toml` target defaults unless explicitly requested
- Do not rename pipe command or argument keys without strong reason
- Keep `target=tab` validation rule: `pane_id` and `tab_index` are mutually exclusive
- Avoid changing `title_with_emojis` output format contract without a request
- If touching restore behavior, verify both pane and tab focus flows
- Prefer additive changes in `src/main.rs` over broad rewrites
- Maintain compatibility with README command examples

## Cursor and Copilot Rules Check
No additional Cursor/Copilot rule files were found in this repository:
- `.cursorrules`: not found
- `.cursor/rules/`: not found
- `.github/copilot-instructions.md`: not found

If these files are added later, treat them as higher-priority repository policy and update this guide.
