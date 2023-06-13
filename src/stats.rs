use std::borrow::Cow;
use needletail::{parse_fastx_file, parse_fastx_stdin};
use rayon::prelude::*;


pub fn stats_all(input: &Option<String>, _output: &Option<String>, _rewrite: bool) {

    let mut input_reader = match input {
        Some(input) => parse_fastx_file(input).expect("valid path/file, please check"),
        None => parse_fastx_stdin().expect("valid stdin"),
    };

    let filename = match input {
        Some(input) => input,
        None => "stdin",
    };

    let mut n_bases = 0;
    let mut n_seqs = 0;
    let mut gap = 0;
    let mut min_len = usize::MAX; // init in a max value to cmp
    let mut max_len = 0;
    let mut len_vec = Vec::new();

    // start to read records
    while let Some(record) = input_reader.next() {
        let seq_rec = record.expect("invalid record");
        // keep track of the total number of bases
        let seq_len = seq_rec.num_bases();
        // count N/n count in parallel of Cow<'_, [u8]> slice.
        let n_count = count_n_bases_para(seq_rec.seq());
        // update min_len and max_len
        min_len = seq_len.min(min_len); // min_len why zero?
        max_len = seq_len.max(max_len); // max_len
        // update len_vec
        len_vec.push(seq_len);
        // update n_bases, n_seqs, gap
        n_bases += seq_len; // sum_len
        n_seqs += 1; // num_seqs
        gap += n_count; // sum_gap
    }
    // start to cumulate
    // let avg_len = mean_len_para(&len_vec);
    let avg_len = n_bases as f64 / n_seqs as f64;
    let (q1, q2, q3, n50) = quartiles_n50(&mut len_vec, n_bases, n_seqs);

    println!("file\tformat\tnum_seqs\tsum_len\tmin_len\tavg_len\tmax_len\tQ1\tQ2\tQ3\tsum_gap\tN50\tQ20_percent\tQ30_percent");
    println!("{}\t{}\t{}\t{}\t{}\t{:.2}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
             filename, "fasta", n_seqs, n_bases, min_len, avg_len, max_len, q1, q2, q3, gap, n50, 0, 0);
}

fn count_n_bases_para(seq: Cow<[u8]>) -> i32 {
    let n_base_count = seq
        .par_iter()
        .filter(|&&x| x == 78 || x == 110)
        .count() as i32;
    n_base_count
}

// fn mean_len_para(len_vec: &Vec<usize>) -> f64 {
//     let sum: usize = len_vec.par_iter().sum();
//     let len = len_vec.len();
//     sum as f64 / len as f64
// }

fn quartiles_n50(len_vec: &mut Vec<usize>, sum_len: usize, num_seqs: usize) -> (usize, usize, usize, usize) {
    // todo!("quartiles_n")
    len_vec.sort_unstable();

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

    (q1, q2, q3, n50)

}