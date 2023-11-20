extern crate bam;
extern crate serde;
extern crate typetag;

use bam::record::tags::TagName;
use bam::Record;
use serde::{Deserialize, Serialize};

use utils::BoolOperator;

use crate::utils;
use std::collections::HashMap;

#[typetag::serde(tag = "type")]
pub trait Filtering: CloneFilter {
    fn apply_to(&self, record: &Record) -> bool;

    fn repr(&self) -> String;

    fn name(&self) -> &str;
}

pub trait CloneFilter {
    fn box_clone<'a>(&self) -> Box<dyn Filtering>;
}

impl<T> CloneFilter for T
    where
        T: Filtering + Clone + 'static,
{
    fn box_clone(&self) -> Box<dyn Filtering> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Filtering> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    filters: HashMap<String, Box<dyn Filtering>>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            filters: HashMap::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.filters.len()
    }
    pub fn push(&mut self, key: &str, value: Box<dyn Filtering>) {
        self.filters.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<Box<dyn Filtering>> {
        self.filters.get(&key.to_string()).cloned()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<String, Box<dyn Filtering>> {
        self.filters.iter()
    }
}

// TODO: uniquely aligned
// TODO: individual flag functions
#[derive(Serialize, Deserialize, Clone)]
pub struct CombinedFilter {
    name: String,
    filter1: Box<dyn Filtering>,
    filter2: Box<dyn Filtering>,
    operator: BoolOperator,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LengthFilter {
    name: String,
    min_len: u32,
    max_len: u32,
    opposite: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TagFilter {
    name: String,
    tag_name: TagName,
    tag_value: utils::MinimalTagValue,
    opposite: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MapqFilter {
    name: String,
    min_mapq: u8,
    max_mapq: u8,
    opposite: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RefNameFilter {
    name: String,
    ref_id: i32,
    opposite: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NthNucleotideFilter {
    name: String,
    position: i64,
    nucleotide: char,
    n_is_wildcard: bool,
    opposite: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FlagFilter {
    name: String,
    remove_flags: u16,
    opposite: bool,
}

impl CombinedFilter {
    pub fn new(
        name: String,
        filter1: Box<dyn Filtering>,
        filter2: Box<dyn Filtering>,
        operator: BoolOperator,
    ) -> CombinedFilter {
        CombinedFilter {
            name,
            filter1,
            filter2,
            operator,
        }
    }
}

impl LengthFilter {
    pub fn new(name: String, min_len: u32, max_len: u32, opposite: bool) -> LengthFilter {
        LengthFilter {
            name,
            min_len,
            max_len,
            opposite,
        }
    }
}

impl TagFilter {
    pub fn new(
        name: String,
        tag_name: TagName,
        tag_value: utils::MinimalTagValue,
        opposite: bool,
    ) -> TagFilter {
        TagFilter {
            name,
            tag_name,
            tag_value,
            opposite,
        }
    }
}

impl MapqFilter {
    pub fn new(name: String, min_mapq: u8, max_mapq: u8, opposite: bool) -> MapqFilter {
        MapqFilter {
            name,
            min_mapq,
            max_mapq,
            opposite,
        }
    }
}

impl RefNameFilter {
    pub fn new(name: String, ref_id: i32, opposite: bool) -> RefNameFilter {
        RefNameFilter {
            name,
            ref_id,
            opposite,
        }
    }
}

impl NthNucleotideFilter {
    pub fn new(
        name: String,
        position: i64,
        nucleotide: char,
        n_is_wildcard: bool,
        opposite: bool,
    ) -> NthNucleotideFilter {
        assert!(
            matches!(nucleotide, 'A' | 'C' | 'G' | 'T' | 'N'),
            "Nucleotide must be one of A, C, G, T, or N!"
        );
        NthNucleotideFilter {
            name,
            position,
            nucleotide,
            n_is_wildcard,
            opposite,
        }
    }
}

impl FlagFilter {
    pub fn new(name: String, remove_flags: u16, opposite: bool) -> FlagFilter {
        FlagFilter {
            name,
            remove_flags,
            opposite,
        }
    }
}

#[typetag::serde]
impl Filtering for CombinedFilter {
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
        format!(
            "CombinedFilter(name={}, filter1={}, filter2={}, operator={:?})",
            self.name,
            self.filter1.name(),
            self.filter2.name(),
            self.operator
        )
    }
    fn name(&self) -> &str {
        &self.name
    }
}

#[typetag::serde]
impl Filtering for FlagFilter {
    fn apply_to(&self, record: &Record) -> bool {
        let flags = record.flag();
        return if flags.no_bits(self.remove_flags) {
            utils::_opposite(true, self.opposite)
        } else {
            utils::_opposite(false, self.opposite)
        };
    }

    fn repr(&self) -> String {
        format!(
            "FlagFilter(name={}, remove_flags={}, opposite={})",
            self.name, self.remove_flags, self.opposite
        )
    }
    fn name(&self) -> &str {
        &self.name
    }
}

#[typetag::serde]
impl Filtering for LengthFilter {
    fn apply_to(&self, record: &Record) -> bool {
        let read_len = record.query_len();
        return if read_len < self.min_len || read_len > self.max_len {
            utils::_opposite(false, self.opposite)
        } else {
            utils::_opposite(true, self.opposite)
        };
    }

    fn repr(&self) -> String {
        format!(
            "LengthFilter(name={}, min_len={}, max_len={}, opposite={})",
            self.name, self.min_len, self.max_len, self.opposite
        )
    }
    fn name(&self) -> &str {
        &self.name
    }
}

#[typetag::serde]
impl Filtering for TagFilter {
    fn apply_to(&self, record: &Record) -> bool {
        return if let Some(tag) = record.tags().get(&self.tag_name) {
            let expanded_tag_val = utils::_minimal_tag_to_tag(&self.tag_value);
            if utils::_are_tag_values_equal(&tag, &expanded_tag_val) {
                utils::_opposite(true, self.opposite)
            } else {
                utils::_opposite(false, self.opposite)
            }
        } else {
            utils::_opposite(false, self.opposite)
        };
    }

    fn repr(&self) -> String {
        format!(
            "TagFilter(name={}, tag_name={:#?}, tag_value={}, opposite={})",
            self.name, self.tag_name, self.tag_value, self.opposite
        )
    }
    fn name(&self) -> &str {
        &self.name
    }
}

#[typetag::serde]
impl Filtering for MapqFilter {
    fn apply_to(&self, record: &Record) -> bool {
        let mapq = record.mapq();
        return if mapq < self.min_mapq || mapq > self.max_mapq {
            utils::_opposite(false, self.opposite)
        } else {
            utils::_opposite(true, self.opposite)
        };
    }

    fn repr(&self) -> String {
        format!(
            "MapqFilter(name={}, min_mapq={}, max_mapq={}, opposite={})",
            self.name, self.min_mapq, self.max_mapq, self.opposite
        )
    }
    fn name(&self) -> &str {
        &self.name
    }
}

#[typetag::serde]
impl Filtering for RefNameFilter {
    fn apply_to(&self, record: &Record) -> bool {
        let this_ref_id = record.ref_id();
        return utils::_opposite(this_ref_id == self.ref_id, self.opposite);
    }

    fn repr(&self) -> String {
        format!(
            "RefNameFilter(name={}, ref_id={}, opposite={})",
            self.name, self.ref_id, self.opposite
        )
    }
    fn name(&self) -> &str {
        &self.name
    }
}

#[typetag::serde]
impl Filtering for NthNucleotideFilter {
    fn apply_to(&self, record: &Record) -> bool {
        let seq = record.sequence();
        if !seq.available() {
            return utils::_opposite(false, self.opposite);
        }

        let len = seq.len() as i64;
        if self.position.abs() >= len {
            return utils::_opposite(false, self.opposite);
        }
        // todo: does negative index work in rust?
        //todo: check if read was aligned to reverse strand?

        let this_nuc = seq.at(self.position as usize) as char;
        return if self.n_is_wildcard && (this_nuc == 'N') {
            utils::_opposite(true, self.opposite)
        } else {
            utils::_opposite(this_nuc == self.nucleotide, self.opposite)
        };
    }

    fn repr(&self) -> String {
        format!(
            "NthNucleotideFilter(name={}, position={}, nucleotide={:?}, opposite={})",
            self.name, self.position, self.nucleotide, self.opposite
        )
    }
    fn name(&self) -> &str {
        &self.name
    }
}
