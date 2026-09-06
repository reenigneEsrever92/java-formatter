//! ENABLE_JAVADOC_FORMATTING + JD_* — the javadoc layout engine: a standalone
//! `/** … */` block comment whose structure parses cleanly is laid out per the
//! javadoc options (paragraph merging, `<p>` on empty lines, blank lines,
//! aligned `@param` / `@throws` descriptions, empty / unknown tag drops, the
//! leading `*` and one-line forms); anything unparseable echoes verbatim.
//! With the gate absent or `false` javadoc output is byte-identical to the
//! verbatim echo.
//!
//! The `JD_*` options are knobs of one engine, so this family lives in one
//! file (the change request sanctions the deviation from the per-option-file
//! convention).
//!
//! Fixtures live under tests/java/javadoc_formatting/.

use super::common::*;
use java_formatter_core::config::JavaStyle;

const CLEAN: &str = include_str!("../java/javadoc_formatting/clean.java");
const CLEAN_OUT: &str = include_str!("../java/javadoc_formatting/clean.out.java");
const CLEAN_ABSENT_OUT: &str = include_str!("../java/javadoc_formatting/clean_absent.out.java");
const CLEAN_ALIGN_PARAM_OFF_OUT: &str =
    include_str!("../java/javadoc_formatting/clean_align_param_off.out.java");
const CLEAN_ALIGN_EXCEPTION_OFF_OUT: &str =
    include_str!("../java/javadoc_formatting/clean_align_exception_off.out.java");
const CLEAN_BLANK_PARM_ON_OUT: &str =
    include_str!("../java/javadoc_formatting/clean_blank_parm_on.out.java");
const CLEAN_BLANK_RETURN_ON_OUT: &str =
    include_str!("../java/javadoc_formatting/clean_blank_return_on.out.java");
const CLEAN_BLANK_DESC_OFF_OUT: &str =
    include_str!("../java/javadoc_formatting/clean_blank_desc_off.out.java");
const CLEAN_P_OFF_OUT: &str = include_str!("../java/javadoc_formatting/clean_p_off.out.java");
const CLEAN_KEEP_EMPTY_OFF_OUT: &str =
    include_str!("../java/javadoc_formatting/clean_keep_empty_off.out.java");
const CLEAN_INVALID_OFF_OUT: &str =
    include_str!("../java/javadoc_formatting/clean_invalid_off.out.java");
const CLEAN_EMPTY_PARAM_OFF_OUT: &str =
    include_str!("../java/javadoc_formatting/clean_empty_param_off.out.java");
const CLEAN_EMPTY_EXCEPTION_OFF_OUT: &str =
    include_str!("../java/javadoc_formatting/clean_empty_exception_off.out.java");
const CLEAN_THROWS_OFF_OUT: &str =
    include_str!("../java/javadoc_formatting/clean_throws_off.out.java");
const CLEAN_ASTERISKS_OFF_OUT: &str =
    include_str!("../java/javadoc_formatting/clean_asterisks_off.out.java");
const CLEAN_PARAM_NEWLINE_ON_OUT: &str =
    include_str!("../java/javadoc_formatting/clean_param_newline_on.out.java");
const CLEAN_PRESERVE_ON_OUT: &str =
    include_str!("../java/javadoc_formatting/clean_preserve_on.out.java");
const EMPTY_RETURN: &str = include_str!("../java/javadoc_formatting/empty_return.java");
const EMPTY_RETURN_OUT: &str = include_str!("../java/javadoc_formatting/empty_return.out.java");
const EMPTY_RETURN_DROPPED_OUT: &str =
    include_str!("../java/javadoc_formatting/empty_return_dropped.out.java");
const CONTINUATION: &str = include_str!("../java/javadoc_formatting/continuation.java");
const CONTINUATION_OUT: &str = include_str!("../java/javadoc_formatting/continuation.out.java");
const CONTINUATION_INDENT_OUT: &str =
    include_str!("../java/javadoc_formatting/continuation_indent.out.java");
const ONELINE: &str = include_str!("../java/javadoc_formatting/oneline.java");
const ONELINE_KEPT_OUT: &str = include_str!("../java/javadoc_formatting/oneline_kept.out.java");
const ONELINE_EXPANDED_OUT: &str =
    include_str!("../java/javadoc_formatting/oneline_expanded.out.java");
const MESSY: &str = include_str!("../java/javadoc_formatting/messy.java");
const MESSY_OUT: &str = include_str!("../java/javadoc_formatting/messy.out.java");
const CLASS_DOC: &str = include_str!("../java/javadoc_formatting/class_doc.java");
const CLASS_DOC_OUT: &str = include_str!("../java/javadoc_formatting/class_doc.out.java");
const CLASS_DOC_ABSENT_OUT: &str =
    include_str!("../java/javadoc_formatting/class_doc_absent.out.java");
