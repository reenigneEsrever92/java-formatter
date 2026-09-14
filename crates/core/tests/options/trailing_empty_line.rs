//! End-of-file policy: every output ends with exactly one trailing empty line.
//!
//! No XML option governs this — it is a fixed output policy, not a scheme
//! setting — so the fixtures live directly under
//! `tests/java/trailing_empty_line/`.

use super::common::*;

const NO_BLANK: &str = include_str!("../java/trailing_empty_line/no_blank.java");
const NO_BLANK_OUT: &str = include_str!("../java/trailing_empty_line/no_blank.out.java");
const BLANK_RUN: &str = include_str!("../java/trailing_empty_line/blank_run.java");
const BLANK_RUN_OUT: &str = include_str!("../java/trailing_empty_line/blank_run.out.java");
const EMPTY: &str = include_str!("../java/trailing_empty_line/empty.java");
const EMPTY_OUT: &str = include_str!("../java/trailing_empty_line/empty.out.java");
const WHITESPACE_ONLY: &str = include_str!("../java/trailing_empty_line/whitespace_only.java");
const WHITESPACE_ONLY_OUT: &str =
    include_str!("../java/trailing_empty_line/whitespace_only.out.java");

#[test]
fn a_source_without_a_trailing_blank_line_gains_one() {
    assert_eq!(format(NO_BLANK), NO_BLANK_OUT);
}

#[test]
fn a_run_of_trailing_blank_lines_collapses_to_one() {
    // Three blank lines after the closing brace collapse to exactly one.
    assert_eq!(format(BLANK_RUN), BLANK_RUN_OUT);
}

#[test]
fn empty_input_becomes_a_single_empty_line() {
    assert_eq!(format(EMPTY), EMPTY_OUT);
}

#[test]
fn whitespace_only_input_becomes_a_single_empty_line() {
    assert_eq!(format(WHITESPACE_ONLY), WHITESPACE_ONLY_OUT);
}

#[test]
fn the_trailing_empty_line_is_idempotent() {
    // Re-formatting an output reproduces it byte-for-byte (R6).
    assert_eq!(format(NO_BLANK_OUT), NO_BLANK_OUT);
    assert_eq!(format(BLANK_RUN_OUT), BLANK_RUN_OUT);
    assert_eq!(format(EMPTY_OUT), EMPTY_OUT);
    assert_eq!(format(WHITESPACE_ONLY_OUT), WHITESPACE_ONLY_OUT);
}
