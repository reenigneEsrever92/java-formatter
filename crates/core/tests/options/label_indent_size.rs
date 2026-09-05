//! LABEL_INDENT_SIZE — relative indent added to the statement indent for
//! `label:` lines.
//! Fixtures live under tests/java/label_indent_size/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const LABELS: &str = include_str!("../java/label_indent_size/labels.java");
const LABELS_OUT: &str = include_str!("../java/label_indent_size/labels.out.java");
const LABELS_DEFAULT_OUT: &str = include_str!("../java/label_indent_size/labels_default.out.java");

/// A relative label indent of 4: the label sits at the statement indent plus
/// 4 columns (the outer label at 12, the nested one at 16 — nesting counts).
fn style_with() -> JavaStyle {
    style(|s| s.label_indent_size = 4)
}

#[test]
fn label_indent_size_shifts_labels_relative_to_the_statement() {
    assert_eq!(format_with(LABELS, &style_with()), LABELS_OUT);
}

#[test]
fn label_indent_size_idempotent() {
    assert_eq!(format_with(LABELS_OUT, &style_with()), LABELS_OUT);
}

#[test]
fn absent_label_indent_size_keeps_labels_at_the_statement_indent() {
    assert_eq!(format(LABELS), LABELS_DEFAULT_OUT);
}
