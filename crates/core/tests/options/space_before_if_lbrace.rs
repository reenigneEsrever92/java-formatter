//! SPACE_BEFORE_IF_LBRACE — space before the opening brace of an `if` body. Defaults to on.
//! Fixtures live under tests/java/space_before_if_lbrace/.

use super::common::*;

const IF_BODY: &str = include_str!("../java/space_before_if_lbrace/if_body.java");
const IF_BODY_OUT: &str = include_str!("../java/space_before_if_lbrace/if_body.out.java");
const IF_BODY_DEFAULT_OUT: &str =
    include_str!("../java/space_before_if_lbrace/if_body_default.out.java");

#[test]
fn off_tightens() {
    let s = style(|st| st.space_before_if_lbrace = false);
    assert_eq!(format_with(IF_BODY, &s), IF_BODY_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(IF_BODY), IF_BODY_DEFAULT_OUT);
}
