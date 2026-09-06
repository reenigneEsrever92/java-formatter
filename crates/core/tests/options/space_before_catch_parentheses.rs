//! SPACE_BEFORE_CATCH_PARENTHESES — space between `catch` and its parameter. Defaults to on.
//! Fixtures live under tests/java/space_before_catch_parentheses/.

use super::common::*;

const TRY_CATCH: &str = include_str!("../java/space_before_catch_parentheses/try_catch.java");
const TRY_CATCH_OUT: &str =
    include_str!("../java/space_before_catch_parentheses/try_catch.out.java");
const TRY_CATCH_DEFAULT_OUT: &str =
    include_str!("../java/space_before_catch_parentheses/try_catch_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_catch_parentheses = false);
    assert_eq!(format_with(TRY_CATCH, &s), TRY_CATCH_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(TRY_CATCH), TRY_CATCH_DEFAULT_OUT);
}
