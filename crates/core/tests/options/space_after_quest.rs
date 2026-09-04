//! SPACE_AFTER_QUEST — space after `?` in a ternary expression. Defaults to
//! on (a ? b : c).
//! Fixtures live under tests/java/space_after_quest/.

use super::common::*;

const MIXED: &str = include_str!("../java/space_after_quest/mixed.java");
const MIXED_OUT: &str = include_str!("../java/space_after_quest/mixed.out.java");
const MIXED_DEFAULT_OUT: &str =
    include_str!("../java/space_after_quest/mixed_default.out.java");

#[test]
fn off_glues_question_mark_to_consequence() {
    let style = style(|s| s.space_after_quest = false);
    assert_eq!(format_with(MIXED, &style), MIXED_OUT);
}

#[test]
fn absent_option_uses_default_spacing() {
    assert_eq!(format(MIXED), MIXED_DEFAULT_OUT);
}
