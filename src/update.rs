use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use console::style;
use serde::{Deserialize, Serialize};

const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;
const GITHUB_API_URL: &str = "https://api.github.com/repos/kimrgrey/claudini/releases/latest";

#[derive(Serialize, Deserialize)]
struct VersionCache {
    last_checked: u64,
    latest_version: String,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

fn cache_path(claudini_dir: &Path) -> std::path::PathBuf {
    claudini_dir.join("version_check.json")
}

fn now_secs() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

fn is_newer(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.split('.')
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let l = parse(latest);
    let c = parse(current);
    l > c
}

fn fetch_latest_version() -> Option<String> {
    let output = std::process::Command::new("curl")
        .args(["-fsSL", GITHUB_API_URL])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let release: GitHubRelease = serde_json::from_slice(&output.stdout).ok()?;
    let version = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    Some(version.to_string())
}

fn print_update_notice(current: &str, latest: &str) {
    eprintln!(
        "\n{} {} → {} — run `{}` to update",
        style("Update available:").yellow().bold(),
        style(current).dim(),
        style(latest).green().bold(),
        style(
            "curl -fsSL https://raw.githubusercontent.com/kimrgrey/claudini/main/install.sh | sh"
        )
        .cyan(),
    );
}

pub fn check_and_notify(claudini_dir: &Path, is_json: bool) {
    if is_json {
        return;
    }

    let _ = check_and_notify_inner(claudini_dir);
}

fn check_and_notify_inner(claudini_dir: &Path) -> Option<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    let cache_file = cache_path(claudini_dir);
    let now = now_secs()?;

    let cache = std::fs::read_to_string(&cache_file)
        .ok()
        .and_then(|data| serde_json::from_str::<VersionCache>(&data).ok());

    let latest_version = if let Some(ref c) = cache {
        if now.saturating_sub(c.last_checked) < CHECK_INTERVAL_SECS {
            c.latest_version.clone()
        } else {
            let v = fetch_latest_version()?;
            let new_cache = VersionCache {
                last_checked: now,
                latest_version: v.clone(),
            };
            let _ = std::fs::write(&cache_file, serde_json::to_string(&new_cache).ok()?);
            v
        }
    } else {
        let v = fetch_latest_version()?;
        let new_cache = VersionCache {
            last_checked: now,
            latest_version: v.clone(),
        };
        let _ = std::fs::write(&cache_file, serde_json::to_string(&new_cache).ok()?);
        v
    };

    if is_newer(&latest_version, current_version) {
        print_update_notice(current_version, &latest_version);
    }

    Some(())
}
