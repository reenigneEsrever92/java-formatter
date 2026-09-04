//! KEEP_CONTROL_STATEMENT_IN_ONE_LINE — keeping a brace-less control-statement
//! body on its header's line.
//!
//! With the option on (the IntelliJ default), a brace-less body that the source
//! already has on the header's line — `if (x) foo();`, `while (go) step();`,
//! `for (…) use(i);`, `do tick(); while (go);` — stays there; a body that the
//! source has on its own line keeps the own-line layout. With it off, every
//! brace-less body is moved to its own line (today's historical layout).
//! Whitespace/layout only (R5); formatting formatted output is a no-op (R6).
//!
//! Fixtures live under tests/java/keep_control_statement_in_one_line/.

use super::common::*;

const INLINE_BODIES: &str =
    include_str!("../java/keep_control_statement_in_one_line/inline_bodies.java");
const INLINE_BODIES_DEFAULT_OUT: &str =
    include_str!("../java/keep_control_statement_in_one_line/inline_bodies_default.out.java");
const INLINE_BODIES_OFF_OUT: &str =
    include_str!("../java/keep_control_statement_in_one_line/inline_bodies_off.out.java");
const OWN_LINE_BODIES: &str =
    include_str!("../java/keep_control_statement_in_one_line/own_line_bodies.java");
const OWN_LINE_BODIES_DEFAULT_OUT: &str =
    include_str!("../java/keep_control_statement_in_one_line/own_line_bodies_default.out.java");

#[test]
fn default_keeps_same_line_brace_less_bodies_inline() {
    // The option's built-in default is on: same-line bodies stay put.
    assert_eq!(format(INLINE_BODIES), INLINE_BODIES_DEFAULT_OUT);
}

#[test]
fn option_off_splits_same_line_brace_less_bodies_onto_their_own_lines() {
    let style = style(|s| s.keep_control_statement_in_one_line = false);
    assert_eq!(format_with(INLINE_BODIES, &style), INLINE_BODIES_OFF_OUT);
}

#[test]
fn default_keeps_own_line_brace_less_bodies_on_their_own_lines() {
    // A body the source already has on its own line is not joined: the
    // default output is byte-identical to the multi-line input.
    assert_eq!(format(OWN_LINE_BODIES), OWN_LINE_BODIES_DEFAULT_OUT);
}
