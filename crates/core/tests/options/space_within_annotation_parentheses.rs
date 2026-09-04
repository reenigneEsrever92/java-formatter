//! SPACE_WITHIN_ANNOTATION_PARENTHESES — padding inside annotation argument
//! parentheses (`@Anno( args )`); a bare `@A()` stays tight.
//! Fixtures live under tests/java/space_within_annotation_parentheses/.

use super::common::*;

const ANNOTATION: &str =
    include_str!("../java/space_within_annotation_parentheses/annotation.java");
const ANNOTATION_OUT: &str =
    include_str!("../java/space_within_annotation_parentheses/annotation.out.java");
const ANNOTATION_DEFAULT_OUT: &str =
    include_str!("../java/space_within_annotation_parentheses/annotation_default.out.java");

#[test]
fn pads_annotation_parentheses_when_on() {
    let s = style(|st| st.space_within_annotation_parentheses = true);
    assert_eq!(format_with(ANNOTATION, &s), ANNOTATION_OUT);
}

#[test]
fn tight_by_default() {
    assert_eq!(format(ANNOTATION), ANNOTATION_DEFAULT_OUT);
}
