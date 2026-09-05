//! EXTENDS_KEYWORD_WRAP — put the `extends` / `implements` keyword of a
//! wrapped type-declaration list on its own line.
//! Fixtures live under tests/java/extends_keyword_wrap/.
//!
//! The keyword moves only when the list actually wraps (`extends_list_wrap`
//! is on and the list overflows); the flag governs both the `implements` and
//! the `extends` keywords.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_HEADERS: &str = include_str!("../java/extends_keyword_wrap/long_headers.java");
const LONG_HEADERS_KEYWORD_OUT: &str =
    include_str!("../java/extends_keyword_wrap/long_headers.out.java");
const LONG_HEADERS_KEYWORD_OFF_OUT: &str =
    include_str!("../java/extends_keyword_wrap/long_headers_keyword_off.out.java");

fn style_with(keyword_on_next_line: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.extends_list_wrap = WrapStyle::WrapIfLong;
        s.extends_keyword_wrap = keyword_on_next_line;
    })
}

#[test]
fn keyword_on_next_line_puts_implements_and_extends_on_their_own_lines() {
    assert_eq!(
        format_with(LONG_HEADERS, &style_with(true)),
        LONG_HEADERS_KEYWORD_OUT
    );
}

#[test]
fn keyword_off_keeps_the_keyword_with_the_first_type() {
    assert_eq!(
        format_with(LONG_HEADERS, &style_with(false)),
        LONG_HEADERS_KEYWORD_OFF_OUT
    );
}
