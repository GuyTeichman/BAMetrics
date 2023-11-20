extern crate bam;
extern crate clap;
extern crate strum_macros;

use strum_macros::{Display, EnumString};

use bam::record::tags::{IntegerType, StringType, TagName, TagValue};
use clap::ValueEnum;
use serde::{
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};

pub fn str_to_tag_name(s: &str) -> TagName {
    if s.len() != 2 {
        panic!("Tag name must be 2 characters long");
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    let second = chars.next().unwrap();
    let tag_name = [first as u8, second as u8];
    return tag_name;
}
#[derive(Serialize, Deserialize, Clone, Display)]
pub enum MinimalTagValue {
    Char(u8),
    Int(i64),
    Float(f32),
    String(String),
}

#[derive(clap::ValueEnum, Clone, Display)]
pub enum CliTagType {
    Char,
    Int,
    Float,
    String,
}
#[derive(Debug, PartialEq, Clone, ValueEnum, EnumString)]
pub enum SupportedFormats {
    BAM,
    SAM,
}

#[derive(Debug, PartialEq, Clone, ValueEnum, EnumString, Serialize, Deserialize)]
pub enum BoolOperator {
    AND,
    OR,
    XOR,
    XNOR,
    NAND,
    NOR,
    IMPLIES,
}

pub fn _opposite(boolean: bool, opposite: bool) -> bool {
    return if opposite { !boolean } else { boolean };
}

pub fn convert_to_minimal_tag_value(
    cli_value: CliTagType,
    data: &str,
) -> Result<MinimalTagValue, String> {
    match cli_value {
        CliTagType::Char => data
            .parse::<u8>()
            .map(MinimalTagValue::Char)
            .map_err(|_| "Invalid data for Char".to_string()),
        CliTagType::Int => data
            .parse::<i64>()
            .map(MinimalTagValue::Int)
            .map_err(|_| "Invalid data for Int".to_string()),
        CliTagType::Float => data
            .parse::<f32>()
            .map(MinimalTagValue::Float)
            .map_err(|_| "Invalid data for Float".to_string()),
        CliTagType::String => Ok(MinimalTagValue::String(data.to_string())),
    }
}
pub fn _minimal_tag_to_tag(tag: &MinimalTagValue) -> TagValue {
    match tag {
        MinimalTagValue::Char(c) => TagValue::Char(*c),
        MinimalTagValue::Int(i) => {
            let int_type = if *i < 0 {
                IntegerType::I32
            } else {
                IntegerType::U32
            };
            TagValue::Int(*i, IntegerType::I32)
        }
        MinimalTagValue::Float(f) => TagValue::Float(*f),
        MinimalTagValue::String(s) => TagValue::String(s.as_bytes(), StringType::String),
    }
}
pub fn _are_tag_values_equal(a: &TagValue, b: &TagValue) -> bool {
    match (a, b) {
        (TagValue::Char(a), TagValue::Char(b)) => a == b,
        (TagValue::Int(a, a_type), TagValue::Int(b, b_type)) => a == b && a_type == b_type,
        (TagValue::Float(a), TagValue::Float(b)) => a == b,
        (TagValue::String(a, a_type), TagValue::String(b, b_type)) => a == b && a_type == b_type,
        (TagValue::IntArray(a), TagValue::IntArray(b)) => a.raw() == b.raw(),
        (TagValue::FloatArray(a), TagValue::FloatArray(b)) => a.raw() == b.raw(),
        _ => false, // Handle all other cases as not equal
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(true, false, true)]
    #[case(true, true, false)]
    #[case(false, false, false)]
    #[case(false, true, true)]
    #[test]
    fn test_opposite(#[case] boolean: bool, #[case] opposite: bool, #[case] expected: bool) {
        let result = _opposite(boolean, opposite);
        assert_eq!(result, expected);
    }
}
