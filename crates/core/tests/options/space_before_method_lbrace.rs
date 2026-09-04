//! SPACE_BEFORE_METHOD_LBRACE — space before the opening brace of method and constructor bodies. Defaults to on.
//! Fixtures live under tests/java/space_before_method_lbrace/.

use super::common::*;

const METHODS: &str = include_str!("../java/space_before_method_lbrace/methods.java");
const METHODS_OUT: &str = include_str!("../java/space_before_method_lbrace/methods.out.java");
const METHODS_DEFAULT_OUT: &str =
    include_str!("../java/space_before_method_lbrace/methods_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_method_lbrace = false);
    assert_eq!(format_with(METHODS, &s), METHODS_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(METHODS), METHODS_DEFAULT_OUT);
}
