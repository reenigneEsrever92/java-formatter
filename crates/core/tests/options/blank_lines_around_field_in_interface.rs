//! BLANK_LINES_AROUND_FIELD_IN_INTERFACE — min blank lines around fields
//! declared in interfaces.
//! Fixtures live under tests/java/blank_lines_around_field_in_interface/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const FIELDS: &str = include_str!("../java/blank_lines_around_field_in_interface/fields.java");
const FIELDS_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_around_field_in_interface/fields_default.out.java");
const FIELDS_3_OUT: &str =
    include_str!("../java/blank_lines_around_field_in_interface/fields_3.out.java");

fn around_field(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_around_field_in_interface = min)
}

#[test]
fn default_minimum_zero_keeps_interface_fields_glued() {
    // The absent-option default is the IntelliJ built-in minimum of 0; the
    // interface method still takes its own minimum.
    assert_eq!(format(FIELDS), FIELDS_DEFAULT_OUT);
}

#[test]
fn minimum_zero_behaviour_matches_the_default() {
    assert_eq!(format_with(FIELDS, &around_field(0)), FIELDS_DEFAULT_OUT);
}

#[test]
fn minimum_three_inserts_three_blank_lines_between_interface_fields() {
    assert_eq!(format_with(FIELDS, &around_field(3)), FIELDS_3_OUT);
}
