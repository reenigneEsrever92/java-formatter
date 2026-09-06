//! `DECONSTRUCTION_LIST_WRAP` — wrapping of record-pattern component lists in
//! `case` labels. `0` keeps a single long label, `1` (wrap if long) and `5`
//! (chop down if long) wrap an over-margin list identically (a component is
//! atomic), `2` (wrap always) wraps even a label that fits. The absent option
//! keeps the compact single-line form; both the arrow-form rules and the
//! colon-form groups wrap their labels. Expression-position switches fall back
//! to the multi-line layout when the single line cannot represent a wrapped
//! label. Unmodelled labels (`case String s`, a guarded record pattern,
//! comma-separated constants) round-trip verbatim (R4).
//! Fixtures live under tests/java/deconstruction_list_wrap/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const CASE_WRAP: &str = include_str!("../java/deconstruction_list_wrap/case_wrap.java");
const CASE_WRAP_DEFAULT_OUT: &str =
    include_str!("../java/deconstruction_list_wrap/case_wrap_default.out.java");
const CASE_WRAP_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/deconstruction_list_wrap/case_wrap_do_not_wrap.out.java");
const CASE_WRAP_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/deconstruction_list_wrap/case_wrap_wrap_if_long.out.java");
const CASE_WRAP_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/deconstruction_list_wrap/case_wrap_wrap_always.out.java");
const CASE_WRAP_CHOP_DOWN_OUT: &str =
    include_str!("../java/deconstruction_list_wrap/case_wrap_chop_down.out.java");
const CASE_WRAP_SELF: &str = include_str!("../java/deconstruction_list_wrap/case_wrap_self.java");
const CASE_WRAP_SELF_OUT: &str =
    include_str!("../java/deconstruction_list_wrap/case_wrap_self.out.java");
const CASE_GROUP: &str = include_str!("../java/deconstruction_list_wrap/case_group.java");
const CASE_GROUP_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/deconstruction_list_wrap/case_group_wrap_if_long.out.java");
const CASE_EXPR: &str = include_str!("../java/deconstruction_list_wrap/case_expr.java");
const CASE_EXPR_DEFAULT_OUT: &str =
    include_str!("../java/deconstruction_list_wrap/case_expr_default.out.java");
const VERBATIM: &str = include_str!("../java/deconstruction_list_wrap/verbatim.java");
const VERBATIM_OUT: &str = include_str!("../java/deconstruction_list_wrap/verbatim.out.java");

/// A narrow margin so the three-component label's list overflows.
fn narrow(wrap: WrapStyle) -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.deconstruction_list_wrap = wrap;
    })
}

#[test]
fn absent_option_keeps_the_compact_single_line_label() {
    // The option defaults to do-not-wrap, so the label stays as written.
    assert_eq!(format(CASE_WRAP), CASE_WRAP_DEFAULT_OUT);
}

#[test]
fn do_not_wrap_keeps_a_single_long_line() {
    let s = narrow(WrapStyle::DoNotWrap);
    assert_eq!(format_with(CASE_WRAP, &s), CASE_WRAP_DO_NOT_WRAP_OUT);
}

#[test]
fn wrap_if_long_breaks_an_over_margin_list() {
    let s = narrow(WrapStyle::WrapIfLong);
    assert_eq!(format_with(CASE_WRAP, &s), CASE_WRAP_WRAP_IF_LONG_OUT);
}

#[test]
fn wrap_always_breaks_a_list_that_would_fit() {
    // The default margin would fit the flat label; wrap-always still breaks it.
    let s = style(|st| st.deconstruction_list_wrap = WrapStyle::WrapAlways);
    assert_eq!(format_with(CASE_WRAP, &s), CASE_WRAP_WRAP_ALWAYS_OUT);
}

#[test]
fn chop_down_wraps_identically_to_wrap_if_long() {
    // A deconstruction component is atomic, so codes 1 and 5 agree.
    let s = narrow(WrapStyle::ChopDownIfLong);
    assert_eq!(format_with(CASE_WRAP, &s), CASE_WRAP_CHOP_DOWN_OUT);
}

#[test]
fn reformatting_the_wrapped_label_is_a_no_op() {
    let s = narrow(WrapStyle::WrapIfLong);
    assert_eq!(format_with(CASE_WRAP_SELF, &s), CASE_WRAP_SELF_OUT);
}

#[test]
fn wraps_a_colon_form_group_label_too() {
    // The colon-form group label lays out through the same wrap decision; the
    // ':' glues after the wrapped label's last line.
    let s = narrow(WrapStyle::WrapIfLong);
    assert_eq!(format_with(CASE_GROUP, &s), CASE_GROUP_WRAP_IF_LONG_OUT);
}

#[test]
fn compact_expression_switch_keeps_its_one_line_form() {
    // The whole switch fits, so the modelled label stays on the single line.
    assert_eq!(format(CASE_EXPR), CASE_EXPR_DEFAULT_OUT);
}

#[test]
fn unmodelled_labels_round_trip_verbatim() {
    // A type pattern, a guarded record pattern and comma-separated constants
    // are outside the modelled shape and keep their source text (R4).
    assert_eq!(format(VERBATIM), VERBATIM_OUT);
}
