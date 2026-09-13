//! Lambda block bodies in argument position — regression coverage for the bug
//! where a `//` comment inside such a body was joined onto the code on the same
//! line (so the comment swallowed everything after it, silently deleting the
//! following statement's header), and where multi-statement bodies were joined
//! with `"; "` (doubling semicolons) or with the raw multi-line text of a
//! control-flow statement.
//!
//! A block that holds a `//` comment or a statement that cannot render flat
//! (control flow, multi-line text) has no safe one-line form: it keeps its
//! multi-line layout and every statement stays intact. A block of
//! self-terminating statements still collapses when it fits. Under
//! `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE` only a *single*-statement body collapses;
//! multi-statement bodies stay multi-line. The comment column toggles are
//! pinned off so the fixtures show the code indent.
//!
//! Fixtures live under tests/java/lambda_body_comments/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const COMMENT_IF: &str = include_str!("../java/lambda_body_comments/comment_if.java");
const COMMENT_IF_OUT: &str = include_str!("../java/lambda_body_comments/comment_if.out.java");
const MULTI_STATEMENT: &str = include_str!("../java/lambda_body_comments/multi_statement.java");
const MULTI_STATEMENT_OUT: &str =
    include_str!("../java/lambda_body_comments/multi_statement.out.java");
const CHAIN: &str = include_str!("../java/lambda_body_comments/chain.java");
const CHAIN_OUT: &str = include_str!("../java/lambda_body_comments/chain.out.java");
const SIMPLE_COLLAPSE: &str = include_str!("../java/lambda_body_comments/simple_collapse.java");
const SIMPLE_COLLAPSE_OUT: &str =
    include_str!("../java/lambda_body_comments/simple_collapse.out.java");

/// The comment column toggles off so the block layout is the only thing under
/// test (the default places `//` comments at column 1).
fn indented() -> JavaStyle {
    style(|s| {
        s.line_comment_at_first_column = false;
        s.block_comment_at_first_column = false;
    })
}

/// A style with `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE` on, to exercise the
/// one-line collapse path (the default flush `{s}` body presentation).
fn collapse() -> JavaStyle {
    style(|s| {
        s.line_comment_at_first_column = false;
        s.block_comment_at_first_column = false;
        s.keep_simple_lambdas_in_one_line = true;
    })
}

/// The comment, its `if` header and the statement all survive; the output is a
/// fixed point.
fn golden(input: &str, expected: &str) {
    assert_eq!(format_with(input, &indented()), expected);
    assert_eq!(format_with(expected, &indented()), expected);
}

#[test]
fn a_comment_before_an_if_does_not_swallow_the_if() {
    golden(COMMENT_IF, COMMENT_IF_OUT);
}

#[test]
fn a_multi_statement_body_collapses_only_when_it_stays_flat() {
    golden(MULTI_STATEMENT, MULTI_STATEMENT_OUT);
}

#[test]
fn a_comment_lambda_in_a_chain_link_keeps_its_statements() {
    golden(CHAIN, CHAIN_OUT);
}

#[test]
fn one_line_collapse_applies_only_to_a_single_simple_statement() {
    // `single` collapses; the commented body and the multi-statement body
    // stay multi-line. The output is a fixed point.
    assert_eq!(
        format_with(SIMPLE_COLLAPSE, &collapse()),
        SIMPLE_COLLAPSE_OUT
    );
    assert_eq!(
        format_with(SIMPLE_COLLAPSE_OUT, &collapse()),
        SIMPLE_COLLAPSE_OUT
    );
}
