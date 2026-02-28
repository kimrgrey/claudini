use anyhow::{bail, Context, Result};
use keyring::Entry;

const SERVICE_NAME: &str = "Claude Code-credentials";

fn username() -> String {
    std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())
}

fn entry_for(service: &str) -> Result<Entry> {
    Entry::new(service, &username()).context("Failed to create keychain entry")
}

fn entry() -> Result<Entry> {
    entry_for(SERVICE_NAME)
}

/// Read the credential from macOS Keychain.
pub fn read() -> Result<String> {
    entry()?.get_password().context("Failed to read credential from keychain")
}

/// Write a credential to macOS Keychain.
///
/// Uses the `security` CLI with `-T` flags so that both Claude Code and claudini
/// can access the entry without triggering a permission prompt.
pub fn write(data: &str) -> Result<()> {
    let user = username();

    // Delete existing entry (ignore error if not found)
    let _ = std::process::Command::new("security")
        .args(["delete-generic-password", "-s", SERVICE_NAME, "-a", &user])
        .output();

    let mut args = vec![
        "add-generic-password".to_string(),
        "-s".to_string(), SERVICE_NAME.to_string(),
        "-a".to_string(), user,
        "-w".to_string(), data.to_string(),
    ];

    // Trust Claude Code and claudini specifically
    let trusted = trusted_app_paths();
    if trusted.is_empty() {
        // Fallback: allow any application if we can't resolve paths
        args.push("-A".to_string());
    } else {
        for path in &trusted {
            args.push("-T".to_string());
            args.push(path.clone());
        }
    }

    let output = std::process::Command::new("security")
        .args(&args)
        .output()
        .context("Failed to run 'security' command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to write credential to keychain: {}", stderr.trim());
    }

    Ok(())
}

/// Resolve paths to binaries that should be trusted to access the active credential.
fn trusted_app_paths() -> Vec<String> {
    let mut paths = Vec::new();

    // Claude Code binary
    if let Some(p) = resolve_binary_path("claude") {
        paths.push(p);
    }

    // claudini binary (ourselves)
    if let Ok(exe) = std::env::current_exe() {
        if let Ok(canonical) = exe.canonicalize() {
            paths.push(canonical.to_string_lossy().into_owned());
        }
    }

    // security binary (Claude Code may use it to read the credential)
    paths.push("/usr/bin/security".to_string());

    paths
}

fn resolve_binary_path(name: &str) -> Option<String> {
    let output = std::process::Command::new("which")
        .arg(name)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path_str = String::from_utf8(output.stdout).ok()?;
    let path = std::path::Path::new(path_str.trim());
    path.canonicalize()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

/// Delete the credential from macOS Keychain.
pub fn delete() -> Result<()> {
    entry()?
        .delete_credential()
        .context("Failed to delete credential from keychain")
}

// --- Profile credentials ---

pub fn profile_service_name(name: &str) -> String {
    format!("claudini-profile-{name}")
}

pub fn read_profile(name: &str) -> Result<String> {
    let service = profile_service_name(name);
    entry_for(&service)?
        .get_password()
        .with_context(|| format!("Failed to read credential for profile '{name}' from keychain"))
}

pub fn write_profile(name: &str, data: &str) -> Result<()> {
    let service = profile_service_name(name);
    entry_for(&service)?
        .set_password(data)
        .with_context(|| format!("Failed to write credential for profile '{name}' to keychain"))
}

pub fn delete_profile(name: &str) -> Result<()> {
    let service = profile_service_name(name);
    entry_for(&service)?
        .delete_credential()
        .with_context(|| format!("Failed to delete credential for profile '{name}' from keychain"))
}

// --- Backup credentials ---

pub fn backup_service_name(name: &str) -> String {
    format!("claudini-backup-{name}")
}

pub fn read_backup(name: &str) -> Result<String> {
    let service = backup_service_name(name);
    entry_for(&service)?
        .get_password()
        .with_context(|| format!("Failed to read credential for backup '{name}' from keychain"))
}

pub fn write_backup(name: &str, data: &str) -> Result<()> {
    let service = backup_service_name(name);
    entry_for(&service)?
        .set_password(data)
        .with_context(|| format!("Failed to write credential for backup '{name}' to keychain"))
}

pub fn delete_backup(name: &str) -> Result<()> {
    let service = backup_service_name(name);
    entry_for(&service)?
        .delete_credential()
        .with_context(|| format!("Failed to delete credential for backup '{name}' from keychain"))
}
