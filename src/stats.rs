use std::borrow::Cow;
use std::io::Write;
use needletail::{parse_fastx_file, parse_fastx_stdin};
use rayon::prelude::*;


pub fn stats_all(input: &Option<String>, writer: &mut dyn Write) {

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
    // let mut min_len = usize::MAX; // init in a max value to cmp
    // let mut max_len = 0;
    let mut len_vec = Vec::new();

    // start to read first record and get format
    let first_rec = if
        let Some(first_rec) = input_reader.next() { first_rec }
        else { panic!("invalid record") };
    let first_seq_rec = first_rec.expect("invalid record");
    let format = first_seq_rec.format();
    let format_symbol = format.start_char();
    let file_format = if let '>' = format_symbol { "fasta" }
        else if let '@' = format_symbol { "fastq" }
        else { "unknown" };
    println!("format: {:?}", format);
    let seq_len = first_seq_rec.num_bases();
    len_vec.push(seq_len);
    n_bases += seq_len; // sum_len
    n_seqs += 1; // num_seqs
    let n_count = count_n_bases_para(first_seq_rec.seq());
    gap += n_count;

    // start to read rest records
    while let Some(record) = input_reader.next() {
        let seq_rec = record.expect("invalid record");
        // keep track of the total number of bases
        let seq_len = seq_rec.num_bases();
        // count N/n count in parallel of Cow<'_, [u8]> slice.
        let n_count = count_n_bases_para(seq_rec.seq());
        // update min_len and max_len
        // min_len = seq_len.min(min_len); // min_len why zero?
        // max_len = seq_len.max(max_len); // max_len
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
    let (min_len, max_len, q1, q2, q3, n50) = quartiles_n50_min_max(&mut len_vec, n_bases, n_seqs);

    writer.write_all(b"file\tformat\tnum_seqs\tsum_len\tmin_len\tavg_len\tmax_len\tQ1\tQ2\tQ3\tsum_gap\tN50\tQ20_percent\tQ30_percent\n").unwrap();
    // println!("file\tformat\tnum_seqs\tsum_len\tmin_len\tavg_len\tmax_len\tQ1\tQ2\tQ3\tsum_gap\tN50\tQ20_percent\tQ30_percent");
    writer.write_all(format!("{}\t{}\t{}\t{}\t{}\t{:.2}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                             filename, file_format, n_seqs, n_bases, min_len, avg_len, max_len, q1, q2, q3, gap, n50, 0, 0).as_bytes()).unwrap();
    // println!("{}\t{}\t{}\t{}\t{}\t{:.2}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
    //          filename, "fasta", n_seqs, n_bases, min_len, avg_len, max_len, q1, q2, q3, gap, n50, 0, 0);
}

fn count_n_bases_para(seq: Cow<[u8]>) -> i32 {
    let n_base_count = seq
        .par_iter()
        .filter(|&&x| x == 78 || x == 110)
        .count() as i32;
    n_base_count
}

// fn sum_mean_len_para(len_vec: &Vec<usize>) -> (usize, usize, f32) {
//     let sum_len = len_vec.par_iter().sum::<usize>() as usize;
//     let num_seqs = len_vec.len() as usize;
//     let avg_len = sum_len as f32 / num_seqs as f32;
//     println!("sum_len: {}, num_seqs: {}, avg_len: {}", sum_len, num_seqs, avg_len);
//     (sum_len, num_seqs, avg_len)
// }

fn quartiles_n50_min_max(len_vec: &mut Vec<usize>, sum_len: usize, num_seqs: usize) -> (usize, usize, usize, usize, usize, usize) {
    // todo!("quartiles_n")
    len_vec.sort_unstable();
    println!("len_vec: {:?}", len_vec);

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