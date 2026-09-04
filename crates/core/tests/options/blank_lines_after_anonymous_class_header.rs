//! BLANK_LINES_AFTER_ANONYMOUS_CLASS_HEADER — min blank lines after an
//! anonymous class header, before its first member.
//! Fixtures live under tests/java/blank_lines_after_anonymous_class_header/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const ANON: &str = include_str!("../java/blank_lines_after_anonymous_class_header/anon.java");
const ANON_DEFAULT_OUT: &str =
    include_str!("../java/blank_lines_after_anonymous_class_header/anon_default.out.java");
const ANON_2_OUT: &str =
    include_str!("../java/blank_lines_after_anonymous_class_header/anon_2.out.java");

fn after_anon_header(min: u32) -> JavaStyle {
    style(|s| s.blank_lines_after_anonymous_class_header = min)
}

#[test]
fn default_minimum_zero_starts_the_first_member_on_the_next_line() {
    // The absent-option default is the IntelliJ built-in minimum of 0.
    assert_eq!(format(ANON), ANON_DEFAULT_OUT);
}

#[test]
fn minimum_two_inserts_two_blank_lines_after_the_anonymous_class_header() {
    assert_eq!(format_with(ANON, &after_anon_header(2)), ANON_2_OUT);
}
