//! Comments in a declaration's modifier area — regression coverage for the bug
//! where a comment between the modifiers and the type was treated as a keyword
//! modifier (a `//` then commented out the rest of the declaration) or dropped
//! outright.
//!
//! A comment is preserved at its source position on its own line: before the
//! declaration it leads, or after the modifier it follows; no code ever shares
//! a line with a `//`, and the output is a fixed point. Comments a construct
//! cannot carry (a parameter without an own-line form, an enum constant) keep
//! their source verbatim (R4).
//!
//! The `*_AT_FIRST_COLUMN` toggles are pinned off so the comment sits at the
//! declaration's indent, which makes the placement observable; under the
//! pristine default both toggles are on and a line comment moves to column 1
//! (see `default_column.java`).
//!
//! Fixtures live under tests/java/comment_in_modifiers/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const FIELD: &str = include_str!("../java/comment_in_modifiers/field.java");
const FIELD_OUT: &str = include_str!("../java/comment_in_modifiers/field.out.java");
const METHOD: &str = include_str!("../java/comment_in_modifiers/method.java");
const METHOD_OUT: &str = include_str!("../java/comment_in_modifiers/method.out.java");
const NESTED_TYPE: &str = include_str!("../java/comment_in_modifiers/nested_type.java");
const NESTED_TYPE_OUT: &str = include_str!("../java/comment_in_modifiers/nested_type.out.java");
const PARAMETER: &str = include_str!("../java/comment_in_modifiers/parameter.java");
const PARAMETER_OUT: &str = include_str!("../java/comment_in_modifiers/parameter.out.java");
const LOCAL_VARIABLE: &str = include_str!("../java/comment_in_modifiers/local_variable.java");
const LOCAL_VARIABLE_OUT: &str =
    include_str!("../java/comment_in_modifiers/local_variable.out.java");
const ENUM_CONSTANT: &str = include_str!("../java/comment_in_modifiers/enum_constant.java");
const ENUM_CONSTANT_OUT: &str = include_str!("../java/comment_in_modifiers/enum_constant.out.java");
const RECORD_COMPONENT: &str = include_str!("../java/comment_in_modifiers/record_component.java");
const RECORD_COMPONENT_OUT: &str =
    include_str!("../java/comment_in_modifiers/record_component.out.java");
const DEFAULT_COLUMN: &str = include_str!("../java/comment_in_modifiers/default_column.java");
const DEFAULT_COLUMN_OUT: &str =
    include_str!("../java/comment_in_modifiers/default_column.out.java");

/// The comment column toggles off so an indented comment stays at the code
/// indent, making the own-line placement observable in every fixture.
fn indented() -> JavaStyle {
    style(|s| {
        s.line_comment_at_first_column = false;
        s.block_comment_at_first_column = false;
    })
}

/// The comment survives at its source position and the output is a fixed point.
fn golden(input: &str, expected: &str) {
    assert_eq!(format_with(input, &indented()), expected);
    assert_eq!(format_with(expected, &indented()), expected);
}

#[test]
fn field_comment_between_annotations_and_modifier_is_not_joined() {
    golden(FIELD, FIELD_OUT);
}

#[test]
fn method_comment_in_the_modifier_area_is_not_joined_or_dropped() {
    golden(METHOD, METHOD_OUT);
}

#[test]
fn nested_type_comment_is_not_joined() {
    golden(NESTED_TYPE, NESTED_TYPE_OUT);
}

#[test]
fn parameter_comment_keeps_its_position() {
    golden(PARAMETER, PARAMETER_OUT);
}

#[test]
fn local_variable_comment_keeps_its_position() {
    golden(LOCAL_VARIABLE, LOCAL_VARIABLE_OUT);
}

#[test]
fn enum_constant_comment_is_preserved_verbatim() {
    golden(ENUM_CONSTANT, ENUM_CONSTANT_OUT);
}

#[test]
fn record_component_comment_keeps_its_position() {
    golden(RECORD_COMPONENT, RECORD_COMPONENT_OUT);
}

#[test]
fn absent_options_use_the_built_in_first_column_defaults() {
    // The IntelliJ built-in default for both toggles is true, so the pristine
    // default style pins the line comment to column 1.
    assert_eq!(format(DEFAULT_COLUMN), DEFAULT_COLUMN_OUT);
}
