//! SPACE_BEFORE_ANOTATION_PARAMETER_LIST — space between an annotation name and its parameter list (XML name spelled as in IntelliJ). Defaults to off.
//! Fixtures live under tests/java/space_before_anotation_parameter_list/.

use super::common::*;

const ANNOS: &str = include_str!("../java/space_before_anotation_parameter_list/annos.java");
const ANNOS_OUT: &str = include_str!("../java/space_before_anotation_parameter_list/annos.out.java");
const ANNOS_DEFAULT_OUT: &str =
    include_str!("../java/space_before_anotation_parameter_list/annos_default.out.java");

#[test]
fn on_spaces() {
    let s = style(|st| st.space_before_anotation_parameter_list = true);
    assert_eq!(format_with(ANNOS, &s), ANNOS_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(ANNOS), ANNOS_DEFAULT_OUT);
}
