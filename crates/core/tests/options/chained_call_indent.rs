//! CHAINED_CALL_INDENT — per-construct continuation indent for wrapped
//! chained method calls.
//! Fixtures live under tests/java/chained_call_indent/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const CONSTRUCTS: &str = include_str!("../java/chained_call_indent/constructs.java");
const CONSTRUCTS_OUT: &str = include_str!("../java/chained_call_indent/constructs.out.java");
const CONSTRUCTS_DEFAULT_OUT: &str =
    include_str!("../java/chained_call_indent/constructs_default.out.java");

/// An explicit width of 2 for chained calls only: the chain's link lines
/// indent 2 columns from the statement, while the sibling constructs
/// (declaration parameters, call arguments, array elements) keep their default
/// widths.
fn style_with() -> JavaStyle {
    style(|s| s.chained_call_indent = 2)
}

#[test]
fn chained_call_indent_overrides_chained_call_links_only() {
    assert_eq!(format_with(CONSTRUCTS, &style_with()), CONSTRUCTS_OUT);
}

#[test]
fn chained_call_indent_idempotent() {
    assert_eq!(format_with(CONSTRUCTS_OUT, &style_with()), CONSTRUCTS_OUT);
}

#[test]
fn absent_chained_call_indent_keeps_the_default_width() {
    // `-1` (default) inherits today's layout for every construct.
    assert_eq!(format(CONSTRUCTS), CONSTRUCTS_DEFAULT_OUT);
}
