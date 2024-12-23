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
}
fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { dir } => match timemachine::initialize_directory(dir) {
            Ok(_) => eprintln!("Initialization complete with metadata! {}", dir),
            Err(e) => eprintln!("Initialization failed {:?}", e),
        },
        Commands::Snapshot {dir} => match timemachine::take_snapshot(dir) {
            Ok(_) => eprintln!("Snapshot taken succesfully!"),
            Err(e) => eprintln!("Snapshot failed")
        }
    }
}
