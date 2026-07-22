//! ASSIGNMENT_WRAP — wrapping behaviour for assignment statements and variable /
//! field initialisers.
//! Fixtures live under tests/java/assignment_wrap/.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const LONG_INIT_DEFAULT: &str = include_str!("../java/assignment_wrap/long_init_default.java");
const LONG_INIT_DEFAULT_OUT: &str =
    include_str!("../java/assignment_wrap/long_init_default.out.java");
const LONG_INIT_WRAP_IF_LONG: &str =
    include_str!("../java/assignment_wrap/long_init_wrap_if_long.java");
const LONG_INIT_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/assignment_wrap/long_init_wrap_if_long.out.java");
const LONG_INIT_DO_NOT_WRAP: &str =
    include_str!("../java/assignment_wrap/long_init_do_not_wrap.java");
const LONG_INIT_DO_NOT_WRAP_OUT: &str =
    include_str!("../java/assignment_wrap/long_init_do_not_wrap.out.java");
const SHORT_INIT_WRAP_IF_LONG: &str =
    include_str!("../java/assignment_wrap/short_init_wrap_if_long.java");
const SHORT_INIT_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/assignment_wrap/short_init_wrap_if_long.out.java");
const SHORT_INIT_WRAP_ALWAYS: &str =
    include_str!("../java/assignment_wrap/short_init_wrap_always.java");
const SHORT_INIT_WRAP_ALWAYS_OUT: &str =
    include_str!("../java/assignment_wrap/short_init_wrap_always.out.java");
const FIELD_LONG_WRAP_IF_LONG: &str =
    include_str!("../java/assignment_wrap/field_long_wrap_if_long.java");
const FIELD_LONG_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/assignment_wrap/field_long_wrap_if_long.out.java");
const COMPOUND_WRAP_IF_LONG: &str =
    include_str!("../java/assignment_wrap/compound_wrap_if_long.java");
const COMPOUND_WRAP_IF_LONG_OUT: &str =
    include_str!("../java/assignment_wrap/compound_wrap_if_long.out.java");

#[test]
fn default_style_keeps_long_assignments_on_one_line() {
    assert_eq!(format(LONG_INIT_DEFAULT), LONG_INIT_DEFAULT_OUT);
}

#[test]
fn wrap_if_long_moves_rhs_to_next_line() {
    let style = style(|s| s.assignment_wrap = WrapStyle::WrapIfLong);
    assert_eq!(
        format_with(LONG_INIT_WRAP_IF_LONG, &style),
        LONG_INIT_WRAP_IF_LONG_OUT
    );
}

#[test]
fn do_not_wrap_keeps_single_line_even_when_long() {
    let style = style(|s| {
        s.right_margin = 40;
        s.assignment_wrap = WrapStyle::DoNotWrap;
    });
    assert_eq!(
        format_with(LONG_INIT_DO_NOT_WRAP, &style),
        LONG_INIT_DO_NOT_WRAP_OUT
    );
}

#[test]
fn wrap_if_long_keeps_short_assignments_flat() {
    let style = style(|s| s.assignment_wrap = WrapStyle::WrapIfLong);
    assert_eq!(
        format_with(SHORT_INIT_WRAP_IF_LONG, &style),
        SHORT_INIT_WRAP_IF_LONG_OUT
    );
}

#[test]
fn wrap_always_breaks_even_short_assignments() {
    let style = style(|s| s.assignment_wrap = WrapStyle::WrapAlways);
    assert_eq!(
        format_with(SHORT_INIT_WRAP_ALWAYS, &style),
        SHORT_INIT_WRAP_ALWAYS_OUT
    );
}

#[test]
fn field_initialiser_is_wrapped_too() {
    let style = style(|s| {
        s.right_margin = 60;
        s.assignment_wrap = WrapStyle::WrapIfLong;
    });
    assert_eq!(
        format_with(FIELD_LONG_WRAP_IF_LONG, &style),
        FIELD_LONG_WRAP_IF_LONG_OUT
    );
}

#[test]
fn compound_assignment_is_preserved() {
    let style = style(|s| s.assignment_wrap = WrapStyle::WrapIfLong);
    assert_eq!(
        format_with(COMPOUND_WRAP_IF_LONG, &style),
        COMPOUND_WRAP_IF_LONG_OUT
    );
}
