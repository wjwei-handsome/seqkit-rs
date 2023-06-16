extern crate core;

#[warn(non_camel_case_types)]


mod stats;
mod logger;
mod io;

use clap::{Parser, Subcommand};
use crate::logger::init_logger;
use crate::stats::{stat_all_inputs};
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
            // println!("input: {:?}", input);
            // stats_all(input, &mut writer);
            stat_all_inputs(input, &mut writer);
        }
    }
}

#[derive(Parser)]
// #[command(author, version, about = "seqkit in rust", long_about)]
#[command(name = "seqkit-rs")]
#[command(about = "a cross-platform and ultrafast toolkit for FASTA/Q file manipulation")]
#[command(long_about = "long_about todo!!!")]
#[command(author, version)]
#[command(
    help_template =
    "{name} -- {about}\n\nVersion: {version}\n\nAuthors: {author}\
    \n\n{usage-heading} {usage}\n{all-args}"
    ) // change template more!
]
struct Cli {
    /// Output file ("-" for stdout)
    #[arg(long, short, global = true, default_value = "-")]
    outfile: String,
    /// Rewrite output file [default: false]
    #[arg(long, short, global = true, default_value = "false")]
    rewrite: bool,
    /// Subcommands
    #[command(subcommand)]
    command: Commands,

}

#[derive(Subcommand)]
enum Commands {
    /// Print statistics of the fastx file
    #[command(visible_alias = "stat")]
    Stats {
        /// Input FAST[A,Q] file
        #[arg(required = false, global = true)]
        input: Option<Vec<String>>,
    },

}