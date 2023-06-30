use core::fmt;
use std::io::Write;
use log::{error, info, warn};
use needletail::parser::{Format, IndexedReader, SequenceRecord};
use crate::io::{input_reader, output_writer};
use memchr::memchr;
use std::str;
use regex::Regex;

pub fn extract(input: &String, regions: &Vec<String>) {

    check_suffix(input);
    let mut fai_reader = match IndexedReader::from_path(input) {
        Ok(reader) => reader,
        Err(err) => {
            error!("{}, please create first", err.msg);
            std::process::exit(1);
        }
    };

    for region_str in regions {
        // 1  +2 +3 +4 +5 +6 +7 +8
        // -8 -7 -6 -5 -4 -3 -2 -1
        let region = ExprRegion::from(region_str.clone());
        match fai_reader.fetch(region.name.as_str(), None, None) {
            Ok(_) => {},
            Err(err) => {
                warn!("Continued: {}", err.msg);
                continue
            }
        };
        let record_len = fai_reader.fetched_id.as_ref().unwrap().len;
        let positive_region = match ExprRegion::positive_region(
            region.name.clone(),
            region.start,
            region.end,
            record_len,
        ) {
            Some(positive_region) => positive_region,
            None => {
                warn!("Continued: Invalid Input Region: `{}`", region);
                continue
            }
        };
        let subseq = match fai_reader.subseq(
            positive_region.name.as_str(),
            Some(positive_region.start as u64 - 1),
            Some(positive_region.end as u64)
        ) {
            Ok(subseq) => subseq,
            Err(e) => {
                warn!("Continued: {}", e.msg); // In theory, it will not appear.
                continue
            }
        };
        println!("{}", subseq);
        }
        // println!("{}", positive_region);
        // if region.start < 0 {
        //     println!("start < 0 : todo");
        //     if region.end < -region.start {
        //         // exp: 1 - -2
        //         println!("start < 0 || end < -start : todo");
        //     } else {
        //         // exp: 1 - -1
        //         println!("start < 0 || end >= -start : todo");
        //     }
        // } else if region.start == 0 {
        //     println!("start == 0 : todo");
        // } else {
        //     if region.end == 0 {
        //         // exp: 1 - 0
        //         println!("start > 0 || end == 0 : todo");
        //     } else if region.end < 0 {
        //         // exp: 2 - -1
        //         println!("start > 0 || end < 0 : todo");
        //     } else {
        //         // 1 - 8
        //         if region.start <= region.end {
        //             let subseq = match fai_reader.subseq(
        //                 region.name.as_str(),
        //                 Some(region.start as u64 - 1),
        //                 Some(region.end as u64)
        //             ) {
        //                 Ok(subseq) => subseq,
        //                 Err(e) => {
        //                     warn!("Continued: {}", e.msg);
        //                     continue
        //                 }
        //             };
        //             println!("{}", subseq);
        //         } else {
        //             // 6 - 2
        //             warn!("Continued: Invalid Input Region: `{}`", region)
        //         }
        //     }
        // }
}

#[derive(Debug)]
struct ExprRegion {
    name: String,
    start: isize,
    end: isize,
}

impl fmt::Display for ExprRegion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}-{}", self.name, self.start, self.end)
    }
}

impl ExprRegion {
    fn positive_region(name: String, start: isize, end: isize, length: u64) -> Option<ExprRegion> {
        //0 +1  +2 +3 +4 +5 +6 +7 +8 0
        //0 -8  -7 -6 -5 -4 -3 -2 -1 0
        // length = 8
        let positive_start = if start > 0 {
            start
        } else if start == 0 {
            1
        } else {
            length as isize + start + 1
        };
        let positive_end = if end > 0 {
            end
        } else if end == 0 {
            length as isize
        } else {
            length as isize + end + 1
        };
        if positive_start <= positive_end {
            Some(ExprRegion {
                name,
                start: positive_start,
                end: positive_end,
            })
        } else {
            None
        }
    }
}

impl From<String> for ExprRegion {
    fn from(value: String) -> Self {
        // actually, unnecessary to match err due to regex
        // NOTE: `faidx` is 1-based ,but `subseq` is 0-based.
        let re_region_full = Regex::new(r"^(.+?):(-?\d+)-(-?\d+)$").unwrap();
        let re_region_one_base = Regex::new(r"^(.+?):(\d+)$").unwrap();
        let re_region_only_begin = Regex::new(r"^(.+?):(-?\d+)-$").unwrap();
        let re_region_only_end = Regex::new(r"^(.+?):-(-?\d+)$").unwrap();

        if re_region_full.is_match(&*value) {
            let caps = re_region_full.captures(&*value).unwrap();
            let name = caps.get(1).unwrap().as_str().to_string();
            let start = caps.get(2).unwrap().as_str().parse::<isize>().unwrap();
            let end = caps.get(3).unwrap().as_str().parse::<isize>().unwrap();
            ExprRegion { name, start, end }
        } else if re_region_one_base.is_match(&*value) {
            let caps = re_region_one_base.captures(&*value).unwrap();
            let name = caps.get(1).unwrap().as_str().to_string();
            let start = caps.get(2).unwrap().as_str().parse::<isize>().unwrap();
            let end = start;
            ExprRegion { name, start, end }
        } else if re_region_only_begin.is_match(&*value) {
            let caps = re_region_only_begin.captures(&*value).unwrap();
            let name = caps.get(1).unwrap().as_str().to_string();
            let start = caps.get(2).unwrap().as_str().parse::<isize>().unwrap();
            let end = -1;
            ExprRegion { name, start, end }
        } else if re_region_only_end.is_match(&*value) {
            let caps = re_region_only_end.captures(&*value).unwrap();
            let name = caps.get(1).unwrap().as_str().to_string();
            let start = 1;
            let end = caps.get(2).unwrap().as_str().parse::<isize>().unwrap();
            ExprRegion { name, start, end }
        } else {
            ExprRegion {
                name: value,
                start: 1,
                end: -1,
            }
        }
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