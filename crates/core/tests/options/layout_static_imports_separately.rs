//! LAYOUT_STATIC_IMPORTS_SEPARATELY — keep static imports in their own section
//! (default) or inline them with the ordinary package sections.
//! Fixtures live under tests/java/layout_static_imports_separately/.

use super::common::*;

const STATIC_AND_PLAIN: &str =
    include_str!("../java/layout_static_imports_separately/static_and_plain.java");
const SEPARATE_OUT: &str =
    include_str!("../java/layout_static_imports_separately/static_and_plain_separate.out.java");
const INLINE_OUT: &str =
    include_str!("../java/layout_static_imports_separately/static_and_plain_inline.out.java");
const SELF: &str =
    include_str!("../java/layout_static_imports_separately/static_and_plain_self.java");
const SELF_OUT: &str =
    include_str!("../java/layout_static_imports_separately/static_and_plain_self.out.java");

#[test]
fn static_imports_get_their_own_section_by_default() {
    // The static imports join the table's empty-name static catch-all after
    // the `java.*` group; absent defaults to `true`.
    assert_eq!(format(STATIC_AND_PLAIN), SEPARATE_OUT);
}

#[test]
fn static_imports_join_the_ordinary_sections_when_disabled() {
    // The `static` attribute of the table entries is ignored: the `java.*`
    // static member import joins the `java.*` group and the other static
    // import joins the catch-all group inline.
    let style = style(|s| s.layout_static_imports_separately = false);
    assert_eq!(format_with(STATIC_AND_PLAIN, &style), INLINE_OUT);
}

#[test]
fn reformatting_the_separated_output_is_a_no_op() {
    assert_eq!(format(SELF), SELF_OUT);
}
