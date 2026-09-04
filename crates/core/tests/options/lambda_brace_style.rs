//! LAMBDA_BRACE_STYLE — brace placement for block lambda bodies.
//!
//! The option reuses the IntelliJ brace codes of BRACE_STYLE, but governs the
//! lambda body alone: `EndOfLine` (the IntelliJ default, code 1) keeps the
//! `{` on the `->` line, the `NextLine` family (codes 2–4) starts a fresh
//! line at the statement indent, and `NextLineIfWrapped` (code 5) behaves
//! like `EndOfLine`. It is independent of `BRACE_STYLE` — here the lambda goes
//! `NextLine` while the `if` block in the same method stays end-of-line.
//! Simple one-statement bodies are only collapsed under an inline-compatible
//! lambda brace style.
//!
//! Fixtures live under tests/java/lambda_brace_style/.

use super::common::*;
use java_formatter_core::config::BraceStyle;

const LAMBDA_BLOCKS: &str = include_str!("../java/lambda_brace_style/lambda_blocks.java");
const LAMBDA_BLOCKS_DEFAULT_OUT: &str =
    include_str!("../java/lambda_brace_style/lambda_blocks_default.out.java");
const LAMBDA_BLOCKS_NEXT_LINE_OUT: &str =
    include_str!("../java/lambda_brace_style/lambda_blocks_next_line.out.java");

#[test]
fn default_end_of_line_keeps_lambda_braces_on_the_arrow_line() {
    assert_eq!(format(LAMBDA_BLOCKS), LAMBDA_BLOCKS_DEFAULT_OUT);
}

#[test]
fn next_line_style_puts_lambda_braces_on_their_own_line_independent_of_brace_style() {
    // LAMBDA_BRACE_STYLE = NextLine while BRACE_STYLE stays end-of-line: the
    // lambda's `{` moves to a fresh line but the `if` block's `{` does not.
    let style = style(|s| s.lambda_brace_style = BraceStyle::NextLine);
    assert_eq!(
        format_with(LAMBDA_BLOCKS, &style),
        LAMBDA_BLOCKS_NEXT_LINE_OUT
    );
}
