//! STRIP_WHITESPACE_FROM_BLANK_LINES_IN_TEXT_BLOCKS — strip trailing
//! whitespace from blank lines inside text blocks.
//! Fixtures live under tests/java/strip_whitespace_from_blank_lines_in_text_blocks/.
//!
//! With the option on, every whitespace-only line inside a multiline text
//! block is trimmed to empty (its whitespace is never part of the text-block
//! value, so only layout changes). Lines with visible content are untouched.
//! Off and absent (the default) preserve the block byte for byte — the strip
//! is an intentional, opt-in deviation from the verbatim echo, limited to
//! whitespace-only blank lines.

use super::common::*;

const BLOCK: &str =
    include_str!("../java/strip_whitespace_from_blank_lines_in_text_blocks/block.java");
const BLOCK_STRIP_OUT: &str =
    include_str!("../java/strip_whitespace_from_blank_lines_in_text_blocks/block_strip.out.java");
const BLOCK_OFF_OUT: &str =
    include_str!("../java/strip_whitespace_from_blank_lines_in_text_blocks/block_off.out.java");
const BLOCK_ABSENT_OUT: &str =
    include_str!("../java/strip_whitespace_from_blank_lines_in_text_blocks/block_absent.out.java");
const STRIPPED: &str =
    include_str!("../java/strip_whitespace_from_blank_lines_in_text_blocks/stripped.java");
const STRIPPED_OUT: &str =
    include_str!("../java/strip_whitespace_from_blank_lines_in_text_blocks/stripped.out.java");

#[test]
fn strip_on_trims_whitespace_only_lines_to_empty() {
    // The blank content lines (spaces, and a tab run) become empty lines;
    // the visible `alpha` / `beta` / `gamma` lines keep their indentation and
    // the closing delimiter is untouched.
    let style = style(|s| s.strip_whitespace_from_blank_lines_in_text_blocks = true);
    assert_eq!(format_with(BLOCK, &style), BLOCK_STRIP_OUT);
}

#[test]
fn strip_off_preserves_the_block_byte_for_byte() {
    let style = style(|s| s.strip_whitespace_from_blank_lines_in_text_blocks = false);
    assert_eq!(format_with(BLOCK, &style), BLOCK_OFF_OUT);
}

#[test]
fn absent_option_preserves_the_block_byte_for_byte() {
    // The option defaults to false, so today's verbatim text-block echo is
    // unchanged under the default style (R4).
    assert_eq!(format(BLOCK), BLOCK_ABSENT_OUT);
}

#[test]
fn reformatting_the_stripped_block_is_a_no_op() {
    // A self-golden: the stripped fixture formats to itself under the option
    // (R6).
    let style = style(|s| s.strip_whitespace_from_blank_lines_in_text_blocks = true);
    assert_eq!(format_with(STRIPPED, &style), STRIPPED_OUT);
}
