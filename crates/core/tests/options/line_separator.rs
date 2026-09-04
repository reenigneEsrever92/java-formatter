//! LINE_SEPARATOR — the line separator emitted at every line end.
//!
//! Fixtures live under tests/java/line_separator/.

use super::common::*;
use java_formatter_core::config::LineSeparator;

const LINES: &str = include_str!("../java/line_separator/lines.java");
const LINES_DEFAULT_OUT: &str = include_str!("../java/line_separator/lines_default.out.java");
const LINES_CRLF_OUT: &str = include_str!("../java/line_separator/lines_crlf.out.java");
const LINES_CR_OUT: &str = include_str!("../java/line_separator/lines_cr.out.java");

#[test]
fn default_system_separator_is_lf() {
    // `System` resolves to LF on the test hosts; absent-option default check.
    assert_eq!(format(LINES), LINES_DEFAULT_OUT);
}

#[test]
fn crlf_separator_ends_every_line_including_the_last() {
    let style = style(|s| s.line_separator = LineSeparator::Crlf);
    assert_eq!(format_with(LINES, &style), LINES_CRLF_OUT);
}

#[test]
fn cr_separator_ends_every_line_including_the_last() {
    let style = style(|s| s.line_separator = LineSeparator::Cr);
    assert_eq!(format_with(LINES, &style), LINES_CR_OUT);
}
