//! KEEP_BLANK_LINES_IN_CODE — max blank lines kept between statements inside code blocks.
//! Fixtures live under tests/java/keep_blank_lines_in_code/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const RUNS: &str = include_str!("../java/keep_blank_lines_in_code/runs.java");
const RUNS_DEFAULT_OUT: &str =
    include_str!("../java/keep_blank_lines_in_code/runs_default.out.java");
const RUNS_CAP0_OUT: &str = include_str!("../java/keep_blank_lines_in_code/runs_cap0.out.java");
const RUNS_CAP1_OUT: &str = include_str!("../java/keep_blank_lines_in_code/runs_cap1.out.java");
const RUNS_CAP3_OUT: &str = include_str!("../java/keep_blank_lines_in_code/runs_cap3.out.java");

fn keep(cap: u32) -> JavaStyle {
    style(|s| s.keep_blank_lines_in_code = cap)
}

#[test]
fn default_cap_two_keeps_up_to_two_blank_lines_in_code() {
    // The absent-option default is the IntelliJ built-in cap of 2: runs of 1
    // survive, runs of 3 are truncated to 2.
    assert_eq!(format(RUNS), RUNS_DEFAULT_OUT);
}

#[test]
fn cap_zero_removes_blank_lines_between_statements() {
    assert_eq!(format_with(RUNS, &keep(0)), RUNS_CAP0_OUT);
}

#[test]
fn cap_one_truncates_runs_to_one_blank_line() {
    assert_eq!(format_with(RUNS, &keep(1)), RUNS_CAP1_OUT);
}

#[test]
fn cap_three_keeps_runs_up_to_three_blank_lines() {
    assert_eq!(format_with(RUNS, &keep(3)), RUNS_CAP3_OUT);
}
