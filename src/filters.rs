extern crate bam;

use bam::Record;
use bam::record::tags::{TagName, TagValue};
use crate::utils;
use utils::BoolOperator;

pub trait Filtering {
    fn apply_to(&self, record: &Record) -> bool;

    fn repr(&self) -> String;
}


struct CombinedFilter {
    filter1: Box<dyn Filtering>,
    filter2: Box<dyn Filtering>,
    operator: BoolOperator,
}

struct LengthFilter {
    min_len: u32,
    max_len: u32,
    opposite: bool,
}

struct TagFilter<'a> {
    tag_name: TagName,
    tag_value: TagValue<'a>,
    opposite: bool,
}

struct MapqFilter {
    min_mapq: u8,
    max_mapq: u8,
    opposite: bool,
}

struct RefNameFilter {
    ref_id: i32,
    opposite: bool,
}

struct NthNucleotideFilter {
    position: i64,
    nucleotide: char,
    n_is_wildcard: bool,
    opposite: bool,
}

struct FlagFilter {
    remove_flags: u16,
    opposite: bool,
}

impl CombinedFilter {
    pub fn new(filter1: Box<dyn Filtering>, filter2: Box<dyn Filtering>, operator: BoolOperator) -> CombinedFilter {
        CombinedFilter {
            filter1,
            filter2,
            operator,
        }
    }
}

impl LengthFilter {
    pub fn new(min_len: u32, max_len: u32, opposite: bool) -> LengthFilter {
        LengthFilter {
            min_len,
            max_len,
            opposite,
        }
    }
}

impl TagFilter<'_> {
    pub fn new(tag_name: TagName, tag_value: TagValue, opposite: bool) -> TagFilter {
        TagFilter {
            tag_name,
            tag_value,
            opposite,
        }
    }
}

impl MapqFilter {
    pub fn new(min_mapq: u8, max_mapq: u8, opposite: bool) -> MapqFilter {
        MapqFilter {
            min_mapq,
            max_mapq,
            opposite,
        }
    }
}

impl RefNameFilter {
    pub fn new(ref_id: i32, opposite: bool) -> RefNameFilter {
        RefNameFilter { ref_id, opposite }
    }
}

impl NthNucleotideFilter {
    pub fn new(position: i64, nucleotide: char, n_is_wildcard: bool, opposite: bool) -> NthNucleotideFilter {
        assert!(matches!(nucleotide, 'A' | 'C' | 'G' | 'T' | 'N'), "Nucleotide must be one of A, C, G, T, or N!");
        NthNucleotideFilter {
            position,
            nucleotide,
            n_is_wildcard,
            opposite,
        }
    }
}

impl FlagFilter {
    pub fn new(remove_flags: u16, opposite: bool) -> FlagFilter {
        FlagFilter {
            remove_flags,
            opposite,
        }
    }
}

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
        format!("CombinedFilter(filter1={}, filter2={}, operator={:?})", self.filter1.repr(), self.filter2.repr(), self.operator)
    }
}
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
        format!("FlagFilter(remove_flags={}, opposite={})", self.remove_flags, self.opposite)
    }
}


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
        format!("LengthFilter(min_len={}, max_len={}, opposite={})", self.min_len, self.max_len, self.opposite)
    }
}

impl Filtering for TagFilter<'_> {
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
        format!("TagFilter(tag_name={:?}, tag_value={:?}, opposite={})", self.tag_name, self.tag_value, self.opposite)
    }
}

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
        format!("MapqFilter(min_mapq={}, max_mapq={}, opposite={})", self.min_mapq, self.max_mapq, self.opposite)
    }
}

impl Filtering for RefNameFilter {
    fn apply_to(&self, record: &Record) -> bool {
        let this_ref_id = record.ref_id();
        return utils::_opposite(this_ref_id == self.ref_id, self.opposite);
    }

    fn repr(&self) -> String {
        format!("RefNameFilter(ref_id={}, opposite={})", self.ref_id, self.opposite)
    }
}

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
        format!("NthNucleotideFilter(position={}, nucleotide={:?}, opposite={})", self.position, self.nucleotide, self.opposite)
    }
}





