extern crate bam;
extern crate clap;
extern crate serde;
extern crate serde_json;

use std::fs::OpenOptions;
use std::io::{BufRead, Read, Write};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use bam::{RecordReader, RecordWriter};
use bam::record::Record;
use clap::{command, Parser, Subcommand};
use serde::{Deserialize, Serialize};

use crate::filters::Filtering;
use crate::utils::BoolOperator;

mod filters;
mod utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'p', long, env)]
    bametric_path: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum CreateCommands {
    /// Create a filter based on read length
    Length {
        /// Minimum read length (inclusive)
        min_len: u32,
        /// Maximum read length (inclusive)
        max_len: u32,
    },
    /// Create a filter based on a tag:value pair
    Tag {
        /// Tag name
        tag_name: String,
        /// Tag value type
        tag_type: utils::CliTagType,
        /// Tag value
        tag_value: String,
    },
    /// Create a filter based on mapping quality
    Mapq {
        /// Minimum mapping quality (inclusive)
        min_mapq: u8,
        /// Maximum mapping quality (inclusive)
        max_mapq: u8,
    },
    /// Create a filter based on the reference name
    RefName { ref_id: i32 },
    /// Create a filter based on the identity of a nucleotide at a given position (e.g. G at the 1st position)
    Nucleotide {
        /// Position in the read to examine. 0-based.
        /// Positive values (and zero) are relative to the start of the read (5'),
        /// negative values are relative to the end of the read (3').
        position: i64,
        /// Nucleotide to be matched
        nucleotide: char,
        /// Treat 'N' nucleotides as a wildcard, matching them to any other nucleotide.
        #[clap(short = 'w', long, action = clap::ArgAction::SetTrue)]
        n_is_wildcard: bool,
    },
    /// Create a filter based on the bitwise SAM flags
    Flag {
        /// bitwise SAM flags to be matched.
        /// Any read that matches at least one of the specified flags will be removed.
        remove_flags: u16,
    },
}

#[derive(Subcommand)]
enum Commands {
    /// Initiate a new BAMetric session
    Init {},

    /// Create a new filter
    Create {
        ///  Optionally set a name for the filter. If not specified, a name will be generated automatically
        #[clap(short = 'n', long)]
        name: Option<String>,
        ///  Optionally invert the filter logic
        #[clap(short = 'o', long)]
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
        input: Vec<PathBuf>,
        /// Output directory
        #[clap(short = 'o', long)]
        output: PathBuf,
        /// Number of threads to use (supported for BAM files only)
        #[clap(short = 'p', long, default_value = "1")]
        threads: u16,
        /// Toggle verbose output
        #[clap(short = 'v', long, required = false)]
        verbose: bool,
    },

    /// Import filters from a JSON file
    Import {
        ///  Path to the JSON file containing the filters to be imported
        import_path: PathBuf,
    },

    /// Export filters to a JSON file
    Export {
        ///  Path to the JSON file to which the filters will be exported. If not specified, the filters will be printed to stdout.
        export_path: Option<PathBuf>,
    },
    /// View the list of defined filters
    View {},
}

fn deserialize_from_json(s: &str) -> Result<filters::Config, serde_json::Error> {
    serde_json::from_str(s)
}

fn serialize_to_json(config: &filters::Config) -> Result<String, serde_json::Error> {
    serde_json::to_string(config)
}

// Define filter creation and combining logic
fn create_filter(
    filter_name: Option<String>,
    opposite: bool,
    args: CreateCommands,
    config_path: &Path,
) {
    // Implement filter creation logic based on args
    let name = match filter_name {
        Some(s) => s,
        None => "combined filter 1".to_string(), //TODO: name generator
    };
    let filter: Box<dyn Filtering> = match args {
        CreateCommands::Length { min_len, max_len } => Box::new(filters::LengthFilter::new(
            name.clone(),
            min_len,
            max_len,
            opposite,
        )),
        CreateCommands::Tag {
            tag_name,
            tag_type,
            tag_value,
        } => {
            let tag_value = utils::convert_to_minimal_tag_value(tag_type, &tag_value).unwrap();
            let tag_name = utils::str_to_tag_name(&tag_name);
            Box::new(filters::TagFilter::new(
                name.clone(),
                tag_name,
                tag_value,
                opposite,
            ))
        }
        CreateCommands::Mapq { min_mapq, max_mapq } => Box::new(filters::MapqFilter::new(
            name.clone(),
            min_mapq,
            max_mapq,
            opposite,
        )),
        CreateCommands::RefName { ref_id } => {
            Box::new(filters::RefNameFilter::new(name.clone(), ref_id, opposite))
        }
        CreateCommands::Nucleotide {
            position,
            nucleotide,
            n_is_wildcard,
        } => Box::new(filters::NthNucleotideFilter::new(
            name.clone(),
            position,
            nucleotide,
            n_is_wildcard,
            opposite,
        )),
        CreateCommands::Flag { remove_flags } => Box::new(filters::FlagFilter::new(
            name.clone(),
            remove_flags,
            opposite,
        )),
    };
    store_filter(filter, &name, config_path);
}

