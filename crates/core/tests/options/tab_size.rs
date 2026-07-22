//! TAB_SIZE — tab-stop width used by tab-character indentation.
//! Fixtures live under tests/java/tab_size/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const TAB_INDENT: &str = include_str!("../java/tab_size/tab_indent.java");
const TAB_INDENT_TAB4_OUT: &str = include_str!("../java/tab_size/tab_indent_tab4.out.java");
const TAB_INDENT_TAB2_OUT: &str = include_str!("../java/tab_size/tab_indent_tab2.out.java");
const TAB_INDENT_TAB8_OUT: &str = include_str!("../java/tab_size/tab_indent_tab8.out.java");

/// Tab output at `indent_size 4` / `continuation_indent_size 8` and a tight
/// margin so wrapped lines exercise continuation indentation too.
fn tab_style(tab_size: u32) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.binary_operation_wrap = WrapStyle::WrapIfLong;
        s.call_parameters_wrap = WrapStyle::WrapIfLong;
        s.indent_size = 4;
        s.continuation_indent_size = 8;
        s.tab_size = tab_size;
        s.use_tab_character = true;
    })
}

#[test]
fn tab_indent_tab4() {
    // indent 4 / tab 4 = one tab per level.
    assert_eq!(format_with(TAB_INDENT, &tab_style(4)), TAB_INDENT_TAB4_OUT);
}

#[test]
fn tab_indent_tab2() {
    // indent 4 / tab 2 = two tabs per level.
    assert_eq!(format_with(TAB_INDENT, &tab_style(2)), TAB_INDENT_TAB2_OUT);
}

#[test]
fn tab_indent_tab8() {
    // indent 4 / tab 8 = spaces for the remainder of each level, with tabs
    // on the cumulative width where a full tab stop fits.
    assert_eq!(format_with(TAB_INDENT, &tab_style(8)), TAB_INDENT_TAB8_OUT);
}
