//! SPACE_WITHIN_EMPTY_ARRAY_INITIALIZER_BRACES — `{ }` vs `{}`.
//! Fixtures live under tests/java/space_within_empty_array_initializer_braces/.

use super::common::*;

const EMPTY_ARRAY_INIT: &str =
    include_str!("../java/space_within_empty_array_initializer_braces/empty_array_init.java");
const EMPTY_ARRAY_INIT_OUT: &str =
    include_str!("../java/space_within_empty_array_initializer_braces/empty_array_init.out.java");
const EMPTY_ARRAY_INIT_DEFAULT_OUT: &str = include_str!(
    "../java/space_within_empty_array_initializer_braces/empty_array_init_default.out.java"
);

#[test]
fn pads_empty_array_initializer_braces_when_on() {
    let s = style(|st| st.space_within_empty_array_initializer_braces = true);
    assert_eq!(format_with(EMPTY_ARRAY_INIT, &s), EMPTY_ARRAY_INIT_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(EMPTY_ARRAY_INIT), EMPTY_ARRAY_INIT_DEFAULT_OUT);
}
