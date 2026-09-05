//! USE_SINGLE_CLASS_IMPORTS — single-class vs on-demand (`pkg.*`) imports for
//! ordinary (unlisted) packages.
//! Fixtures live under tests/java/use_single_class_imports/.

use super::common::*;

const PREFERS_ON_DEMAND: &str =
    include_str!("../java/use_single_class_imports/prefers_on_demand.java");
const PREFERS_ON_DEMAND_OUT: &str =
    include_str!("../java/use_single_class_imports/prefers_on_demand.out.java");
const SINGLE_IMPORT_MERGES: &str =
    include_str!("../java/use_single_class_imports/single_import_merges.java");
const SINGLE_IMPORT_MERGES_OUT: &str =
    include_str!("../java/use_single_class_imports/single_import_merges.out.java");
const DEFAULT_KEEPS_SINGLE_IMPORTS: &str =
    include_str!("../java/use_single_class_imports/default_keeps_single_imports.java");
const DEFAULT_KEEPS_SINGLE_IMPORTS_OUT: &str =
    include_str!("../java/use_single_class_imports/default_keeps_single_imports.out.java");
const WILDCARD_GUARD: &str = include_str!("../java/use_single_class_imports/wildcard_guard.java");
const WILDCARD_GUARD_OUT: &str =
    include_str!("../java/use_single_class_imports/wildcard_guard.out.java");
const AMBIGUITY_GUARD: &str = include_str!("../java/use_single_class_imports/ambiguity_guard.java");
const AMBIGUITY_GUARD_OUT: &str =
    include_str!("../java/use_single_class_imports/ambiguity_guard.out.java");
const LOCAL_GUARD: &str = include_str!("../java/use_single_class_imports/local_guard.java");
const LOCAL_GUARD_OUT: &str = include_str!("../java/use_single_class_imports/local_guard.out.java");
const SELF: &str = include_str!("../java/use_single_class_imports/prefers_on_demand_self.java");
const SELF_OUT: &str =
    include_str!("../java/use_single_class_imports/prefers_on_demand_self.out.java");

fn single_class_off_style() -> java_formatter_core::config::JavaStyle {
    style(|s| s.use_single_class_imports = false)
}

#[test]
fn off_prefers_on_demand_below_the_class_count() {
    // With USE_SINGLE_CLASS_IMPORTS off, a.one's two imports (below the class
    // count of five) merge into `a.one.*`, and b.two's single import into
    // `b.two.*` — each emitted at its first import's position, so the merged
    // `a.one.*` keeps the slot of the interleaved first a.one import.
    assert_eq!(
        format_with(PREFERS_ON_DEMAND, &single_class_off_style()),
        PREFERS_ON_DEMAND_OUT
    );
}

#[test]
fn off_merges_even_a_count_of_one() {
    assert_eq!(
        format_with(SINGLE_IMPORT_MERGES, &single_class_off_style()),
        SINGLE_IMPORT_MERGES_OUT
    );
}

#[test]
fn on_and_absent_keep_single_imports_below_the_class_count() {
    // The default (true) keeps single-class imports below the class count.
    assert_eq!(
        format(DEFAULT_KEEPS_SINGLE_IMPORTS),
        DEFAULT_KEEPS_SINGLE_IMPORTS_OUT
    );
}

#[test]
fn wildcard_present_disables_merging_when_off() {
    assert_eq!(
        format_with(WILDCARD_GUARD, &single_class_off_style()),
        WILDCARD_GUARD_OUT
    );
}

#[test]
fn conflicting_simple_names_disable_merging_when_off() {
    // `C` is imported from a.one and b.other; neither group collapses.
    assert_eq!(
        format_with(AMBIGUITY_GUARD, &single_class_off_style()),
        AMBIGUITY_GUARD_OUT
    );
}

#[test]
fn local_type_with_same_name_disables_merging_when_off() {
    assert_eq!(
        format_with(LOCAL_GUARD, &single_class_off_style()),
        LOCAL_GUARD_OUT
    );
}

#[test]
fn reformatting_the_merged_output_is_a_no_op() {
    assert_eq!(format_with(SELF, &single_class_off_style()), SELF_OUT);
}
