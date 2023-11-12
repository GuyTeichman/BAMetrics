use bam::record::tags::TagValue;

#[derive(Debug, PartialEq)]
pub enum BoolOperator {
    AND,
    OR,
    XOR,
    XNOR,
    NAND,
    NOR,
    IMPLIES
}

pub fn _opposite(boolean: bool, opposite: bool) -> bool {
    return if opposite {
        !boolean
    } else {
        boolean
    };
}

pub fn _are_tag_values_equal(a: &TagValue, b: &TagValue) -> bool {
    match (a, b) {
        (TagValue::Char(a), TagValue::Char(b)) => a == b,
        (TagValue::Int(a, a_type), TagValue::Int(b, b_type)) => {
            a == b && a_type == b_type
        }
        (TagValue::Float(a), TagValue::Float(b)) => a == b,
        (TagValue::String(a, a_type), TagValue::String(b, b_type)) => {
            a == b && a_type == b_type
        }
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