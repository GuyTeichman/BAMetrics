mod utils;
mod filters;

extern crate bam;
extern crate clap;
extern crate serde;
extern crate serde_json;

use std::path::PathBuf;

use std::io::{BufRead, Read, Write};
use serde::{Deserialize, Serialize};
use bam::{RecordReader, RecordWriter};
use bam::record::Record;
use bam::record::tags::TagValue;
use clap::{arg, command, Command, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'p', long, env)]
    bametric_path: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum CreateCommands {
    /// Create a filter based on read length
    Length {
        /// Minimum read length (inclusive)
        min_len: u32,
        /// Maximum read length (inclusive)
        max_len: u32,
    },
    Tag {
        /// Tag name
        tag_name: String,
        /// Tag value
        tag_value: String,
    },
    Mapq {
        /// Minimum mapping quality (inclusive)
        min_mapq: u8,
        /// Maximum mapping quality (inclusive)
        max_mapq: u8,
    },
    RefName { ref_id: i32 },
    Nucleotide {
        /// Position in the read to examine. 0-based.
        /// Positive values (and zero) are relative to the start of the read (5'),
        /// negative values are relative to the end of the read (3').
        position: i64,
        /// Nucleotide to be matched
        nucleotide: char,
        /// Treat 'N' nucleotides as a wildcard, matching them to any other nucleotide.
        n_is_wildcard: bool,
    },
    Flag {
        /// bitwise SAM flags to be matched.
        /// Any read that matches at least one of the specified flags will be removed.
        remove_flags: u16
    },
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initiate a new BAMetric session
    Init {},

    /// Create a new filter
    Create {
        ///  Optionally set a name for the filter. If not specified, a name will be generated automatically
        #[clap(short = 'n', long)]
        name: Option<String>,

        ///  Optionally invert the filter logic
        #[clap(short, long)]
        opposite: bool,
        #[command(subcommand)]
        cmd: CreateCommands,
    },

    /// Combine two existing filters using a boolean operator
    Combine {
        /// Name of the first filter to be combined
        #[clap(index = 1)]
        filter1: String,
        /// The boolean operator to be used for combining the two filters
        #[clap(index = 2)]
        operator: utils::BoolOperator,
        /// Name of the second filter to be combined
        #[clap(index = 3)]
        filter2: String,
        /// Optionally set a name for the combined filter. If not specified, a name will be generated automatically
        #[clap(short = 'n', long)]
        name: Option<String>,
    },

    /// Apply a filter to BAM/SAM files
    Apply {
        ///  Name of the filter to be applied
        filter_name: String,
        /// Input BAM/SAM files
        input: Vec<String>,
        /// Output directory
        #[clap(short = 'o', long)]
        output: Option<String>,
        /// Output file format
        #[clap(short = 'f', long, default_value = "bam")]
        format: utils::SupportedFormats,
        /// Toggle verbose output
        #[clap(short = 'v', long, required = false)]
        verbose: bool,
    },

    /// Import filters from a JSON file
    Import {
        ///  Path to the JSON file containing the filters to be imported
        import_path: String,
    },

    /// Export filters to a JSON file
    Export {
        ///  Path to the JSON file to which the filters will be exported. If not specified, the filters will be printed to stdout.
        export_path: Option<String>,
    },
    /// View the list of defined filters
    View {},
}

// Define filter creation and combining logic
fn create_filter(filter_name: &str, args: Vec<&str>) {
    // Implement filter creation logic based on args
}

fn combine_filters(target_filter: &str, filter1: &str, operator: &str, filter2: &str) {
    // Implement filter combination logic using specified operator
}

// Define filter application logic
fn apply_filter(filter: &str, input_file: &str, output_file: &str, threads: u16) {
    // Implement the logic to apply the filter to the input BAM file
    assert!(input_file.ends_with(".bam") || input_file.ends_with(".sam"), "Input file must be a BAM or SAM file!");
    assert!(threads > 0, "Number of threads must be greater than 0!");


    let reader: Box<dyn RecordReader<Item=Result<Record, std::io::Error>>> = if input_file.ends_with(".bam") {
        Box::new(bam::BamReader::from_path(input_file, threads - 1).unwrap())
    } else {
        Box::new(bam::SamReader::from_path(input_file).unwrap())
    };

    for record in reader {
        let record: Record = record.unwrap();
        // Do something.
    };
}

fn read_files(input_file: &str, output_file: &str) {
    // Implement the logic to read the input BAM file and write the output BAM file
}


fn main() {
    let args = Args::parse();
    dbg!(args);
}


#[cfg(test)]
mod tests {
    use rstest::{*};

    #[fixture]
    fn bam_record() {}
}