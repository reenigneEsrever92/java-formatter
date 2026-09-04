//! BLANK_LINES_AFTER_PACKAGE — min blank lines after the package declaration.
//! Fixtures live under tests/java/blank_lines_after_package/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const AFTER_PACKAGE: &str = include_str!("../java/blank_lines_after_package/after_package.java");
const AFTER_PACKAGE_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_after_package/after_package_default.out.java");
const AFTER_PACKAGE_0_OUT: &str =
    include_str!("../java/blank_lines_after_package/after_package_0.out.java");
const AFTER_PACKAGE_3_OUT: &str =
    include_str!("../java/blank_lines_after_package/after_package_3.out.java");

fn after_package(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_after_package = min)
}

#[test]
fn default_minimum_one_inserts_one_blank_line_after_the_package() {
    // The absent-option default is the IntelliJ built-in minimum of 1.
    assert_eq!(format(AFTER_PACKAGE), AFTER_PACKAGE_DEFAULT_OUT);
}

#[test]
fn minimum_zero_keeps_the_type_glued_to_the_package() {
    assert_eq!(
        format_with(AFTER_PACKAGE, &after_package(0)),
        AFTER_PACKAGE_0_OUT
    );
}

#[test]
fn minimum_three_inserts_three_blank_lines_after_the_package() {
    assert_eq!(
        format_with(AFTER_PACKAGE, &after_package(3)),
        AFTER_PACKAGE_3_OUT
    );
}
