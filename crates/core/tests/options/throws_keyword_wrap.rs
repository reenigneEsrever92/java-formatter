//! THROWS_KEYWORD_WRAP — put the `throws` keyword of a wrapped throws list on
//! its own line.
//! Fixtures live under tests/java/throws_keyword_wrap/.
//!
//! The keyword moves only when the list actually wraps (`throws_list_wrap` is
//! on and the list overflows).

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_THROWS: &str = include_str!("../java/throws_keyword_wrap/long_throws.java");
const LONG_THROWS_KEYWORD_OUT: &str =
    include_str!("../java/throws_keyword_wrap/long_throws.out.java");
const LONG_THROWS_KEYWORD_OFF_OUT: &str =
    include_str!("../java/throws_keyword_wrap/long_throws_keyword_off.out.java");

fn style_with(keyword_on_next_line: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.throws_list_wrap = WrapStyle::WrapIfLong;
        s.throws_keyword_wrap = keyword_on_next_line;
    })
}

#[test]
fn keyword_on_next_line_puts_throws_on_its_own_line() {
    assert_eq!(
        format_with(LONG_THROWS, &style_with(true)),
        LONG_THROWS_KEYWORD_OUT
    );
}

#[test]
fn keyword_off_keeps_throws_with_the_first_exception() {
    assert_eq!(
        format_with(LONG_THROWS, &style_with(false)),
        LONG_THROWS_KEYWORD_OFF_OUT
    );
}
