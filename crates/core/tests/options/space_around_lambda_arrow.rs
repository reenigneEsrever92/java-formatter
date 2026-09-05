//! SPACE_AROUND_LAMBDA_ARROW — space around the lambda arrow (->), on
//! expression-bodied and one-line block lambdas.
//! Fixtures live under tests/java/space_around_lambda_arrow/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_around_lambda_arrow/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_around_lambda_arrow/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_around_lambda_arrow/mixed_default.out.java");

#[test]
fn off_tightens_lambda_arrow() {
    let style = style(|s| {
        s.space_around_lambda_arrow = false;
        s.keep_simple_lambdas_in_one_line = true;
        // Keep the goldens' padded `{ s }` one-line lambda blocks (the
        // absent padding default is flush `{s}`).
        s.spaces_inside_block_braces_when_body_is_present = true;
    });
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
