//! ALIGN_MULTILINE_TEXT_BLOCKS — align multiline text blocks to the
//! statement's continuation column.
//! Fixtures live under tests/java/align_multiline_text_blocks/.
//!
//! With the option on, a multiline text block rendered as a statement value
//! is realigned to the output the formatter emits: every non-opening line
//! that carries visible content (the content lines and the closing-delimiter
//! line) shifts by one uniform delta so the first content line sits at the
//! canonical continuation column for the statement, preserving the block's
//! relative indentation and its stripped string value. Off and absent (the
//! default) echo the block verbatim, byte for byte. The option never touches
//! ordinary strings, single-line text blocks or flat (no-column) contexts.

use super::common::*;

const BLOCK: &str = include_str!("../java/align_multiline_text_blocks/block.java");
const BLOCK_ALIGN_OUT: &str =
    include_str!("../java/align_multiline_text_blocks/block_align.out.java");
const BLOCK_OFF_OUT: &str = include_str!("../java/align_multiline_text_blocks/block_off.out.java");
const BLOCK_ABSENT_OUT: &str =
    include_str!("../java/align_multiline_text_blocks/block_absent.out.java");
const ALIGNED: &str = include_str!("../java/align_multiline_text_blocks/aligned.java");
const ALIGNED_OUT: &str = include_str!("../java/align_multiline_text_blocks/aligned.out.java");

#[test]
fn align_on_realigns_the_misindented_block() {
    // The misindented content (column 4) moves so its first line sits at the
    // statement's continuation column (16 for a method-body statement at the
    // default indent); the closing delimiter shifts with it and the relative
    // content indentation is preserved.
    let style = style(|s| s.align_multiline_text_blocks = true);
    assert_eq!(format_with(BLOCK, &style), BLOCK_ALIGN_OUT);
}

#[test]
fn align_off_echoes_the_block_verbatim() {
    let style = style(|s| s.align_multiline_text_blocks = false);
    assert_eq!(format_with(BLOCK, &style), BLOCK_OFF_OUT);
}

#[test]
fn absent_option_echoes_the_block_verbatim() {
    // The option defaults to false, so today's byte-for-byte text-block echo
    // is unchanged under the default style (R4).
    assert_eq!(format(BLOCK), BLOCK_ABSENT_OUT);
}

#[test]
fn reformatting_the_aligned_block_is_a_no_op() {
    // A self-golden: the aligned block formats to itself under the option
    // (R6).
    let style = style(|s| s.align_multiline_text_blocks = true);
    assert_eq!(format_with(ALIGNED, &style), ALIGNED_OUT);
}
