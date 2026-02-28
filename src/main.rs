mod backup;
mod config;
mod keychain;
mod profile;
mod sync;
mod update;

use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use comfy_table::{Cell, Color, Table, presets::UTF8_FULL_CONDENSED};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser)]
#[command(
    name = "claudini",
    about = "CLI for switching Claude Code accounts",
    version
)]
struct Cli {
    /// Output as JSON (machine-readable, no colors/spinners)
    #[arg(long, global = true)]
    json: bool,

    /// Override Claude home directory (default: ~, env: CLAUDINI_CLAUDE_HOME)
    #[arg(long, global = true)]
    claude_home: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize claudini (create ~/.claudini/ directory structure)
    Init,

    /// Switch to a different profile (shortcut for `profile use`)
    Use {
        /// Profile name
        name: String,

        /// Launch claude after switching
        #[arg(short, long)]
        launch: bool,
    },

    /// Catch-all: treat unknown subcommand as profile name, switch and launch claude
    #[command(external_subcommand)]
    Run(Vec<String>),

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

        /// Launch claude after switching
        #[arg(short, long)]
        launch: bool,
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
    /// Delete a named backup
    Delete {
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

    if let Ok(claudini_dir) = config::claudini_dir() {
        update::check_and_notify(&claudini_dir, is_json);
    }
}

fn run(cli: Cli) -> Result<()> {
    let claudini_dir = config::claudini_dir()?;
    let claude_home = config::resolve_claude_home(cli.claude_home.as_deref())?;
    let is_json = cli.json;

    console::set_colors_enabled(!is_json && std::io::stdout().is_terminal());

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            let (name, email) = profile::current(&claudini_dir, &claude_home)?;

            if is_json {
                println!("{}", serde_json::json!({ "profile": name, "email": email }));
            } else {
                println!("{} {}", style("Profile:").bold(), style(&name).cyan());
                match email {
                    Some(e) => println!("{} {}", style("Email:").bold(), style(e).cyan()),
                    None => println!("{} (unknown)", style("Email:").bold()),
                }
            }
            return Ok(());
        }
    };

    match command {
        Command::Init => match profile::init(&claudini_dir)? {
            profile::InitResult::Initialized => {
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
            profile::InitResult::AlreadyInitialized {
                profiles_migrated,
                backups_migrated,
            } => {
                let total = profiles_migrated + backups_migrated;
                if is_json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "status": "already_initialized",
                            "credentials_migrated": {
                                "profiles": profiles_migrated,
                                "backups": backups_migrated,
                            }
                        })
                    );
                } else if total > 0 {
                    println!(
                        "{} Already initialized. Migrated {} credential(s) to Keychain ({} profile, {} backup).",
                        style("✓").green().bold(),
                        total,
                        profiles_migrated,
                        backups_migrated
                    );
                } else {
                    println!(
                        "{} Already initialized (no legacy credentials to migrate).",
                        style("✓").green().bold(),
                    );
                }
            }
        },

        Command::Use { name, launch } => {
            switch_profile(&claudini_dir, &claude_home, &name, launch, is_json)?;
        }

        Command::Run(args) => {
            let name = args.first().context("Profile name required")?;
            if args.len() > 1 {
                bail!(
                    "Unexpected arguments after profile name: {}",
                    args[1..].join(" ")
                );
            }
            switch_profile(&claudini_dir, &claude_home, name, true, is_json)?;
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

            ProfileCommand::Use { name, launch } => {
                switch_profile(&claudini_dir, &claude_home, &name, launch, is_json)?;
            }

            ProfileCommand::List => {
                let profiles = profile::list(&claudini_dir)?;

                if is_json {
                    let items: Vec<_> = profiles
                        .iter()
                        .map(|(name, active)| serde_json::json!({ "name": name, "active": active }))
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&items)?);
                } else if profiles.is_empty() {
                    println!("No profiles. Use 'claudini profile add <name>' to create one.");
                } else {
                    let mut table = Table::new();
                    table.load_preset(UTF8_FULL_CONDENSED);
                    table.set_header(vec!["Profile", "Status"]);
                    let use_color = console::colors_enabled();
                    for (name, active) in &profiles {
                        if *active {
                            let mut name_cell = Cell::new(name);
                            let mut status_cell = Cell::new("active");
                            if use_color {
                                name_cell = name_cell.fg(Color::Cyan);
                                status_cell = status_cell.fg(Color::Green);
                            }
                            table.add_row(vec![name_cell, status_cell]);
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
                    println!("{}", serde_json::json!({ "profile": name, "email": email }));
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

            BackupCommand::Delete { name } => {
                backup::delete(&claudini_dir, &name)?;

                if is_json {
                    println!(
                        "{}",
                        serde_json::json!({ "status": "deleted", "name": name })
                    );
                } else {
                    println!(
                        "{} Backup '{}' deleted",
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

fn switch_profile(
    claudini_dir: &std::path::Path,
    claude_home: &std::path::Path,
    name: &str,
    launch: bool,
    is_json: bool,
) -> Result<()> {
    let spinner = make_spinner(is_json, "Switching profile...");
    profile::switch(claudini_dir, claude_home, name)?;
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
            style(name).cyan()
        );
    }

    if launch && !is_json {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new("claude").exec();
        bail!("Failed to launch claude: {}", err);
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
