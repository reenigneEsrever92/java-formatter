//! BLANK_LINES_AROUND_METHOD_IN_INTERFACE — min blank lines around methods
//! declared in interfaces.
//! Fixtures live under tests/java/blank_lines_around_method_in_interface/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const METHODS: &str = include_str!("../java/blank_lines_around_method_in_interface/methods.java");
const METHODS_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_around_method_in_interface/methods_default.out.java");
const METHODS_0_OUT: &str =
    include_str!("../java/blank_lines_around_method_in_interface/methods_0.out.java");
const METHODS_3_OUT: &str =
    include_str!("../java/blank_lines_around_method_in_interface/methods_3.out.java");

fn around_method(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_around_method_in_interface = min)
}

#[test]
fn default_minimum_one_separates_interface_methods_with_one_blank_line() {
    // The absent-option default is the IntelliJ built-in minimum of 1.
    assert_eq!(format(METHODS), METHODS_DEFAULT_OUT);
}

#[test]
fn minimum_zero_glues_interface_methods() {
    assert_eq!(format_with(METHODS, &around_method(0)), METHODS_0_OUT);
}

#[test]
fn minimum_three_inserts_three_blank_lines_between_interface_methods() {
    assert_eq!(format_with(METHODS, &around_method(3)), METHODS_3_OUT);
}
