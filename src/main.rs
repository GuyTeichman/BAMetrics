mod utils;

extern crate bam;
extern crate clap;
extern crate serde;
extern crate serde_json;

// use std::fs::{File, OpenOptions};
// use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
// use std::collections::HashMap;

// use bam::{RecordReader, RecordWriter};
use bam::record::Record;
use bam::record::tags::{TagName, TagValue};
// use clap::App;
use utils::Nucleotide;

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
fn apply_filter(filter: &str, input_file: &str, output_file: &str) {
    // Implement the logic to apply the filter to the input BAM file
}

fn read_files(input_file: &str, output_file: &str) {
    // Implement the logic to read the input BAM file and write the output BAM file
}

fn filter_by_nth_nucleotide(record: &Record, position: i64, nucleotide: Nucleotide, opposite: bool) -> bool {
    let seq = record.sequence();
    if !seq.available() {
        return utils::_opposite(false, opposite);
    }
    else {
        return false;
    }
}

fn filter_by_length(record: &Record, min_len: u32, max_len: u32, opposite: bool) -> bool {
    let read_len = record.query_len();
    return if read_len < min_len || read_len > max_len {
        utils::_opposite(false, opposite)
    } else {
        utils::_opposite(true, opposite)
    }
}


fn filter_by_flags(record: &Record, remove_flags: u16, opposite: bool)  -> bool{
    let flags = record.flag();
    return if flags.no_bits(remove_flags) {
        utils::_opposite(true, opposite)
    } else {
        utils::_opposite(false, opposite)
    }

}


fn filter_by_mapq(record: &Record, min_mapq: u8, max_mapq: u8, opposite: bool)  -> bool{
    let mapq:u8 = record.mapq();
    return if mapq < min_mapq || mapq > max_mapq {
        utils::_opposite(false, opposite)
    } else {
        utils::_opposite(true, opposite)
    }
}

fn filter_by_ref_name(record: &Record, ref_id: i32, opposite: bool) -> bool {
    let this_ref_id = record.ref_id();
    return utils::_opposite(this_ref_id == ref_id, opposite)
}

fn filter_by_tag(record:Record, tag_name:TagName, exp_tag_value:TagValue,opposite:bool)->bool {
    return if let Some(tag) = record.tags().get(&tag_name) {
        if utils::_are_tag_values_equal(&tag, &exp_tag_value) {
            utils::_opposite(true, opposite)
        } else {
            utils::_opposite(false, opposite)
        }
    } else {
        utils::_opposite(false, opposite)
    }
}
fn main() {
    println!("Hello, world!");
}