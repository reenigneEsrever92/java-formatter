//! CONTINUATION_INDENT_SIZE — extra indent for wrapped continuation lines.
//! Fixtures live under tests/java/continuation_indent_size/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const CONTINUATION: &str = include_str!("../java/continuation_indent_size/continuation.java");
const CONTINUATION_OUT: &str =
    include_str!("../java/continuation_indent_size/continuation.out.java");
const CONTINUATION_8_OUT: &str =
    include_str!("../java/continuation_indent_size/continuation_8.out.java");

/// A tight margin plus binary wrapping so the long sum breaks onto
/// continuation lines.
fn wrapped(continuation_indent_size: u32) -> JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.binary_operation_wrap = WrapStyle::WrapIfLong;
        s.continuation_indent_size = continuation_indent_size;
    })
}

#[test]
fn continuation_indent_four_indents_wrapped_lines_by_four() {
    assert_eq!(format_with(CONTINUATION, &wrapped(4)), CONTINUATION_OUT);
}

#[test]
fn default_continuation_indent_eight_is_used() {
    assert_eq!(format_with(CONTINUATION, &wrapped(8)), CONTINUATION_8_OUT);
}
