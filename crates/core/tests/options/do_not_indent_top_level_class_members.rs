//! DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS — members of a top-level class sit
//! at the class declaration indent (no extra level).
//! Fixtures live under tests/java/do_not_indent_top_level_class_members/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const MEMBERS: &str = include_str!("../java/do_not_indent_top_level_class_members/members.java");
const MEMBERS_OUT: &str =
    include_str!("../java/do_not_indent_top_level_class_members/members.out.java");
const MEMBERS_DEFAULT_OUT: &str =
    include_str!("../java/do_not_indent_top_level_class_members/members_default.out.java");

/// The option on: top-level members at column 0; a nested class keeps its own
/// members indented one level (it is not a top-level class).
fn style_on() -> JavaStyle {
    style(|s| s.do_not_indent_top_level_class_members = true)
}

#[test]
fn do_not_indent_top_level_class_members_flush_members() {
    assert_eq!(format_with(MEMBERS, &style_on()), MEMBERS_OUT);
}

#[test]
fn do_not_indent_top_level_class_members_idempotent() {
    assert_eq!(format_with(MEMBERS_OUT, &style_on()), MEMBERS_OUT);
}

#[test]
fn absent_do_not_indent_top_level_class_members_keeps_today_indent() {
    assert_eq!(format(MEMBERS), MEMBERS_DEFAULT_OUT);
}
