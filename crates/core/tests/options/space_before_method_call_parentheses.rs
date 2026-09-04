//! SPACE_BEFORE_METHOD_CALL_PARENTHESES — space between a method name and its call parentheses (calls, chains, constructors). Defaults to off.
//! Fixtures live under tests/java/space_before_method_call_parentheses/.

use super::common::*;

const CALLS: &str = include_str!("../java/space_before_method_call_parentheses/calls.java");
const CALLS_OUT: &str = include_str!("../java/space_before_method_call_parentheses/calls.out.java");
const CALLS_DEFAULT_OUT: &str =
    include_str!("../java/space_before_method_call_parentheses/calls_default.out.java");

#[test]
fn on_spaces() {
    let s = style(|st| st.space_before_method_call_parentheses = true);
    assert_eq!(format_with(CALLS, &s), CALLS_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(CALLS), CALLS_DEFAULT_OUT);
}
