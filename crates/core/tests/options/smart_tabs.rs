//! SMART_TABS — with USE_TAB_CHARACTER, tab characters are emitted only for
//! indentation that lands exactly on a tab stop; other indents use spaces.
//! Fixtures live under tests/java/smart_tabs/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const TAB_INDENT: &str = include_str!("../java/smart_tabs/tab_indent.java");
const TAB_INDENT_OUT: &str = include_str!("../java/smart_tabs/tab_indent.out.java");
const TAB_INDENT_NOSMART_OUT: &str = include_str!("../java/smart_tabs/tab_indent_nosmart.out.java");
const TAB_INDENT_DEFAULT_OUT: &str = include_str!("../java/smart_tabs/tab_indent_default.out.java");

/// Tab output at `indent 4 / tab 4` with a continuation indent of 6 (not a
/// whole number of tabs) and a tight margin so wrapped lines exercise the
/// continuation columns that cannot land on a tab stop.
fn tab_style(smart_tabs: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.binary_operation_wrap = WrapStyle::WrapIfLong;
        s.call_parameters_wrap = WrapStyle::WrapIfLong;
        s.indent_size = 4;
        s.continuation_indent_size = 6;
        s.tab_size = 4;
        s.use_tab_character = true;
        s.smart_tabs = smart_tabs;
    })
}

#[test]
fn smart_tabs_emit_spaces_for_off_stop_continuations() {
    // Continuation widths 14 / 18 are not multiples of the 4-column tab stop,
    // so smart tabs emits pure spaces there while block indents stay tabs.
    assert_eq!(format_with(TAB_INDENT, &tab_style(true)), TAB_INDENT_OUT);
}

#[test]
fn smart_tabs_idempotent() {
    assert_eq!(
        format_with(TAB_INDENT_OUT, &tab_style(true)),
        TAB_INDENT_OUT
    );
}

#[test]
fn smart_tabs_off_keeps_the_tab_stop_mix() {
    // The default (smart tabs off) keeps today's tabs-plus-spaces mix for the
    // same 14 / 18-column continuation widths.
    assert_eq!(
        format_with(TAB_INDENT, &tab_style(false)),
        TAB_INDENT_NOSMART_OUT
    );
}

#[test]
fn smart_tabs_inert_without_use_tab_character() {
    // Without USE_TAB_CHARACTER the default style is all spaces, untouched.
    assert_eq!(format(TAB_INDENT), TAB_INDENT_DEFAULT_OUT);
}
