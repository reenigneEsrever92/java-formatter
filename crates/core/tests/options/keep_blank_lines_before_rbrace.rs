//! KEEP_BLANK_LINES_BEFORE_RBRACE — max blank lines kept before a closing `}`.
//! Fixtures live under tests/java/keep_blank_lines_before_rbrace/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const RBRACE: &str = include_str!("../java/keep_blank_lines_before_rbrace/rbrace.java");
const RBRACE_DEFAULT_OUT: &str =
    include_str!("../java/keep_blank_lines_before_rbrace/rbrace_default.out.java");
const RBRACE_CAP0_OUT: &str =
    include_str!("../java/keep_blank_lines_before_rbrace/rbrace_cap0.out.java");
const RBRACE_CAP1_OUT: &str =
    include_str!("../java/keep_blank_lines_before_rbrace/rbrace_cap1.out.java");
const RBRACE_CAP3_OUT: &str =
    include_str!("../java/keep_blank_lines_before_rbrace/rbrace_cap3.out.java");

fn keep(cap: u32) -> JavaStyle {
    style(|s| s.keep_blank_lines_before_rbrace = cap)
}

#[test]
fn default_cap_two_keeps_up_to_two_blank_lines_before_closing_braces() {
    // The absent-option default is the IntelliJ built-in cap of 2: the run of
    // 3 before the method brace is truncated, runs of 1 stay.
    assert_eq!(format(RBRACE), RBRACE_DEFAULT_OUT);
}

#[test]
fn cap_zero_removes_blank_lines_before_closing_braces() {
    assert_eq!(format_with(RBRACE, &keep(0)), RBRACE_CAP0_OUT);
}

#[test]
fn cap_one_keeps_at_most_one_blank_line_before_a_closing_brace() {
    assert_eq!(format_with(RBRACE, &keep(1)), RBRACE_CAP1_OUT);
}

#[test]
fn cap_three_keeps_the_whole_three_blank_line_run() {
    assert_eq!(format_with(RBRACE, &keep(3)), RBRACE_CAP3_OUT);
}
