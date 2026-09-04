//! SPACE_BEFORE_ARRAY_INITIALIZER_LBRACE — space between `new T[]` and its array-initializer brace. Defaults to off.
//! Fixtures live under tests/java/space_before_array_initializer_lbrace/.

use super::common::*;

const ARRAYS: &str = include_str!("../java/space_before_array_initializer_lbrace/arrays.java");
const ARRAYS_OUT: &str = include_str!("../java/space_before_array_initializer_lbrace/arrays.out.java");
const ARRAYS_DEFAULT_OUT: &str =
    include_str!("../java/space_before_array_initializer_lbrace/arrays_default.out.java");

#[test]
fn on_spaces() {
    let s = style(|st| st.space_before_array_initializer_lbrace = true);
    assert_eq!(format_with(ARRAYS, &s), ARRAYS_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(ARRAYS), ARRAYS_DEFAULT_OUT);
}
