# claudini

CLI for switching between multiple Claude Code accounts on macOS.

## Why

Claude Code doesn't support multiple accounts. If you use Claude Code with different accounts — say a personal one and a work one — switching means manually replacing `~/.claude.json` and swapping the credential stored in the macOS Keychain. Do it wrong and you lose your config, break your session, or leak one account's credential into another.

**claudini** solves this by managing named profiles that bundle both the config file and the keychain credential. You switch accounts with a single command (`claudini use work`), and everything else — syncing shared settings, swapping credentials, updating symlinks — happens automatically. All credentials stay in the macOS Keychain; nothing is ever written to disk as plain text.

## How it works

- Each profile stores its own `claude.json` on disk and its credential in the macOS Keychain
- `~/.claude.json` becomes a **symlink** pointing to the active profile's `claude.json`
- On switch, the keychain credential is swapped and **shared fields** (projects, settings, usage history) are synced from the outgoing profile to the incoming one
- Account-specific fields (OAuth tokens, user ID, org caches) stay per-profile
- Credentials never touch the filesystem — they are read from and written to the Keychain directly

### Storage layout

```
~/.claudini/
  config.json                  # { "active_profile": "personal" }
  profiles/
    personal/
      claude.json              # account-specific claude.json
    work/
      claude.json
  backups/
    before-upgrade/
      claude.json              # snapshot of ~/.claude.json
      claude/                  # snapshot of ~/.claude/ directory

macOS Keychain:
  "Claude Code-credentials"          # active credential (read by Claude Code)
  "claudini-profile-personal"        # personal profile credential
  "claudini-profile-work"            # work profile credential
  "claudini-backup-before-upgrade"   # backup credential
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

### Before you start

Create a backup of your current Claude Code configuration before making any changes:

```bash
claudini backup create init
```

This saves a snapshot of your `~/.claude.json`, keychain credential, and `~/.claude/` directory so you can restore if anything goes wrong.

### First-time setup

```bash
claudini init
```

Creates the `~/.claudini/` directory structure and `config.json`. If already initialized, migrates any legacy plain text credential files to the Keychain.

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
claudini use work
```

Syncs shared fields, swaps the symlink and keychain credential. `claudini use` is a shortcut for `claudini profile use`.

To switch and immediately launch Claude Code in one step:

```bash
claudini use work --launch
# or short form:
claudini use work -l
```

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

Deletes a profile's stored data and its Keychain credential. Cannot remove the currently active profile.

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

### Delete a backup

```bash
claudini backup delete before-upgrade
```

Deletes a backup's stored data and its Keychain credential.

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
- `claude` CLI on PATH (for `add --login` and `use --launch`)
