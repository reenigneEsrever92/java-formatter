//! SPACE_WITHIN_EMPTY_METHOD_CALL_PARENTHESES — `f( )` vs `f()`.
//! Fixtures live under tests/java/space_within_empty_method_call_parentheses/.

use super::common::*;

const EMPTY_CALL: &str =
    include_str!("../java/space_within_empty_method_call_parentheses/empty_call.java");
const EMPTY_CALL_OUT: &str =
    include_str!("../java/space_within_empty_method_call_parentheses/empty_call.out.java");
const EMPTY_CALL_DEFAULT_OUT: &str =
    include_str!("../java/space_within_empty_method_call_parentheses/empty_call_default.out.java");

#[test]
fn pads_empty_call_parentheses_when_on() {
    let s = style(|st| st.space_within_empty_method_call_parentheses = true);
    assert_eq!(format_with(EMPTY_CALL, &s), EMPTY_CALL_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(EMPTY_CALL), EMPTY_CALL_DEFAULT_OUT);
}
