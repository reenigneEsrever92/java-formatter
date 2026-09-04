//! SPACE_BEFORE_ANNOTATION_ARRAY_INITIALIZER_LBRACE — space between an annotation's `(` and a bare array-initializer argument. Defaults to off.
//! Fixtures live under tests/java/space_before_annotation_array_initializer_lbrace/.

use super::common::*;

const ANN_ARRAY: &str = include_str!("../java/space_before_annotation_array_initializer_lbrace/ann_array.java");
const ANN_ARRAY_OUT: &str = include_str!("../java/space_before_annotation_array_initializer_lbrace/ann_array.out.java");
const ANN_ARRAY_DEFAULT_OUT: &str =
    include_str!("../java/space_before_annotation_array_initializer_lbrace/ann_array_default.out.java");

#[test]
fn on_spaces() {
    let s = style(|st| st.space_before_annotation_array_initializer_lbrace = true);
    assert_eq!(format_with(ANN_ARRAY, &s), ANN_ARRAY_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(ANN_ARRAY), ANN_ARRAY_DEFAULT_OUT);
}
