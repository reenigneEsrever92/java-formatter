//! SPACES_WITHIN_ANGLE_BRACKETS — spaces inside the angle brackets of type
//! arguments and type parameters (`< T >` vs `<T>`). Defaults to off.
//! Fixtures live under tests/java/spaces_within_angle_brackets/.

use super::common::*;

const MIXED: &str = include_str!("../java/spaces_within_angle_brackets/mixed.java");
const MIXED_OUT: &str = include_str!("../java/spaces_within_angle_brackets/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/spaces_within_angle_brackets/mixed_default.out.java");
const MIXED_SELF: &str = include_str!("../java/spaces_within_angle_brackets/mixed_self.java");
const MIXED_SELF_OUT: &str =
    include_str!("../java/spaces_within_angle_brackets/mixed_self.out.java");
const COMPOSED: &str = include_str!("../java/spaces_within_angle_brackets/composed.java");
const COMPOSED_OUT: &str = include_str!("../java/spaces_within_angle_brackets/composed.out.java");

#[test]
fn on_pads_inside_angle_brackets() {
    let style = style(|s| s.spaces_within_angle_brackets = true);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_keeps_canonical_output() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}

#[test]
fn composes_with_space_after_comma_in_type_arguments() {
    let style = style(|s| {
        s.spaces_within_angle_brackets = true;
        s.space_after_comma_in_type_arguments = false;
    });
    assert_eq!(format_with(COMPOSED, &style), COMPOSED_OUT);
}

#[test]
fn padded_output_is_idempotent() {
    let style = style(|s| s.spaces_within_angle_brackets = true);
    assert_eq!(format_with(MIXED_SELF, &style), MIXED_SELF_OUT);
}
