//! BRACE_STYLE — brace placement for statement blocks (`other_brace_style`).
//! Fixtures live under tests/java/brace_style/.
//!
//! In this formatter the placement of `if`/`for`/`while` block braces does not
//! change between brace styles; what the option governs is whether a simple
//! one-statement block may be collapsed onto the header line when
//! `KEEP_SIMPLE_BLOCKS_IN_ONE_LINE` is enabled. `EndOfLine` allows the
//! collapse, `NextLine` forces the multi-line form.

use super::common::*;
use java_formatter_core::config::BraceStyle;

const BLOCK_BRACE: &str = include_str!("../java/brace_style/block_brace.java");
const BLOCK_BRACE_EOL_OUT: &str =
    include_str!("../java/brace_style/block_brace_eol.out.java");
const BLOCK_BRACE_NEXT_LINE_OUT: &str =
    include_str!("../java/brace_style/block_brace_next_line.out.java");
const BLOCK_BRACE_DEFAULT_OUT: &str =
    include_str!("../java/brace_style/block_brace_default.out.java");

#[test]
fn end_of_line_brace_style_allows_one_line_simple_blocks() {
    let style = style(|s| {
        s.keep_simple_blocks_in_one_line = true;
        // Keep the goldens' padded `{ s }` one-line blocks (the absent
        // padding default is flush `{s}`).
        s.spaces_inside_block_braces_when_body_is_present = true;
    });
    assert_eq!(format_with(BLOCK_BRACE, &style), BLOCK_BRACE_EOL_OUT);
}

#[test]
fn next_line_brace_style_keeps_simple_blocks_multiline() {
    let style = style(|s| {
        s.keep_simple_blocks_in_one_line = true;
        s.other_brace_style = BraceStyle::NextLine;
    });
    assert_eq!(format_with(BLOCK_BRACE, &style), BLOCK_BRACE_NEXT_LINE_OUT);
}

#[test]
fn default_style_expands_simple_blocks() {
    // Without KEEP_SIMPLE_BLOCKS_IN_ONE_LINE both brace styles expand.
    assert_eq!(format(BLOCK_BRACE), BLOCK_BRACE_DEFAULT_OUT);
}
