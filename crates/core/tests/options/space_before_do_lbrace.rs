//! SPACE_BEFORE_DO_LBRACE — space between `do` and its body's opening brace. Defaults to on.
//! Fixtures live under tests/java/space_before_do_lbrace/.

use super::common::*;

const DO_BODY: &str = include_str!("../java/space_before_do_lbrace/do_body.java");
const DO_BODY_OUT: &str = include_str!("../java/space_before_do_lbrace/do_body.out.java");
const DO_BODY_DEFAULT_OUT: &str =
    include_str!("../java/space_before_do_lbrace/do_body_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_do_lbrace = false);
    assert_eq!(format_with(DO_BODY, &s), DO_BODY_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(DO_BODY), DO_BODY_DEFAULT_OUT);
}
