//! KEEP_LINE_BREAKS — keeping (true) or reflowing (false) an existing
//! multi-line call or parameter list that would otherwise fit on one line.
//!
//! Fixtures live under tests/java/keep_line_breaks/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const CALL: &str = include_str!("../java/keep_line_breaks/call.java");
const CALL_OUT: &str = include_str!("../java/keep_line_breaks/call.out.java");
const CALL_REFLOW_OUT: &str = include_str!("../java/keep_line_breaks/call_reflow.out.java");
const PARAMS: &str = include_str!("../java/keep_line_breaks/params.java");
const PARAMS_OUT: &str = include_str!("../java/keep_line_breaks/params.out.java");
const PARAMS_REFLOW_OUT: &str = include_str!("../java/keep_line_breaks/params_reflow.out.java");

#[test]
fn default_keeps_the_multiline_call() {
    // Absent-option default: keep_line_breaks is true, so the call's source
    // break survives even though the flat form fits.
    assert_eq!(format(CALL), CALL_OUT);
}

#[test]
fn keep_line_breaks_preserves_the_multiline_call() {
    let style = style(|_| {});
    assert_eq!(format_with(CALL, &style), CALL_OUT);
}

#[test]
fn keep_line_breaks_false_reflows_the_call() {
    let style = keep_false();
    assert_eq!(format_with(CALL, &style), CALL_REFLOW_OUT);
}

#[test]
fn default_keeps_the_multiline_parameter_list() {
    assert_eq!(format(PARAMS), PARAMS_OUT);
}

#[test]
fn keep_line_breaks_false_reflows_the_parameter_list() {
    let style = keep_false();
    assert_eq!(format_with(PARAMS, &style), PARAMS_REFLOW_OUT);
}

fn keep_false() -> JavaStyle {
    style(|s| s.keep_line_breaks = false)
}
