//! SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT — spaces inside `{ … }`
//! of a non-empty one-line block when `SPACE_WITHIN_BRACES` is off.
//!
//! Faithful to the IntelliJ default (absent/false), a one-line block body is
//! flush — `if (c) {use();}` — and the toggle adds the inner spaces —
//! `if (c) { use(); }`. Applied at the keep-simple one-line-body sites (the
//! `KEEP_SIMPLE_*_IN_ONE_LINE` collapses); flat contexts (argument lambdas,
//! one-line switches) keep their pinned `{ … }` layout.
//!
//! Fixtures live under tests/java/spaces_inside_block_braces_when_body_is_present/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const BODIES: &str =
    include_str!("../java/spaces_inside_block_braces_when_body_is_present/bodies.java");
const BODIES_FLUSH_OUT: &str =
    include_str!("../java/spaces_inside_block_braces_when_body_is_present/bodies_flush.out.java");
const BODIES_PADDED_OUT: &str =
    include_str!("../java/spaces_inside_block_braces_when_body_is_present/bodies_padded.out.java");
const SELF_GOLDEN: &str =
    include_str!("../java/spaces_inside_block_braces_when_body_is_present/self_golden.java");
const SELF_GOLDEN_OUT: &str =
    include_str!("../java/spaces_inside_block_braces_when_body_is_present/self_golden.out.java");

fn keep_simple_style(pad: bool) -> JavaStyle {
    style(|s| {
        s.keep_simple_blocks_in_one_line = true;
        s.keep_simple_methods_in_one_line = true;
        s.spaces_inside_block_braces_when_body_is_present = pad;
    })
}

#[test]
fn absent_and_false_render_flush_one_line_bodies() {
    // The IntelliJ built-in default (absent/false): a non-empty one-line
    // block is flush — `if (c) {use();}`.
    assert_eq!(
        format_with(BODIES, &keep_simple_style(false)),
        BODIES_FLUSH_OUT
    );
}

#[test]
fn option_on_pads_the_one_line_bodies() {
    // The toggle adds the single inner spaces: `if (c) { use(); }`.
    assert_eq!(
        format_with(BODIES, &keep_simple_style(true)),
        BODIES_PADDED_OUT
    );
}

#[test]
fn reformatting_padded_bodies_is_a_no_op() {
    // A self-golden: the padded one-line bodies format to themselves (R6).
    assert_eq!(
        format_with(SELF_GOLDEN, &keep_simple_style(true)),
        SELF_GOLDEN_OUT
    );
}