fn combine_filters(
    combined_name: Option<String>,
    filter1: &str,
    operator: BoolOperator,
    filter2: &str,
    config_path: &Path,
) {
    // Implement filter combination logic using specified operator
    let mut objs = get_filters(vec![filter1, filter2], config_path);
    let f2_obj = objs.pop().unwrap();
    let f1_obj = objs.pop().unwrap();
    let name = match combined_name {
        Some(s) => s,
        None => "combined filter 1".to_string(), //TODO: name generator
    };
    let combined = filters::CombinedFilter::new(name.clone(), f1_obj, f2_obj, operator);
    store_filter(Box::new(combined), &name, config_path);
}

fn get_filters(filter_names: Vec<&str>, config_path: &Path) -> Vec<Box<dyn Filtering>> {
    let mut config = load_config(config_path);
    let mut filters = Vec::new();
    for name in filter_names {
        let filter = config.get(name).unwrap();
        filters.push(filter);
    }
    return filters;
}

fn store_filter(filter: Box<dyn Filtering>, name: &str, config_path: &Path) {
    let mut config = load_config(config_path);
    config.push(name, filter);
    save_config(&config, config_path)
}

fn load_config(config_path: &Path) -> filters::Config {
    let mut config_file = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(config_path)
        .unwrap();
    let mut config_str = String::new();
    config_file.read_to_string(&mut config_str).unwrap();
    let config = deserialize_from_json(&config_str).unwrap();
    return config;
}

fn save_config(config: &filters::Config, config_path: &Path) {
    let mut config_file = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(config_path)
        .unwrap();
    let json_str = serialize_to_json(config).unwrap();
    config_file.write_all(json_str.as_bytes()).unwrap();
}

fn import_filters(import_path: &Path, config_path: &Path) {
    let config = load_config(import_path);
    save_config(&config, config_path);
}

fn export_filters(export_path: Option<&Path>, config_path: &Path) -> Option<String> {
    let config = load_config(config_path);
    if export_path.is_none() {
        return Some(serialize_to_json(&config).unwrap());
    }
    save_config(&config, export_path.unwrap());
    return None;
}

fn init(config_path: &Path) {
    let config = filters::Config::new();
    save_config(&config, config_path);

    eprintln!("Initialized BAMetric session at {}", config_path.display());
}

// Define filter application logic
fn apply_filter(
    filter: &str,
    input_file: &Path,
    output_file: &Path,
    threads: u16,
    config_path: &Path,
) {
    println!("Applying filter {} to file {}", filter, input_file.display());
    println!("Output will be written to {}", output_file.display());
    let suffix = input_file.extension().unwrap();
    // Implement the logic to apply the filter to the input BAM file
    assert!(suffix == "sam" || suffix == "bam", "Input file must be a BAM or SAM file!");
    assert!(threads > 0, "Number of threads must be greater than 0!");

    let filter = get_filters(vec![filter], config_path).pop().unwrap();

    let reader: Box<dyn RecordReader<Item=Result<Record, std::io::Error>>> =
        if suffix == "bam" {
            Box::new(bam::BamReader::from_path(input_file, threads - 1).unwrap())
        } else {
            Box::new(bam::SamReader::from_path(input_file).unwrap())
        };

    let reader_header: bam::Header = if suffix == "bam" {
        bam::BamReader::from_path(input_file, threads - 1)
            .unwrap()
            .header()
            .clone()
    } else {
        bam::SamReader::from_path(input_file)
            .unwrap()
            .header()
            .clone()
    };

    let mut writer: Box<dyn RecordWriter> = if output_file.extension().unwrap() == "bam" {
        Box::new(bam::BamWriter::from_path(output_file, reader_header).unwrap())
    } else {
        Box::new(bam::SamWriter::from_path(output_file, reader_header).unwrap())
    };

    for record in reader {
        let record: Record = record.unwrap();
        let res = filter.apply_to(&record);
        if res {
            writer.write(&record).unwrap()
        }
    }
    writer.finish().unwrap();
}

fn view_filters(config_path: &Path) {
    let config = load_config(config_path);
    for (name, filter) in config.iter() {
        println!("{}: {}", name, filter.repr());
    }
}

fn read_files(input_file: &str, output_file: &str) {
    // Implement the logic to read the input BAM file and write the output BAM file
}

fn main() {
    let args = Args::parse();
    let config_path = match args.bametric_path {
        Some(s) => s,
        None => PathBuf::from_str("bametric.json").unwrap(),
    };
    match args.cmd {
        Commands::Init {} => init(&config_path),
        Commands::Create {
            name,
            opposite,
            cmd,
        } => create_filter(name, opposite, cmd, &config_path),
        Commands::Combine {
            filter1,
            operator,
            filter2,
            name,
        } => combine_filters(name, &filter1, operator, &filter2, &config_path),
        Commands::Apply {
            filter_name,
            input,
            output,
            threads,
            verbose,
        } => {
            for this_input in input {
                if verbose {
                    eprintln!("Processing file {}", this_input.display());
                }
                apply_filter(&filter_name, &this_input, &output, threads, &config_path);
            }
        }
        Commands::Import { import_path } => import_filters(&import_path, &config_path),
        Commands::Export { export_path } => {
            let out = export_filters(export_path.as_deref(), &config_path);
            match out {
                Some(s) => {
                    println!("{}", s);
                }
                None => {}
            }
        }
        Commands::View {} => view_filters(&config_path),
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    #[fixture]
    fn bam_record() {}
}
