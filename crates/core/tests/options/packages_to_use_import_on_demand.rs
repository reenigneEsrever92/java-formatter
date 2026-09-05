//! PACKAGES_TO_USE_IMPORT_ON_DEMAND — packages whose single-type imports always
//! merge into `pkg.*` on demand, regardless of count.
//! Fixtures live under tests/java/packages_to_use_import_on_demand/.

use super::common::*;

const LISTED_PACKAGE_MERGES: &str =
    include_str!("../java/packages_to_use_import_on_demand/listed_package_merges.java");
const LISTED_PACKAGE_MERGES_OUT: &str =
    include_str!("../java/packages_to_use_import_on_demand/listed_package_merges.out.java");
const LISTED_SINGLE_MERGES: &str =
    include_str!("../java/packages_to_use_import_on_demand/listed_single_merges.java");
const LISTED_SINGLE_MERGES_OUT: &str =
    include_str!("../java/packages_to_use_import_on_demand/listed_single_merges.out.java");
const DEFAULT_LIST_JAVA_AWT: &str =
    include_str!("../java/packages_to_use_import_on_demand/default_list_java_awt.java");
const DEFAULT_LIST_JAVA_AWT_OUT: &str =
    include_str!("../java/packages_to_use_import_on_demand/default_list_java_awt.out.java");
const UNLISTED_STAYS_SINGLE: &str =
    include_str!("../java/packages_to_use_import_on_demand/unlisted_stays_single.java");
const UNLISTED_STAYS_SINGLE_OUT: &str =
    include_str!("../java/packages_to_use_import_on_demand/unlisted_stays_single.out.java");
const WILDCARD_GUARD: &str =
    include_str!("../java/packages_to_use_import_on_demand/wildcard_guard.java");
const WILDCARD_GUARD_OUT: &str =
    include_str!("../java/packages_to_use_import_on_demand/wildcard_guard.out.java");
const AMBIGUITY_GUARD: &str =
    include_str!("../java/packages_to_use_import_on_demand/ambiguity_guard.java");
const AMBIGUITY_GUARD_OUT: &str =
    include_str!("../java/packages_to_use_import_on_demand/ambiguity_guard.out.java");
const LOCAL_GUARD: &str = include_str!("../java/packages_to_use_import_on_demand/local_guard.java");
const LOCAL_GUARD_OUT: &str =
    include_str!("../java/packages_to_use_import_on_demand/local_guard.out.java");
const SELF: &str =
    include_str!("../java/packages_to_use_import_on_demand/listed_package_merges_self.java");
const SELF_OUT: &str =
    include_str!("../java/packages_to_use_import_on_demand/listed_package_merges_self.out.java");

/// A scheme listing `java.awt` (replacing the built-in pair), so the other
/// fixtures' ordinary packages are unlisted and only `java.awt` merges.
fn awt_only_style() -> java_formatter_core::config::JavaStyle {
    style(|s| {
        s.packages_to_use_import_on_demand = vec!["java.awt".to_string()];
    })
}

#[test]
fn listed_package_merges_below_the_class_count() {
    // java.awt has two imports here — below CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND
    // (5) — but being listed it still collapses into `java.awt.*`; the unlisted
    // com.other.Widget stays single-class.
    assert_eq!(
        format_with(LISTED_PACKAGE_MERGES, &awt_only_style()),
        LISTED_PACKAGE_MERGES_OUT
    );
}

#[test]
fn a_single_listed_import_merges_too() {
    assert_eq!(
        format_with(LISTED_SINGLE_MERGES, &awt_only_style()),
        LISTED_SINGLE_MERGES_OUT
    );
}

#[test]
fn the_default_list_collapses_a_lone_java_awt_import() {
    // The built-in packages are java.awt and javax.swing, so under the default
    // style a single java.awt import already merges (format = no scheme).
    assert_eq!(format(DEFAULT_LIST_JAVA_AWT), DEFAULT_LIST_JAVA_AWT_OUT);
}

#[test]
fn unlisted_packages_stay_single_class() {
    // a.one is not listed and its two imports sit below the class count, so it
    // keeps its single-type imports.
    assert_eq!(
        format_with(UNLISTED_STAYS_SINGLE, &awt_only_style()),
        UNLISTED_STAYS_SINGLE_OUT
    );
}

#[test]
fn wildcard_present_disables_listed_merging() {
    assert_eq!(
        format_with(WILDCARD_GUARD, &awt_only_style()),
        WILDCARD_GUARD_OUT
    );
}

#[test]
fn conflicting_simple_names_disable_listed_merging() {
    // `Button` comes from java.awt and javax.swing; collapsing java.awt could
    // change which Button is bound, so it stays single-class.
    assert_eq!(
        format_with(AMBIGUITY_GUARD, &awt_only_style()),
        AMBIGUITY_GUARD_OUT
    );
}

#[test]
fn local_type_with_same_name_disables_listed_merging() {
    assert_eq!(format_with(LOCAL_GUARD, &awt_only_style()), LOCAL_GUARD_OUT);
}

#[test]
fn reformatting_the_merged_output_is_a_no_op() {
    assert_eq!(format_with(SELF, &awt_only_style()), SELF_OUT);
}
