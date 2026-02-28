# claudini

CLI tool for switching between multiple Claude Code accounts on macOS.

## Project structure

```
claudini/
  Cargo.toml
  src/
    main.rs        # CLI entry point (clap derive), subcommand enums, output formatting
    config.rs      # Path helpers, Config struct, directory layout
    keychain.rs    # macOS Keychain read/write/delete via keyring crate
    profile.rs     # Profile operations: init, add, add --login, use, list, remove, rename, current
    backup.rs      # Backup operations: create, restore, list
    sync.rs        # Shared field sync between profiles
```

## CLI

```
claudini init
claudini profile add <name> [--login]
claudini profile use <name>
claudini profile list
claudini profile remove <name>
claudini profile rename <old> <new>
claudini profile current
claudini backup create <name>
claudini backup restore <name>
claudini backup list
```

## Architecture

- Profiles stored under `~/.claudini/profiles/<name>/` with `claude.json` + `credentials`
- Backups stored under `~/.claudini/backups/<name>/` with `claude.json`, `credentials`, and `claude/` dir
- `~/.claude.json` is a symlink to the active profile's `claude.json`
- Keychain entry `Claude Code-credentials` is swapped on profile switch
- Shared fields (non-account-specific) are synced from outgoing to incoming profile on switch

## Key conventions

- All errors use `anyhow` for context-rich error chains
- Two output modes: human (colored, spinners, tables) and JSON (`--json` flag)
- Claude home directory overridable via `--claude-home` flag or `CLAUDINI_CLAUDE_HOME` env var
- Account-specific fields listed in `sync.rs::ACCOUNT_SPECIFIC_FIELDS`

## Git & PR conventions

- Do not add Claude as a co-author to commit messages and PR descriptions
- Do not commit unless explicitly asked to
- Do not push branches unless explicitly asked to
- Never force push (`git push --force` / `git push -f`) branches to GitHub
- To resolve conflicts with main, use `git pull origin main` instead of `git rebase` (rebase requires force push)
- Always ask before using `--admin` flag when merging PRs — it bypasses branch protection checks
- Try to keep branch names short but readable
- Always remove previously added but now unused code

## Build & run

```bash
cargo build
cargo run -- <command>
```

## Dependencies

- `clap` (derive) — CLI parsing
- `serde` + `serde_json` — JSON handling
- `dirs` — home directory resolution
- `anyhow` — error handling
- `keyring` (apple-native) — macOS Keychain access
- `console` — terminal colors/styling
- `indicatif` — progress spinners
- `comfy-table` — formatted table output
