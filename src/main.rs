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
    }
}
