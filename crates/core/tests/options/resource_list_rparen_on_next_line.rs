//! RESOURCE_LIST_RPAREN_ON_NEXT_LINE — put the ')' of a wrapped resource list
//! on its own line.
//! Fixtures live under tests/java/resource_list_rparen_on_next_line/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const LONG_RESOURCES: &str =
    include_str!("../java/resource_list_rparen_on_next_line/long_resources.java");
const LONG_RESOURCES_RPAREN_OUT: &str =
    include_str!("../java/resource_list_rparen_on_next_line/long_resources.out.java");
const LONG_RESOURCES_RPAREN_OFF_OUT: &str =
    include_str!("../java/resource_list_rparen_on_next_line/long_resources_rparen_off.out.java");

#[test]
fn rparen_on_next_line_puts_the_rparen_on_its_own_line() {
    let style = style(|s| {
        s.right_margin = 60;
        s.resource_list_wrap = WrapStyle::WrapIfLong;
        s.resource_list_rparen_on_next_line = true;
    });
    assert_eq!(
        format_with(LONG_RESOURCES, &style),
        LONG_RESOURCES_RPAREN_OUT
    );
}

#[test]
fn rparen_off_attaches_the_rparen_to_the_last_resource() {
    let style = style(|s| {
        s.right_margin = 60;
        s.resource_list_wrap = WrapStyle::WrapIfLong;
    });
    assert_eq!(
        format_with(LONG_RESOURCES, &style),
        LONG_RESOURCES_RPAREN_OFF_OUT
    );
}
