//! BLANK_LINES_AFTER_CLASS_HEADER — min blank lines after the class header,
//! before the first member.
//! Fixtures live under tests/java/blank_lines_after_class_header/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const HEADER: &str = include_str!("../java/blank_lines_after_class_header/header.java");
const HEADER_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_after_class_header/header_default.out.java");
const HEADER_2_OUT: &str = include_str!("../java/blank_lines_after_class_header/header_2.out.java");

fn after_class_header(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_after_class_header = min)
}

#[test]
fn default_minimum_zero_starts_the_first_member_on_the_next_line() {
    // The absent-option default is the IntelliJ built-in minimum of 0.
    assert_eq!(format(HEADER), HEADER_DEFAULT_OUT);
}

#[test]
fn minimum_two_inserts_two_blank_lines_after_the_class_header() {
    assert_eq!(format_with(HEADER, &after_class_header(2)), HEADER_2_OUT);
}
