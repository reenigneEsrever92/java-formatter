//! NEW_LINE_WHEN_BODY_IS_PRESENTED — put the body of a one-line block on a
//! new line.
//!
//! With the toggle on, a collapsed one-line block (a `KEEP_SIMPLE_*_IN_ONE_LINE`
//! body) starts on a fresh line below its statement head at the head's own
//! indent — `if (c)` then `{ s }` on the next line — instead of following the
//! head on the same line. Absent keeps the block after the head. The block
//! presentation composes with `SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT`.
//!
//! Fixtures live under tests/java/new_line_when_body_is_presented/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const BODIES: &str = include_str!("../java/new_line_when_body_is_presented/bodies.java");
const BODIES_NEWLINE_OUT: &str =
    include_str!("../java/new_line_when_body_is_presented/bodies_newline.out.java");
const BODIES_AFTER_HEAD_OUT: &str =
    include_str!("../java/new_line_when_body_is_presented/bodies_after_head.out.java");
const SELF_GOLDEN: &str = include_str!("../java/new_line_when_body_is_presented/self_golden.java");
const SELF_GOLDEN_OUT: &str =
    include_str!("../java/new_line_when_body_is_presented/self_golden.out.java");

fn keep_simple_style(new_line: bool) -> JavaStyle {
    style(|s| {
        s.keep_simple_blocks_in_one_line = true;
        s.keep_simple_methods_in_one_line = true;
        s.spaces_inside_block_braces_when_body_is_present = true;
        s.new_line_when_body_is_presented = new_line;
    })
}

#[test]
fn option_on_places_each_one_line_body_on_its_own_line() {
    // Each collapsed block starts on a fresh line at its statement head's
    // indent.
    assert_eq!(
        format_with(BODIES, &keep_simple_style(true)),
        BODIES_NEWLINE_OUT
    );
}

#[test]
fn absent_keeps_the_body_after_the_head() {
    // Without the option the same keep-simple bodies follow their heads on
    // the head's line.
    assert_eq!(
        format_with(BODIES, &keep_simple_style(false)),
        BODIES_AFTER_HEAD_OUT
    );
}

#[test]
fn reformatting_newline_presented_bodies_is_a_no_op() {
    // A self-golden: the own-line one-line bodies format to themselves (R6).
    assert_eq!(
        format_with(SELF_GOLDEN, &keep_simple_style(true)),
        SELF_GOLDEN_OUT
    );
}
