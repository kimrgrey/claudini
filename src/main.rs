mod backup;
mod config;
mod keychain;
mod profile;
mod sync;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, Color, Table};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser)]
#[command(name = "claudini", about = "CLI for switching Claude Code accounts")]
struct Cli {
    /// Output as JSON (machine-readable, no colors/spinners)
    #[arg(long, global = true)]
    json: bool,

    /// Override Claude home directory (default: ~, env: CLAUDINI_CLAUDE_HOME)
    #[arg(long, global = true)]
    claude_home: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize claudini (create ~/.claudini/ directory structure)
    Init,

    /// Manage profiles
    Profile {
        #[command(subcommand)]
        command: ProfileCommand,
    },

    /// Manage backups
    Backup {
        #[command(subcommand)]
        command: BackupCommand,
    },
}

#[derive(Subcommand)]
enum ProfileCommand {
    /// Save current auth as a named profile
    Add {
        /// Profile name
        name: String,

        /// Clear auth and launch `claude` for OAuth login
        #[arg(long)]
        login: bool,
    },

    /// Switch to a different profile
    Use {
        /// Profile name
        name: String,
    },

    /// List all profiles
    List,

    /// Remove a profile
    Remove {
        /// Profile name
        name: String,
    },

    /// Rename a profile
    Rename {
        /// Current profile name
        old_name: String,
        /// New profile name
        new_name: String,
    },

    /// Show active profile name and email
    Current,
}

#[derive(Subcommand)]
enum BackupCommand {
    /// Create a backup of current Claude config and credentials
    Create {
        /// Backup name
        name: String,
    },
    /// Restore from a named backup
    Restore {
        /// Backup name
        name: String,
    },
    /// List available backups
    List,
}

