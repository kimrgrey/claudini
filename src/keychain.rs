use anyhow::{Context, Result, bail};

const SERVICE_NAME: &str = "Claude Code-credentials";

fn username() -> String {
    std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())
}

fn security_read(service: &str) -> Result<String> {
    let user = username();
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", service, "-a", &user, "-w"])
        .output()
        .context("Failed to run 'security' command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to read credential from keychain: {}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_string())
}

fn security_write(service: &str, data: &str, trusted_apps: &[String]) -> Result<()> {
    let user = username();

    // Delete existing entry (ignore error if not found)
    let _ = std::process::Command::new("security")
        .args(["delete-generic-password", "-s", service, "-a", &user])
        .output();

    let mut args = vec![
        "add-generic-password".to_string(),
        "-s".to_string(),
        service.to_string(),
        "-a".to_string(),
        user,
        "-w".to_string(),
        data.to_string(),
    ];

    if trusted_apps.is_empty() {
        bail!(
            "No trusted applications resolved — refusing to create a keychain entry with unrestricted access"
        );
    }
    for path in trusted_apps {
        args.push("-T".to_string());
        args.push(path.clone());
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

fn security_delete(service: &str) -> Result<()> {
    let user = username();
    let output = std::process::Command::new("security")
        .args(["delete-generic-password", "-s", service, "-a", &user])
        .output()
        .context("Failed to run 'security' command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Failed to delete credential from keychain: {}",
            stderr.trim()
        );
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
    if let Ok(exe) = std::env::current_exe()
        && let Ok(canonical) = exe.canonicalize()
    {
        paths.push(canonical.to_string_lossy().into_owned());
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

// --- Active credential (shared with Claude Code) ---

pub fn read() -> Result<String> {
    security_read(SERVICE_NAME)
}

/// Write with `-T` flags so that Claude Code, claudini, and the security CLI
/// can all access the entry without triggering a permission prompt.
pub fn write(data: &str) -> Result<()> {
    security_write(SERVICE_NAME, data, &trusted_app_paths())
}

pub fn delete() -> Result<()> {
    security_delete(SERVICE_NAME)
}

// --- Profile credentials ---

pub fn profile_service_name(name: &str) -> String {
    format!("claudini-profile-{name}")
}

pub fn read_profile(name: &str) -> Result<String> {
    security_read(&profile_service_name(name))
        .with_context(|| format!("Failed to read credential for profile '{name}' from keychain"))
}

pub fn write_profile(name: &str, data: &str) -> Result<()> {
    security_write(&profile_service_name(name), data, &trusted_app_paths())
        .with_context(|| format!("Failed to write credential for profile '{name}' to keychain"))
}

pub fn delete_profile(name: &str) -> Result<()> {
    security_delete(&profile_service_name(name))
        .with_context(|| format!("Failed to delete credential for profile '{name}' from keychain"))
}

// --- Backup credentials ---

pub fn backup_service_name(name: &str) -> String {
    format!("claudini-backup-{name}")
}

pub fn read_backup(name: &str) -> Result<String> {
    security_read(&backup_service_name(name))
        .with_context(|| format!("Failed to read credential for backup '{name}' from keychain"))
}

pub fn write_backup(name: &str, data: &str) -> Result<()> {
    security_write(&backup_service_name(name), data, &trusted_app_paths())
        .with_context(|| format!("Failed to write credential for backup '{name}' to keychain"))
}

pub fn delete_backup(name: &str) -> Result<()> {
    security_delete(&backup_service_name(name))
        .with_context(|| format!("Failed to delete credential for backup '{name}' from keychain"))
}
