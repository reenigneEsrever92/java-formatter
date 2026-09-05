//! SPACE_WITHIN_RECORD_HEADER — put one space just inside the parens of a
//! record header that share a line with a component.
//! Fixtures live under tests/java/space_within_record_header/.

use super::common::*;
use java_formatter_core::config::{JavaStyle, WrapStyle};

const HEADER: &str = include_str!("../java/space_within_record_header/header.java");
const HEADER_SPACED_OUT: &str =
    include_str!("../java/space_within_record_header/header_spaced.out.java");
const HEADER_DEFAULT_OUT: &str =
    include_str!("../java/space_within_record_header/header_default.out.java");
const COMPONENT_WRAP: &str = include_str!("../java/space_within_record_header/component_wrap.java");
const COMPONENT_WRAP_SPACED_OUT: &str =
    include_str!("../java/space_within_record_header/component_wrap_spaced.out.java");
const COMPONENT_WRAP_PLAIN_OUT: &str =
    include_str!("../java/space_within_record_header/component_wrap_plain.out.java");
const COMPONENT_WRAP_SELF: &str =
    include_str!("../java/space_within_record_header/component_wrap_self.java");
const COMPONENT_WRAP_SELF_OUT: &str =
    include_str!("../java/space_within_record_header/component_wrap_self.out.java");

fn spaced_wrapped() -> JavaStyle {
    style(|s| {
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.align_multiline_records = false;
        s.space_within_record_header = true;
    })
}

#[test]
fn pads_the_flat_header_when_enabled() {
    let style = style(|s| s.space_within_record_header = true);
    assert_eq!(format_with(HEADER, &style), HEADER_SPACED_OUT);
}

#[test]
fn absent_option_keeps_the_flat_header_unpadded() {
    // The absent option defaults to false, so the header stays unchanged.
    assert_eq!(format(HEADER), HEADER_DEFAULT_OUT);
}

#[test]
fn wraps_the_lparen_attached_header_with_padded_shared_parens() {
    // '( first,' and the glued 'last )' carry the pad; the wrapped layout's
    // own continuation lines are unaffected.
    let style = spaced_wrapped();
    assert_eq!(
        format_with(COMPONENT_WRAP, &style),
        COMPONENT_WRAP_SPACED_OUT
    );
}

#[test]
fn wrapped_lparen_attached_header_is_unpadded_when_disabled() {
    let style = style(|s| {
        s.record_components_wrap = WrapStyle::WrapAlways;
        s.align_multiline_records = false;
    });
    assert_eq!(
        format_with(COMPONENT_WRAP, &style),
        COMPONENT_WRAP_PLAIN_OUT
    );
}

#[test]
fn reformatting_the_spaced_wrapped_header_is_a_no_op() {
    let style = spaced_wrapped();
    assert_eq!(
        format_with(COMPONENT_WRAP_SELF, &style),
        COMPONENT_WRAP_SELF_OUT
    );
}