const HEADER: &str = include_str!("../java/javadoc_formatting/header.java");
const HEADER_OUT: &str = include_str!("../java/javadoc_formatting/header.out.java");
const HEADER_ABSENT_OUT: &str = include_str!("../java/javadoc_formatting/header_absent.out.java");

/// The gate on, every other option at its built-in default.
fn gate() -> JavaStyle {
    style(|s| s.enable_javadoc_formatting = true)
}

/// The gate on plus one knob tweak.
fn with(configure: impl FnOnce(&mut JavaStyle)) -> JavaStyle {
    style(|s| {
        s.enable_javadoc_formatting = true;
        configure(s);
    })
}

#[test]
fn gate_on_formats_the_clean_multi_tag_javadoc_at_the_defaults() {
    assert_eq!(format_with(CLEAN, &gate()), CLEAN_OUT);
}

#[test]
fn gate_absent_keeps_the_javadoc_byte_identical() {
    assert_eq!(format(CLEAN), CLEAN_ABSENT_OUT);
}

#[test]
fn gate_false_keeps_the_javadoc_byte_identical() {
    assert_eq!(format_with(CLEAN, &style(|_| ())), CLEAN_ABSENT_OUT);
}

#[test]
fn align_param_comments_off_leaves_param_descriptions_unaligned() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_align_param_comments = false)),
        CLEAN_ALIGN_PARAM_OFF_OUT
    );
}

#[test]
fn align_exception_comments_off_leaves_throws_descriptions_unaligned() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_align_exception_comments = false)),
        CLEAN_ALIGN_EXCEPTION_OFF_OUT
    );
}

#[test]
fn add_blank_after_parm_comments_on_inserts_a_blank_after_the_params() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_add_blank_after_parm_comments = true)),
        CLEAN_BLANK_PARM_ON_OUT
    );
}

#[test]
fn add_blank_after_return_on_inserts_a_blank_after_the_return_tag() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_add_blank_after_return = true)),
        CLEAN_BLANK_RETURN_ON_OUT
    );
}

#[test]
fn add_blank_after_description_off_removes_the_blank_before_the_tags() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_add_blank_after_description = false)),
        CLEAN_BLANK_DESC_OFF_OUT
    );
}

#[test]
fn p_at_empty_lines_off_renders_a_bare_blank_line() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_p_at_empty_lines = false)),
        CLEAN_P_OFF_OUT
    );
}

#[test]
fn keep_empty_lines_off_drops_the_empty_description_line() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_keep_empty_lines = false)),
        CLEAN_KEEP_EMPTY_OFF_OUT
    );
}

#[test]
fn keep_invalid_tags_off_drops_the_unknown_tag() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_keep_invalid_tags = false)),
        CLEAN_INVALID_OFF_OUT
    );
}

#[test]
fn keep_empty_parameter_off_drops_the_empty_param_tag() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_keep_empty_parameter = false)),
        CLEAN_EMPTY_PARAM_OFF_OUT
    );
}

#[test]
fn keep_empty_exception_off_drops_the_empty_throws_tag() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_keep_empty_exception = false)),
        CLEAN_EMPTY_EXCEPTION_OFF_OUT
    );
}

#[test]
fn use_throws_not_exception_off_keeps_the_exception_tag() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_use_throws_not_exception = false)),
        CLEAN_THROWS_OFF_OUT
    );
}

#[test]
fn leading_asterisks_off_renders_the_asterisk_less_form() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_leading_asterisks_are_enabled = false)),
        CLEAN_ASTERISKS_OFF_OUT
    );
}

#[test]
fn param_description_on_new_line_moves_param_descriptions_below_the_tag() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_param_description_on_new_line = true)),
        CLEAN_PARAM_NEWLINE_ON_OUT
    );
}

#[test]
fn preserve_line_feeds_on_keeps_the_description_line_breaks() {
    assert_eq!(
        format_with(CLEAN, &with(|s| s.jd_preserve_line_feeds = true)),
        CLEAN_PRESERVE_ON_OUT
    );
}

#[test]
fn empty_return_tag_is_kept_at_the_defaults() {
    assert_eq!(format_with(EMPTY_RETURN, &gate()), EMPTY_RETURN_OUT);
}

#[test]
fn keep_empty_return_off_drops_the_empty_return_tag() {
    assert_eq!(
        format_with(EMPTY_RETURN, &with(|s| s.jd_keep_empty_return = false)),
        EMPTY_RETURN_DROPPED_OUT
    );
}

