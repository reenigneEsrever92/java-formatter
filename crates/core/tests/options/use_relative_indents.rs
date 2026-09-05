//! USE_RELATIVE_INDENTS — with USE_TAB_CHARACTER, continuation indents are
//! measured from the construct's own indent level instead of added to the
//! full level columns.
//! Fixtures live under tests/java/use_relative_indents/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const REL: &str = include_str!("../java/use_relative_indents/relative_indents.java");
const REL_OUT: &str = include_str!("../java/use_relative_indents/relative_indents.out.java");
const REL_ABS_OUT: &str =
    include_str!("../java/use_relative_indents/relative_indents_abs.out.java");
const REL_DEFAULT_OUT: &str =
    include_str!("../java/use_relative_indents/relative_indents_default.out.java");

/// Tab output at `indent 4 / tab 4 / continuation 8` with a tight margin. The
/// continuation offset here is one indent unit over a level, so a relative
/// continuation sits one unit closer to the statement than the absolute one.
fn tab_style(relative: bool) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.binary_operation_wrap = WrapStyle::WrapIfLong;
        s.indent_size = 4;
        s.continuation_indent_size = 8;
        s.tab_size = 4;
        s.use_tab_character = true;
        s.use_relative_indents = relative;
    })
}

#[test]
fn use_relative_indents_shortens_continuations_by_one_level() {
    // Relative: the level-2 continuation lands at 12 (3 tabs) and the level-3
    // one at 16 (4 tabs).
    assert_eq!(format_with(REL, &tab_style(true)), REL_OUT);
}

#[test]
fn use_relative_indents_idempotent() {
    assert_eq!(format_with(REL_OUT, &tab_style(true)), REL_OUT);
}

#[test]
fn relative_indents_off_keeps_the_absolute_continuation_columns() {
    // Off (the default): the level-2 continuation sits at 16 (4 tabs) and the
    // level-3 one at 20 (5 tabs).
    assert_eq!(format_with(REL, &tab_style(false)), REL_ABS_OUT);
}

#[test]
fn relative_indents_inert_without_tab_character() {
    // The option is gated on USE_TAB_CHARACTER; the plain default style is
    // untouched.
    assert_eq!(format(REL), REL_DEFAULT_OUT);
}
