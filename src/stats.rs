use std::borrow::Cow;
use std::io::Write;
use rayon::prelude::*;
use needletail::parser::Format;
use tabled::{Table, Tabled};
use crate::io::{input_reader, format_table_style};
use crate::PrintFormat;

// define a tabled struct for display and output
#[derive(Tabled)]
struct FastxStat {
    filename: String,
    format: String,
    num_seqs: usize,
    sum_len: usize,
    min_len: usize,
    #[tabled(display_with = "float2")]
    avg_len: f64,
    max_len: usize,
    #[tabled(rename = "Q1")]
    q1: usize,
    #[tabled(rename = "Q1")]
    q2: usize,
    #[tabled(rename = "Q3")]
    q3: usize,
    sum_gap: usize,
    #[tabled(rename = "N50")]
    n50: usize,
    #[tabled(rename = "Q20(%)", display_with = "float2")]
    q20: f64,
    #[tabled(rename = "Q30(%)", display_with = "float2")]
    q30: f64,
}

/// It's only for `display_with` in tabled
fn float2(n: &f64) -> String {
    format!("{:.2}", n)
}


/// public function to stat all inputs in parallel
pub fn stat_all_inputs(input_list: &Option<Vec<String>>, writer: &mut dyn Write, format: &PrintFormat) {

    let result_vec = match input_list {
        Some(input_list) =>
            {
                input_list
                .par_iter()
                .map(|input|
                    {
                        stat(&Some(input.clone())) // just clone it;light mem
                    }
                )
                .collect::<Vec<_>>()
            },
        None =>
            {
                vec![stat(&None)]
            }
    };

    let mut table = Table::new(&result_vec);

    table = format_table_style(table, format);

    writeln!(writer, "{}", table).unwrap();

}


/// get statistics of a single input
fn stat(input: &Option<String>) -> FastxStat {
    // get input reader
    let mut input_reader = input_reader(input);

    let filename = match input {
        Some(input) => input,
        None => "stdin",
    };

    let mut sum_len = 0;
    let mut num_seqs = 0;
    let mut sum_gap = 0;
    let mut len_vec = Vec::new();

    // start to read first record and get format
    let first_rec = if
    let Some(first_rec) = input_reader.next() { first_rec }
    else { panic!("invalid record") };
    let first_seq_rec = first_rec.expect("invalid record");
    let format = first_seq_rec.format();
    let file_format = if let Format::Fasta = format { "fasta" } else { "fastq" };
    let seq_len = first_seq_rec.num_bases();
    len_vec.push(seq_len);
    sum_len += seq_len; // sum_len
    num_seqs += 1; // num_seqs
    let n_count = count_n_bases_para(first_seq_rec.seq());
    sum_gap += n_count;

    // start to read rest records
    while let Some(record) = input_reader.next() {
        let seq_rec = record.expect("invalid record");
        // keep track of the total number of bases
        let seq_len = seq_rec.num_bases();
        // count N/n count in parallel of Cow<'_, [u8]> slice.
        let n_count = count_n_bases_para(seq_rec.seq());
        // update len_vec
        len_vec.push(seq_len);
        // update n_bases, n_seqs, gap
        sum_len += seq_len; // sum_len
        num_seqs += 1; // num_seqs
        sum_gap += n_count; // sum_gap
    }

    let avg_len = sum_len as f64 / num_seqs as f64;
    let (min_len, max_len, q1, q2, q3, n50) = quartiles_n50_min_max(&mut len_vec, sum_len, num_seqs);

    FastxStat {
        filename: filename.to_string(),
        format: file_format.to_string(),
        num_seqs,
        sum_len,
        min_len,
        avg_len,
        max_len,
        q1,
        q2,
        q3,
        sum_gap,
        n50,
        q20: 0.0,
        q30: 0.0,
    }
}


/// count N/n bases in parallel, 78 for N, 110 for n
fn count_n_bases_para(seq: Cow<[u8]>) -> usize {
    let n_base_count = seq
        .par_iter()
        .filter(|&&x| x == 78 || x == 110)
        .count() as usize;
    n_base_count
}

//// compute the quartiles and n50
fn quartiles_n50_min_max(len_vec: &mut Vec<usize>, sum_len: usize, num_seqs: usize) -> (usize, usize, usize, usize, usize, usize) {

    len_vec.sort_unstable();

    let min_len = len_vec[0];
    let max_len = len_vec[num_seqs - 1];

    let half_len = sum_len / 2;

    // cursum to get the n50
    let cumulataion = len_vec
        .iter()
        .scan(0, |state, &x| {
            *state += x;
            Some(*state)
        })
        .collect::<Vec<_>>();

    let n50_offset = cumulataion
        .par_iter()
        .position_first(|&x| x >= half_len)
        .unwrap(); // usually its a Some

    let q1_index = num_seqs / 4;
    let q1 = len_vec[q1_index];
    let q2_index = num_seqs / 2;
    let q2 = len_vec[q2_index];
    let q3_index = q1_index + q2_index;
    let q3 = len_vec[q3_index];

    let n50 = len_vec[n50_offset];

    (min_len, max_len, q1, q2, q3, n50)

}