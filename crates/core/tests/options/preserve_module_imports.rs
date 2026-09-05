//! PRESERVE_MODULE_IMPORTS — keep or drop `import module …;` lines on
//! reformat.
//! Fixtures live under tests/java/preserve_module_imports/.

use super::common::*;

const MODULE_IMPORTS: &str = include_str!("../java/preserve_module_imports/module_imports.java");
const KEPT_OUT: &str = include_str!("../java/preserve_module_imports/module_imports_kept.out.java");
const DROPPED_OUT: &str =
    include_str!("../java/preserve_module_imports/module_imports_dropped.out.java");
const SELF: &str = include_str!("../java/preserve_module_imports/module_imports_self.java");
const SELF_OUT: &str = include_str!("../java/preserve_module_imports/module_imports_self.out.java");

#[test]
fn module_imports_are_kept_in_the_module_slot_by_default() {
    // The module line is preserved at the layout table's module slot (before
    // the ordinary imports); absent defaults to `true`.
    assert_eq!(format(MODULE_IMPORTS), KEPT_OUT);
}

#[test]
fn module_imports_are_removed_when_preserve_is_false() {
    // PRESERVE_MODULE_IMPORTS=false is the one sanctioned removal: the module
    // lines vanish and the rest of the import section keeps its layout.
    let style = style(|s| s.preserve_module_imports = false);
    assert_eq!(format_with(MODULE_IMPORTS, &style), DROPPED_OUT);
}

#[test]
fn reformatting_the_kept_output_is_a_no_op() {
    assert_eq!(format(SELF), SELF_OUT);
}
