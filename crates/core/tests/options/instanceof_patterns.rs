//! `instanceof` pattern matching — type patterns, `final` modifiers, and
//! record patterns are preserved on the right-hand side of `instanceof`.
//! Fixtures live under tests/java/instanceof_patterns/.
//!
//! The `instanceof` renderer used to rebuild only `left instanceof right`,
//! dropping the pattern variable name, an optional `final`, and any record
//! pattern. Under the default style the whole pattern survives verbatim on one
//! line, and the deconstruction-list spacing options apply to a record pattern
//! identically in `instanceof` and in switch labels (covered per-option by the
//! deconstruction spacing suites).

use super::common::*;

const TYPE_PATTERN: &str = include_str!("../java/instanceof_patterns/type_pattern.java");
const TYPE_PATTERN_OUT: &str = include_str!("../java/instanceof_patterns/type_pattern.out.java");
const FINAL_PATTERN: &str = include_str!("../java/instanceof_patterns/final_pattern.java");
const FINAL_PATTERN_OUT: &str = include_str!("../java/instanceof_patterns/final_pattern.out.java");
const RECORD_PATTERN: &str = include_str!("../java/instanceof_patterns/record_pattern.java");
const RECORD_PATTERN_OUT: &str =
    include_str!("../java/instanceof_patterns/record_pattern.out.java");
const RECORD_PATTERN_SPACED: &str =
    include_str!("../java/instanceof_patterns/record_pattern_spaced.java");
const RECORD_PATTERN_SPACED_OUT: &str =
    include_str!("../java/instanceof_patterns/record_pattern_spaced.out.java");
const COMBINED: &str = include_str!("../java/instanceof_patterns/combined.java");
const COMBINED_OUT: &str = include_str!("../java/instanceof_patterns/combined.out.java");

#[test]
fn type_pattern_keeps_its_variable_name() {
    assert_eq!(format(TYPE_PATTERN), TYPE_PATTERN_OUT);
}

#[test]
fn final_modifier_and_name_are_preserved() {
    assert_eq!(format(FINAL_PATTERN), FINAL_PATTERN_OUT);
}

#[test]
fn record_pattern_is_preserved() {
    assert_eq!(format(RECORD_PATTERN), RECORD_PATTERN_OUT);
}

#[test]
fn deconstruction_spacing_options_apply_to_instanceof_record_patterns() {
    let style = style(|s| {
        s.space_within_deconstruction_list = true;
        s.space_before_deconstruction_list = true;
    });
    assert_eq!(
        format_with(RECORD_PATTERN_SPACED, &style),
        RECORD_PATTERN_SPACED_OUT
    );
}

#[test]
fn patterns_survive_in_boolean_while_and_ternary_contexts() {
    assert_eq!(format(COMBINED), COMBINED_OUT);
}
