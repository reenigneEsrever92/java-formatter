//! DELETE_UNUSED_MODULE_IMPORTS — remove clearly-unused `import module …;`
//! lines on reformat.
//! Fixtures live under tests/java/delete_unused_module_imports/.

use super::common::*;

const DUPLICATES: &str = include_str!("../java/delete_unused_module_imports/duplicates.java");
const DUPLICATES_DELETE_OUT: &str =
    include_str!("../java/delete_unused_module_imports/duplicates_delete.out.java");
const DUPLICATES_DEFAULT_OUT: &str =
    include_str!("../java/delete_unused_module_imports/duplicates_default.out.java");
const ALREADY_DEDUPED: &str =
    include_str!("../java/delete_unused_module_imports/already_deduped.java");
const ALREADY_DEDUPED_OUT: &str =
    include_str!("../java/delete_unused_module_imports/already_deduped.out.java");

#[test]
fn duplicate_module_import_beyond_the_first_is_removed() {
    // Only clearly-unused module imports are dropped: the repeated `java.base`
    // line adds nothing, while the single `java.sql` line is doubtful and kept.
    let style = style(|s| s.delete_unused_module_imports = true);
    assert_eq!(format_with(DUPLICATES, &style), DUPLICATES_DELETE_OUT);
}

#[test]
fn absent_option_keeps_the_duplicate() {
    // delete_unused defaults to false: every module import is preserved.
    assert_eq!(format(DUPLICATES), DUPLICATES_DEFAULT_OUT);
}

#[test]
fn reformatting_the_deduped_output_is_a_no_op() {
    let style = style(|s| s.delete_unused_module_imports = true);
    assert_eq!(format_with(ALREADY_DEDUPED, &style), ALREADY_DEDUPED_OUT);
}
