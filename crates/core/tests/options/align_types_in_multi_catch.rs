//! ALIGN_TYPES_IN_MULTI_CATCH — align wrapped multi-catch types under the
//! first type.
//! Fixtures live under tests/java/align_types_in_multi_catch/.
//!
//! When a multi-catch type list wraps (`MULTI_CATCH_TYPES_WRAP` engaged), the
//! continuation lines' `|` operators start at the first type's column with
//! the option on (the default) instead of at the continuation indent. The
//! fixtures use a continuation indent of 4 so the two layouts are visibly
//! different from the aligned column.

use super::common::*;
use java_formatter_core::config::WrapStyle;

const CATCH_LIST: &str = include_str!("../java/align_types_in_multi_catch/catch_list.java");
const CATCH_LIST_ALIGN_OUT: &str =
    include_str!("../java/align_types_in_multi_catch/catch_list_align.out.java");
const CATCH_LIST_CONT_OUT: &str =
    include_str!("../java/align_types_in_multi_catch/catch_list_cont.out.java");
const CATCH_LIST_ABSENT_OUT: &str =
    include_str!("../java/align_types_in_multi_catch/catch_list_absent.out.java");
const WRAPPED: &str = include_str!("../java/align_types_in_multi_catch/wrapped.java");
const WRAPPED_OUT: &str = include_str!("../java/align_types_in_multi_catch/wrapped.out.java");

fn wrap(align: bool) -> java_formatter_core::config::JavaStyle {
    style(|s| {
        s.right_margin = 40;
        s.multi_catch_types_wrap = WrapStyle::WrapAlways;
        s.align_types_in_multi_catch = align;
        // A narrower continuation indent keeps the two layouts distinct.
        s.continuation_indent_size = 4;
    })
}

#[test]
fn align_on_starts_continuation_lines_at_the_first_types_column() {
    // The `|` operators line up under the first type (which starts right
    // after `catch (`).
    assert_eq!(format_with(CATCH_LIST, &wrap(true)), CATCH_LIST_ALIGN_OUT);
}

#[test]
fn align_off_uses_the_plain_continuation_indent() {
    assert_eq!(format_with(CATCH_LIST, &wrap(false)), CATCH_LIST_CONT_OUT);
}

#[test]
fn absent_option_defaults_to_aligned() {
    // `ALIGN_TYPES_IN_MULTI_CATCH` defaults to true, so a scheme that sets
    // only the wrap engages the aligned layout (mirroring the record-alignment
    // default).
    let style = style(|s| {
        s.right_margin = 40;
        s.multi_catch_types_wrap = WrapStyle::WrapAlways;
        s.continuation_indent_size = 4;
    });
    assert_eq!(format_with(CATCH_LIST, &style), CATCH_LIST_ABSENT_OUT);
}

#[test]
fn reformatting_the_aligned_layout_is_a_no_op() {
    // A self-golden: the aligned fixture formats to itself under the option
    // (R6).
    assert_eq!(format_with(WRAPPED, &wrap(true)), WRAPPED_OUT);
}
