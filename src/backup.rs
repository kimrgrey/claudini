use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::config::{backup_dir, claude_json_path, list_backups as config_list_backups};
use crate::keychain;

/// Create a backup of current Claude config, credentials, and data directory.
pub fn create(claudini_dir: &Path, claude_home: &Path, name: &str) -> Result<()> {
    let bdir = backup_dir(claudini_dir, name);
    if bdir.exists() {
        bail!("Backup '{}' already exists", name);
    }
    std::fs::create_dir_all(&bdir)?;

    // Copy claude.json (resolving symlink)
    let cj = claude_json_path(claude_home);
    if cj.exists() {
        std::fs::copy(&cj, bdir.join("claude.json"))
            .context("Failed to copy claude.json")?;
    } else {
        bail!("~/.claude.json not found — nothing to back up");
    }

    // Save keychain credential
    if let Ok(cred) = keychain::read() {
        std::fs::write(bdir.join("credentials"), &cred)?;
    }

    // Copy ~/.claude/ directory
    let claude_data_dir = claude_home.join(".claude");
    if claude_data_dir.is_dir() {
        copy_dir_recursive(&claude_data_dir, &bdir.join("claude"))
            .context("Failed to copy ~/.claude/ directory")?;
    }

    Ok(())
}

/// Restore from a named backup.
pub fn restore(claudini_dir: &Path, claude_home: &Path, name: &str) -> Result<()> {
    let bdir = backup_dir(claudini_dir, name);
    if !bdir.exists() {
        bail!("Backup '{}' does not exist", name);
    }

    // Restore claude.json
    let backup_cj = bdir.join("claude.json");
    if backup_cj.exists() {
        let cj = claude_json_path(claude_home);
        if cj.exists() || cj.is_symlink() {
            std::fs::remove_file(&cj).context("Failed to remove existing .claude.json")?;
        }
        std::fs::copy(&backup_cj, &cj).context("Failed to restore claude.json")?;
    }

    // Restore credentials to keychain
    let backup_cred = bdir.join("credentials");
    if backup_cred.exists() {
        let cred = std::fs::read_to_string(&backup_cred)
            .context("Failed to read backup credentials")?;
        keychain::write(&cred)?;
    }

    // Restore ~/.claude/ directory
    let backup_claude_dir = bdir.join("claude");
    if backup_claude_dir.is_dir() {
        let claude_data_dir = claude_home.join(".claude");
        if claude_data_dir.exists() {
            std::fs::remove_dir_all(&claude_data_dir)
                .context("Failed to remove existing ~/.claude/ directory")?;
        }
        copy_dir_recursive(&backup_claude_dir, &claude_data_dir)
            .context("Failed to restore ~/.claude/ directory")?;
    }

    Ok(())
}

/// List available backups.
pub fn list(claudini_dir: &Path) -> Result<Vec<String>> {
    config_list_backups(claudini_dir)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
