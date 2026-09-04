//! SPACE_WITHIN_METHOD_PARENTHESES — padding inside method / constructor
//! declaration parameter lists.
//! Fixtures live under tests/java/space_within_method_parentheses/.

use super::common::*;

const METHOD: &str = include_str!("../java/space_within_method_parentheses/method.java");
const METHOD_OUT: &str = include_str!("../java/space_within_method_parentheses/method.out.java");
const METHOD_DEFAULT_OUT: &str =
    include_str!("../java/space_within_method_parentheses/method_default.out.java");

#[test]
fn pads_declaration_parentheses_when_on() {
    let s = style(|st| st.space_within_method_parentheses = true);
    assert_eq!(format_with(METHOD, &s), METHOD_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(METHOD), METHOD_DEFAULT_OUT);
}
