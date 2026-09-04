//! BLANK_LINES_AFTER_IMPORTS — min blank lines after the import section.
//! Fixtures live under tests/java/blank_lines_after_imports/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const AFTER_IMPORTS: &str = include_str!("../java/blank_lines_after_imports/after_imports.java");
const AFTER_IMPORTS_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_after_imports/after_imports_default.out.java");
const AFTER_IMPORTS_0_OUT: &str =
    include_str!("../java/blank_lines_after_imports/after_imports_0.out.java");
const AFTER_IMPORTS_3_OUT: &str =
    include_str!("../java/blank_lines_after_imports/after_imports_3.out.java");

fn after_imports(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_after_imports = min)
}

#[test]
fn default_minimum_one_inserts_one_blank_line_after_the_imports() {
    // The absent-option default is the IntelliJ built-in minimum of 1.
    assert_eq!(format(AFTER_IMPORTS), AFTER_IMPORTS_DEFAULT_OUT);
}

#[test]
fn minimum_zero_keeps_the_type_glued_to_the_imports() {
    assert_eq!(
        format_with(AFTER_IMPORTS, &after_imports(0)),
        AFTER_IMPORTS_0_OUT
    );
}

#[test]
fn minimum_three_inserts_three_blank_lines_after_the_imports() {
    assert_eq!(
        format_with(AFTER_IMPORTS, &after_imports(3)),
        AFTER_IMPORTS_3_OUT
    );
}
