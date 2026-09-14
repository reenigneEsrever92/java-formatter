//! Anonymous class bodies — regression coverage for the bug where a
//! `new X() { … }` rendered through the flat (one-line) path dropped its whole
//! implementation block. The flat renderers resolved the anonymous `class_body`
//! with a field-name lookup the grammar never sets (`object_creation_expression`
//! exposes the body as an unnamed positional child), so the body vanished
//! wherever the creation was rendered flat: a call / `new` argument, a chain
//! receiver, a ternary side, a binary operand and a lambda body. The body is now
//! preserved — glued on the call line for a single argument, re-indented
//! canonically everywhere else — and every fixture is a fixed point.
//!
//! Fixtures live under tests/java/anonymous_class_bodies/.

use super::common::*;

const ARGUMENT: &str = include_str!("../java/anonymous_class_bodies/argument.java");
const ARGUMENT_OUT: &str = include_str!("../java/anonymous_class_bodies/argument.out.java");
const ARGUMENT_AMONG_OTHERS: &str =
    include_str!("../java/anonymous_class_bodies/argument_among_others.java");
const ARGUMENT_AMONG_OTHERS_OUT: &str =
    include_str!("../java/anonymous_class_bodies/argument_among_others.out.java");
const NEW_ARGUMENT: &str = include_str!("../java/anonymous_class_bodies/new_argument.java");
const NEW_ARGUMENT_OUT: &str = include_str!("../java/anonymous_class_bodies/new_argument.out.java");
const CHAIN_RECEIVER: &str = include_str!("../java/anonymous_class_bodies/chain_receiver.java");
const CHAIN_RECEIVER_OUT: &str =
    include_str!("../java/anonymous_class_bodies/chain_receiver.out.java");
const TERNARY: &str = include_str!("../java/anonymous_class_bodies/ternary.java");
const TERNARY_OUT: &str = include_str!("../java/anonymous_class_bodies/ternary.out.java");
const BINARY: &str = include_str!("../java/anonymous_class_bodies/binary.java");
const BINARY_OUT: &str = include_str!("../java/anonymous_class_bodies/binary.out.java");
const LAMBDA_BODY: &str = include_str!("../java/anonymous_class_bodies/lambda_body.java");
const LAMBDA_BODY_OUT: &str = include_str!("../java/anonymous_class_bodies/lambda_body.out.java");
const INITIALISER_CONTROL: &str =
    include_str!("../java/anonymous_class_bodies/initialiser_control.java");
const INITIALISER_CONTROL_OUT: &str =
    include_str!("../java/anonymous_class_bodies/initialiser_control.out.java");

/// The body survives and the output is a fixed point.
fn golden(input: &str, expected: &str) {
    assert_eq!(format(input), expected);
    assert_eq!(format(expected), expected);
}

#[test]
fn a_single_anonymous_argument_is_glued_on_the_call_line() {
    golden(ARGUMENT, ARGUMENT_OUT);
}

#[test]
fn an_anonymous_argument_among_others_keeps_its_body() {
    golden(ARGUMENT_AMONG_OTHERS, ARGUMENT_AMONG_OTHERS_OUT);
}

#[test]
fn an_anonymous_argument_to_new_keeps_its_body() {
    golden(NEW_ARGUMENT, NEW_ARGUMENT_OUT);
}

#[test]
fn an_anonymous_receiver_of_a_call_keeps_its_body() {
    golden(CHAIN_RECEIVER, CHAIN_RECEIVER_OUT);
}

#[test]
fn an_anonymous_ternary_side_keeps_its_body() {
    golden(TERNARY, TERNARY_OUT);
}

#[test]
fn an_anonymous_binary_operand_keeps_its_body() {
    golden(BINARY, BINARY_OUT);
}

#[test]
fn an_anonymous_lambda_body_keeps_its_body() {
    golden(LAMBDA_BODY, LAMBDA_BODY_OUT);
}

#[test]
fn the_contexts_that_already_worked_are_unchanged() {
    golden(INITIALISER_CONTROL, INITIALISER_CONTROL_OUT);
}
