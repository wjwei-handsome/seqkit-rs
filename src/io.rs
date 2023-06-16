use std::fs::File;
use std::io::{BufWriter, stdout, Write};
use std::path::Path;
use log::{warn, error};
use needletail::{FastxReader, parse_fastx_file, parse_fastx_stdin};


/// check if output file exists and if rewrite
fn outfile_exist(output_file: &String, rewrite: bool) {
    // check if output file exists
    let path = Path::new(output_file);
    if path.exists() {
        if rewrite {
            // rewrite the file
            warn!("file {} exist, will rewrite it", output_file);
        } else {
            // exit
            error!("file {} exist, use -r to rewrite it", output_file);
            std::process::exit(1);
        }
    }
}


/// get output writer
pub fn output_writer(output_file: &String, rewrite: bool) -> Box<dyn Write> {
    if output_file == "-" {
        Box::new(stdout())
    } else {
        outfile_exist(output_file, rewrite);
        let file = File::create(output_file).unwrap();
        Box::new(BufWriter::new(file))
    }
}

/// get input reader and catch error
pub fn input_reader(input: &Option<String>) -> Box<dyn FastxReader>{
    match input {
        Some(input) => {
            match parse_fastx_file(input) {
                Ok(reader) => reader,
                Err(err) => {
                    error!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        None => {
            // check if stdin is empty
            if atty::is(atty::Stream::Stdin) {
                error!("no input content detected");
                std::process::exit(1);
            }
            parse_fastx_stdin().expect("valid stdin, please contact author")
        }
    }
}