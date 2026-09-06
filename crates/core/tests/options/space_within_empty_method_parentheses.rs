//! SPACE_WITHIN_EMPTY_METHOD_PARENTHESES — `void f( )` vs `void f()`.
//! Fixtures live under tests/java/space_within_empty_method_parentheses/.

use super::common::*;

const EMPTY_METHOD: &str =
    include_str!("../java/space_within_empty_method_parentheses/empty_method.java");
const EMPTY_METHOD_OUT: &str =
    include_str!("../java/space_within_empty_method_parentheses/empty_method.out.java");
const EMPTY_METHOD_DEFAULT_OUT: &str =
    include_str!("../java/space_within_empty_method_parentheses/empty_method_default.out.java");

#[test]
fn pads_empty_declaration_parentheses_when_on() {
    let s = style(|st| st.space_within_empty_method_parentheses = true);
    assert_eq!(format_with(EMPTY_METHOD, &s), EMPTY_METHOD_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(EMPTY_METHOD), EMPTY_METHOD_DEFAULT_OUT);
}
