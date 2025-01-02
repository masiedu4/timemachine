use clap::CommandFactory;
use clap::Parser;
use clap_complete::{generate_to, shells::*};
use std::path::PathBuf;
use timemachine;

#[derive(Parser)]
#[command(
    name = "timemachine",
    version = env!("CARGO_PKG_VERSION"),
    author = "Michael Asiedu",
    about = "A version control system for directories that helps track and manage file changes over time",
    long_about = "TimeMachine is a powerful file versioning tool that creates snapshots of directories and allows you to track, restore, and manage changes over time. It provides an easy way to backup and version control any directory on your system."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    #[command(
        about = "Initialize a directory for version tracking",
        long_about = "Prepares a directory for version tracking by creating necessary metadata structures. This command must be run before using other commands on a directory."
    )]
    Init {
        #[arg(
            value_name = "DIRECTORY",
            help = "Path to the directory to initialize",
            long_help = "Absolute or relative path to the directory that will be tracked. The directory must exist and be writable."
        )]
        dir: String,
    },

    #[command(
        about = "Create a new snapshot of the current directory state",
        long_about = "Takes a snapshot of the current state of the directory, including all files and their contents. Each snapshot is assigned a unique ID that can be used for future operations."
    )]
    Snapshot {
        #[arg(
            value_name = "DIRECTORY",
            help = "Path to the directory to snapshot",
            long_help = "Path to an initialized directory. The directory must have been previously initialized using the init command."
        )]
        dir: String,
    },

    #[command(
        about = "List all snapshots for a directory",
        long_about = "Displays a list of all snapshots taken for the specified directory. When used with --detailed, shows additional information like space usage and file counts."
    )]
    List {
        #[arg(
            value_name = "DIRECTORY",
            help = "Path to the directory",
            long_help = "Path to an initialized directory whose snapshots you want to list."
        )]
        dir: String,
        #[arg(
            long,
            default_value_t = false,
            help = "Show detailed information including space usage",
            long_help = "When enabled, shows additional information for each snapshot including: total size, number of files, and space usage statistics."
        )]
        detailed: bool,
    },

    #[command(
        about = "Show the current status of a directory",
        long_about = "Displays the current state of the directory compared to its last snapshot, showing modified, added, and deleted files. Also shows available space and latest snapshot information."
    )]
    Status {
        #[arg(
            value_name = "DIRECTORY",
            help = "Path to the directory to check status",
            long_help = "Path to an initialized directory. Shows changes made since the last snapshot, if any."
        )]
        dir: String,
    },

    #[command(
        about = "Delete a specific snapshot",
        long_about = "Removes a snapshot from the directory's history. When used with --cleanup, also removes any stored file contents that are no longer referenced by other snapshots."
    )]
    Delete {
        #[arg(
            value_name = "DIRECTORY",
            help = "Path to the directory",
            long_help = "Path to an initialized directory containing the snapshot to delete."
        )]
        dir: String,
        #[arg(
            value_name = "SNAPSHOT_ID",
            help = "ID of the snapshot to delete",
            long_help = "Numeric ID of the snapshot to remove. Use the list command to see available snapshot IDs."
        )]
        snapshot_id: usize,
        #[arg(
            long,
            default_value_t = false,
            help = "Clean up unused content after deletion",
            long_help = "When enabled, removes stored file contents that are no longer referenced by any remaining snapshots, freeing up space."
        )]
        cleanup: bool,
    },

    #[command(
        about = "Compare two snapshots",
        long_about = "Shows the differences between two snapshots, including added, modified, and deleted files. Useful for understanding changes between different points in time."
    )]
    Diff {
        #[arg(
            value_name = "DIRECTORY",
            help = "Path to the directory",
            long_help = "Path to an initialized directory containing the snapshots to compare."
        )]
        dir: String,
        #[arg(
            value_name = "SNAPSHOT_ID_1",
            help = "ID of the first snapshot",
            long_help = "Numeric ID of the first snapshot for comparison. Use the list command to see available snapshot IDs."
        )]
        snapshot_id1: usize,
        #[arg(
            value_name = "SNAPSHOT_ID_2",
            help = "ID of the second snapshot",
            long_help = "Numeric ID of the second snapshot for comparison. Use the list command to see available snapshot IDs."
        )]
        snapshot_id2: usize,
    },

    #[command(
        about = "Restore directory to a specific snapshot state",
        long_about = "Restores the directory to the state it was in at a specific snapshot. Use --dry-run to preview changes without applying them."
    )]
    Restore {
        #[arg(
            value_name = "DIRECTORY",
            help = "Path to the directory to restore",
            long_help = "Path to an initialized directory that you want to restore to a previous state."
        )]
        dir: String,
        #[arg(
            value_name = "SNAPSHOT_ID",
            help = "ID of the snapshot to restore to",
            long_help = "Numeric ID of the snapshot to restore to. Use the list command to see available snapshot IDs."
        )]
        snapshot_id: usize,
        #[arg(
            long,
            default_value_t = false,
            help = "Perform a trial run with no changes made",
            long_help = "When enabled, shows what would be changed by the restore operation without actually making any changes. Useful for previewing the effects of a restore."
        )]
        #[arg(
            long,
            help = "Force restore even if there are uncommitted changes",
            long_help = "WARNING: Using --force will override any uncommitted changes in your current directory state. A backup snapshot will be automatically created before forcing the restore."
        )]
        force: bool,
        #[arg(
            long,
            help = "Perform a dry run without making any changes",
            long_help = "Show what changes would be made without actually performing the restore operation."
        )]
        dry_run: bool,
    },

    #[command(
        hide = true,
        about = "Generate shell completions",
        long_about = "Generate shell completion scripts for various shells"
    )]
    Completions {
        #[arg(
            value_name = "SHELL",
            help = "Shell to generate completions for",
            long_help = "Supported shells: bash, zsh, fish, powershell"
        )]
        shell: Option<String>,
    },
}

