//! Formatter control tags — `// @formatter:off` … `// @formatter:on` regions
//! preserved byte-for-byte, per the four root-level options
//! (`FORMATTER_TAGS_ENABLED`, `FORMATTER_OFF_TAG`, `FORMATTER_ON_TAG`,
//! `FORMATTER_TAGS_ACCEPT_REGEXP`).
//!
//! Markers are recognised comment-scoped: only the content of a `//` line
//! comment or a `/* */` block comment is compared to the tag, so a tag inside
//! a string literal or prose never triggers (a deliberate divergence from
//! IntelliJ's raw substring scan). A region spans from the start of the off
//! marker's line through the start of the on marker's line and is emitted
//! byte-for-byte; an unmatched off protects to end of file; a second off is
//! ignored; an on without off is ignored.
//! Fixtures live under tests/java/formatter_tags/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const CLASS_MEMBERS: &str = include_str!("../java/formatter_tags/class_members.java");
const CLASS_MEMBERS_OUT: &str = include_str!("../java/formatter_tags/class_members.out.java");
const METHOD_BODY: &str = include_str!("../java/formatter_tags/method_body.java");
const METHOD_BODY_OUT: &str = include_str!("../java/formatter_tags/method_body.out.java");
const TOP_LEVEL_TYPES: &str = include_str!("../java/formatter_tags/top_level_types.java");
const TOP_LEVEL_TYPES_OUT: &str = include_str!("../java/formatter_tags/top_level_types.out.java");
const UNMATCHED_OFF: &str = include_str!("../java/formatter_tags/unmatched_off.java");
const UNMATCHED_OFF_OUT: &str = include_str!("../java/formatter_tags/unmatched_off.out.java");
const BLOCK_MARKER: &str = include_str!("../java/formatter_tags/block_marker.java");
const BLOCK_MARKER_OUT: &str = include_str!("../java/formatter_tags/block_marker.out.java");
const CASE_INSENSITIVE: &str = include_str!("../java/formatter_tags/case_insensitive.java");
const CASE_INSENSITIVE_OUT: &str = include_str!("../java/formatter_tags/case_insensitive.out.java");
const STRING_NOT_MARKER: &str = include_str!("../java/formatter_tags/string_not_marker.java");
const STRING_NOT_MARKER_OUT: &str =
    include_str!("../java/formatter_tags/string_not_marker.out.java");
const ON_WITHOUT_OFF: &str = include_str!("../java/formatter_tags/on_without_off.java");
const ON_WITHOUT_OFF_OUT: &str = include_str!("../java/formatter_tags/on_without_off.out.java");
const SECOND_OFF: &str = include_str!("../java/formatter_tags/second_off.java");
const SECOND_OFF_OUT: &str = include_str!("../java/formatter_tags/second_off.out.java");
const TRAILING_MARKER: &str = include_str!("../java/formatter_tags/trailing_marker.java");
const TRAILING_MARKER_OUT: &str = include_str!("../java/formatter_tags/trailing_marker.out.java");
const DISABLED: &str = include_str!("../java/formatter_tags/disabled.java");
const DISABLED_OUT: &str = include_str!("../java/formatter_tags/disabled.out.java");
const CUSTOM_TAGS: &str = include_str!("../java/formatter_tags/custom_tags.java");
const CUSTOM_TAGS_OUT: &str = include_str!("../java/formatter_tags/custom_tags.out.java");
const REGEXP_TAGS: &str = include_str!("../java/formatter_tags/regexp_tags.java");
const REGEXP_TAGS_OUT: &str = include_str!("../java/formatter_tags/regexp_tags.out.java");
const MALFORMED_REGEXP: &str = include_str!("../java/formatter_tags/malformed_regexp.java");
const MALFORMED_REGEXP_OUT: &str = include_str!("../java/formatter_tags/malformed_regexp.out.java");
const WRAP_LONG_LINES_SKIP: &str = include_str!("../java/formatter_tags/wrap_long_lines_skip.java");
const WRAP_LONG_LINES_SKIP_OUT: &str =
    include_str!("../java/formatter_tags/wrap_long_lines_skip.out.java");
const IMPORTS_REGION: &str = include_str!("../java/formatter_tags/imports_region.java");
const IMPORTS_REGION_OUT: &str = include_str!("../java/formatter_tags/imports_region.out.java");
const TOP_OF_FILE: &str = include_str!("../java/formatter_tags/top_of_file.java");
const TOP_OF_FILE_OUT: &str = include_str!("../java/formatter_tags/top_of_file.out.java");
const TOPLEVEL_CLASS: &str = include_str!("../java/formatter_tags/toplevel_class.java");
const TOPLEVEL_CLASS_OUT: &str = include_str!("../java/formatter_tags/toplevel_class.out.java");

