//! CASE_STATEMENT_ON_NEW_LINE — placing the statement after a `case` /
//! `default` label on a new line. Defaults to on. Off, the group's first
//! (single-line) statement is joined onto the label's line
//! (`case 1: foo();`); following statements still start their own lines at
//! the statement indent. Whitespace/layout only (R5); formatting formatted
//! output is a no-op (R6).
//!
//! Fixtures live under tests/java/case_statement_on_new_line/.

use super::common::*;

const CASES: &str = include_str!("../java/case_statement_on_new_line/cases.java");
const CASES_DEFAULT_OUT: &str =
    include_str!("../java/case_statement_on_new_line/cases_default.out.java");
const CASES_OFF_OUT: &str = include_str!("../java/case_statement_on_new_line/cases.out.java");

#[test]
fn off_joins_the_first_statement_onto_the_label_line() {
    let s = style(|st| st.case_statement_on_new_line = false);
    assert_eq!(format_with(CASES, &s), CASES_OFF_OUT);
    assert_eq!(format_with(CASES_OFF_OUT, &s), CASES_OFF_OUT);
}

#[test]
fn absent_option_puts_statements_on_new_lines() {
    assert_eq!(format(CASES), CASES_DEFAULT_OUT);
    assert_eq!(format(CASES_DEFAULT_OUT), CASES_DEFAULT_OUT);
}
