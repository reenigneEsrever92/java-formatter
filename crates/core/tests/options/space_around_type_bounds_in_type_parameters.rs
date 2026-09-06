//! SPACE_AROUND_TYPE_BOUNDS_IN_TYPE_PARAMETERS — spaces around the `&`-joined
//! bounds of a type parameter (`T extends A & B` vs `T extends A&B`). Defaults
//! to on; the mandatory space after `extends` and wildcard bound spacing stay.
//! Fixtures live under tests/java/space_around_type_bounds_in_type_parameters/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_around_type_bounds_in_type_parameters/mixed.java");
const MIXED_OUT: &str =
    include_str!("../java/space_around_type_bounds_in_type_parameters/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_around_type_bounds_in_type_parameters/mixed_default.out.java");
const MIXED_SELF: &str =
    include_str!("../java/space_around_type_bounds_in_type_parameters/mixed_self.java");
const MIXED_SELF_OUT: &str =
    include_str!("../java/space_around_type_bounds_in_type_parameters/mixed_self.out.java");
const COMPOSED: &str =
    include_str!("../java/space_around_type_bounds_in_type_parameters/composed.java");
const COMPOSED_OUT: &str =
    include_str!("../java/space_around_type_bounds_in_type_parameters/composed.out.java");

#[test]
fn off_compresses_type_bound_spacing() {
    let style = style(|s| s.space_around_type_bounds_in_type_parameters = false);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_keeps_canonical_output() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}

#[test]
fn composes_with_wildcard_bound_spacing() {
    let style = style(|s| s.space_around_type_bounds_in_type_parameters = false);
    assert_eq!(format_with(COMPOSED, &style), COMPOSED_OUT);
}

#[test]
fn compressed_output_is_idempotent() {
    let style = style(|s| s.space_around_type_bounds_in_type_parameters = false);
    assert_eq!(format_with(MIXED_SELF, &style), MIXED_SELF_OUT);
}
