extern crate bam;

use bam::Record;
use bam::record::tags::{TagName, TagValue};
use crate::utils;
use utils::BoolOperator;

extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};

pub trait Filtering {
    fn apply_to(&self, record: &Record) -> bool;

    fn repr(&self) -> String;

    fn name(&self) -> &str;
}

// TODO: uniquely aligned
// TODO: individual flag functions
pub struct CombinedFilter<'a> {
    name: &'a str,
    filter1: Box<dyn Filtering>,
    filter2: Box<dyn Filtering>,
    operator: BoolOperator,
}

#[derive(Serialize, Deserialize)]
pub struct LengthFilter<'a> {
    name: &'a str,
    min_len: u32,
    max_len: u32,
    opposite: bool,
}


pub struct TagFilter<'a> {
    name: &'a str,
    tag_name: TagName,
    tag_value: TagValue<'a>,
    opposite: bool,
}

#[derive(Serialize, Deserialize)]
pub struct MapqFilter<'a> {
    name: &'a str,
    min_mapq: u8,
    max_mapq: u8,
    opposite: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RefNameFilter<'a> {
    name: &'a str,
    ref_id: i32,
    opposite: bool,
}

#[derive(Serialize, Deserialize)]
pub struct NthNucleotideFilter<'a> {
    name: &'a str,
    position: i64,
    nucleotide: char,
    n_is_wildcard: bool,
    opposite: bool,
}

#[derive(Serialize, Deserialize)]
pub struct FlagFilter<'a> {
    name: &'a str,
    remove_flags: u16,
    opposite: bool,
}

impl <'a> CombinedFilter<'a> {
    pub fn new(name: &'a str, filter1: Box<dyn Filtering>, filter2: Box<dyn Filtering>, operator: BoolOperator,
    ) -> CombinedFilter<'a> {
        assert!(matches!(operator, BoolOperator::AND | BoolOperator::OR | BoolOperator::XOR |
            BoolOperator::XNOR | BoolOperator::NAND | BoolOperator::NOR | BoolOperator::IMPLIES),
                "Operator must be one of AND, OR, XOR, XNOR, NAND, NOR, or IMPLIES!");
        CombinedFilter {
            name,
            filter1,
            filter2,
            operator,
        }
    }
}

impl <'a>LengthFilter<'a> {
    pub fn new(name: &'a str, min_len: u32, max_len: u32, opposite: bool) -> LengthFilter<'a> {
        LengthFilter {
            name,
            min_len,
            max_len,
            opposite,
        }
    }
}

impl<'a> TagFilter<'a> {
    pub fn new(name: &'a str, tag_name: TagName, tag_value: TagValue<'a>, opposite: bool) -> TagFilter<'a> {
        TagFilter {
            name,
            tag_name,
            tag_value,
            opposite,
        }
    }
}

impl <'a>MapqFilter<'a> {
    pub fn new(name: &'a str, min_mapq: u8, max_mapq: u8, opposite: bool) -> MapqFilter {
        MapqFilter {
            name,
            min_mapq,
            max_mapq,
            opposite,
        }
    }
}

impl <'a>RefNameFilter<'a> {
    pub fn new(name: &'a str, ref_id: i32, opposite: bool) -> RefNameFilter {
        RefNameFilter {
            name,
            ref_id,
            opposite,
        }
    }
}

impl <'a>NthNucleotideFilter<'a> {
    pub fn new(name: &'a str, position: i64, nucleotide: char, n_is_wildcard: bool, opposite: bool) -> NthNucleotideFilter {
        assert!(matches!(nucleotide, 'A' | 'C' | 'G' | 'T' | 'N'), "Nucleotide must be one of A, C, G, T, or N!");
        NthNucleotideFilter {
            name,
            position,
            nucleotide,
            n_is_wildcard,
            opposite,
        }
    }
}

impl <'a>FlagFilter<'a> {
    pub fn new(name: &'a str, remove_flags: u16, opposite: bool) -> FlagFilter {
        FlagFilter {
            name,
            remove_flags,
            opposite,
        }
    }
}

impl<'a> Filtering for CombinedFilter<'a> {
    fn apply_to(&self, record: &Record) -> bool {
        let result1 = self.filter1.apply_to(record);
        let result2 = self.filter2.apply_to(record);
        return match self.operator {
            BoolOperator::AND => result1 && result2,
            BoolOperator::OR => result1 || result2,
            BoolOperator::XOR => result1 ^ result2,
            BoolOperator::XNOR => !(result1 ^ result2),
            BoolOperator::NAND => !(result1 && result2),
            BoolOperator::NOR => !(result1 || result2),
            BoolOperator::IMPLIES => !result1 || result2,
        };
    }