#[test]
fn indent_on_continuation_off_keeps_the_continuation_at_the_asterisk() {
    assert_eq!(format_with(CONTINUATION, &gate()), CONTINUATION_OUT);
}

#[test]
fn indent_on_continuation_on_aligns_the_continuation_under_the_description() {
    assert_eq!(
        format_with(CONTINUATION, &with(|s| s.jd_indent_on_continuation = true)),
        CONTINUATION_INDENT_OUT
    );
}

#[test]
fn do_not_wrap_one_line_comments_true_keeps_the_one_line_form() {
    assert_eq!(
        format_with(
            ONELINE,
            &with(|s| s.jd_do_not_wrap_one_line_comments = true)
        ),
        ONELINE_KEPT_OUT
    );
}

#[test]
fn do_not_wrap_one_line_comments_false_expands_the_one_line_form() {
    assert_eq!(format_with(ONELINE, &gate()), ONELINE_EXPANDED_OUT);
}

#[test]
fn messy_javadoc_echoes_byte_for_byte_with_the_gate_on() {
    assert_eq!(format_with(MESSY, &gate()), MESSY_OUT);
}

#[test]
fn gate_on_formats_the_class_level_javadoc_between_package_and_type() {
    assert_eq!(format_with(CLASS_DOC, &gate()), CLASS_DOC_OUT);
}

#[test]
fn gate_absent_keeps_the_class_level_javadoc_byte_identical() {
    assert_eq!(format(CLASS_DOC), CLASS_DOC_ABSENT_OUT);
}

#[test]
fn gate_on_formats_the_file_header_javadoc() {
    assert_eq!(format_with(HEADER, &gate()), HEADER_OUT);
}

#[test]
fn gate_absent_keeps_the_file_header_javadoc_byte_identical() {
    assert_eq!(format(HEADER), HEADER_ABSENT_OUT);
}

#[test]
fn formatted_goldens_reproduce_themselves() {
    let cases: Vec<(&str, JavaStyle)> = vec![
        (CLEAN_OUT, gate()),
        (
            CLEAN_ALIGN_PARAM_OFF_OUT,
            with(|s| s.jd_align_param_comments = false),
        ),
        (
            CLEAN_ALIGN_EXCEPTION_OFF_OUT,
            with(|s| s.jd_align_exception_comments = false),
        ),
        (
            CLEAN_BLANK_PARM_ON_OUT,
            with(|s| s.jd_add_blank_after_parm_comments = true),
        ),
        (
            CLEAN_BLANK_RETURN_ON_OUT,
            with(|s| s.jd_add_blank_after_return = true),
        ),
        (
            CLEAN_BLANK_DESC_OFF_OUT,
            with(|s| s.jd_add_blank_after_description = false),
        ),
        (CLEAN_P_OFF_OUT, with(|s| s.jd_p_at_empty_lines = false)),
        (
            CLEAN_KEEP_EMPTY_OFF_OUT,
            with(|s| s.jd_keep_empty_lines = false),
        ),
        (
            CLEAN_INVALID_OFF_OUT,
            with(|s| s.jd_keep_invalid_tags = false),
        ),
        (
            CLEAN_EMPTY_PARAM_OFF_OUT,
            with(|s| s.jd_keep_empty_parameter = false),
        ),
        (
            CLEAN_EMPTY_EXCEPTION_OFF_OUT,
            with(|s| s.jd_keep_empty_exception = false),
        ),
        (
            CLEAN_THROWS_OFF_OUT,
            with(|s| s.jd_use_throws_not_exception = false),
        ),
        (
            CLEAN_ASTERISKS_OFF_OUT,
            with(|s| s.jd_leading_asterisks_are_enabled = false),
        ),
        (
            CLEAN_PARAM_NEWLINE_ON_OUT,
            with(|s| s.jd_param_description_on_new_line = true),
        ),
        (
            CLEAN_PRESERVE_ON_OUT,
            with(|s| s.jd_preserve_line_feeds = true),
        ),
        (EMPTY_RETURN_OUT, gate()),
        (
            EMPTY_RETURN_DROPPED_OUT,
            with(|s| s.jd_keep_empty_return = false),
        ),
        (CONTINUATION_OUT, gate()),
        (
            CONTINUATION_INDENT_OUT,
            with(|s| s.jd_indent_on_continuation = true),
        ),
        (
            ONELINE_KEPT_OUT,
            with(|s| s.jd_do_not_wrap_one_line_comments = true),
        ),
        (ONELINE_EXPANDED_OUT, gate()),
        (CLASS_DOC_OUT, gate()),
        (HEADER_OUT, gate()),
    ];
    for (golden, style) in cases {
        assert_eq!(format_with(golden, &style), golden);
    }
}
