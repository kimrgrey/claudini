use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub active_profile: Option<String>,
}

impl Config {
    pub fn load(claudini_dir: &Path) -> Result<Self> {
        let path = config_json_path(claudini_dir);
        let data = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        serde_json::from_str(&data).with_context(|| "Failed to parse config.json")
    }

    pub fn save(&self, claudini_dir: &Path) -> Result<()> {
        let path = config_json_path(claudini_dir);
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, data).with_context(|| format!("Failed to write {}", path.display()))
    }
}

/// Returns the path to `~/.claudini/`.
pub fn claudini_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".claudini"))
}

/// Returns the path to `~/.claudini/config.json`.
pub fn config_json_path(claudini_dir: &Path) -> PathBuf {
    claudini_dir.join("config.json")
}

/// Returns the path to `~/.claudini/profiles/`.
pub fn profiles_dir(claudini_dir: &Path) -> PathBuf {
    claudini_dir.join("profiles")
}

/// Returns the path to `~/.claudini/profiles/<name>/`.
pub fn profile_dir(claudini_dir: &Path, name: &str) -> PathBuf {
    profiles_dir(claudini_dir).join(name)
}

/// Returns the path to `~/.claudini/profiles/<name>/claude.json`.
pub fn profile_claude_json(claudini_dir: &Path, name: &str) -> PathBuf {
    profile_dir(claudini_dir, name).join("claude.json")
}

/// Returns the path to `~/.claudini/backups/`.
pub fn backups_dir(claudini_dir: &Path) -> PathBuf {
    claudini_dir.join("backups")
}

/// Returns the path to `~/.claudini/backups/<name>/`.
pub fn backup_dir(claudini_dir: &Path, name: &str) -> PathBuf {
    backups_dir(claudini_dir).join(name)
}

/// Returns the path to `<claude_home>/.claude.json`.
pub fn claude_json_path(claude_home: &Path) -> PathBuf {
    claude_home.join(".claude.json")
}

/// Resolves the Claude home directory from CLI flag, env var, or default (~).
pub fn resolve_claude_home(cli_override: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = cli_override {
        return Ok(p.to_path_buf());
    }
    if let Ok(val) = std::env::var("CLAUDINI_CLAUDE_HOME") {
        return Ok(PathBuf::from(val));
    }
    dirs::home_dir().context("Could not determine home directory")
}

/// Checks whether claudini has been initialized (config.json exists).
pub fn is_initialized(claudini_dir: &Path) -> bool {
    config_json_path(claudini_dir).exists()
}

/// Lists profile names by reading subdirectories of the profiles dir.
pub fn list_profiles(claudini_dir: &Path) -> Result<Vec<String>> {
    let dir = profiles_dir(claudini_dir);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && let Some(name) = entry.file_name().to_str()
        {
            names.push(name.to_string());
        }
    }
    names.sort();
    Ok(names)
}

/// Lists backup names by reading subdirectories of the backups dir.
pub fn list_backups(claudini_dir: &Path) -> Result<Vec<String>> {
    let dir = backups_dir(claudini_dir);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && let Some(name) = entry.file_name().to_str()
        {
            names.push(name.to_string());
        }
    }
    names.sort();
    Ok(names)
}
