//! DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION_IN_PARAMETER — keep a lone parameter
//! annotation on the same line as the parameter.
//! Fixtures live under tests/java/do_not_wrap_after_single_annotation_in_parameter/.
//!
//! When `true`, a formal parameter carrying exactly one annotation keeps that
//! annotation inline with the parameter (on the parameter's own line in the
//! wrapped list) regardless of `PARAMETER_ANNOTATION_WRAP`; parameters with
//! multiple annotations still break per the wrap code.

use super::common::*;

const SINGLE_PARAM: &str =
    include_str!("../java/do_not_wrap_after_single_annotation_in_parameter/single_param.java");
const SINGLE_PARAM_FALSE_OUT: &str = include_str!(
    "../java/do_not_wrap_after_single_annotation_in_parameter/single_param_false.out.java"
);
const SINGLE_PARAM_TRUE_OUT: &str = include_str!(
    "../java/do_not_wrap_after_single_annotation_in_parameter/single_param_true.out.java"
);
const SINGLE_PARAM_DEFAULT_OUT: &str = include_str!(
    "../java/do_not_wrap_after_single_annotation_in_parameter/single_param_default.out.java"
);
const SINGLE_PARAM_SELF: &str =
    include_str!("../java/do_not_wrap_after_single_annotation_in_parameter/single_param_self.java");
const SINGLE_PARAM_SELF_OUT: &str = include_str!(
    "../java/do_not_wrap_after_single_annotation_in_parameter/single_param_self.out.java"
);

fn style_with(exempt: bool) -> java_formatter_core::config::JavaStyle {
    style(|s| {
        s.parameter_annotation_wrap = java_formatter_core::config::WrapStyle::WrapAlways;
        s.do_not_wrap_after_single_annotation_in_parameter = exempt;
    })
}

#[test]
fn off_breaks_single_and_multiple_parameter_annotations_alike() {
    assert_eq!(
        format_with(SINGLE_PARAM, &style_with(false)),
        SINGLE_PARAM_FALSE_OUT
    );
}

#[test]
fn on_keeps_a_lone_parameter_annotation_inline() {
    assert_eq!(
        format_with(SINGLE_PARAM, &style_with(true)),
        SINGLE_PARAM_TRUE_OUT
    );
}

#[test]
fn absent_option_defaults_to_off() {
    let s =
        style(|s| s.parameter_annotation_wrap = java_formatter_core::config::WrapStyle::WrapAlways);
    assert_eq!(format_with(SINGLE_PARAM, &s), SINGLE_PARAM_DEFAULT_OUT);
}

#[test]
fn reformatting_exempted_parameter_output_is_a_no_op() {
    assert_eq!(
        format_with(SINGLE_PARAM_SELF, &style_with(true)),
        SINGLE_PARAM_SELF_OUT
    );
}
