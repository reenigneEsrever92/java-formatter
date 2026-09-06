//! SPACE_AFTER_CLOSING_ANGLE_BRACKET_IN_TYPE_ARGUMENT — space after a closing
//! `>` of an explicit type-argument list where it abuts a following token
//! (`a.<T>b()` vs `a.<T> b()`). Defaults to off.
//! Fixtures live under tests/java/space_after_closing_angle_bracket_in_type_argument/.

use super::common::*;

const MIXED: &str =
    include_str!("../java/space_after_closing_angle_bracket_in_type_argument/mixed.java");
const MIXED_OUT: &str =
    include_str!("../java/space_after_closing_angle_bracket_in_type_argument/mixed.out.java");
const MIXED_DEFAULT_OUT: &str = include_str!(
    "../java/space_after_closing_angle_bracket_in_type_argument/mixed_default.out.java"
);
const MIXED_SELF: &str =
    include_str!("../java/space_after_closing_angle_bracket_in_type_argument/mixed_self.java");
const MIXED_SELF_OUT: &str =
    include_str!("../java/space_after_closing_angle_bracket_in_type_argument/mixed_self.out.java");
const COMPOSED: &str =
    include_str!("../java/space_after_closing_angle_bracket_in_type_argument/composed.java");
const COMPOSED_OUT: &str =
    include_str!("../java/space_after_closing_angle_bracket_in_type_argument/composed.out.java");

#[test]
fn on_spaces_after_closing_angle_bracket() {
    let style = style(|s| s.space_after_closing_angle_bracket_in_type_argument = true);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_keeps_canonical_output() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}

#[test]
fn composes_with_space_after_comma_in_type_arguments() {
    let style = style(|s| {
        s.space_after_closing_angle_bracket_in_type_argument = true;
        s.space_after_comma_in_type_arguments = false;
    });
    assert_eq!(format_with(COMPOSED, &style), COMPOSED_OUT);
}

#[test]
fn spaced_output_is_idempotent() {
    let style = style(|s| s.space_after_closing_angle_bracket_in_type_argument = true);
    assert_eq!(format_with(MIXED_SELF, &style), MIXED_SELF_OUT);
}
