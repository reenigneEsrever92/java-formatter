//! BLANK_LINES_BEFORE_METHOD_BODY — min blank lines at the start of a method /
//! constructor body.
//! Fixtures live under tests/java/blank_lines_before_method_body/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const BODIES: &str = include_str!("../java/blank_lines_before_method_body/bodies.java");
const BODIES_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_before_method_body/bodies_default.out.java");
const BODIES_3_OUT: &str = include_str!("../java/blank_lines_before_method_body/bodies_3.out.java");

fn before_method_body(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_before_method_body = min)
}

#[test]
fn default_minimum_zero_starts_bodies_on_the_next_line() {
    // The absent-option default is the IntelliJ built-in minimum of 0.
    assert_eq!(format(BODIES), BODIES_DEFAULT_OUT);
}

#[test]
fn minimum_three_inserts_three_blank_lines_after_the_opening_brace() {
    assert_eq!(format_with(BODIES, &before_method_body(3)), BODIES_3_OUT);
}
