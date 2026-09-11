//! Comments that trail code on the same source line — regression coverage for
//! the bug where every such comment was moved onto its own line.
//!
//! A `//` or single-line `/* … */` behind a statement, a member, a type or a
//! list element stays on that line. A trailing comment ignores the column
//! options (`*_AT_FIRST_COLUMN`, `KEEP_FIRST_COLUMN_COMMENT`), so the fixtures
//! pin them off to show the code indent; a list element's trailing comment sits
//! after the separator comma when it is a `//` (never swallowing the comma) and
//! a trailing `//` on the last element pushes the closing delimiter to its own
//! line.
//!
//! Fixtures live under tests/java/trailing_comments/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const STATEMENT: &str = include_str!("../java/trailing_comments/statement.java");
const STATEMENT_OUT: &str = include_str!("../java/trailing_comments/statement.out.java");
const MEMBER: &str = include_str!("../java/trailing_comments/member.java");
const MEMBER_OUT: &str = include_str!("../java/trailing_comments/member.out.java");
const LIST: &str = include_str!("../java/trailing_comments/list.java");
const LIST_OUT: &str = include_str!("../java/trailing_comments/list.out.java");
const DEFAULT_COLUMN: &str = include_str!("../java/trailing_comments/default_column.java");
const DEFAULT_COLUMN_OUT: &str = include_str!("../java/trailing_comments/default_column.out.java");

/// The comment column toggles off so a trailing comment's own placement is the
/// only thing under test (a trailing comment ignores them anyway).
fn indented() -> JavaStyle {
    style(|s| {
        s.line_comment_at_first_column = false;
        s.block_comment_at_first_column = false;
    })
}

/// The comment stays behind its code and the output is a fixed point.
fn golden(input: &str, expected: &str) {
    assert_eq!(format_with(input, &indented()), expected);
    assert_eq!(format_with(expected, &indented()), expected);
}

#[test]
fn statement_comment_stays_behind_the_statement() {
    golden(STATEMENT, STATEMENT_OUT);
}

#[test]
fn member_comment_stays_behind_the_member_and_its_braces() {
    golden(MEMBER, MEMBER_OUT);
}

#[test]
fn list_element_comment_stays_behind_the_element() {
    golden(LIST, LIST_OUT);
}

#[test]
fn a_trailing_comment_ignores_the_column_options() {
    // The pristine default has both `*_AT_FIRST_COLUMN` toggles on (comments on
    // their own line go to column 1), but a trailing comment stays put.
    assert_eq!(format(DEFAULT_COLUMN), DEFAULT_COLUMN_OUT);
}
