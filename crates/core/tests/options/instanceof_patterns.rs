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
//!
//! A record pattern whose type is a *qualified* name is a tree-sitter-java
//! 0.23.5 grammar gap (the grammar models only a simple record type before the
//! component list), so the parser recovers with an `ERROR` node; the affected
//! statement is preserved verbatim rather than losing the deconstruction —
//! whether the `ERROR` sits inside the statement or is recovered as a fragment
//! directly in the enclosing block (which merges it back into its statement).

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
const SCOPED_RECORD_PATTERN: &str =
    include_str!("../java/instanceof_patterns/scoped_record_pattern.java");
const SCOPED_RECORD_PATTERN_OUT: &str =
    include_str!("../java/instanceof_patterns/scoped_record_pattern.out.java");
const SCOPED_RECORD_PATTERN_IN_EXPRESSION: &str =
    include_str!("../java/instanceof_patterns/scoped_record_pattern_in_expression.java");
const SCOPED_RECORD_PATTERN_IN_EXPRESSION_OUT: &str =
    include_str!("../java/instanceof_patterns/scoped_record_pattern_in_expression.out.java");

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

/// A record pattern whose type is a qualified name (`Outer.Inner.Record(var v)`)
/// is valid Java but a tree-sitter-java 0.23.5 grammar gap: the parser recovers
/// with an `ERROR` node carrying the deconstruction. The affected statement is
/// emitted verbatim (R4) instead of rebuilt from its fields, which used to drop
/// the deconstruction and leave non-compiling output.
#[test]
fn scoped_record_patterns_are_preserved_verbatim() {
    assert_eq!(format(SCOPED_RECORD_PATTERN), SCOPED_RECORD_PATTERN_OUT);
}

/// When the parser recovers a broken statement as a parsed prefix plus an
/// `ERROR` fragment sitting directly in the block (a ternary over a scoped
/// record pattern, a declaration, a `return`), the fragment is merged back into
/// its statement and the whole run is emitted verbatim rather than split into
/// separate — and invalid — statements.
#[test]
fn scoped_record_patterns_in_expressions_are_preserved_verbatim() {
    assert_eq!(
        format(SCOPED_RECORD_PATTERN_IN_EXPRESSION),
        SCOPED_RECORD_PATTERN_IN_EXPRESSION_OUT
    );
}
