//! KEEP_BLANK_LINES_BETWEEN_IMPORTS — preserve the source's blank lines
//! between the imports of one group.
//! Fixtures live under tests/java/keep_blank_lines_between_imports/.

use super::common::*;

const GROUP_GAPS: &str = include_str!("../java/keep_blank_lines_between_imports/group_gaps.java");
const GROUP_GAPS_KEPT_OUT: &str =
    include_str!("../java/keep_blank_lines_between_imports/group_gaps_kept.out.java");
const GROUP_GAPS_DROPPED_OUT: &str =
    include_str!("../java/keep_blank_lines_between_imports/group_gaps_dropped.out.java");
const GROUP_GAPS_SELF: &str =
    include_str!("../java/keep_blank_lines_between_imports/group_gaps_self.java");
const GROUP_GAPS_SELF_OUT: &str =
    include_str!("../java/keep_blank_lines_between_imports/group_gaps_self.out.java");

#[test]
fn keep_blank_lines_true_preserves_the_gaps_inside_a_group() {
    // The blank lines inside the single catch-all group survive; the section
    // keeps its group order (no re-sorting of imports).
    let style = style(|s| s.keep_blank_lines_between_imports = true);
    assert_eq!(format_with(GROUP_GAPS, &style), GROUP_GAPS_KEPT_OUT);
}

#[test]
fn keep_blank_lines_false_drops_the_gaps() {
    let style = style(|s| s.keep_blank_lines_between_imports = false);
    assert_eq!(format_with(GROUP_GAPS, &style), GROUP_GAPS_DROPPED_OUT);
}

#[test]
fn absent_option_drops_the_gaps_like_false() {
    // keep_blank_lines defaults to false: today's gap-free layout.
    assert_eq!(format(GROUP_GAPS), GROUP_GAPS_DROPPED_OUT);
}

#[test]
fn reformatting_the_kept_output_is_a_no_op() {
    let style = style(|s| s.keep_blank_lines_between_imports = true);
    assert_eq!(format_with(GROUP_GAPS_SELF, &style), GROUP_GAPS_SELF_OUT);
}
