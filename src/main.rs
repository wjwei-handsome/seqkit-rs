#[warn(non_camel_case_types)]


mod stats;
mod logger;

use clap::{Parser, Subcommand};
use crate::logger::init_logger;
use crate::stats::stats_all;

fn main() {
    init_logger();
    let cli = Cli::parse();
    match &cli.command {
        Commands::Stats { input, output, rewrite } => {
            stats_all(input, output, *rewrite);
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print statistics of the fastx file
    Stats {
        /// Input fastx file
        #[arg(short, long, required = false)]
        input: Option<String>,
        /// Output file path, if not set, output to STDOUT
        #[arg(short, long, required = false)]
        output: Option<String>,
        /// Rewrite output file, default is false
        #[arg(short, long, default_value = "false", required = false)]
        rewrite: bool,
    },
}