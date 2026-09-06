//! SPACE_BEFORE_OPENING_ANGLE_BRACKET_IN_TYPE_PARAMETER — space between a
//! class / interface / record name and its type-parameter list (`class Foo<T>`
//! vs `class Foo <T>`). Defaults to off; composes with the shipped
//! SPACE_BEFORE_TYPE_PARAMETER_LIST at the same join (either on inserts the
//! single space).
//! Fixtures live under tests/java/space_before_opening_angle_bracket_in_type_parameter/.

use super::common::*;

const MIXED: &str =
    include_str!("../java/space_before_opening_angle_bracket_in_type_parameter/mixed.java");
const MIXED_OUT: &str =
    include_str!("../java/space_before_opening_angle_bracket_in_type_parameter/mixed.out.java");
const MIXED_DEFAULT_OUT: &str = include_str!(
    "../java/space_before_opening_angle_bracket_in_type_parameter/mixed_default.out.java"
);
const MIXED_SELF: &str =
    include_str!("../java/space_before_opening_angle_bracket_in_type_parameter/mixed_self.java");
const MIXED_SELF_OUT: &str = include_str!(
    "../java/space_before_opening_angle_bracket_in_type_parameter/mixed_self.out.java"
);
const COMPOSED: &str =
    include_str!("../java/space_before_opening_angle_bracket_in_type_parameter/composed.java");
const COMPOSED_OUT: &str =
    include_str!("../java/space_before_opening_angle_bracket_in_type_parameter/composed.out.java");

#[test]
fn on_spaces_before_type_parameter_list() {
    let style = style(|s| s.space_before_opening_angle_bracket_in_type_parameter = true);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_keeps_canonical_output() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}

#[test]
fn composes_with_space_before_type_parameter_list() {
    let style = style(|s| {
        s.space_before_opening_angle_bracket_in_type_parameter = true;
        s.space_before_type_parameter_list = true;
    });
    assert_eq!(format_with(COMPOSED, &style), COMPOSED_OUT);
}

#[test]
fn spaced_output_is_idempotent() {
    let style = style(|s| s.space_before_opening_angle_bracket_in_type_parameter = true);
    assert_eq!(format_with(MIXED_SELF, &style), MIXED_SELF_OUT);
}
