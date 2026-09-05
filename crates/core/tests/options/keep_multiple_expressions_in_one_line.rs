//! KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE — keep the multiple expressions of a
//! statement (a classic `for` header's init / update clause lists, a
//! multi-declarator field / local declaration) on one line.
//!
//! The engine never splits these lists: `for` per-clause breaks land at the
//! semicolons (`FOR_STATEMENT_WRAP`), never inside a comma-separated slot,
//! and multi-declarator joins have no per-declarator break layout — so the
//! option is honoured by construction and its on / off / absent output is
//! identical (pinned below). The option becomes load-bearing should a
//! per-expression break ever be added.
//!
//! Fixtures live under tests/java/keep_multiple_expressions_in_one_line/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const FOR_DECLS: &str =
    include_str!("../java/keep_multiple_expressions_in_one_line/for_decls.java");
const FOR_DECLS_ON_OUT: &str =
    include_str!("../java/keep_multiple_expressions_in_one_line/on.out.java");
const FOR_DECLS_OFF_OUT: &str =
    include_str!("../java/keep_multiple_expressions_in_one_line/off.out.java");
const FOR_DECLS_DEFAULT_OUT: &str =
    include_str!("../java/keep_multiple_expressions_in_one_line/default.out.java");
const COMPOSITION: &str =
    include_str!("../java/keep_multiple_expressions_in_one_line/composition.java");
const COMPOSITION_OUT: &str =
    include_str!("../java/keep_multiple_expressions_in_one_line/composition.out.java");
const SELF_GOLDEN: &str =
    include_str!("../java/keep_multiple_expressions_in_one_line/self_golden.java");
const SELF_GOLDEN_OUT: &str =
    include_str!("../java/keep_multiple_expressions_in_one_line/self_golden.out.java");

fn on_style() -> JavaStyle {
    style(|s| s.keep_multiple_expressions_in_one_line = true)
}

fn off_style() -> JavaStyle {
    style(|s| s.keep_multiple_expressions_in_one_line = false)
}

/// A keep-simple-blocks style whose collapsed body keeps the multi-expression
/// `for` header inline.
fn composition_style() -> JavaStyle {
    style(|s| {
        s.keep_simple_blocks_in_one_line = true;
        s.keep_multiple_expressions_in_one_line = true;
        s.spaces_inside_block_braces_when_body_is_present = true;
    })
}

#[test]
fn option_on_keeps_the_multi_expression_statements_on_one_line() {
    assert_eq!(format_with(FOR_DECLS, &on_style()), FOR_DECLS_ON_OUT);
}

#[test]
fn option_off_keeps_the_multi_expression_statements_on_one_line() {
    // Off (the built-in default): the joins are still never split — the
    // engine has no per-expression break layout, so the output is identical
    // to the option-on golden.
    assert_eq!(format_with(FOR_DECLS, &off_style()), FOR_DECLS_OFF_OUT);
}

#[test]
fn absent_option_uses_the_built_in_false_default() {
    assert_eq!(format(FOR_DECLS), FOR_DECLS_DEFAULT_OUT);
}

#[test]
fn collapsed_keep_simple_body_keeps_the_multi_expression_header_inline() {
    // KEEP_SIMPLE_BLOCKS_IN_ONE_LINE collapses the loop body; the header's
    // multi-clause init/update list stays joined on the header line.
    assert_eq!(
        format_with(COMPOSITION, &composition_style()),
        COMPOSITION_OUT
    );
}

#[test]
fn reformatting_collapsed_multi_expression_output_is_a_no_op() {
    // A self-golden: the collapsed composition formats to itself (R6).
    assert_eq!(
        format_with(SELF_GOLDEN, &composition_style()),
        SELF_GOLDEN_OUT
    );
}
