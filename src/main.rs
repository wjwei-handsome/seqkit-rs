extern crate core;

#[warn(non_camel_case_types)]


mod stats;
mod logger;
mod io;

use clap::{Parser, Subcommand};
use crate::logger::init_logger;
use crate::stats::stats_all;
use crate::io::output_writer;

fn main() {
    init_logger();
    let cli = Cli::parse();
    // process outfile, if `-` then stdout, else write to file, or gzip file
    let outfile = cli.outfile;
    // println!("outfile: {}", outfile);
    let mut writer = output_writer(&outfile, cli.rewrite);
    // println!("if rewrite: {}", cli.rewrite);
    match &cli.command {
        Commands::Stats { input } => {
            stats_all(input, &mut writer);
        }
    }
}

#[derive(Parser)]
#[command(author, version, about = "seqkit in rust", long_about)]
struct Cli {
    /// Output file ("-" for stdout)
    #[arg(long, short, global = true, default_value = "-")]
    outfile: String,
    /// Rewrite output file [default: false]
    #[arg(long, short, global = true, default_value = "false")]
    rewrite: bool,
    /// Subcommands
    #[command(subcommand, name = "ww")]
    command: Commands,

}

#[derive(Subcommand)]
enum Commands {
    /// Print statistics of the fastx file
    #[command(visible_alias = "stat")]
    Stats {
        /// Input fastx file
        #[arg(short, long, required = false)]
        input: Option<String>,
    },

}