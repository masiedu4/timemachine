use clap::Parser;
use timemachine;

#[derive(Parser)]
#[command(name = "timemachine", version = "0.1.0", author = "Michael Asiedu")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Init {
        #[arg(value_name = "DIRECTORY")]
        dir: String,
    },
    Snapshot {
        #[arg(value_name = "DIRECTORY")]
        dir: String,
    },
    List {
        #[arg(value_name = "DIRECTORY")]
        dir: String,
        /// Show detailed information including space usage
        #[arg(long, default_value_t = false)]
        detailed: bool,
    },
    Status {
        #[arg(value_name = "DIRECTORY")]
        dir: String,
    },
    Delete {
        #[arg(value_name = "DIRECTORY")]
        dir: String,
        #[arg(value_name = "SNAPSHOT_ID")]
        snapshot_id: usize,
        /// Clean up unused content after deletion
        #[arg(long, default_value_t = false)]
        cleanup: bool,
    },
    Diff {
        #[arg(value_name = "DIRECTORY")]
        dir: String,
        #[arg(value_name = "SNAPSHOT_ID_1")]
        snapshot_id1: usize,
        #[arg(value_name = "SNAPSHOT_ID_2")]
        snapshot_id2: usize,
    },
    Restore {
        #[arg(value_name = "DIRECTORY")]
        dir: String,
        #[arg(value_name = "SNAPSHOT_ID")]
        snapshot_id: usize,
        /// Perform a trial run with no changes made
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
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
        },
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
        } => match timemachine::restore_snapshot(dir, *snapshot_id, *dry_run) {
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
        },
    }
}
