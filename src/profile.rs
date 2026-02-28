use std::path::Path;

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::config::{
    claude_json_path, is_initialized, list_profiles, profile_claude_json, profile_credentials,
    profile_dir, profiles_dir, Config,
};
use crate::keychain;
use crate::sync::sync_shared_fields;

/// Initialize claudini: create directory structure and config.json.
pub fn init(claudini_dir: &Path) -> Result<()> {
    if is_initialized(claudini_dir) {
        bail!("Already initialized (config.json exists at {})", claudini_dir.display());
    }

    std::fs::create_dir_all(profiles_dir(claudini_dir))
        .context("Failed to create profiles directory")?;

    let cfg = Config {
        active_profile: None,
    };
    cfg.save(claudini_dir)?;

    Ok(())
}

/// Add the current auth as a named profile.
pub fn add(claudini_dir: &Path, claude_home: &Path, name: &str) -> Result<()> {
    ensure_initialized(claudini_dir)?;

    let pdir = profile_dir(claudini_dir, name);
    if pdir.exists() {
        bail!("Profile '{}' already exists", name);
    }
    std::fs::create_dir_all(&pdir)?;

    // Save keychain credential
    let cred = keychain::read().context("No keychain credential found — are you logged in?")?;
    std::fs::write(profile_credentials(claudini_dir, name), &cred)?;

    // Handle claude.json
    let cj = claude_json_path(claude_home);
    let target = profile_claude_json(claudini_dir, name);

    if cj.is_symlink() {
        // Already managed — copy the target file
        std::fs::copy(&cj, &target).context("Failed to copy claude.json")?;
    } else if cj.is_file() {
        // Regular file — move it, create symlink back
        std::fs::rename(&cj, &target).context("Failed to move claude.json")?;
    } else {
        bail!("~/.claude.json not found");
    }

    // Ensure symlink points to the new profile
    if cj.exists() || cj.is_symlink() {
        std::fs::remove_file(&cj)?;
    }
    std::os::unix::fs::symlink(&target, &cj)
        .context("Failed to create symlink for .claude.json")?;

    // Update config
    let mut cfg = Config::load(claudini_dir)?;
    cfg.active_profile = Some(name.to_string());
    cfg.save(claudini_dir)?;

    Ok(())
}

/// Clear auth, launch `claude` for OAuth login, then save as a new profile.
pub fn add_with_login(claudini_dir: &Path, claude_home: &Path, name: &str) -> Result<()> {
    ensure_initialized(claudini_dir)?;

    let pdir = profile_dir(claudini_dir, name);
    if pdir.exists() {
        bail!("Profile '{}' already exists", name);
    }

    let cj = claude_json_path(claude_home);
    let cfg = Config::load(claudini_dir)?;

    // Save current keychain credential to current active profile (if any)
    if let Some(ref active) = cfg.active_profile {
        if let Ok(cred) = keychain::read() {
            let _ = std::fs::write(profile_credentials(claudini_dir, active), &cred);
        }
    }

    // Remove symlink / file so claude starts fresh
    if cj.exists() || cj.is_symlink() {
        std::fs::remove_file(&cj)?;
    }

    // Delete keychain credential
    let _ = keychain::delete();

    // Spawn claude interactively
    let status = std::process::Command::new("claude")
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to launch 'claude'. Is it installed and on your PATH?")?;

    if !status.success() {
        bail!("claude exited with status {}", status);
    }

    // Grab new auth
    if !cj.is_file() {
        bail!("claude did not create ~/.claude.json — login may have failed");
    }

    std::fs::create_dir_all(&pdir)?;

    let cred = keychain::read().context("No keychain credential after login")?;
    std::fs::write(profile_credentials(claudini_dir, name), &cred)?;

    let target = profile_claude_json(claudini_dir, name);
    std::fs::rename(&cj, &target).context("Failed to move new claude.json")?;

    std::os::unix::fs::symlink(&target, &cj)
        .context("Failed to create symlink for .claude.json")?;

    let mut cfg = Config::load(claudini_dir)?;
    cfg.active_profile = Some(name.to_string());
    cfg.save(claudini_dir)?;

    Ok(())
}

