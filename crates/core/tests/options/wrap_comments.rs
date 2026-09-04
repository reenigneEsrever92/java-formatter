//! WRAP_COMMENTS — wrap single-line comments longer than the right margin at
//! word boundaries (true). Continuation lines repeat the comment's column
//! prefix: `//` for line comments, aligned ` * ` text for block comments.
//!
//! Fixtures live under tests/java/wrap_comments/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const LONG_LINE: &str = include_str!("../java/wrap_comments/long_line.java");
const LONG_LINE_WRAPPED_OUT: &str =
    include_str!("../java/wrap_comments/long_line_wrapped.out.java");
const LONG_LINE_UNWRAPPED_OUT: &str =
    include_str!("../java/wrap_comments/long_line_unwrapped.out.java");
const LONG_LINE_DEFAULT_OUT: &str =
    include_str!("../java/wrap_comments/long_line_default.out.java");
const LONG_BLOCK: &str = include_str!("../java/wrap_comments/long_block.java");
const LONG_BLOCK_WRAPPED_OUT: &str =
    include_str!("../java/wrap_comments/long_block_wrapped.out.java");
const LONG_BLOCK_UNWRAPPED_OUT: &str =
    include_str!("../java/wrap_comments/long_block_unwrapped.out.java");
const LONG_BLOCK_DEFAULT_OUT: &str =
    include_str!("../java/wrap_comments/long_block_default.out.java");

/// A tight right margin so the long comment overflows; the two
/// `*_AT_FIRST_COLUMN` toggles are pinned off so the comment stays at the code
/// indent and only the wrap toggle is observable.
fn wrap(wrap_comments: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.wrap_comments = wrap_comments;
        s.line_comment_at_first_column = false;
        s.block_comment_at_first_column = false;
    })
}

#[test]
fn wrap_true_breaks_an_over_margin_line_comment_at_word_boundaries() {
    assert_eq!(format_with(LONG_LINE, &wrap(true)), LONG_LINE_WRAPPED_OUT);
}

#[test]
fn wrap_false_leaves_the_long_line_comment_intact() {
    assert_eq!(
        format_with(LONG_LINE, &wrap(false)),
        LONG_LINE_UNWRAPPED_OUT
    );
}

#[test]
fn wrap_true_breaks_an_over_margin_block_comment_into_aligned_lines() {
    assert_eq!(format_with(LONG_BLOCK, &wrap(true)), LONG_BLOCK_WRAPPED_OUT);
}

#[test]
fn wrap_false_leaves_the_long_block_comment_intact() {
    assert_eq!(
        format_with(LONG_BLOCK, &wrap(false)),
        LONG_BLOCK_UNWRAPPED_OUT
    );
}

#[test]
fn absent_option_uses_the_built_in_false_default() {
    // The built-in default is false (and the margin is the built-in 120, which
    // the fixture fits), so neither comment is wrapped. (The pristine default
    // style additionally pins the comments to column 1 via the
    // `*_AT_FIRST_COLUMN` built-in defaults.)
    assert_eq!(format(LONG_LINE), LONG_LINE_DEFAULT_OUT);
    assert_eq!(format(LONG_BLOCK), LONG_BLOCK_DEFAULT_OUT);
}
