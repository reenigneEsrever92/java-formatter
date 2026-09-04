//! SPACE_BEFORE_CLASS_LBRACE — space before the opening brace of class / interface / enum / record and anonymous-class bodies. Defaults to on.
//! Fixtures live under tests/java/space_before_class_lbrace/.

use super::common::*;

const TYPES: &str = include_str!("../java/space_before_class_lbrace/types.java");
const TYPES_OUT: &str = include_str!("../java/space_before_class_lbrace/types.out.java");
const TYPES_DEFAULT_OUT: &str =
    include_str!("../java/space_before_class_lbrace/types_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_class_lbrace = false);
    assert_eq!(format_with(TYPES, &s), TYPES_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(TYPES), TYPES_DEFAULT_OUT);
}