fn generate_completions(shell_name: Option<String>) -> std::io::Result<()> {
    let shells = vec!["bash", "zsh", "fish", "powershell"];
    let out_dir = PathBuf::from("completions");
    std::fs::create_dir_all(&out_dir)?;

    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    match shell_name {
        Some(shell) => match shell.as_str() {
            "bash" => {
                generate_to(Bash, &mut cmd, &bin_name, &out_dir)?;
                Ok(())
            }
            "zsh" => {
                generate_to(Zsh, &mut cmd, &bin_name, &out_dir)?;
                Ok(())
            }
            "fish" => {
                generate_to(Fish, &mut cmd, &bin_name, &out_dir)?;
                Ok(())
            }
            "powershell" => {
                generate_to(PowerShell, &mut cmd, &bin_name, &out_dir)?;
                Ok(())
            }
            _ => {
                eprintln!("Unsupported shell. Available shells: {}", shells.join(", "));
                Ok(())
            }
        },
        None => {
            // Generate for all shells
            generate_to(Bash, &mut cmd, &bin_name, &out_dir)?;
            generate_to(Zsh, &mut cmd, &bin_name, &out_dir)?;
            generate_to(Fish, &mut cmd, &bin_name, &out_dir)?;
            generate_to(PowerShell, &mut cmd, &bin_name, &out_dir)?;
            println!("Generated completion scripts in ./completions/");
            Ok(())
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { dir } => match timemachine::initialize_timemachine(dir) {
            Ok(_) => eprintln!("Initialization complete for {}", dir),
            Err(e) => eprintln!(
                "Initialization failed for directory '{}': {}. Please check the directory path and try again.",
                dir, e
            )
        },
        Commands::Snapshot { dir } => match timemachine::take_snapshot(dir) {
            Ok(_) => eprintln!("Snapshot for {} taken successfully!", dir),
            Err(e) => eprintln!(
                "Snapshot creation failed for directory '{}': {}. Please ensure the directory is accessible and try again.",
                dir, e
            ),
        },
        Commands::List { dir, detailed } => match timemachine::list_snapshots(dir, *detailed) {
            Ok(snapshots) => {
                if snapshots.is_empty() {
                    eprintln!("No snapshots found in {}", dir);
                } else {
                    eprintln!("Snapshots in {}:", dir);
                    for snapshot in snapshots {
                        if *detailed {
                            eprintln!(
                                "ID: {}, Time: {}, Changes: {}, Size: {} bytes",
                                snapshot.id, snapshot.timestamp, snapshot.changes, snapshot.total_size
                            );
                        } else {
                            eprintln!(
                                "ID: {}, Time: {}, Changes: {}",
                                snapshot.id, snapshot.timestamp, snapshot.changes
                            );
                        }
                    }
                }
            }
            Err(e) => eprintln!(
                "Failed to list snapshots in directory '{}': {}",
                dir, e
            ),
        },
        Commands::Status { dir } => match timemachine::get_status(dir) {
            Ok(status) => {
                eprintln!("Status for {}:", dir);
                if let Some(id) = status.latest_snapshot_id {
                    eprintln!("Latest snapshot: {}", id);
                } else {
                    eprintln!("No snapshots found");
                }
                eprintln!("Available space: {} bytes", status.available_space);

                if status.has_uncommitted_changes {
                    eprintln!("\nUncommitted changes:");
                    if !status.modified_files.is_empty() {
                        eprintln!("Modified files: {:?}", status.modified_files);
                    }
                    if !status.new_files.is_empty() {
                        eprintln!("New files: {:?}", status.new_files);
                    }
                    if !status.deleted_files.is_empty() {
                        eprintln!("Deleted files: {:?}", status.deleted_files);
                    }
                } else {
                    eprintln!("\nWorking directory is clean");
                }
            }
            Err(e) => eprintln!(
                "Failed to get status for directory '{}': {}",
                dir, e
            ),
        },
        Commands::Delete { dir, snapshot_id, cleanup } => {
            match timemachine::delete_snapshot(dir, *snapshot_id, *cleanup) {
                Ok(_) => {
                    eprintln!("Successfully deleted snapshot {}", snapshot_id);
                    if *cleanup {
                        eprintln!("Cleaned up unused content");
                    }
                }
                Err(e) => eprintln!(
                    "Failed to delete snapshot {} in directory '{}': {}",
                    snapshot_id, dir, e
                ),
            }
        }
        Commands::Diff {
            dir,
            snapshot_id1,
            snapshot_id2,
        } => match timemachine::differentiate_snapshots(dir, *snapshot_id1, *snapshot_id2) {
            Ok(comparison) => {
                eprintln!(
                    "Comparison between snapshot {} and snapshot {}:",
                    snapshot_id1, snapshot_id2
                );
                eprintln!("New Files: {:?}", comparison.new_files);
                eprintln!("Modified Files: {:?}", comparison.modified_files);
                eprintln!("Deleted Files: {:?}", comparison.deleted_files);
            }
            Err(e) => eprintln!(
                "Failed to compare snapshots {} and {} in directory '{}': {}. Ensure the snapshots exist and try again.",
                snapshot_id1, snapshot_id2, dir, e
            ),
        },
        Commands::Restore {
            dir,
            snapshot_id,
            dry_run,
            force
        } =>
            {
                eprintln!("Preparing to restore directory: {}", dir);
                if *force {
                    eprintln!("WARNING: Force flag is enabled. This will:");
                    eprintln!("  1. Create a backup snapshot of your current state");
                    eprintln!("  2. Override any uncommitted changes");
                    eprintln!("  3. Restore to the specified snapshot");
                }

            match timemachine::restore_snapshot(dir, *snapshot_id, *dry_run, *force) {
                Ok(report) => {
                    if report.added.is_empty() && report.modified.is_empty() && report.deleted.is_empty() {
                        eprintln!("No changes needed - files are already at the target state.");
                    } else {
                        eprintln!("Changes to be made:");
                        if !report.added.is_empty() {
                            eprintln!("Files to add: {:?}", report.added);
                        }
                        if !report.modified.is_empty() {
                            eprintln!("Files to modify: {:?}", report.modified);
                        }
                        if !report.deleted.is_empty() {
                            eprintln!("Files to delete: {:?}", report.deleted);
                        }
                    }

                    if *dry_run {
                        eprintln!("Dry run complete. No changes were made.");
                    } else {
                        eprintln!("Restore complete.");
                    }
                }
                Err(e) => eprintln!(
                    "Failed to restore snapshot {} in directory '{}': {}",
                    snapshot_id, dir, e
                ),
            }
        },
        Commands::Completions { shell } => {
            if let Err(e) = generate_completions(shell.clone()) {
                eprintln!("Failed to generate completions: {}", e);
            }
        }
    }
}
