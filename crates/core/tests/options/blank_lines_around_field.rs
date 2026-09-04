//! BLANK_LINES_AROUND_FIELD — min blank lines around fields.
//! Fixtures live under tests/java/blank_lines_around_field/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const FIELDS: &str = include_str!("../java/blank_lines_around_field/fields.java");
const FIELDS_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_around_field/fields_default.out.java");
const FIELDS_3_OUT: &str = include_str!("../java/blank_lines_around_field/fields_3.out.java");

fn around_field(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_around_field = min)
}

#[test]
fn default_minimum_zero_keeps_fields_glued_and_only_separates_from_methods() {
    // The absent-option default is the IntelliJ built-in minimum of 0 for
    // plain fields; the gap to a method still takes the method's minimum.
    assert_eq!(format(FIELDS), FIELDS_DEFAULT_OUT);
}

#[test]
fn minimum_zero_behaviour_matches_the_default() {
    assert_eq!(format_with(FIELDS, &around_field(0)), FIELDS_DEFAULT_OUT);
}

#[test]
fn minimum_three_inserts_three_blank_lines_between_fields() {
    assert_eq!(format_with(FIELDS, &around_field(3)), FIELDS_3_OUT);
}
