//! KEEP_BLANK_LINES_IN_DECLARATIONS — max blank lines kept between class-body members.
//! Fixtures live under tests/java/keep_blank_lines_in_declarations/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const MEMBERS: &str = include_str!("../java/keep_blank_lines_in_declarations/members.java");
const MEMBERS_DEFAULT_OUT: &str =
    include_str!("../java/keep_blank_lines_in_declarations/members_default.out.java");
const MEMBERS_CAP0_OUT: &str =
    include_str!("../java/keep_blank_lines_in_declarations/members_cap0.out.java");
const MEMBERS_CAP1_OUT: &str =
    include_str!("../java/keep_blank_lines_in_declarations/members_cap1.out.java");
const MEMBERS_CAP3_OUT: &str =
    include_str!("../java/keep_blank_lines_in_declarations/members_cap3.out.java");

fn keep(cap: u32) -> JavaStyle {
    style(|s| s.keep_blank_lines_in_declarations = cap)
}

#[test]
fn default_cap_two_keeps_up_to_two_blank_lines_between_members() {
    // The absent-option default is the IntelliJ built-in cap of 2; the
    // per-member `BLANK_LINES_*` minimums still apply on top (methods stay
    // separated by one blank even at cap 0).
    assert_eq!(format(MEMBERS), MEMBERS_DEFAULT_OUT);
}

#[test]
fn cap_zero_removes_blank_lines_between_members() {
    assert_eq!(format_with(MEMBERS, &keep(0)), MEMBERS_CAP0_OUT);
}

#[test]
fn cap_one_truncates_runs_to_one_blank_line() {
    assert_eq!(format_with(MEMBERS, &keep(1)), MEMBERS_CAP1_OUT);
}

#[test]
fn cap_three_keeps_runs_up_to_three_blank_lines() {
    assert_eq!(format_with(MEMBERS, &keep(3)), MEMBERS_CAP3_OUT);
}
