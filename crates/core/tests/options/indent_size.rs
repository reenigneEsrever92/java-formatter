//! INDENT_SIZE — indentation width per nesting level in spaces.
//! Fixtures live under tests/java/indent_size/.

use super::common::*;

const INDENT: &str = include_str!("../java/indent_size/indent.java");
const INDENT_TWO_OUT: &str = include_str!("../java/indent_size/indent.out.java");
const INDENT_FOUR_OUT: &str = include_str!("../java/indent_size/indent_four.out.java");

#[test]
fn indent_size_two_spaces_indents_each_level_by_two() {
    let style = style(|s| s.indent_size = 2);
    assert_eq!(format_with(INDENT, &style), INDENT_TWO_OUT);
}

#[test]
fn default_indent_size_four_stays_four_spaces_per_level() {
    assert_eq!(format(INDENT), INDENT_FOUR_OUT);
}
