//! BLOCK_COMMENT_AT_FIRST_COLUMN — whether `/* */` block comments are placed
//! at the first column (true, no indent) or at the surrounding code's indent
//! (false).
//!
//! Fixtures live under tests/java/block_comment_at_first_column/.

use super::common::*;

const INDENTED: &str = include_str!("../java/block_comment_at_first_column/indented.java");
const INDENTED_TRUE_OUT: &str =
    include_str!("../java/block_comment_at_first_column/indented_true.out.java");
const INDENTED_FALSE_OUT: &str =
    include_str!("../java/block_comment_at_first_column/indented_false.out.java");

#[test]
fn first_column_pins_an_indented_block_comment_to_column_1() {
    let style = style(|s| s.block_comment_at_first_column = true);
    assert_eq!(format_with(INDENTED, &style), INDENTED_TRUE_OUT);
}

#[test]
fn absent_option_uses_the_built_in_true_default() {
    // The IntelliJ built-in default is true, so the pristine default style
    // places the indented comment at column 1 too.
    assert_eq!(format(INDENTED), INDENTED_TRUE_OUT);
}

#[test]
fn not_first_column_keeps_the_comment_at_the_code_indent() {
    let style = style(|s| s.block_comment_at_first_column = false);
    assert_eq!(format_with(INDENTED, &style), INDENTED_FALSE_OUT);
}
