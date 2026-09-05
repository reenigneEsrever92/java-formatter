//! LAYOUT_ON_DEMAND_IMPORT_FROM_SAME_PACKAGE_FIRST — put the file's own
//! package's on-demand (`pkg.*`) import before its group's other imports.
//! Fixtures live under tests/java/layout_on_demand_import_from_same_package_first/.

use super::common::*;

const SAME_PACKAGE_WILDCARDS: &str = include_str!(
    "../java/layout_on_demand_import_from_same_package_first/same_package_wildcards.java"
);
const OWN_FIRST_OUT: &str = include_str!(
    "../java/layout_on_demand_import_from_same_package_first/same_package_wildcards_own_first.out.java"
);
const SOURCE_ORDER_OUT: &str = include_str!(
    "../java/layout_on_demand_import_from_same_package_first/same_package_wildcards_source_order.out.java"
);
const SELF: &str = include_str!(
    "../java/layout_on_demand_import_from_same_package_first/same_package_wildcards_self.java"
);
const SELF_OUT: &str = include_str!(
    "../java/layout_on_demand_import_from_same_package_first/same_package_wildcards_self.out.java"
);

#[test]
fn own_package_on_demand_import_moves_first_by_default() {
    // The file's own package (`com.example`) wildcard leads its group; the
    // other imports keep their relative order. The default is `true`, so the
    // absent-option output equals the on state.
    assert_eq!(format(SAME_PACKAGE_WILDCARDS), OWN_FIRST_OUT);
}

#[test]
fn disabled_keeps_the_source_order() {
    let style = style(|s| s.layout_on_demand_import_from_same_package_first = false);
    assert_eq!(
        format_with(SAME_PACKAGE_WILDCARDS, &style),
        SOURCE_ORDER_OUT
    );
}

#[test]
fn reformatting_the_own_first_output_is_a_no_op() {
    assert_eq!(format(SELF), SELF_OUT);
}
