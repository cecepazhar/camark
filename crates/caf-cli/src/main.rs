use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

/// camarkctl — headless verification CLI for CAMark v2.
#[derive(Parser, Debug)]
#[command(name = "camarkctl", version = caf_core::CORE_VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print the resolved runtime data directory and store location.
    Env(JsonFlag),
    /// Error code registry.
    Errors {
        #[command(subcommand)]
        action: ErrorsAction,
    },
    /// Vault management.
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },
    /// Profiles management.
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Notes sample slice management.
    Notes {
        #[command(subcommand)]
        action: NotesAction,
    },
    /// GCC License management (status, activate, verify, deactivate).
    License {
        #[command(subcommand)]
        action: LicenseAction,
    },
}

#[derive(Subcommand, Debug)]
enum ErrorsAction {
    List,
}

#[derive(Subcommand, Debug)]
enum VaultAction {
    Unlock { password: String },
    Lock,
    Status,
}

#[derive(Subcommand, Debug)]
enum ProfileAction {
    List,
    Add {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "owner")]
        role: String,
        #[arg(long)]
        pin: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum NotesAction {
    List,
    Search {
        query: String,
    },
    Add {
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "")]
        content: String,
        #[arg(long, default_value = "shared")]
        visibility: String,
    },
}

#[derive(Subcommand, Debug)]
enum LicenseAction {
    /// Print local and server license status.
    Status {
        #[arg(long)]
        gcc_url: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Activate a GCC license key for this device.
    Activate {
        key: String,
        #[arg(long)]
        device_name: Option<String>,
        #[arg(long)]
        gcc_url: Option<String>,
    },
    /// Verify active license with GCC server.
    Verify {
        #[arg(long)]
        key: Option<String>,
        #[arg(long)]
        gcc_url: Option<String>,
    },
    /// Deactivate license from this device.
    Deactivate {
        #[arg(long)]
        key: Option<String>,
        #[arg(long)]
        gcc_url: Option<String>,
    },
}

#[derive(Args, Debug, Default)]
struct JsonFlag {
    #[arg(long)]
    json: bool,
}

#[derive(Serialize)]
struct EnvOutput {
    data_dir: PathBuf,
    db_path: PathBuf,
    source: String,
    device_id: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Env(flag)) => {
            let info = caf_core::paths::resolve_data_dir()?;
            let db = caf_core::paths::db_path(&info.path);
            let device_id = caf_core::paths::device_id().unwrap_or_else(|_| "unknown".into());

            if flag.json {
                let out = EnvOutput {
                    data_dir: info.path,
                    db_path: db,
                    source: info.source.as_str().to_string(),
                    device_id,
                };
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("Data Directory: {}", info.path.display());
                println!("Database Path:  {}", db.display());
                println!("Source:         {}", info.source.as_str());
                println!("Device ID:      {}", device_id);
            }
        }
        Some(Commands::Errors {
            action: ErrorsAction::List,
        }) => {
            println!("CMRK-001: General Error");
            println!("CMRK-101: Vault Authentication Failed");
            println!("CMRK-102: Vault Not Initialized");
            println!("CMRK-201: Profile Not Found");
            println!("CMRK-301: Storage Encryption Error");
        }
        Some(Commands::Vault { action }) => match action {
            VaultAction::Status => {
                let initialized = caf_core::vault::is_vault_initialized()?;
                let is_unlocked = caf_core::vault::is_unlocked()?;
                println!("Vault Initialized: {initialized}");
                println!("Vault Unlocked:    {is_unlocked}");
            }
            VaultAction::Unlock { password } => {
                let ok = caf_core::vault::validate_password(&password)?;
                if ok {
                    println!("Vault unlocked successfully.");
                } else {
                    return Err("Invalid master password.".into());
                }
            }
            VaultAction::Lock => {
                caf_core::vault::lock();
                println!("Vault locked.");
            }
        },
        Some(Commands::Profile { action }) => match action {
            ProfileAction::List => {
                let list = caf_core::profiles::list_profiles()?;
                for p in list {
                    println!("- [{}] {} ({}) [PIN: {}]", p.id, p.name, p.role, p.has_pin);
                }
            }
            ProfileAction::Add { name, role, pin } => {
                let profile = caf_core::profiles::save_profile(
                    caf_core::profiles::ProfileInput {
                        id: None,
                        name,
                        role,
                        avatar: None,
                        pin,
                    },
                    "cli-root",
                )?;
                println!("Created profile: {} ({})", profile.name, profile.id);
            }
        },
        Some(Commands::Notes { action }) => match action {
            NotesAction::List => {
                let notes = caf_core::notes::list_notes("cli-root", true)?;
                for n in notes {
                    println!("- [{}] {} (visibility: {})", n.id, n.title, n.visibility);
                }
            }
            NotesAction::Add {
                title,
                content,
                visibility,
            } => {
                let note = caf_core::notes::save_note(
                    caf_core::notes::NoteInput {
                        id: None,
                        title,
                        content,
                        tags: vec![],
                        visibility: Some(visibility),
                        owner_profile_id: "cli-root".to_string(),
                    },
                    "cli-root",
                )?;
                println!("Created note: {} ({})", note.title, note.id);
            }
            NotesAction::Search { query } => {
                let results = caf_core::notes::search_notes(&query, "cli-root", true)?;
                if results.is_empty() {
                    println!("No notes matching '{query}'.");
                } else {
                    println!("Found {} notes matching '{}':", results.len(), query);
                    for n in results {
                        println!("- [{}] {} (tags: {:?})", n.id, n.title, n.tags);
                    }
                }
            }
        },
        Some(Commands::License { action }) => match action {
            LicenseAction::Status { gcc_url: _, json } => {
                let status = caf_core::license_client::get_license_status()?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&status)?);
                } else {
                    println!("=== CAMark GCC License Status ===");
                    println!("Active:          {}", status.active);
                    println!("Valid:           {}", status.valid);
                    println!(
                        "License Key:     {}",
                        status.license_key.unwrap_or_else(|| "-".into())
                    );
                    println!(
                        "Product:         {}",
                        status.product_sku.unwrap_or_else(|| "CAMARK".into())
                    );
                    println!(
                        "Plan Tier:       {}",
                        status.plan_tier.unwrap_or_else(|| "community".into())
                    );
                    println!(
                        "Device ID:       {}",
                        status.device_id.unwrap_or_else(|| "-".into())
                    );
                    println!(
                        "Expires At:      {}",
                        status.expires_at.unwrap_or_else(|| "-".into())
                    );
                    println!("Offline Capable: {}", status.offline_capable);
                }
            }
            LicenseAction::Activate {
                key,
                device_name,
                gcc_url,
            } => {
                println!("Activating CAMark license with GCC backend...");
                let result = caf_core::license_client::activate_license(
                    gcc_url.as_deref(),
                    &key,
                    device_name.as_deref(),
                )?;
                println!(
                    "Success! License activated for device {}.",
                    result.device_id
                );
                println!("Product:    {}", result.product_name);
                println!("Plan Tier:  {}", result.plan_tier);
                println!("Key:        {}", result.license_key);
                if let Some(exp) = result.expires_at {
                    println!("Expires:    {}", exp);
                } else {
                    println!("Expires:    Lifetime");
                }
            }
            LicenseAction::Verify { key, gcc_url } => {
                println!("Verifying license with GCC server...");
                let valid =
                    caf_core::license_client::verify_license(gcc_url.as_deref(), key.as_deref())?;
                if valid {
                    println!("License is VALID on GCC backend.");
                } else {
                    println!("License is INVALID or not registered for this device.");
                }
            }
            LicenseAction::Deactivate { key, gcc_url } => {
                println!("Deactivating license on this device...");
                caf_core::license_client::deactivate_license(gcc_url.as_deref(), key.as_deref())?;
                println!("License deactivated and local license store cleared.");
            }
        },
        None => {
            println!("camarkctl v{}", caf_core::CORE_VERSION);
            println!("Use --help for available subcommands.");
        }
    }

    Ok(())
}
