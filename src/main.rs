// Import required dependencies
// clap provides command line argument parsing functionality
use clap::Parser;
// Import our timemachine library functionality
use timemachine;

// Define the main CLI structure using clap's derive feature
// This automatically generates the command-line parser based on the struct definition
#[derive(Parser)]
#[command(name = "timemachine", version = "0.1.0", author = "Michael Asiedu")]
struct Cli {
    // Define that this CLI will have subcommands (init, snapshot)
    #[command(subcommand)]
    command: Commands,
}

// Define the available subcommands and their arguments
#[derive(clap::Subcommand)]
enum Commands {
    // 'init' subcommand - Initializes a new timemachine directory
    Init {
        // Required argument: the directory to initialize
        #[arg(value_name = "DIRECTORY")]
        dir: String,
    },
    // 'snapshot' subcommand - Takes a snapshot of the current directory state
    Snapshot {
        // Required argument: the directory to snapshot
        #[arg(value_name = "DIRECTORY")]
        dir: String,
    },
}

// Main entry point of the CLI application
fn main() {
    // Parse command line arguments into our Cli struct
    let cli = Cli::parse();

    // Match on the subcommand and execute the appropriate functionality
    match &cli.command {
        // Handle the 'init' command
        Commands::Init { dir } => match timemachine::initialize_directory(dir) {
            Ok(_) => eprintln!("Initialization complete with metadata! {}", dir),
            Err(e) => eprintln!("Initialization failed {:?}", e),
        },
        // Handle the 'snapshot' command
        Commands::Snapshot {dir} => match timemachine::take_snapshot(dir) {
            Ok(_) => eprintln!("Snapshot taken succesfully!"),
            Err(e) => eprintln!("Snapshot failed {}", e)
        }
    }
}
