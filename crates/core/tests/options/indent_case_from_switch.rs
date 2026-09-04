//! INDENT_CASE_FROM_SWITCH — indenting `case` / `default` labels from the
//! `switch`. Defaults to on: labels sit one level below the `switch` keyword
//! and their statements a further level. Off, labels sit at the `switch`
//! indent (statements one level below the `switch`). Whitespace/layout only
//! (R5); formatting formatted output is a no-op (R6).
//!
//! Fixtures live under tests/java/indent_case_from_switch/.

use super::common::*;

const CASES: &str = include_str!("../java/indent_case_from_switch/cases.java");
const CASES_DEFAULT_OUT: &str =
    include_str!("../java/indent_case_from_switch/cases_default.out.java");
const CASES_OFF_OUT: &str = include_str!("../java/indent_case_from_switch/cases.out.java");

#[test]
fn off_puts_labels_at_the_switch_indent() {
    let s = style(|st| st.indent_case_from_switch = false);
    assert_eq!(format_with(CASES, &s), CASES_OFF_OUT);
    assert_eq!(format_with(CASES_OFF_OUT, &s), CASES_OFF_OUT);
}

#[test]
fn absent_option_uses_default_label_indent() {
    assert_eq!(format(CASES), CASES_DEFAULT_OUT);
    assert_eq!(format(CASES_DEFAULT_OUT), CASES_DEFAULT_OUT);
}
