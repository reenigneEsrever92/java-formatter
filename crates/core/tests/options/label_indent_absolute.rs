//! LABEL_INDENT_ABSOLUTE — label lines are indented by LABEL_INDENT_SIZE from
//! the left margin regardless of nesting.
//! Fixtures live under tests/java/label_indent_absolute/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const LABELS: &str = include_str!("../java/label_indent_absolute/labels.java");
const LABELS_OUT: &str = include_str!("../java/label_indent_absolute/labels.out.java");
const LABELS_DEFAULT_OUT: &str =
    include_str!("../java/label_indent_absolute/labels_default.out.java");

/// An absolute label indent of 12: both the outer (statement at column 8) and
/// the nested (statement at column 12) labels sit at column 12 from the
/// margin, independent of nesting.
fn style_with() -> JavaStyle {
    style(|s| {
        s.label_indent_size = 12;
        s.label_indent_absolute = true;
    })
}

#[test]
fn label_indent_absolute_pins_labels_to_the_margin_width() {
    assert_eq!(format_with(LABELS, &style_with()), LABELS_OUT);
}

#[test]
fn label_indent_absolute_idempotent() {
    assert_eq!(format_with(LABELS_OUT, &style_with()), LABELS_OUT);
}

#[test]
fn absent_label_indent_absolute_keeps_labels_at_the_statement_indent() {
    assert_eq!(format(LABELS), LABELS_DEFAULT_OUT);
}
