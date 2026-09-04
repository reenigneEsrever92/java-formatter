//! KEEP_FIRST_COLUMN_COMMENT — keep comments that start in the first column of
//! the source at the first column (true) instead of indenting them with the
//! surrounding code (false). The two `*_AT_FIRST_COLUMN` toggles are pinned
//! off here so the keep behaviour is observable in isolation.
//!
//! Fixtures live under tests/java/keep_first_column_comment/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const FIRST_COLUMN: &str = include_str!("../java/keep_first_column_comment/first_column.java");
const FIRST_COLUMN_KEEP_OUT: &str =
    include_str!("../java/keep_first_column_comment/first_column_keep.out.java");
const FIRST_COLUMN_INDENT_OUT: &str =
    include_str!("../java/keep_first_column_comment/first_column_indent.out.java");
const FIRST_COLUMN_DEFAULT_OUT: &str =
    include_str!("../java/keep_first_column_comment/first_column_default.out.java");

/// The keep flag off/on styles pin the `*_AT_FIRST_COLUMN` toggles off so only
/// KEEP_FIRST_COLUMN_COMMENT decides whether a first-column comment moves.
fn keep(keep: bool) -> JavaStyle {
    style(|s| {
        s.line_comment_at_first_column = false;
        s.block_comment_at_first_column = false;
        s.keep_first_column_comment = keep;
    })
}

#[test]
fn keep_true_leaves_first_column_comments_in_column_1() {
    assert_eq!(
        format_with(FIRST_COLUMN, &keep(true)),
        FIRST_COLUMN_KEEP_OUT
    );
}

#[test]
fn keep_false_indents_first_column_comments_with_the_code() {
    assert_eq!(
        format_with(FIRST_COLUMN, &keep(false)),
        FIRST_COLUMN_INDENT_OUT
    );
}

#[test]
fn absent_option_uses_the_built_in_true_default() {
    // The IntelliJ built-in default is true (and the `*_AT_FIRST_COLUMN`
    // defaults also pin comments to column 1), so the pristine default style
    // keeps both first-column comments in place.
    assert_eq!(format(FIRST_COLUMN), FIRST_COLUMN_DEFAULT_OUT);
}
