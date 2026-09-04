//! BLANK_LINES_BEFORE_CLASS_END — min blank lines before a class's closing brace.
//! Fixtures live under tests/java/blank_lines_before_class_end/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const END: &str = include_str!("../java/blank_lines_before_class_end/end.java");
const END_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_before_class_end/end_default.out.java");
const END_2_OUT: &str = include_str!("../java/blank_lines_before_class_end/end_2.out.java");

fn before_class_end(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_before_class_end = min)
}

#[test]
fn default_minimum_zero_glues_the_closing_brace_to_the_last_member() {
    // The absent-option default is the IntelliJ built-in minimum of 0.
    assert_eq!(format(END), END_DEFAULT_OUT);
}

#[test]
fn minimum_two_inserts_two_blank_lines_before_the_class_end() {
    assert_eq!(format_with(END, &before_class_end(2)), END_2_OUT);
}
