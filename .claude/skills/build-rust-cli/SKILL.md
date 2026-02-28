---
name: build-rust-cli
description: Best practices for building CLI tools in Rust
---

# Rust CLI Best Practices

Reference guide for building and testing CLI applications in Rust. Synthesized from the [Rust CLI Book](https://rust-cli.github.io/book/), [clap examples](https://github.com/clap-rs/clap/tree/master/examples), and [ripgrep](https://github.com/BurntSushi/ripgrep).

## Project Structure

- **Keep `main.rs` thin.** Parse args, load settings, dispatch to command handlers. Business logic lives in dedicated modules (`commands.rs`, `client.rs`, etc.).
- **One module per concern.** Separate argument parsing, configuration, HTTP client, and command handlers into distinct files.
- **Extract params structs** when command handlers need more than ~7 arguments. Use `'a` lifetime params borrowing from the caller.
- For larger CLIs, consider a **workspace** with focused sub-crates (ripgrep pattern). For smaller CLIs, modules within a single crate are fine.

## Argument Parsing (clap derive)

- **Use `#[derive(Parser)]` and `#[derive(Subcommand)]`** for structured, type-safe argument parsing.
- **Always add `version`** to `#[command()]` so `--version` works.
- **Use `ValueEnum`** for constrained string choices (e.g., priority levels, output formats). This gives free validation, help text, and tab completion instead of opaque API errors from invalid values.
- **Use doc comments** (`///`) on struct fields — clap uses them as help text automatically.
- **Use `#[arg(short, long)]`** for common flags.
- **Use `#[arg(long, default_value = "...")]`** for options with sensible defaults.
- **Use `Vec<T>`** for repeatable flags (e.g., `--label id1 --label id2`).

## Error Handling

- **Use `anyhow`** for application-level errors. Use `thiserror` only for library crates.
- **Always add `.context()`** or `.with_context()` to give errors meaning. "failed to read config" is better than "No such file or directory".
- **Never `.unwrap()` in production paths.** Use `?` with context instead.
- **Handle errors in main explicitly** rather than returning `Result` from `main()`:
  ```rust
  fn main() {
      if let Err(err) = run() {
          eprintln!("error: {err:#}");
          std::process::exit(1);
      }
  }
  ```
  This lets you style the error output (red, bold prefix) and control the format.
- **Handle HTTP status codes specifically**: 401 (unauthorized), 404 (not found), 429 (rate limited), 5xx (server error) — each with a user-friendly message.
- **Graceful partial failure**: when iterating over multiple items (files, API resources), collect errors and continue rather than aborting on the first failure.

## Output (stdout vs stderr)

- **stdout is for data.** This is what gets piped to other programs or redirected to files.
- **stderr is for diagnostics.** Errors, warnings, spinners, progress bars, log messages.
- **Spinners/progress bars** (`indicatif`) default to stderr — keep it that way.
- **Provide a `--json` flag** for machine-readable output. When active, disable spinners and colors. Use `serde_json::to_string_pretty()` for human-readable JSON.
- **Use `comfy_table`** for tabular human output with UTF-8 borders and colored headers.
- **Use `console`** crate for styled text (bold, colors, dim).
- **Detect if stdout is a terminal** (`IsTerminal` trait, stable since Rust 1.70) to auto-disable colors when piped.

## Configuration

- **Layer configuration** with clear precedence (each layer overrides the previous):
  1. Hardcoded defaults (`Default` impl)
  2. System/project config files (JSON, TOML)
  3. User-local config files (git-ignored)
  4. Environment variables (`APP_KEY`, `APP_BASE_URL`, etc.)
  5. CLI arguments (highest priority)
- **Use deep merge** for nested config objects — overlay keys from higher layers, preserve unset keys from lower layers.
- **Silently skip missing config files** (not an error if the file doesn't exist).
- **Error on malformed config files** with context including the file path.

## Testing

### Unit Tests (in-module `#[cfg(test)]`)
- Test pure logic: config merging, data transformation, validation.
- Use `wiremock` for HTTP mocking — create a `MockServer`, mount expectations, pass the mock URI as the base URL.
- Use `tempfile::TempDir` for filesystem tests.
- Use `temp-env::with_vars` + `serial_test::serial` for environment variable tests (prevents contamination between tests).
- Group tests by concern with separator comments: `// ── Construction ──`, `// ── Error handling ──`.

### Integration Tests (`tests/cli.rs`)
- Use `assert_cmd` to run the compiled binary as a subprocess.
- Use `predicates` for composable stdout/stderr assertions.
- **Test both JSON and human output modes** for every command.
- **Test error conditions**: missing required args, API errors (401, 404), invalid enum values.
- **Test empty results**: verify "No X found." messages.
- **Test query parameter forwarding**: use `wiremock::matchers::query_param` to verify filters reach the server.
- **Create a helper function** (`plane_cmd_with(mock_uri)`) to reduce boilerplate in tests that need a mock server.
- **Use `.expect(1)`** on mocks when you want to verify the request was actually made.

### Test patterns from ripgrep
- **Dir + TestCommand pattern**: create an isolated temp directory per test, run the binary, assert on stdout/stderr/exit code.
- **Self-contained flag tests**: each flag definition has its own test that parses args and verifies the resulting config.
- **Snapshot testing** (`trycmd`/`snapbox`): write expected input/output as files, run them automatically. Good for regression testing help text and output format.

## Exit Codes

- **0** = success.
- **1** = general failure (default when `main()` returns `Err`).
- **2** = usage error (invalid arguments — clap handles this automatically).
- For search-like tools, follow grep convention: 0 = found, 1 = not found, 2 = error.
- Use `std::process::exit(code)` when you need explicit control.

## Dependencies (recommended stack)

| Purpose | Crate |
|---------|-------|
| Arg parsing | `clap` (derive) |
| Error handling | `anyhow` |
| Serialization | `serde` + `serde_json` |
| HTTP client | `reqwest` (json, rustls-tls) |
| Async runtime | `tokio` |
| Terminal colors | `console` |
| Progress bars | `indicatif` |
| Tables | `comfy-table` |
| HTTP mocking | `wiremock` |
| CLI testing | `assert_cmd` + `predicates` |
| Temp files | `tempfile` |
| Env var testing | `temp-env` + `serial_test` |

## Misc

- **Run `cargo fmt` and `cargo clippy --all-targets -- -D warnings`** before every commit.
- **`edition = "2024"`** (or latest stable) in Cargo.toml.
- **Fill in Cargo.toml metadata**: description, license, repository, authors.
- **Use `rustls-tls`** instead of native-tls for easier cross-compilation.
- **Set `default-features = false`** on `reqwest` to avoid pulling in OpenSSL.
