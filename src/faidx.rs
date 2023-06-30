use std::io::Write;
use log::{error, info};
use needletail::parser::{Format, IndexedReader, SequenceRecord};
use crate::io::{input_reader, output_writer};
use memchr::memchr;
use std::str;

pub fn extract(input: &String, regions: &Vec<String>) {
    let mut fai_reader = IndexedReader::from_path(input).unwrap();
    let subseq = fai_reader.subseq("x", Some(6), Some(12)).unwrap();
    println!("{}", subseq);
}

struct region {
    name: String,
    start: usize,
    end: usize,
}

impl From<String> for region {
    fn from(value: String) -> Self {
        todo!()
    }
}



pub fn create(input: &String, rewrite: bool) {
    let fai_file_name = format!("{}.fai", input);
    info!("create index for {}", &input);

    let mut input_reader = input_reader(&Some(input.clone()));

    check_suffix(input); // check if input file is plain text

    // init offset and index records
    let mut offset = 0;

    // read first record and check format
    let first_rec = if
    let Some(first_rec) = input_reader.next() { first_rec }
    else { panic!("invalid record") };
    let first_seq_rec = first_rec.expect("invalid record");
    match first_seq_rec.format() {
        Format::Fasta => {},
        _ => {
            error!("only support fasta format");
            std::process::exit(1);
        }
    };

    let mut writer = output_writer(&fai_file_name, rewrite);

    fill_index_records(first_seq_rec, &mut offset, &mut writer);

    // start to read rest records
    while let Some(record) = input_reader.next() {
        let seq_rec = record.expect("invalid record");
        fill_index_records(seq_rec, &mut offset, &mut writer);
    }

    info!("Successfully create index file: {}", &fai_file_name);

}

fn get_first_line_pos(raw_seq: &[u8]) -> usize {
    match memchr(b'\n', raw_seq) {
        Some(pos) => pos,
        None => {
            raw_seq.len()
        }
    }
}

fn fill_index_records(
    record: SequenceRecord,
    offset: &mut u64,
    writer: &mut Box<dyn Write>,
) {
    let seq_name_str = str::from_utf8(record.id().clone()).unwrap();
    let seq_name_len = seq_name_str.len() + 1 + 1; // +1 for \n, +1 for >
    *offset += seq_name_len as u64;
    let seq_len = record.num_bases();
    let raw_seq = record.raw_seq();
    let first_next_line = get_first_line_pos(raw_seq);

    writeln!(
        writer,
        "{}\t{}\t{}\t{}\t{}",
        seq_name_str.to_string(),
        seq_len,
        *offset,
        first_next_line,
        first_next_line + 1,
    ).expect("TODO: panic message");

    *offset += raw_seq.len() as u64 + 1;

}

const INVALID_SUFFIX: [&str; 3] = [".gz", ".xz", ".bz2"];

fn check_suffix(input: &String) {
    for suffix in INVALID_SUFFIX.iter() {
        if input.ends_with(suffix) {
            error!("only support plain text file");
            std::process::exit(1);
        }
    }
}