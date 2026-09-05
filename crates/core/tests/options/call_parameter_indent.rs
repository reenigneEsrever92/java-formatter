//! CALL_PARAMETER_INDENT — per-construct continuation indent for wrapped
//! method-call / constructor-call argument lists.
//! Fixtures live under tests/java/call_parameter_indent/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const CONSTRUCTS: &str = include_str!("../java/call_parameter_indent/constructs.java");
const CONSTRUCTS_OUT: &str = include_str!("../java/call_parameter_indent/constructs.out.java");
const CONSTRUCTS_DEFAULT_OUT: &str =
    include_str!("../java/call_parameter_indent/constructs_default.out.java");

/// An explicit width of 2 for call arguments only: the wrapped argument list
/// indents 2 columns from the statement, while the sibling constructs
/// (declaration parameters, chained call, array elements) keep their default
/// widths.
fn style_with() -> JavaStyle {
    style(|s| s.call_parameter_indent = 2)
}

#[test]
fn call_parameter_indent_overrides_call_arguments_only() {
    assert_eq!(format_with(CONSTRUCTS, &style_with()), CONSTRUCTS_OUT);
}

#[test]
fn call_parameter_indent_idempotent() {
    assert_eq!(format_with(CONSTRUCTS_OUT, &style_with()), CONSTRUCTS_OUT);
}

#[test]
fn absent_call_parameter_indent_keeps_the_default_width() {
    // `-1` (default) inherits today's layout for every construct.
    assert_eq!(format(CONSTRUCTS), CONSTRUCTS_DEFAULT_OUT);
}
