//! SPACE_BEFORE_TRY_PARENTHESES — space between `try` and its resource list. Defaults to on.
//! Fixtures live under tests/java/space_before_try_parentheses/.

use super::common::*;

const TRY_RESOURCES: &str = include_str!("../java/space_before_try_parentheses/try_resources.java");
const TRY_RESOURCES_OUT: &str = include_str!("../java/space_before_try_parentheses/try_resources.out.java");
const TRY_RESOURCES_DEFAULT_OUT: &str =
    include_str!("../java/space_before_try_parentheses/try_resources_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_try_parentheses = false);
    assert_eq!(format_with(TRY_RESOURCES, &s), TRY_RESOURCES_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(TRY_RESOURCES), TRY_RESOURCES_DEFAULT_OUT);
}
