//! INDENT_BREAK_FROM_CASE — indenting `break` / `continue` / `return`
//! statements one level from the `case` / `default` label. Defaults to on.
//! Off, those jump statements line up with the label instead of the
//! statement indent. Whitespace/layout only (R5); formatting formatted
//! output is a no-op (R6).
//!
//! Fixtures live under tests/java/indent_break_from_case/.

use super::common::*;

const CASES: &str = include_str!("../java/indent_break_from_case/cases.java");
const CASES_DEFAULT_OUT: &str =
    include_str!("../java/indent_break_from_case/cases_default.out.java");
const CASES_OFF_OUT: &str = include_str!("../java/indent_break_from_case/cases.out.java");

#[test]
fn off_aligns_jump_statements_with_the_label() {
    let s = style(|st| st.indent_break_from_case = false);
    assert_eq!(format_with(CASES, &s), CASES_OFF_OUT);
    assert_eq!(format_with(CASES_OFF_OUT, &s), CASES_OFF_OUT);
}

#[test]
fn absent_option_keeps_jump_statements_indented_from_the_label() {
    assert_eq!(format(CASES), CASES_DEFAULT_OUT);
    assert_eq!(format(CASES_DEFAULT_OUT), CASES_DEFAULT_OUT);
}
