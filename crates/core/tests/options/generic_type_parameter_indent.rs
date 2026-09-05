//! GENERIC_TYPE_PARAMETER_INDENT — per-construct continuation indent for
//! generic type parameters. Inert today: generic parameter lists always render
//! flat (`flat_type_params`), so the explicit width must not disturb any other
//! construct — the set style and the default style produce identical bytes.
//! Fixtures live under tests/java/generic_type_parameter_indent/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const GENERIC: &str = include_str!("../java/generic_type_parameter_indent/generic.java");
const GENERIC_OUT: &str = include_str!("../java/generic_type_parameter_indent/generic.out.java");
const GENERIC_DEFAULT_OUT: &str =
    include_str!("../java/generic_type_parameter_indent/generic_default.out.java");

fn style_with() -> JavaStyle {
    style(|s| s.generic_type_parameter_indent = 6)
}

#[test]
fn generic_type_parameter_indent_is_inert_but_round_trips() {
    // The explicit width parses / serializes (the registry round-trip holds)
    // and leaves the flat generic lists and every sibling construct unchanged.
    assert_eq!(format_with(GENERIC, &style_with()), GENERIC_OUT);
    assert_eq!(GENERIC_OUT, GENERIC_DEFAULT_OUT);
}

#[test]
fn generic_type_parameter_indent_idempotent() {
    assert_eq!(format_with(GENERIC_OUT, &style_with()), GENERIC_OUT);
}

#[test]
fn absent_generic_type_parameter_indent_keeps_the_default_output() {
    assert_eq!(format(GENERIC), GENERIC_DEFAULT_OUT);
}
