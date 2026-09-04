//! BLANK_LINES_AROUND_FIELD_WITH_ANNOTATIONS — min blank lines around fields
//! that carry an annotation.
//! Fixtures live under tests/java/blank_lines_around_field_with_annotations/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const ANN_FIELDS: &str =
    include_str!("../java/blank_lines_around_field_with_annotations/ann_fields.java");
const ANN_FIELDS_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_around_field_with_annotations/ann_fields_default.out.java");
const ANN_FIELDS_3_OUT: &str =
    include_str!("../java/blank_lines_around_field_with_annotations/ann_fields_3.out.java");

fn around_annotated(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_around_field_with_annotations = min)
}

#[test]
fn default_minimum_zero_keeps_annotated_fields_glued() {
    // The absent-option default is the IntelliJ built-in minimum of 0: the
    // annotated field behaves like a plain field.
    assert_eq!(format(ANN_FIELDS), ANN_FIELDS_DEFAULT_OUT);
}

#[test]
fn minimum_zero_behaviour_matches_the_default() {
    assert_eq!(
        format_with(ANN_FIELDS, &around_annotated(0)),
        ANN_FIELDS_DEFAULT_OUT
    );
}

#[test]
fn minimum_three_inserts_three_blank_lines_around_the_annotated_field() {
    assert_eq!(
        format_with(ANN_FIELDS, &around_annotated(3)),
        ANN_FIELDS_3_OUT
    );
}
