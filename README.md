# claudini

CLI for switching between multiple Claude Code accounts on macOS.

Claude Code stores authentication in two places: `~/.claude.json` (account metadata) and the macOS Keychain (`Claude Code-credentials`). **claudini** manages named profiles that bundle both, letting you switch accounts with a single command.

## How it works

- Each profile stores its own copy of `claude.json` and the keychain credential
- `~/.claude.json` becomes a **symlink** pointing to the active profile's `claude.json`
- On switch, the keychain credential is swapped and **shared fields** (projects, settings, usage history) are synced from the outgoing profile to the incoming one
- Account-specific fields (OAuth tokens, user ID, org caches) stay per-profile

### Storage layout

```
~/.claudini/
  config.json                  # { "active_profile": "personal" }
  profiles/
    personal/
      claude.json              # account-specific claude.json
      credentials              # keychain credential for this account
    work/
      claude.json
      credentials
  backups/
    before-upgrade/
      claude.json              # snapshot of ~/.claude.json
      credentials              # snapshot of keychain credential
      claude/                  # snapshot of ~/.claude/ directory
```

## Installation

### Quick install

```bash
curl -fsSL https://raw.githubusercontent.com/kimrgrey/claudini/main/install.sh | sh
```

The script detects your Mac's architecture (Apple Silicon or Intel), downloads the matching binary from the latest GitHub Release, and installs it to `/usr/local/bin/claudini`.

To install to a custom location:

```bash
INSTALL_DIR=~/.local/bin curl -fsSL https://raw.githubusercontent.com/kimrgrey/claudini/main/install.sh | sh
```

### From GitHub Releases

Download the binary for your architecture from the [latest release](https://github.com/kimrgrey/claudini/releases/latest):

- **Apple Silicon:** `claudini-aarch64-apple-darwin`
- **Intel:** `claudini-x86_64-apple-darwin`

Then make it executable and move it to your PATH:

```bash
chmod +x claudini-*
mv claudini-* /usr/local/bin/claudini
```

### From source

```bash
cargo install --path .
```

Or build locally:

```bash
cargo build --release
# binary at target/release/claudini
```

## Usage

### First-time setup

```bash
claudini init
```

Creates the `~/.claudini/` directory structure and `config.json`.

### Save your current account as a profile

```bash
claudini profile add personal
```

Reads the current `~/.claude.json` and keychain credential, saves them as the `personal` profile, and replaces `~/.claude.json` with a symlink.

### Add a new account via OAuth login

```bash
claudini profile add --login work
```

Saves the current credential, clears auth, launches `claude` for interactive OAuth login, then saves the new account as the `work` profile.

### Switch between profiles

```bash
claudini profile use work
```

Syncs shared fields, swaps the symlink and keychain credential.

### List profiles

```bash
claudini profile list
```

Shows all profiles in a table, with the active one marked.

### Show current profile

```bash
claudini profile current
```

Prints the active profile name and associated email address.

### Rename a profile

```bash
claudini profile rename work work-old
```

Renames a profile. If it's the active profile, the symlink and config are updated automatically.

### Remove a profile

```bash
claudini profile remove work
```

Deletes a profile's stored data. Cannot remove the currently active profile.

### Create a backup

```bash
claudini backup create before-upgrade
```

Saves a snapshot of `~/.claude.json`, the keychain credential, and the `~/.claude/` directory.

### Restore a backup

```bash
claudini backup restore before-upgrade
```

Replaces current config, credentials, and `~/.claude/` directory from the named backup.

### List backups

```bash
claudini backup list
```

Shows all available backup names.

## JSON output

All commands support `--json` for machine-readable output:

```bash
claudini --json profile list
claudini --json profile current
claudini --json backup list
```

Errors are returned as `{"error": "..."}`.

## Testing without touching real config

Override the Claude home directory to test safely:

```bash
# Via CLI flag
claudini --claude-home /tmp/test-claude profile add test

# Via environment variable
export CLAUDINI_CLAUDE_HOME=/tmp/test-claude
claudini profile add test
```

When overridden, claudini looks for `<claude-home>/.claude.json` instead of `~/.claude.json`.

## Field sync details

When switching profiles, claudini copies **shared fields** from the outgoing profile to the incoming one. This keeps things like project configs, tips history, and settings consistent across accounts.

**Account-specific fields** (preserved per-profile):
`oauthAccount`, `userID`, `s1mAccessCache`, `groveConfigCache`, `passesEligibilityCache`, `hasShownOpus46Notice`, `cachedGrowthBookFeatures`, `cachedExtraUsageDisabledReason`, `penguinModeOrgEnabled`, `clientDataCache`, `claudeCodeFirstTokenDate`, `hasVisitedExtraUsage`, `hasVisitedPasses`, `passesLastSeenRemaining`

**Shared fields** (synced on switch): everything else — `projects`, `githubRepoPaths`, `tipsHistory`, `toolUsage`, `skillUsage`, `numStartups`, `installMethod`, `autoUpdates`, `hasCompletedOnboarding`, etc.

## Requirements

- macOS (uses macOS Keychain via the `keyring` crate with `apple-native` feature)
- Rust 1.70+
- `claude` CLI on PATH (for `add --login`)
