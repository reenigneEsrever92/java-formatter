//! SPACE_AFTER_COMMA — space after `,` in declarations, calls, arrays,
//! annotations, record components, lambda parameters and type lists. Defaults
//! to on (f(a, b)).
//! Fixtures live under tests/java/space_after_comma/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_after_comma/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_after_comma/mixed.out.java");
const MIXED_DEFAULT_OUT: &str = include_str!("../java/space_after_comma/mixed_default.out.java");

#[test]
fn off_glues_commas_to_their_left_neighbor() {
    let style = style(|s| s.space_after_comma = false);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
