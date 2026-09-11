//! Comments inside comma-separated lists — regression coverage for the bug
//! where a comment between two elements was treated as an element (taking the
//! list's separator comma) which corrupted a flat list or dropped the comment
//! outright.
//!
//! A comment is preserved verbatim and laid out as its own item, attached to
//! the element it precedes (or to the closing delimiter), never receiving a
//! separator comma; a line comment forces the list onto its wrapped layout.
//! Type arguments, lambda parameters, multi-declarator lists and a
//! commented record pattern have no wrapped form, so their construct is kept
//! verbatim (R4).
//! Fixtures live under tests/java/comments_in_lists/.

use super::common::*;

const RECORD_HEADER: &str = include_str!("../java/comments_in_lists/record_header.java");
const RECORD_HEADER_OUT: &str = include_str!("../java/comments_in_lists/record_header.out.java");
const METHOD_PARAMS: &str = include_str!("../java/comments_in_lists/method_params.java");
const METHOD_PARAMS_OUT: &str = include_str!("../java/comments_in_lists/method_params.out.java");
const CALL_ARGS: &str = include_str!("../java/comments_in_lists/call_args.java");
const CALL_ARGS_OUT: &str = include_str!("../java/comments_in_lists/call_args.out.java");
const ARRAY_INIT: &str = include_str!("../java/comments_in_lists/array_init.java");
const ARRAY_INIT_OUT: &str = include_str!("../java/comments_in_lists/array_init.out.java");
const ANNOTATION_ARGS: &str = include_str!("../java/comments_in_lists/annotation_args.java");
const ANNOTATION_ARGS_OUT: &str =
    include_str!("../java/comments_in_lists/annotation_args.out.java");
const TYPE_ARGS: &str = include_str!("../java/comments_in_lists/type_args.java");
const TYPE_ARGS_OUT: &str = include_str!("../java/comments_in_lists/type_args.out.java");
const LAMBDA_PARAMS: &str = include_str!("../java/comments_in_lists/lambda_params.java");
const LAMBDA_PARAMS_OUT: &str = include_str!("../java/comments_in_lists/lambda_params.out.java");
const THROWS_CLAUSE: &str = include_str!("../java/comments_in_lists/throws_clause.java");
const THROWS_CLAUSE_OUT: &str = include_str!("../java/comments_in_lists/throws_clause.out.java");
const DECONSTRUCTION: &str = include_str!("../java/comments_in_lists/deconstruction.java");
const DECONSTRUCTION_OUT: &str = include_str!("../java/comments_in_lists/deconstruction.out.java");
const ENUM_CONSTANTS: &str = include_str!("../java/comments_in_lists/enum_constants.java");
const ENUM_CONSTANTS_OUT: &str = include_str!("../java/comments_in_lists/enum_constants.out.java");
const MULTI_DECLARATOR: &str = include_str!("../java/comments_in_lists/multi_declarator.java");
const MULTI_DECLARATOR_OUT: &str =
    include_str!("../java/comments_in_lists/multi_declarator.out.java");

/// The comment survives, gets no separator, and the output is a fixed point.
fn golden(input: &str, expected: &str) {
    assert_eq!(format(input), expected);
    assert_eq!(format(expected), expected);
}

#[test]
fn record_header_comment_is_not_a_component() {
    golden(RECORD_HEADER, RECORD_HEADER_OUT);
}

#[test]
fn method_parameter_comment_is_not_a_parameter() {
    golden(METHOD_PARAMS, METHOD_PARAMS_OUT);
}

#[test]
fn call_argument_comment_is_not_an_argument() {
    golden(CALL_ARGS, CALL_ARGS_OUT);
}

#[test]
fn array_initializer_comment_is_not_an_element() {
    golden(ARRAY_INIT, ARRAY_INIT_OUT);
}

#[test]
fn annotation_argument_comment_is_not_a_value() {
    golden(ANNOTATION_ARGS, ANNOTATION_ARGS_OUT);
}

#[test]
fn type_argument_comment_keeps_the_type_verbatim() {
    golden(TYPE_ARGS, TYPE_ARGS_OUT);
}

#[test]
fn lambda_parameter_comment_keeps_the_parameters_verbatim() {
    golden(LAMBDA_PARAMS, LAMBDA_PARAMS_OUT);
}

#[test]
fn throws_clause_comment_is_not_an_exception() {
    golden(THROWS_CLAUSE, THROWS_CLAUSE_OUT);
}

#[test]
fn commented_record_pattern_never_collapses_onto_one_line() {
    golden(DECONSTRUCTION, DECONSTRUCTION_OUT);
}

#[test]
fn enum_constant_comment_is_not_dropped() {
    golden(ENUM_CONSTANTS, ENUM_CONSTANTS_OUT);
}

#[test]
fn multi_declarator_comment_is_not_dropped() {
    golden(MULTI_DECLARATOR, MULTI_DECLARATOR_OUT);
}
