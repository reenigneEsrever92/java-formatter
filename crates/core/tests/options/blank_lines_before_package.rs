//! BLANK_LINES_BEFORE_PACKAGE — min blank lines before the package declaration.
//! Fixtures live under tests/java/blank_lines_before_package/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const BEFORE_PACKAGE: &str = include_str!("../java/blank_lines_before_package/before_package.java");
const BEFORE_PACKAGE_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_before_package/before_package_default.out.java");
const BEFORE_PACKAGE_2_OUT: &str =
    include_str!("../java/blank_lines_before_package/before_package_2.out.java");

fn before_package(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_before_package = min)
}

#[test]
fn default_minimum_zero_glues_the_package_to_the_file_header() {
    // The absent-option default is 0 blank lines before the package.
    assert_eq!(format(BEFORE_PACKAGE), BEFORE_PACKAGE_DEFAULT_OUT);
}

#[test]
fn minimum_two_inserts_two_blank_lines_before_the_package() {
    assert_eq!(
        format_with(BEFORE_PACKAGE, &before_package(2)),
        BEFORE_PACKAGE_2_OUT
    );
}
