//! KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE — keeping single-statement lambda block
//! bodies on one line.
//!
//! With the option on, a lambda whose block body holds a single statement is
//! collapsed to one line when it fits the right margin; with it off the block
//! body stays multi-line. Multi-statement block bodies are never collapsed.
//!
//! Fixtures live under tests/java/keep_simple_lambdas_in_one_line/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const LAMBDA: &str = include_str!("../java/keep_simple_lambdas_in_one_line/lambda.java");
const LAMBDA_OUT: &str = include_str!("../java/keep_simple_lambdas_in_one_line/lambda.out.java");
const LAMBDA_DEFAULT_OUT: &str =
    include_str!("../java/keep_simple_lambdas_in_one_line/lambda_default.out.java");

fn on_style() -> JavaStyle {
    // A tight-ish margin keeps the multi-statement body too long to collapse.
    style(|s| {
        s.right_margin = 60;
        s.keep_simple_lambdas_in_one_line = true;
        // Keep the goldens' padded `{ s }` one-line bodies (the absent
        // padding default is flush `{s}`).
        s.spaces_inside_block_braces_when_body_is_present = true;
    })
}

#[test]
fn lambda_default() {
    // Block bodies stay multi-line when the option is off.
    assert_eq!(format(LAMBDA), LAMBDA_DEFAULT_OUT);
}

#[test]
fn lambda() {
    // Single-statement bodies collapse; the multi-statement body stays
    // multi-line (see the golden).
    assert_eq!(format_with(LAMBDA, &on_style()), LAMBDA_OUT);
}
