//! KEEP_SIMPLE_BLOCKS_IN_ONE_LINE — one-line simple block output.
//! Fixtures live under tests/java/keep_simple_blocks_in_one_line/.

use super::common::*;
use java_formatter_core::config::{BraceStyle, JavaStyle};

const SIMPLE_IF_COLLAPSE: &str =
    include_str!("../java/keep_simple_blocks_in_one_line/simple_if_collapse.java");
const SIMPLE_IF_COLLAPSE_OUT: &str =
    include_str!("../java/keep_simple_blocks_in_one_line/simple_if_collapse.out.java");
const KEEP_IF: &str = include_str!("../java/keep_simple_blocks_in_one_line/keep_if.java");
const KEEP_IF_OUT: &str = include_str!("../java/keep_simple_blocks_in_one_line/keep_if.out.java");
const ELSE_CHAIN: &str = include_str!("../java/keep_simple_blocks_in_one_line/else_chain.java");
const ELSE_CHAIN_OUT: &str =
    include_str!("../java/keep_simple_blocks_in_one_line/else_chain.out.java");
const WHILE_FOR_DO: &str = include_str!("../java/keep_simple_blocks_in_one_line/while_for_do.java");
const WHILE_FOR_DO_OUT: &str =
    include_str!("../java/keep_simple_blocks_in_one_line/while_for_do.out.java");
const MULTIPLE_STATEMENTS: &str =
    include_str!("../java/keep_simple_blocks_in_one_line/multiple_statements.java");
const MULTIPLE_STATEMENTS_OUT: &str =
    include_str!("../java/keep_simple_blocks_in_one_line/multiple_statements.out.java");
const NEXT_LINE_BRACE: &str =
    include_str!("../java/keep_simple_blocks_in_one_line/next_line_brace.java");
const NEXT_LINE_BRACE_OUT: &str =
    include_str!("../java/keep_simple_blocks_in_one_line/next_line_brace.out.java");
const TRY_SYNC_ONE_LINE: &str =
    include_str!("../java/keep_simple_blocks_in_one_line/try_sync_one_line.java");
const TRY_SYNC_COLLAPSE_OUT: &str =
    include_str!("../java/keep_simple_blocks_in_one_line/try_sync_collapse.out.java");
const TRY_SYNC_DEFAULT_OUT: &str =
    include_str!("../java/keep_simple_blocks_in_one_line/try_sync_default.out.java");
const TRY_SYNC_NEXT_LINE_OUT: &str =
    include_str!("../java/keep_simple_blocks_in_one_line/try_sync_next_line.out.java");

fn keep_simple() -> JavaStyle {
    // The collapse is exercised with the Java padding toggle on so the
    // goldens keep the padded `{ s }` one-line blocks; the faithful
    // absent/false default is flush `{s}` (see the spaces-inside-block-braces
    // option's own test file).
    style(|s| {
        s.keep_simple_blocks_in_one_line = true;
        s.spaces_inside_block_braces_when_body_is_present = true;
    })
}

fn keep_simple_next_line() -> JavaStyle {
    style(|s| {
        s.keep_simple_blocks_in_one_line = true;
        s.other_brace_style = BraceStyle::NextLine;
    })
}

#[test]
fn simple_if_collapse() {
    // With the option off a one-line block is expanded.
    assert_eq!(format(SIMPLE_IF_COLLAPSE), SIMPLE_IF_COLLAPSE_OUT);
}

#[test]
fn keep_if() {
    assert_eq!(format_with(KEEP_IF, &keep_simple()), KEEP_IF_OUT);
}

#[test]
fn else_chain() {
    assert_eq!(format_with(ELSE_CHAIN, &keep_simple()), ELSE_CHAIN_OUT);
}

#[test]
fn while_for_do() {
    assert_eq!(format_with(WHILE_FOR_DO, &keep_simple()), WHILE_FOR_DO_OUT);
}

#[test]
fn multiple_statements() {
    // A block with more than one statement stays expanded.
    assert_eq!(
        format_with(MULTIPLE_STATEMENTS, &keep_simple()),
        MULTIPLE_STATEMENTS_OUT
    );
}

#[test]
fn next_line_brace() {
    // A one-line block is impossible when the block brace goes on the next line.
    assert_eq!(
        format_with(NEXT_LINE_BRACE, &keep_simple_next_line()),
        NEXT_LINE_BRACE_OUT
    );
}

#[test]
fn try_sync_collapse() {
    // try/catch/finally, try-with-resources and synchronized all collapse.
    assert_eq!(
        format_with(TRY_SYNC_ONE_LINE, &keep_simple()),
        TRY_SYNC_COLLAPSE_OUT
    );
}

#[test]
fn try_sync_default() {
    // Without the option the multi-line layout is unchanged.
    assert_eq!(format(TRY_SYNC_ONE_LINE), TRY_SYNC_DEFAULT_OUT);
}

#[test]
fn try_sync_next_line() {
    // With NextLine brace style nothing collapses.
    assert_eq!(
        format_with(TRY_SYNC_ONE_LINE, &keep_simple_next_line()),
        TRY_SYNC_NEXT_LINE_OUT
    );
}
