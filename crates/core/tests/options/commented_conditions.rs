//! Comments inside keyword conditions and parenthesized expressions —
//! regression coverage for the bug where a comment was taken for the whole
//! inner expression: tree-sitter attaches comments as named extras inside the
//! parens, so a leading comment became `named_child(0)`, the real condition
//! vanished, and the closing `)` (plus the following `{` or `;`) was swallowed
//! into a `//` comment, producing invalid Java. Comments in trailing and
//! mid-expression positions were silently dropped.
//!
//! A break-forcing comment (`//` or a multi-line `/* … */`) inside a condition
//! is laid out on its own line at the continuation indent — the first one glued
//! right after `(` — with the condition after it and the closing `)` alone on
//! its line, so nothing is ever swallowed. A single-line `/* … */` stays inline,
//! and a comment nested inside the expression itself keeps the whole paren
//! verbatim (R4). The comment column toggles are pinned off so the fixtures
//! show the code indent.
//!
//! Fixtures live under tests/java/commented_conditions/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const IF_LEADING: &str = include_str!("../java/commented_conditions/if_leading.java");
const IF_LEADING_OUT: &str = include_str!("../java/commented_conditions/if_leading.out.java");
const WHILE_LEADING: &str = include_str!("../java/commented_conditions/while_leading.java");
const WHILE_LEADING_OUT: &str = include_str!("../java/commented_conditions/while_leading.out.java");
const DO_WHILE_LEADING: &str = include_str!("../java/commented_conditions/do_while_leading.java");
const DO_WHILE_LEADING_OUT: &str =
    include_str!("../java/commented_conditions/do_while_leading.out.java");
const SYNCHRONIZED_LEADING: &str =
    include_str!("../java/commented_conditions/synchronized_leading.java");
const SYNCHRONIZED_LEADING_OUT: &str =
    include_str!("../java/commented_conditions/synchronized_leading.out.java");
const SWITCH_LEADING: &str = include_str!("../java/commented_conditions/switch_leading.java");
const SWITCH_LEADING_OUT: &str =
    include_str!("../java/commented_conditions/switch_leading.out.java");
const PARENS_MID: &str = include_str!("../java/commented_conditions/parens_mid.java");
const PARENS_MID_OUT: &str = include_str!("../java/commented_conditions/parens_mid.out.java");
const IF_TRAILING_BLOCK: &str = include_str!("../java/commented_conditions/if_trailing_block.java");
const IF_TRAILING_BLOCK_OUT: &str =
    include_str!("../java/commented_conditions/if_trailing_block.out.java");
const IF_MID_EXPRESSION: &str = include_str!("../java/commented_conditions/if_mid_expression.java");
const IF_MID_EXPRESSION_OUT: &str =
    include_str!("../java/commented_conditions/if_mid_expression.out.java");

/// The comment column toggles off so the condition layout is the only thing
/// under test (the default places `//` comments at column 1).
fn indented() -> JavaStyle {
    style(|s| {
        s.line_comment_at_first_column = false;
        s.block_comment_at_first_column = false;
    })
}

/// The comment, the condition and the construct all survive; the output is a
/// fixed point.
fn golden(input: &str, expected: &str) {
    assert_eq!(format_with(input, &indented()), expected);
    assert_eq!(format_with(expected, &indented()), expected);
}

#[test]
fn a_leading_comment_in_an_if_condition_does_not_swallow_the_condition() {
    golden(IF_LEADING, IF_LEADING_OUT);
}

#[test]
fn a_leading_comment_in_a_while_condition_is_kept_on_its_own_line() {
    golden(WHILE_LEADING, WHILE_LEADING_OUT);
}

#[test]
fn a_leading_comment_in_a_do_while_condition_does_not_swallow_the_semicolon() {
    golden(DO_WHILE_LEADING, DO_WHILE_LEADING_OUT);
}

#[test]
fn a_leading_comment_in_a_synchronized_lock_expression_is_kept() {
    golden(SYNCHRONIZED_LEADING, SYNCHRONIZED_LEADING_OUT);
}

#[test]
fn a_leading_comment_in_a_switch_expression_is_kept() {
    golden(SWITCH_LEADING, SWITCH_LEADING_OUT);
}

#[test]
fn a_comment_mid_expression_keeps_the_parenthesized_expression_verbatim() {
    golden(PARENS_MID, PARENS_MID_OUT);
}

#[test]
fn a_trailing_block_comment_in_a_condition_stays_inline() {
    golden(IF_TRAILING_BLOCK, IF_TRAILING_BLOCK_OUT);
}

#[test]
fn a_comment_between_binary_operands_is_not_dropped() {
    golden(IF_MID_EXPRESSION, IF_MID_EXPRESSION_OUT);
}
