//! MODIFIER_LIST_WRAP — wrap after the modifier / annotation list of a
//! declaration.
//! Fixtures live under tests/java/modifier_list_wrap/.
//!
//! When `true`, the declaration header breaks after its keyword modifiers
//! (the next token starts at `ind(indent)`); annotations already sit on
//! their own lines. `false` (the default) keeps the modifiers and the rest
//! of the header on one line.

use super::common::*;

const DECLARATIONS: &str = include_str!("../java/modifier_list_wrap/declarations.java");
const DECLARATIONS_WRAP_OUT: &str =
    include_str!("../java/modifier_list_wrap/declarations_wrap.out.java");
const DECLARATIONS_FLAT_OUT: &str =
    include_str!("../java/modifier_list_wrap/declarations_flat.out.java");
const WRAPPED: &str = include_str!("../java/modifier_list_wrap/wrapped.java");
const WRAPPED_OUT: &str = include_str!("../java/modifier_list_wrap/wrapped.out.java");

#[test]
fn modifier_list_wrap_breaks_after_the_keyword_modifiers() {
    let s = style(|st| st.modifier_list_wrap = true);
    assert_eq!(format_with(DECLARATIONS, &s), DECLARATIONS_WRAP_OUT);
}

#[test]
fn default_keeps_the_modifier_list_on_the_header_line() {
    assert_eq!(format(DECLARATIONS), DECLARATIONS_FLAT_OUT);
}

#[test]
fn reformatting_wrapped_declaration_output_is_a_no_op() {
    // A self-golden: the fixture already uses the wrapped layout, so
    // formatting it under the same style is byte-identical (R6).
    let s = style(|st| st.modifier_list_wrap = true);
    assert_eq!(format_with(WRAPPED, &s), WRAPPED_OUT);
}
