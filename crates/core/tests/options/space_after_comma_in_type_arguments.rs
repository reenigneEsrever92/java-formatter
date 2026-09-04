//! SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS — space after `,` in generic type
//! arguments. Defaults to on (Map<String, Integer>).
//! Fixtures live under tests/java/space_after_comma_in_type_arguments/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_after_comma_in_type_arguments/mixed.java");
const MIXED_OUT: &str =
    include_str!("../java/space_after_comma_in_type_arguments/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_after_comma_in_type_arguments/mixed_default.out.java");

#[test]
fn off_glues_type_argument_commas() {
    let style = style(|s| s.space_after_comma_in_type_arguments = false);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
