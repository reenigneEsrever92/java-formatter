//! LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION — insert the space after `//` inside
//! `//noinspection` suppression comments on reformat (true). Ordinary line
//! comments are governed by LINE_COMMENT_ADD_SPACE_ON_REFORMAT instead, so
//! they stay untouched here.
//!
//! Fixtures live under tests/java/line_comment_add_space_in_suppression/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const SUPPRESSION: &str =
    include_str!("../java/line_comment_add_space_in_suppression/suppression.java");
const SUPPRESSION_SPACE_OUT: &str =
    include_str!("../java/line_comment_add_space_in_suppression/suppression_space.out.java");
const SUPPRESSION_PLAIN_OUT: &str =
    include_str!("../java/line_comment_add_space_in_suppression/suppression_plain.out.java");
const SUPPRESSION_DEFAULT_OUT: &str =
    include_str!("../java/line_comment_add_space_in_suppression/suppression_default.out.java");

/// The two `*_AT_FIRST_COLUMN` toggles are pinned off so the comments stay at
/// the code indent and only the suppression toggle is observable.
fn suppress(on: bool) -> JavaStyle {
    style(|s| {
        s.line_comment_at_first_column = false;
        s.block_comment_at_first_column = false;
        s.line_comment_add_space_in_suppression = on;
    })
}

#[test]
fn in_suppression_adds_the_missing_space_inside_a_noinspection_comment() {
    // `//ordinary` is not a suppression comment; its space is governed by
    // LINE_COMMENT_ADD_SPACE_ON_REFORMAT, which is off here, so it stays
    // untouched.
    assert_eq!(
        format_with(SUPPRESSION, &suppress(true)),
        SUPPRESSION_SPACE_OUT
    );
}

#[test]
fn off_keeps_the_suppression_comment_as_written() {
    assert_eq!(
        format_with(SUPPRESSION, &suppress(false)),
        SUPPRESSION_PLAIN_OUT
    );
}

#[test]
fn absent_option_uses_the_built_in_false_default() {
    // The built-in default is false: no space is inserted inside the
    // suppression comment. (The pristine default style additionally pins the
    // comments to column 1 via the `*_AT_FIRST_COLUMN` built-in defaults.)
    assert_eq!(format(SUPPRESSION), SUPPRESSION_DEFAULT_OUT);
}
