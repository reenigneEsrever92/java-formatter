//! BLANK_LINES_AROUND_INITIALIZER — min blank lines around instance / static
//! initializer blocks.
//! Fixtures live under tests/java/blank_lines_around_initializer/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const INITS: &str = include_str!("../java/blank_lines_around_initializer/inits.java");
const INITS_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_around_initializer/inits_default.out.java");
const INITS_0_OUT: &str = include_str!("../java/blank_lines_around_initializer/inits_0.out.java");
const INITS_3_OUT: &str = include_str!("../java/blank_lines_around_initializer/inits_3.out.java");

fn around_initializer(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_around_initializer = min)
}

#[test]
fn default_minimum_one_separates_initializers_with_one_blank_line() {
    // The absent-option default is the IntelliJ built-in minimum of 1.
    assert_eq!(format(INITS), INITS_DEFAULT_OUT);
}

#[test]
fn minimum_zero_glues_initializers_to_adjacent_members() {
    assert_eq!(format_with(INITS, &around_initializer(0)), INITS_0_OUT);
}

#[test]
fn minimum_three_inserts_three_blank_lines_around_initializers() {
    assert_eq!(format_with(INITS, &around_initializer(3)), INITS_3_OUT);
}
