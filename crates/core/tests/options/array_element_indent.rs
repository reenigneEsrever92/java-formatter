//! ARRAY_ELEMENT_INDENT — per-construct continuation indent for wrapped array
//! initializer elements.
//! Fixtures live under tests/java/array_element_indent/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const CONSTRUCTS: &str = include_str!("../java/array_element_indent/constructs.java");
const CONSTRUCTS_OUT: &str = include_str!("../java/array_element_indent/constructs.out.java");
const CONSTRUCTS_DEFAULT_OUT: &str =
    include_str!("../java/array_element_indent/constructs_default.out.java");

/// An explicit width of 2 for array elements only: the element lines indent 2
/// columns from the statement, while the sibling constructs (declaration
/// parameters, call arguments, chained call) keep their default widths.
fn style_with() -> JavaStyle {
    style(|s| s.array_element_indent = 2)
}

#[test]
fn array_element_indent_overrides_array_elements_only() {
    assert_eq!(format_with(CONSTRUCTS, &style_with()), CONSTRUCTS_OUT);
}

#[test]
fn array_element_indent_idempotent() {
    assert_eq!(format_with(CONSTRUCTS_OUT, &style_with()), CONSTRUCTS_OUT);
}

#[test]
fn absent_array_element_indent_keeps_the_default_width() {
    // `-1` (default) inherits today's layout for every construct.
    assert_eq!(format(CONSTRUCTS), CONSTRUCTS_DEFAULT_OUT);
}
