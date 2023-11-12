mod utils;
mod filters;

extern crate bam;
extern crate clap;
extern crate serde;
extern crate serde_json;

use std::io::{BufRead, Read, Write};
use serde::{Deserialize, Serialize};
use bam::{RecordReader, RecordWriter};
use bam::record::Record;
// Define a struct to represent filters
// #[derive(Serialize, Deserialize)]
// struct Filter {
//     criteria: String,
// }

// Define filter creation and combining logic
fn create_filter(filter_name: &str, args: Vec<&str>) {
    // Implement filter creation logic based on args
}

fn combine_filters(target_filter: &str, filter1: &str, operator: &str, filter2: &str) {
    // Implement filter combination logic using specified operator
}

// Define filter application logic
fn apply_filter(filter: &str, input_file: &str, output_file: &str, threads:u16) {
    // Implement the logic to apply the filter to the input BAM file
    assert!(input_file.ends_with(".bam") || input_file.ends_with(".sam"), "Input file must be a BAM or SAM file!");
    assert!(threads>0, "Number of threads must be greater than 0!");


    let reader:Box<dyn RecordReader<Item=Result<Record,std::io::Error>>>  = if input_file.ends_with(".bam") {
        Box::new(bam::BamReader::from_path(input_file, threads - 1).unwrap())
    } else {
        Box::new(bam::SamReader::from_path(input_file).unwrap())
    };

    for record in reader {
        let record:Record = record.unwrap();
        // Do something.
    };
}

fn read_files(input_file: &str, output_file: &str) {
    // Implement the logic to read the input BAM file and write the output BAM file
}

// TODO: uniquely aligned
// TODO: individual flag functions
fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use rstest::{*};

    #[fixture]
    fn bam_record() {}
}