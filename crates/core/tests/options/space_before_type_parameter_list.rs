//! SPACE_BEFORE_TYPE_PARAMETER_LIST — space between a class / interface /
//! record name and its type-parameter list (`<…>`). Defaults to off
//! (class Foo<T>).
//! Fixtures live under tests/java/space_before_type_parameter_list/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_before_type_parameter_list/mixed.java");
const MIXED_OUT: &str =
    include_str!("../java/space_before_type_parameter_list/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_before_type_parameter_list/mixed_default.out.java");

#[test]
fn on_spaces_before_type_parameter_list() {
    let style = style(|s| s.space_before_type_parameter_list = true);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
