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
    Compare {
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
        Commands::Init { dir } => match timemachine::initialize_directory(dir) {
            Ok(_) => eprintln!("Initialization complete with metadata! {}", dir),
            Err(e) => eprintln!("Initialization failed {:?}", e),
        },
        Commands::Snapshot { dir } => match timemachine::take_snapshot(dir) {
            Ok(_) => eprintln!("Snapshot taken succesfully!"),
            Err(e) => eprintln!("Snapshot failed {}", e),
        },
        Commands::Compare {
            dir,
            snapshot_id1,
            snapshot_id2,
        } => match timemachine::compare_snapshots(dir, *snapshot_id1, *snapshot_id2) {
            Ok(comparison) => {
                eprintln!(
                    "Comparison between snapshot {} and snapshot {}:",
                    snapshot_id1, snapshot_id2
                );
                eprintln!("New Files: {:?}", comparison.new_files);
                eprintln!("Modified Files: {:?}", comparison.modified_files);
                eprintln!("Deleted Files: {:?}", comparison.deleted_files);
            }
            Err(e) => eprintln!("Failed to compare snapshots: {}", e),
        },
    }
}
