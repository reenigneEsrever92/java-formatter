//! SPACE_WITHIN_ARRAY_INITIALIZER_BRACES — padding inside non-empty array
//! initialiser braces.
//! Fixtures live under tests/java/space_within_array_initializer_braces/.

use super::common::*;

const ARRAY_INIT: &str =
    include_str!("../java/space_within_array_initializer_braces/array_init.java");
const ARRAY_INIT_OUT: &str =
    include_str!("../java/space_within_array_initializer_braces/array_init.out.java");
const ARRAY_INIT_DEFAULT_OUT: &str =
    include_str!("../java/space_within_array_initializer_braces/array_init_default.out.java");

#[test]
fn pads_array_initializer_braces_when_on() {
    let s = style(|st| st.space_within_array_initializer_braces = true);
    assert_eq!(format_with(ARRAY_INIT, &s), ARRAY_INIT_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(ARRAY_INIT), ARRAY_INIT_DEFAULT_OUT);
}
