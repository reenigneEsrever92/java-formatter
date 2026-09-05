//! KEEP_SIMPLE_CLASSES_IN_ONE_LINE — keep simple class / interface / record
//! bodies on one line.
//!
//! With the option on and an end-of-line class brace style, a body collapses
//! to one line when every member renders without a newline (a member whose
//! own layout is multi-line — a block body, a wrapped field — rejects, as do
//! comments / extras, R4) and the whole declaration fits the right margin;
//! members collapse recursively (a simple method member needs
//! KEEP_SIMPLE_METHODS_IN_ONE_LINE to render one line itself). Enums and
//! anonymous classes are out of scope. Off / absent keeps the multi-line
//! `class_body` layout.
//!
//! Fixtures live under tests/java/keep_simple_classes_in_one_line/.

use super::common::*;
use java_formatter_core::config::{BraceStyle, JavaStyle};

const COLLAPSE: &str = include_str!("../java/keep_simple_classes_in_one_line/collapse.java");
const COLLAPSE_OUT: &str =
    include_str!("../java/keep_simple_classes_in_one_line/collapse.out.java");
const COLLAPSE_DEFAULT_OUT: &str =
    include_str!("../java/keep_simple_classes_in_one_line/collapse_default.out.java");
const NEXT_LINE_BRACE: &str =
    include_str!("../java/keep_simple_classes_in_one_line/next_line_brace.java");
const NEXT_LINE_BRACE_OUT: &str =
    include_str!("../java/keep_simple_classes_in_one_line/next_line_brace.out.java");
const NON_SIMPLE: &str = include_str!("../java/keep_simple_classes_in_one_line/non_simple.java");
const NON_SIMPLE_OUT: &str =
    include_str!("../java/keep_simple_classes_in_one_line/non_simple.out.java");
const TOO_WIDE: &str = include_str!("../java/keep_simple_classes_in_one_line/too_wide.java");
const TOO_WIDE_OUT: &str =
    include_str!("../java/keep_simple_classes_in_one_line/too_wide.out.java");
const SELF_GOLDEN: &str = include_str!("../java/keep_simple_classes_in_one_line/self_golden.java");
const SELF_GOLDEN_OUT: &str =
    include_str!("../java/keep_simple_classes_in_one_line/self_golden.out.java");

/// Collapse on. The Java padding toggle keeps the collapsed `{ s }` one-line
/// bodies readable (the faithful absent padding default is flush `{s}`).
fn on_style() -> JavaStyle {
    style(|s| {
        s.keep_simple_classes_in_one_line = true;
        s.keep_simple_methods_in_one_line = true;
        s.spaces_inside_block_braces_when_body_is_present = true;
    })
}

fn next_line_style() -> JavaStyle {
    style(|s| {
        s.keep_simple_classes_in_one_line = true;
        s.keep_simple_methods_in_one_line = true;
        s.spaces_inside_block_braces_when_body_is_present = true;
        s.class_brace_style = BraceStyle::NextLine;
    })
}

fn tight_style() -> JavaStyle {
    style(|s| {
        s.right_margin = 60;
        s.keep_simple_classes_in_one_line = true;
        s.keep_simple_methods_in_one_line = true;
        s.spaces_inside_block_braces_when_body_is_present = true;
    })
}

#[test]
fn simple_class_interface_and_record_collapse() {
    // Class, interface and record bodies whose members are all simple
    // collapse to one line (the record body is formatted as a class body).
    assert_eq!(format_with(COLLAPSE, &on_style()), COLLAPSE_OUT);
}

#[test]
fn absent_option_keeps_the_multiline_layout() {
    // Without the option (absent → the built-in false default) the bodies
    // stay multi-line.
    assert_eq!(format(COLLAPSE), COLLAPSE_DEFAULT_OUT);
}

#[test]
fn next_line_class_brace_style_does_not_collapse() {
    // The `{` on its own line cannot hold a one-line body.
    assert_eq!(
        format_with(NEXT_LINE_BRACE, &next_line_style()),
        NEXT_LINE_BRACE_OUT
    );
}

#[test]
fn non_simple_members_keep_the_class_multiline() {
    // A method member with a block body (multi-statement) renders
    // multi-line, so the class cannot collapse.
    assert_eq!(format_with(NON_SIMPLE, &on_style()), NON_SIMPLE_OUT);
}

#[test]
fn a_class_too_wide_for_the_margin_stays_multiline() {
    // The collapsed one-line declaration would exceed the 60-column margin,
    // so the multi-line layout is kept (members still collapse where they
    // fit).
    assert_eq!(format_with(TOO_WIDE, &tight_style()), TOO_WIDE_OUT);
}

#[test]
fn reformatting_collapsed_declarations_is_a_no_op() {
    // A self-golden: the collapsed declarations format to themselves (R6).
    assert_eq!(format_with(SELF_GOLDEN, &on_style()), SELF_GOLDEN_OUT);
}
