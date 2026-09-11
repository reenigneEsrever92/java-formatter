//! Array creation — dimension expressions in `new T[expr]`.
//! Fixtures live under tests/java/array_creation/.
//!
//! Regression: `new T[0]` used to render as `new T[[0]]` because the whole
//! `dimensions_expr` node was wrapped in brackets and formatted via the
//! text-echo fallback; the inner expression must be formatted instead.

use super::common::*;

const DIMS: &str = include_str!("../java/array_creation/dims.java");
const DIMS_OUT: &str = include_str!("../java/array_creation/dims.out.java");
const INIT: &str = include_str!("../java/array_creation/init.java");
const INIT_OUT: &str = include_str!("../java/array_creation/init.out.java");

/// `new T[expr]` keeps exactly one bracket pair per dimension, in every
/// context: literal dims, multi-dimensional, sub-expression, expression dims.
#[test]
fn dimension_expressions_keep_single_brackets() {
    assert_eq!(format(DIMS), DIMS_OUT);
    assert!(
        !DIMS_OUT.contains("[[") && !DIMS_OUT.contains("]]"),
        "no doubled bracket pairs in the golden: {DIMS_OUT}"
    );
}

/// Reformatting the formatted output is a no-op (R6).
#[test]
fn dimension_expressions_are_idempotent() {
    assert_eq!(format(DIMS_OUT), DIMS_OUT);
}

/// The `new int[] { ... }` initializer form (a `dimensions` node, not a
/// `dimensions_expr`) is unaffected.
#[test]
fn initializer_form_is_unchanged() {
    assert_eq!(format(INIT), INIT_OUT);
}
