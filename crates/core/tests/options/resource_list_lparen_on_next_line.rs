//! RESOURCE_LIST_LPAREN_ON_NEXT_LINE — put the '(' of a wrapped resource list
//! on its own line.
//! Fixtures live under tests/java/resource_list_lparen_on_next_line/.
//!
//! The layout effect of the option only shows once the closing paren also sits
//! on its own line (`resource_list_rparen_on_next_line`); with the attached
//! `)` the formatter emits the same layout for both lparen settings, so the
//! rparen option is pinned on in this file to isolate the lparen behaviour.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const LONG_RESOURCES: &str =
    include_str!("../java/resource_list_lparen_on_next_line/long_resources.java");
const LONG_RESOURCES_LPAREN_OUT: &str =
    include_str!("../java/resource_list_lparen_on_next_line/long_resources.out.java");
const LONG_RESOURCES_LPAREN_OFF_OUT: &str =
    include_str!("../java/resource_list_lparen_on_next_line/long_resources_lparen_off.out.java");

fn wrapped_with(lparen_on_next_line: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.resource_list_wrap = WrapStyle::WrapIfLong;
        s.resource_list_lparen_on_next_line = lparen_on_next_line;
        s.resource_list_rparen_on_next_line = true;
    })
}

#[test]
fn lparen_on_next_line_breaks_after_the_lparen() {
    assert_eq!(
        format_with(LONG_RESOURCES, &wrapped_with(true)),
        LONG_RESOURCES_LPAREN_OUT
    );
}

#[test]
fn lparen_off_keeps_the_first_resource_on_the_lparen_line() {
    assert_eq!(
        format_with(LONG_RESOURCES, &wrapped_with(false)),
        LONG_RESOURCES_LPAREN_OFF_OUT
    );
}
