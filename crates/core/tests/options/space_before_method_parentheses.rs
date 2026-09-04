//! SPACE_BEFORE_METHOD_PARENTHESES — space between a method / constructor name and its parameter list. Defaults to off.
//! Fixtures live under tests/java/space_before_method_parentheses/.

use super::common::*;

const DECLS: &str = include_str!("../java/space_before_method_parentheses/decls.java");
const DECLS_OUT: &str = include_str!("../java/space_before_method_parentheses/decls.out.java");
const DECLS_DEFAULT_OUT: &str =
    include_str!("../java/space_before_method_parentheses/decls_default.out.java");

#[test]
fn on_spaces() {
    let s = style(|st| st.space_before_method_parentheses = true);
    assert_eq!(format_with(DECLS, &s), DECLS_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(DECLS), DECLS_DEFAULT_OUT);
}