fn main() {
    let cli = Cli::parse();
    let is_json = cli.json;

    if let Err(e) = run(cli) {
        if is_json {
            let err = serde_json::json!({ "error": format!("{:#}", e) });
            println!("{}", serde_json::to_string_pretty(&err).unwrap());
        } else {
            eprintln!("{} {:#}", style("error:").red().bold(), e);
        }
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    let claudini_dir = config::claudini_dir()?;
    let claude_home = config::resolve_claude_home(cli.claude_home.as_deref())?;
    let is_json = cli.json;

    match cli.command {
        Command::Init => {
            profile::init(&claudini_dir)?;
            if is_json {
                println!("{}", serde_json::json!({ "status": "initialized" }));
            } else {
                println!(
                    "{} Initialized claudini at {}",
                    style("✓").green().bold(),
                    style(claudini_dir.display()).cyan()
                );
            }
        }

        Command::Profile { command } => match command {
            ProfileCommand::Add { name, login } => {
                if login {
                    if !is_json {
                        println!(
                            "{} Saving current credentials and launching claude for login...",
                            style("→").blue().bold()
                        );
                    }
                    profile::add_with_login(&claudini_dir, &claude_home, &name)?;
                } else {
                    let spinner = make_spinner(is_json, "Saving profile...");
                    profile::add(&claudini_dir, &claude_home, &name)?;
                    finish_spinner(spinner);
                }

                if is_json {
                    println!(
                        "{}",
                        serde_json::json!({ "status": "added", "profile": name })
                    );
                } else {
                    println!(
                        "{} Profile '{}' added and set as active",
                        style("✓").green().bold(),
                        style(&name).cyan()
                    );
                }
            }

            ProfileCommand::Use { name } => {
                let spinner = make_spinner(is_json, "Switching profile...");
                profile::switch(&claudini_dir, &claude_home, &name)?;
                finish_spinner(spinner);

                if is_json {
                    println!(
                        "{}",
                        serde_json::json!({ "status": "switched", "profile": name })
                    );
                } else {
                    println!(
                        "{} Switched to profile '{}'",
                        style("✓").green().bold(),
                        style(&name).cyan()
                    );
                }
            }

            ProfileCommand::List => {
                let profiles = profile::list(&claudini_dir)?;

                if is_json {
                    let items: Vec<_> = profiles
                        .iter()
                        .map(|(name, active)| {
                            serde_json::json!({ "name": name, "active": active })
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&items)?);
                } else if profiles.is_empty() {
                    println!("No profiles. Use 'claudini profile add <name>' to create one.");
                } else {
                    let mut table = Table::new();
                    table.load_preset(UTF8_FULL_CONDENSED);
                    table.set_header(vec!["Profile", "Status"]);
                    for (name, active) in &profiles {
                        if *active {
                            table.add_row(vec![
                                Cell::new(name).fg(Color::Cyan),
                                Cell::new("active").fg(Color::Green),
                            ]);
                        } else {
                            table.add_row(vec![Cell::new(name), Cell::new("")]);
                        }
                    }
                    println!("{table}");
                }
            }

            ProfileCommand::Remove { name } => {
                profile::remove(&claudini_dir, &name)?;

                if is_json {
                    println!(
                        "{}",
                        serde_json::json!({ "status": "removed", "profile": name })
                    );
                } else {
                    println!(
                        "{} Profile '{}' removed",
                        style("✓").green().bold(),
                        style(&name).cyan()
                    );
                }
            }

            ProfileCommand::Rename { old_name, new_name } => {
                profile::rename(&claudini_dir, &claude_home, &old_name, &new_name)?;

                if is_json {
                    println!(
                        "{}",
                        serde_json::json!({ "status": "renamed", "old_name": old_name, "new_name": new_name })
                    );
                } else {
                    println!(
                        "{} Profile '{}' renamed to '{}'",
                        style("✓").green().bold(),
                        style(&old_name).cyan(),
                        style(&new_name).cyan()
                    );
                }
            }

            ProfileCommand::Current => {
                let (name, email) = profile::current(&claudini_dir, &claude_home)?;

                if is_json {
                    println!(
                        "{}",
                        serde_json::json!({ "profile": name, "email": email })
                    );
                } else {
                    println!("{} {}", style("Profile:").bold(), style(&name).cyan());
                    match email {
                        Some(e) => println!("{} {}", style("Email:").bold(), style(e).cyan()),
                        None => println!("{} (unknown)", style("Email:").bold()),
                    }
                }
            }
        },

        Command::Backup { command } => match command {
            BackupCommand::Create { name } => {
                let spinner = make_spinner(is_json, "Backing up...");
                backup::create(&claudini_dir, &claude_home, &name)?;
                finish_spinner(spinner);

                if is_json {
                    println!(
                        "{}",
                        serde_json::json!({ "status": "backed_up", "name": name })
                    );
                } else {
                    println!(
                        "{} Backup '{}' saved",
                        style("✓").green().bold(),
                        style(&name).cyan()
                    );
                }
            }

            BackupCommand::Restore { name } => {
                let spinner = make_spinner(is_json, "Restoring backup...");
                backup::restore(&claudini_dir, &claude_home, &name)?;
                finish_spinner(spinner);

                if is_json {
                    println!(
                        "{}",
                        serde_json::json!({ "status": "restored", "name": name })
                    );
                } else {
                    println!(
                        "{} Backup '{}' restored",
                        style("✓").green().bold(),
                        style(&name).cyan()
                    );
                }
            }

            BackupCommand::List => {
                let backups = backup::list(&claudini_dir)?;

                if is_json {
                    println!("{}", serde_json::to_string_pretty(&backups)?);
                } else if backups.is_empty() {
                    println!("No backups. Use 'claudini backup create <name>' to create one.");
                } else {
                    for name in &backups {
                        println!("  {}", style(name).cyan());
                    }
                }
            }
        },

    }

    Ok(())
}

fn make_spinner(is_json: bool, msg: &str) -> Option<ProgressBar> {
    if is_json {
        return None;
    }
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    Some(pb)
}

fn finish_spinner(spinner: Option<ProgressBar>) {
    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }
}
