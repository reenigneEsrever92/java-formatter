//! FOR_BRACE_FORCE — whether braces are forced around brace-less for /
//! enhanced-for bodies.
//! Fixtures live under tests/java/for_brace_force/.
//!
//! Force codes (docs/settings/index.md "Force-brace codes"): `0` do not force,
//! `1` force when the body spans multiple lines, `3` always force. Both the
//! classic `for` and the enhanced `for` count as `for` bodies. Bodies that
//! already carry braces are left to the block layout and never stripped.

use super::common::*;
use java_formatter_core::config::{ForceStyle, JavaStyle};

const BODIES: &str = include_str!("../java/for_brace_force/for_brace_force.java");
const BODIES_DEFAULT_OUT: &str =
    include_str!("../java/for_brace_force/for_brace_force_default.out.java");
const BODIES_CODE0_OUT: &str =
    include_str!("../java/for_brace_force/for_brace_force_code0.out.java");
const BODIES_CODE1_OUT: &str =
    include_str!("../java/for_brace_force/for_brace_force_code1.out.java");
const BODIES_CODE3_OUT: &str =
    include_str!("../java/for_brace_force/for_brace_force_code3.out.java");
const BRACED: &str = include_str!("../java/for_brace_force/for_brace_force_braced.java");
const BRACED_OUT: &str = include_str!("../java/for_brace_force/for_brace_force_braced.out.java");

fn force(f: ForceStyle) -> JavaStyle {
    style(|s| s.for_brace_force = f)
}

#[test]
fn absent_option_defaults_to_do_not_force() {
    assert_eq!(format(BODIES), BODIES_DEFAULT_OUT);
}

#[test]
fn do_not_force_preserves_brace_less_bodies() {
    assert_eq!(
        format_with(BODIES, &force(ForceStyle::DoNotForce)),
        BODIES_CODE0_OUT
    );
}

#[test]
fn force_if_multiline_braces_only_multiline_bodies() {
    assert_eq!(
        format_with(BODIES, &force(ForceStyle::ForceIfMultiline)),
        BODIES_CODE1_OUT
    );
}

#[test]
fn force_always_braces_every_brace_less_body() {
    assert_eq!(
        format_with(BODIES, &force(ForceStyle::ForceAlways)),
        BODIES_CODE3_OUT
    );
}

#[test]
fn force_always_output_is_idempotent() {
    assert_eq!(
        format_with(BRACED, &force(ForceStyle::ForceAlways)),
        BRACED_OUT
    );
}