/// The protected region survives untouched and the output is a fixed point
/// (the markers re-protect it, R6).
fn golden(input: &str, expected: &str) {
    assert_eq!(format(input), expected);
    assert_eq!(format(expected), expected);
}

fn golden_with(input: &str, expected: &str, style: &JavaStyle) {
    assert_eq!(format_with(input, style), expected);
    assert_eq!(format_with(expected, style), expected);
}

#[test]
fn region_around_members_is_preserved_byte_for_byte() {
    golden(CLASS_MEMBERS, CLASS_MEMBERS_OUT);
}

#[test]
fn region_inside_a_method_body_is_preserved() {
    golden(METHOD_BODY, METHOD_BODY_OUT);
}

#[test]
fn region_spanning_whole_top_level_types_is_preserved() {
    golden(TOP_LEVEL_TYPES, TOP_LEVEL_TYPES_OUT);
}

#[test]
fn unmatched_off_protects_to_end_of_file() {
    golden(UNMATCHED_OFF, UNMATCHED_OFF_OUT);
}

#[test]
fn block_comment_markers_work() {
    golden(BLOCK_MARKER, BLOCK_MARKER_OUT);
}

#[test]
fn markers_match_case_insensitively() {
    golden(CASE_INSENSITIVE, CASE_INSENSITIVE_OUT);
}

#[test]
fn a_tag_inside_a_string_or_prose_is_not_a_marker() {
    golden(STRING_NOT_MARKER, STRING_NOT_MARKER_OUT);
}

#[test]
fn on_without_off_is_ignored() {
    golden(ON_WITHOUT_OFF, ON_WITHOUT_OFF_OUT);
}

#[test]
fn a_second_off_is_ignored() {
    golden(SECOND_OFF, SECOND_OFF_OUT);
}

#[test]
fn a_trailing_off_marker_freezes_its_whole_line() {
    golden(TRAILING_MARKER, TRAILING_MARKER_OUT);
}

#[test]
fn tags_disabled_make_no_difference() {
    // FORMATTER_TAGS_ENABLED=false (and, implicitly, a tag-free file under the
    // default style) formats normally: the markers are ordinary comments.
    let off = style(|s| s.formatter_tags_enabled = false);
    golden_with(DISABLED, DISABLED_OUT, &off);
    // A tag-free file under the default style is unchanged by the feature.
    assert_eq!(format(CLASS_MEMBERS_OUT), CLASS_MEMBERS_OUT);
}

#[test]
fn custom_tags_are_honoured() {
    let custom = style(|s| {
        s.formatter_off_tag = "OFF".to_string();
        s.formatter_on_tag = "ON".to_string();
    });
    golden_with(CUSTOM_TAGS, CUSTOM_TAGS_OUT, &custom);
}

#[test]
fn regexp_tags_match_per_regex() {
    let rx = style(|s| {
        s.formatter_tags_accept_regexp = true;
        s.formatter_off_tag = "@formatter:\\s*off".to_string();
        s.formatter_on_tag = "@formatter:\\s*on".to_string();
    });
    golden_with(REGEXP_TAGS, REGEXP_TAGS_OUT, &rx);
}

#[test]
fn a_malformed_regexp_falls_back_to_literal_matching() {
    let rx = style(|s| {
        s.formatter_tags_accept_regexp = true;
        s.formatter_off_tag = "@formatter:[".to_string();
        s.formatter_on_tag = "@formatter:on".to_string();
    });
    // `@formatter:[` is not a valid regex, so it falls back to literal
    // comparison — and `// @formatter:off` does not equal it, so nothing is
    // protected and the file formats normally.
    golden_with(MALFORMED_REGEXP, MALFORMED_REGEXP_OUT, &rx);
}

#[test]
fn wrap_long_lines_leaves_protected_lines_alone() {
    let wrap = style(|s| {
        s.right_margin = 40;
        s.wrap_long_lines = true;
    });
    golden_with(WRAP_LONG_LINES_SKIP, WRAP_LONG_LINES_SKIP_OUT, &wrap);
}

#[test]
fn region_around_imports_is_preserved() {
    // Tags in the import section: the imports between the markers stay in
    // place with their source spacing, and the markers stay around them.
    golden(IMPORTS_REGION, IMPORTS_REGION_OUT);
}

#[test]
fn region_at_the_top_of_the_file_covers_package_and_imports() {
    // Tags before the package: package and imports are preserved verbatim.
    golden(TOP_OF_FILE, TOP_OF_FILE_OUT);
}

#[test]
fn region_around_a_whole_top_level_class_is_preserved() {
    // Tags immediately before a top-level class (no package / imports): the
    // class body is preserved verbatim.
    golden(TOPLEVEL_CLASS, TOPLEVEL_CLASS_OUT);
}
