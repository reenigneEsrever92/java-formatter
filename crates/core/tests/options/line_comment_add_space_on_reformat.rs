//! LINE_COMMENT_ADD_SPACE_ON_REFORMAT — insert the space after `//` of an
//! ordinary line comment on reformat when it is absent (true). A
//! `//noinspection` suppression comment is governed by
//! LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION instead, so it stays untouched here
//! even when the toggle is on.
//!
//! Fixtures live under tests/java/line_comment_add_space_on_reformat/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const COMMENTS: &str = include_str!("../java/line_comment_add_space_on_reformat/comments.java");
const COMMENTS_SPACE_OUT: &str =
    include_str!("../java/line_comment_add_space_on_reformat/comments_space.out.java");
const COMMENTS_PLAIN_OUT: &str =
    include_str!("../java/line_comment_add_space_on_reformat/comments_plain.out.java");
const COMMENTS_DEFAULT_OUT: &str =
    include_str!("../java/line_comment_add_space_on_reformat/comments_default.out.java");

/// The two `*_AT_FIRST_COLUMN` toggles are pinned off so the comments stay at
/// the code indent and only the space toggle is observable.
fn spaced(on: bool) -> JavaStyle {
    style(|s| {
        s.line_comment_at_first_column = false;
        s.block_comment_at_first_column = false;
        s.line_comment_add_space_on_reformat = on;
    })
}

#[test]
fn on_reformat_adds_the_missing_space_after_an_ordinary_slash_slash() {
    // `// spaced` already carries the space and `//noinspection …` is the
    // suppression form governed by its own option
    // (LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION), so only `//nospace` gains a
    // space.
    assert_eq!(format_with(COMMENTS, &spaced(true)), COMMENTS_SPACE_OUT);
}

#[test]
fn off_reformat_keeps_the_comments_as_written() {
    assert_eq!(format_with(COMMENTS, &spaced(false)), COMMENTS_PLAIN_OUT);
}

#[test]
fn absent_option_uses_the_built_in_false_default() {
    // The built-in default is false: no space is inserted. (The pristine
    // default style additionally pins the comments to column 1 via the
    // `*_AT_FIRST_COLUMN` built-in defaults.)
    assert_eq!(format(COMMENTS), COMMENTS_DEFAULT_OUT);
}
