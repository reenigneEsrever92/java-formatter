//! SPACE_WITHIN_METHOD_CALL_PARENTHESES — padding inside call, chain and
//! constructor-argument parentheses.
//! Fixtures live under tests/java/space_within_method_call_parentheses/.

use super::common::*;

const CALL: &str = include_str!("../java/space_within_method_call_parentheses/call.java");
const CALL_OUT: &str = include_str!("../java/space_within_method_call_parentheses/call.out.java");
const CALL_DEFAULT_OUT: &str =
    include_str!("../java/space_within_method_call_parentheses/call_default.out.java");

#[test]
fn pads_call_parentheses_when_on() {
    let s = style(|st| st.space_within_method_call_parentheses = true);
    assert_eq!(format_with(CALL, &s), CALL_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(CALL), CALL_DEFAULT_OUT);
}
