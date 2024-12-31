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
        } => {
            // Now proceed with restore
            match timemachine::restore_snapshot(dir, *snapshot_id, *dry_run) {
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
                        eprintln!("Restore completed successfully!");
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
}
