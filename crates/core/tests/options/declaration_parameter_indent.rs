//! DECLARATION_PARAMETER_INDENT — per-construct continuation indent for
//! wrapped method / constructor parameter lists.
//! Fixtures live under tests/java/declaration_parameter_indent/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const CONSTRUCTS: &str = include_str!("../java/declaration_parameter_indent/constructs.java");
const CONSTRUCTS_OUT: &str =
    include_str!("../java/declaration_parameter_indent/constructs.out.java");
const CONSTRUCTS_DEFAULT_OUT: &str =
    include_str!("../java/declaration_parameter_indent/constructs_default.out.java");

/// An explicit width of 2 for declaration parameters only: the wrapped
/// parameter list indents 2 columns from the declaration, while the sibling
/// constructs (call arguments, chained call, array elements) keep their
/// default widths.
fn style_with() -> JavaStyle {
    style(|s| s.declaration_parameter_indent = 2)
}

#[test]
fn declaration_parameter_indent_overrides_declaration_params_only() {
    assert_eq!(format_with(CONSTRUCTS, &style_with()), CONSTRUCTS_OUT);
}

#[test]
fn declaration_parameter_indent_idempotent() {
    assert_eq!(format_with(CONSTRUCTS_OUT, &style_with()), CONSTRUCTS_OUT);
}

#[test]
fn absent_declaration_parameter_indent_keeps_the_default_width() {
    // `-1` (default) inherits today's layout for every construct.
    assert_eq!(format(CONSTRUCTS), CONSTRUCTS_DEFAULT_OUT);
}
