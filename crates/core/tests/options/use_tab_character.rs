//! USE_TAB_CHARACTER — tab-character indentation output.
//! Fixtures live under tests/java/use_tab_character/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const TAB_INDENT: &str = include_str!("../java/use_tab_character/tab_indent.java");
const TAB_INDENT_OUT: &str = include_str!("../java/use_tab_character/tab_indent.out.java");
const TAB_INDENT_DEFAULT_OUT: &str =
    include_str!("../java/use_tab_character/tab_indent_default.out.java");

/// The settings the golden was produced with: tab output at a tight margin
/// with binary and call wrapping enabled so some lines wrap.
fn tab_style() -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.binary_operation_wrap = WrapStyle::WrapIfLong;
        s.call_parameters_wrap = WrapStyle::WrapIfLong;
        s.indent_size = 4;
        s.continuation_indent_size = 8;
        s.tab_size = 4;
        s.use_tab_character = true;
    })
}

#[test]
fn tab_indent_golden() {
    assert_eq!(format_with(TAB_INDENT, &tab_style()), TAB_INDENT_OUT);
}

#[test]
fn tab_indent_default() {
    // Without use_tab_character the default style indents with spaces and
    // emits no tabs at all.
    assert_eq!(format(TAB_INDENT), TAB_INDENT_DEFAULT_OUT);
}
