extern crate core;

mod faidx;
mod io;
mod logger;
mod stats;

use crate::faidx::{create, extract};
use crate::io::output_writer;
use crate::logger::init_logger;
use crate::stats::stat_all_inputs;
use clap::{Parser, Subcommand, ValueEnum};

fn main() {
    init_logger();
    let cli = Cli::parse();
    // process outfile, if `-` then stdout, else write to file, or gzip file
    // TODO: add gzip support
    let outfile = cli.outfile;
    let mut writer = output_writer(&outfile, cli.rewrite);
    match &cli.command {
        Commands::Stats { input, format } => {
            stat_all_inputs(input, &mut writer, format);
        }
        Commands::Faidx { input, regions, .. } => match regions {
            Some(regions) => {
                extract(input, regions, &mut writer, cli.line_width);
            }
            None => {
                create(input, cli.rewrite);
            }
        },
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
    \n\n{usage-heading} {usage}\n\n{all-args}"
    ) // change template more!
]
struct Cli {
    /// Output file ("-" for stdout)
    #[arg(long, short, global = true, default_value = "-")]
    outfile: String,
    /// Bool, if rewrite output file [default: false]
    #[arg(long, short, global = true, default_value = "false")]
    rewrite: bool,
    /// Line width when outputing FASTA format (0 for no wrap)
    #[arg(
        long = "line-width",
        short = 'w',
        global = true,
        default_value = "60",
        required = false
    )]
    line_width: Option<u8>,
    /// Subcommands
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print statistics of the fastx file
    #[command(visible_alias = "stat")]
    Stats {
        /// Input FAST[A,Q] files, None for STDIN
        #[arg(required = false)]
        input: Option<Vec<String>>,
        /// Output format,
        #[arg(long, short, default_value = "markdown")]
        format: PrintFormat,
    },
    /// create FASTA index file and extract subsequence
    #[command(visible_alias = "fai")]
    Faidx {
        /// Input FASTA file
        #[arg(required = true)]
        input: String,
        /// Regions to extract, e.g. chr1:1-100
        #[arg(required = false)]
        regions: Option<Vec<String>>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum PrintFormat {
    Tabular,
    Markdown,
}
