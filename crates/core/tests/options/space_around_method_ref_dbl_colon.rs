//! SPACE_AROUND_METHOD_REF_DBL_COLON — space around the method-reference
//! separator (::). Defaults to off (A::new).
//! Fixtures live under tests/java/space_around_method_ref_dbl_colon/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_around_method_ref_dbl_colon/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_around_method_ref_dbl_colon/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_around_method_ref_dbl_colon/mixed_default.out.java");

#[test]
fn on_spaces_method_reference_colons() {
    let style = style(|s| s.space_around_method_ref_dbl_colon = true);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_stays_space_less() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
