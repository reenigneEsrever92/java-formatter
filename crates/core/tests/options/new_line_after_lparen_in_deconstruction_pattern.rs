//! NEW_LINE_AFTER_LPAREN_IN_DECONSTRUCTION_PATTERN — with the '(' of a wrapped
//! record pattern on its own line every component starts its own line below
//! it; off, the first component stays on the `case` line after the '('.
//! Fixtures live under tests/java/new_line_after_lparen_in_deconstruction_pattern/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const CASE_WRAP: &str =
    include_str!("../java/new_line_after_lparen_in_deconstruction_pattern/case_wrap.java");
const CASE_WRAP_LPAREN_ON_OUT: &str = include_str!(
    "../java/new_line_after_lparen_in_deconstruction_pattern/case_wrap_lparen_on.out.java"
);
const CASE_WRAP_LPAREN_OFF_OUT: &str = include_str!(
    "../java/new_line_after_lparen_in_deconstruction_pattern/case_wrap_lparen_off.out.java"
);

fn wrapped(lparen_on: bool) -> JavaStyle {
    style(|s| {
        s.deconstruction_list_wrap = WrapStyle::WrapAlways;
        s.new_line_after_lparen_in_deconstruction_pattern = lparen_on;
    })
}

#[test]
fn lparen_on_starts_every_component_on_its_own_line() {
    // The '(' stays on the case line and each component begins below it.
    let style = wrapped(true);
    assert_eq!(format_with(CASE_WRAP, &style), CASE_WRAP_LPAREN_ON_OUT);
}

#[test]
fn lparen_off_keeps_the_first_component_inline_after_the_paren() {
    let style = wrapped(false);
    assert_eq!(format_with(CASE_WRAP, &style), CASE_WRAP_LPAREN_OFF_OUT);
}
