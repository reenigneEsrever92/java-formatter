//! ALIGN_MULTILINE_DECONSTRUCTION_LIST_COMPONENTS — align the components of a
//! wrapped record-pattern `case` label under the first component instead of at
//! the continuation indent.
//! Fixtures live under tests/java/align_multiline_deconstruction_list_components/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const CASE_WRAP: &str =
    include_str!("../java/align_multiline_deconstruction_list_components/case_wrap.java");
const CASE_WRAP_ALIGN_OUT: &str =
    include_str!("../java/align_multiline_deconstruction_list_components/case_wrap_align.out.java");
const CASE_WRAP_CONT_OUT: &str =
    include_str!("../java/align_multiline_deconstruction_list_components/case_wrap_cont.out.java");

fn wrapped(align: bool) -> JavaStyle {
    style(|s| {
        s.deconstruction_list_wrap = WrapStyle::WrapAlways;
        s.align_multiline_deconstruction_list_components = align;
    })
}

#[test]
fn align_on_pads_wrapped_components_under_the_first_component() {
    // Components sit one column right of the '(' (the default: align on).
    let style = wrapped(true);
    assert_eq!(format_with(CASE_WRAP, &style), CASE_WRAP_ALIGN_OUT);
}

#[test]
fn align_off_uses_the_continuation_indent() {
    let style = wrapped(false);
    assert_eq!(format_with(CASE_WRAP, &style), CASE_WRAP_CONT_OUT);
}
