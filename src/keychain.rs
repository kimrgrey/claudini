use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE_NAME: &str = "Claude Code-credentials";

fn username() -> String {
    std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())
}

fn entry() -> Result<Entry> {
    Entry::new(SERVICE_NAME, &username()).context("Failed to create keychain entry")
}

/// Read the credential from macOS Keychain.
pub fn read() -> Result<String> {
    entry()?.get_password().context("Failed to read credential from keychain")
}

/// Write a credential to macOS Keychain.
pub fn write(data: &str) -> Result<()> {
    entry()?
        .set_password(data)
        .context("Failed to write credential to keychain")
}

/// Delete the credential from macOS Keychain.
pub fn delete() -> Result<()> {
    entry()?
        .delete_credential()
        .context("Failed to delete credential from keychain")
}