    fn repr(&self) -> String {
        format!("CombinedFilter(name={}, filter1={}, filter2={}, operator={:?})", self.name, self.filter1.name(), self.filter2.name(), self.operator)
    }
    fn name(&self) -> &str {
        self.name
    }
}

impl<'a> Filtering for FlagFilter<'a> {
    fn apply_to(&self, record: &Record) -> bool {
        let flags = record.flag();
        return if flags.no_bits(self.remove_flags) {
            utils::_opposite(true, self.opposite)
        } else {
            utils::_opposite(false, self.opposite)
        };
    }

    fn repr(&self) -> String {
        format!("FlagFilter(name={}, remove_flags={}, opposite={})", self.name, self.remove_flags, self.opposite)
    }
    fn name(&self) -> &str {
        &self.name
    }
}


impl<'a> Filtering for LengthFilter<'a> {
    fn apply_to(&self, record: &Record) -> bool {
        let read_len = record.query_len();
        return if read_len < self.min_len || read_len > self.max_len {
            utils::_opposite(false, self.opposite)
        } else {
            utils::_opposite(true, self.opposite)
        };
    }

    fn repr(&self) -> String {
        format!("LengthFilter(name={}, min_len={}, max_len={}, opposite={})", self.name, self.min_len, self.max_len, self.opposite)
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> Filtering for TagFilter<'a> {
    fn apply_to(&self, record: &Record) -> bool {
        return if let Some(tag) = record.tags().get(&self.tag_name) {
            if utils::_are_tag_values_equal(&tag, &self.tag_value) {
                utils::_opposite(true, self.opposite)
            } else {
                utils::_opposite(false, self.opposite)
            }
        } else {
            utils::_opposite(false, self.opposite)
        };
    }

    fn repr(&self) -> String {
        let formatted_tag = match self.tag_value {
            TagValue::Char(c) => format!("'{}'", c),
            TagValue::Int(i, _) => format!("{}", i),
            TagValue::Float(f) => format!("{}", f),
            TagValue::String(s, _) => format!("'{:#?}'", s),
            TagValue::IntArray(_) => format!("IntArray"),
            TagValue::FloatArray(_) => format!("FloatArray"),
        };
        format!("TagFilter(name={}, tag_name={:#?}, tag_value={}, opposite={})", self.name, self.tag_name, formatted_tag, self.opposite)
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> Filtering for MapqFilter<'a> {
    fn apply_to(&self, record: &Record) -> bool {
        let mapq = record.mapq();
        return if mapq < self.min_mapq || mapq > self.max_mapq {
            utils::_opposite(false, self.opposite)
        } else {
            utils::_opposite(true, self.opposite)
        };
    }

    fn repr(&self) -> String {
        format!("MapqFilter(name={}, min_mapq={}, max_mapq={}, opposite={})", self.name, self.min_mapq, self.max_mapq, self.opposite)
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> Filtering for RefNameFilter<'a> {
    fn apply_to(&self, record: &Record) -> bool {
        let this_ref_id = record.ref_id();
        return utils::_opposite(this_ref_id == self.ref_id, self.opposite);
    }

    fn repr(&self) -> String {
        format!("RefNameFilter(name={}, ref_id={}, opposite={})", self.name, self.ref_id, self.opposite)
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> Filtering for NthNucleotideFilter<'a> {
    fn apply_to(&self, record: &Record) -> bool {
        let seq = record.sequence();
        if !seq.available() {
            return utils::_opposite(false, self.opposite);
        }

        let len = seq.len() as i64;
        if self.position.abs() >= len {
            return utils::_opposite(false, self.opposite);
        }
        //todo: check if read was aligned to reverse strand?

        return if let this_nuc = seq.at(self.position as usize) as char {
            return if self.n_is_wildcard && (this_nuc == 'N') {
                utils::_opposite(true, self.opposite)
            } else {
                utils::_opposite(this_nuc == self.nucleotide, self.opposite)
            };
        } else {
            utils::_opposite(false, self.opposite)
        };
    }

    fn repr(&self) -> String {
        format!("NthNucleotideFilter(name={}, position={}, nucleotide={:?}, opposite={})", self.name, self.position, self.nucleotide, self.opposite)
    }
    fn name(&self) -> &str {
        &self.name
    }
}





