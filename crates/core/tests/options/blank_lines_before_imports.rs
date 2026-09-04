//! BLANK_LINES_BEFORE_IMPORTS — min blank lines before the import section.
//! Fixtures live under tests/java/blank_lines_before_imports/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const BEFORE_IMPORTS: &str = include_str!("../java/blank_lines_before_imports/before_imports.java");
const BEFORE_IMPORTS_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_before_imports/before_imports_default.out.java");
const BEFORE_IMPORTS_0_OUT: &str =
    include_str!("../java/blank_lines_before_imports/before_imports_0.out.java");
const BEFORE_IMPORTS_3_OUT: &str =
    include_str!("../java/blank_lines_before_imports/before_imports_3.out.java");

fn before_imports(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_before_imports = min)
}

#[test]
fn default_minimum_one_inserts_one_blank_line_before_the_imports() {
    // The absent-option default is the IntelliJ built-in minimum of 1 (here
    // after the file-header comment, in a package-less file).
    assert_eq!(format(BEFORE_IMPORTS), BEFORE_IMPORTS_DEFAULT_OUT);
}

#[test]
fn minimum_zero_keeps_the_imports_glued_to_the_file_header() {
    assert_eq!(
        format_with(BEFORE_IMPORTS, &before_imports(0)),
        BEFORE_IMPORTS_0_OUT
    );
}

#[test]
fn minimum_three_inserts_three_blank_lines_before_the_imports() {
    assert_eq!(
        format_with(BEFORE_IMPORTS, &before_imports(3)),
        BEFORE_IMPORTS_3_OUT
    );
}
