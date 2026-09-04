//! KEEP_BLANK_LINES_BETWEEN_PACKAGE_DECLARATION_AND_HEADER — max blank lines
//! kept between a file-header comment and the package declaration.
//! Fixtures live under tests/java/keep_blank_lines_between_package_declaration_and_header/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const HEADER: &str =
    include_str!("../java/keep_blank_lines_between_package_declaration_and_header/header.java");
const HEADER_DEFAULT_OUT: &str = include_str!(
    "../java/keep_blank_lines_between_package_declaration_and_header/header_default.out.java"
);
const HEADER_CAP0_OUT: &str = include_str!(
    "../java/keep_blank_lines_between_package_declaration_and_header/header_cap0.out.java"
);
const HEADER_CAP1_OUT: &str = include_str!(
    "../java/keep_blank_lines_between_package_declaration_and_header/header_cap1.out.java"
);
const HEADER_CAP3_OUT: &str = include_str!(
    "../java/keep_blank_lines_between_package_declaration_and_header/header_cap3.out.java"
);

fn keep(cap: u32) -> JavaStyle {
    style(|s| s.keep_blank_lines_between_package_declaration_and_header = cap)
}

#[test]
fn default_cap_two_keeps_two_of_three_blank_lines_after_the_header() {
    // The absent-option default is the IntelliJ built-in cap of 2.
    assert_eq!(format(HEADER), HEADER_DEFAULT_OUT);
}

#[test]
fn cap_zero_glues_the_package_to_the_header() {
    assert_eq!(format_with(HEADER, &keep(0)), HEADER_CAP0_OUT);
}

#[test]
fn cap_one_truncates_the_run_to_one_blank_line() {
    assert_eq!(format_with(HEADER, &keep(1)), HEADER_CAP1_OUT);
}

#[test]
fn cap_three_keeps_the_whole_three_blank_line_run() {
    assert_eq!(format_with(HEADER, &keep(3)), HEADER_CAP3_OUT);
}
