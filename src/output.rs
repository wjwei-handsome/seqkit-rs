use std::fs::File;
use std::io::{BufWriter, stdout, Write};

pub fn output_writer(output_file: &String) -> Box<dyn Write> {
    if output_file == "-" {
        Box::new(stdout())
    } else {
        let file = File::create(output_file).unwrap();
        Box::new(BufWriter::new(file))
    }
}