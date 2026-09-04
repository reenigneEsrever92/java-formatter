//! BLANK_LINES_AROUND_CLASS — min blank lines around class / interface declarations
//! (both top-level types and nested class members).
//! Fixtures live under tests/java/blank_lines_around_class/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const CLASSES: &str = include_str!("../java/blank_lines_around_class/classes.java");
const CLASSES_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_around_class/classes_default.out.java");
const CLASSES_0_OUT: &str = include_str!("../java/blank_lines_around_class/classes_0.out.java");
const CLASSES_3_OUT: &str = include_str!("../java/blank_lines_around_class/classes_3.out.java");

fn around_class(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_around_class = min)
}

#[test]
fn default_minimum_one_inserts_one_blank_line_around_classes() {
    // The absent-option default is the IntelliJ built-in minimum of 1.
    assert_eq!(format(CLASSES), CLASSES_DEFAULT_OUT);
}

#[test]
fn minimum_zero_glues_nested_and_top_level_classes() {
    assert_eq!(format_with(CLASSES, &around_class(0)), CLASSES_0_OUT);
}

#[test]
fn minimum_three_inserts_three_blank_lines_around_classes() {
    assert_eq!(format_with(CLASSES, &around_class(3)), CLASSES_3_OUT);
}