/// Switch to a different profile.
pub fn switch(claudini_dir: &Path, claude_home: &Path, name: &str) -> Result<()> {
    ensure_initialized(claudini_dir)?;

    let pdir = profile_dir(claudini_dir, name);
    if !pdir.exists() {
        bail!("Profile '{}' does not exist", name);
    }

    let cfg = Config::load(claudini_dir)?;
    let cj = claude_json_path(claude_home);

    // Save current keychain credential to outgoing profile
    if let Some(ref active) = cfg.active_profile {
        if let Ok(cred) = keychain::read() {
            let _ = std::fs::write(profile_credentials(claudini_dir, active), &cred);
        }

        // Sync shared fields from outgoing → incoming
        if active != name {
            if let Ok(from_data) = std::fs::read_to_string(&cj) {
                if let Ok(from_val) = serde_json::from_str::<Value>(&from_data) {
                    let target_path = profile_claude_json(claudini_dir, name);
                    if let Ok(to_data) = std::fs::read_to_string(&target_path) {
                        if let Ok(mut to_val) = serde_json::from_str::<Value>(&to_data) {
                            sync_shared_fields(&from_val, &mut to_val);
                            let _ = std::fs::write(
                                &target_path,
                                serde_json::to_string_pretty(&to_val)?,
                            );
                        }
                    }
                }
            }
        }
    }

    // Remove old symlink/file
    if cj.exists() || cj.is_symlink() {
        std::fs::remove_file(&cj)?;
    }

    // Create new symlink
    let target = profile_claude_json(claudini_dir, name);
    std::os::unix::fs::symlink(&target, &cj)
        .context("Failed to create symlink for .claude.json")?;

    // Write incoming profile's credential to keychain
    let cred_path = profile_credentials(claudini_dir, name);
    let cred = std::fs::read_to_string(&cred_path)
        .with_context(|| format!("No credentials file for profile '{}'", name))?;
    keychain::write(&cred)?;

    // Update config
    let mut cfg = Config::load(claudini_dir)?;
    cfg.active_profile = Some(name.to_string());
    cfg.save(claudini_dir)?;

    Ok(())
}

/// List all profiles, returning (name, is_active) pairs.
pub fn list(claudini_dir: &Path) -> Result<Vec<(String, bool)>> {
    ensure_initialized(claudini_dir)?;

    let cfg = Config::load(claudini_dir)?;
    let names = list_profiles(claudini_dir)?;
    let active = cfg.active_profile.as_deref();

    Ok(names
        .into_iter()
        .map(|n| {
            let is_active = active == Some(n.as_str());
            (n, is_active)
        })
        .collect())
}

/// Remove a profile.
pub fn remove(claudini_dir: &Path, name: &str) -> Result<()> {
    ensure_initialized(claudini_dir)?;

    let cfg = Config::load(claudini_dir)?;
    if cfg.active_profile.as_deref() == Some(name) {
        bail!(
            "Cannot remove the active profile '{}'. Switch to another profile first.",
            name
        );
    }

    let pdir = profile_dir(claudini_dir, name);
    if !pdir.exists() {
        bail!("Profile '{}' does not exist", name);
    }

    std::fs::remove_dir_all(&pdir).context("Failed to remove profile directory")?;

    Ok(())
}

/// Rename a profile.
pub fn rename(claudini_dir: &Path, claude_home: &Path, old_name: &str, new_name: &str) -> Result<()> {
    ensure_initialized(claudini_dir)?;

    let old_dir = profile_dir(claudini_dir, old_name);
    if !old_dir.exists() {
        bail!("Profile '{}' does not exist", old_name);
    }

    let new_dir = profile_dir(claudini_dir, new_name);
    if new_dir.exists() {
        bail!("Profile '{}' already exists", new_name);
    }

    std::fs::rename(&old_dir, &new_dir).context("Failed to rename profile directory")?;

    // Update config if this was the active profile
    let mut cfg = Config::load(claudini_dir)?;
    if cfg.active_profile.as_deref() == Some(old_name) {
        cfg.active_profile = Some(new_name.to_string());
        cfg.save(claudini_dir)?;

        // Re-point the symlink to the new path
        let cj = claude_json_path(claude_home);
        if cj.is_symlink() {
            std::fs::remove_file(&cj)?;
            let target = profile_claude_json(claudini_dir, new_name);
            std::os::unix::fs::symlink(&target, &cj)
                .context("Failed to update symlink for .claude.json")?;
        }
    }

    Ok(())
}

/// Get info about the current profile: (name, email).
pub fn current(claudini_dir: &Path, claude_home: &Path) -> Result<(String, Option<String>)> {
    ensure_initialized(claudini_dir)?;

    let cfg = Config::load(claudini_dir)?;
    let name = cfg
        .active_profile
        .context("No active profile. Use 'claudini add <name>' to create one.")?;

    let cj = claude_json_path(claude_home);
    let email = read_email_from_claude_json(&cj);

    Ok((name, email))
}

fn read_email_from_claude_json(path: &Path) -> Option<String> {
    let data = std::fs::read_to_string(path).ok()?;
    let val: Value = serde_json::from_str(&data).ok()?;
    val.get("oauthAccount")?
        .get("emailAddress")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn ensure_initialized(claudini_dir: &Path) -> Result<()> {
    if !is_initialized(claudini_dir) {
        bail!("Not initialized. Run 'claudini init' first.");
    }
    Ok(())
}
