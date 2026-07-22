//! CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND — merging of single imports into
//! on-demand (`pkg.*`) imports.
//! Fixtures live under tests/java/class_count_to_use_import_on_demand/.

use super::common::*;

const MERGES_PAST_THRESHOLD: &str =
    include_str!("../java/class_count_to_use_import_on_demand/merges_past_threshold.java");
const MERGES_PAST_THRESHOLD_OUT: &str = include_str!(
    "../java/class_count_to_use_import_on_demand/merges_past_threshold.out.java"
);
const RESPECTS_THRESHOLD: &str =
    include_str!("../java/class_count_to_use_import_on_demand/respects_threshold.java");
const RESPECTS_THRESHOLD_OUT: &str = include_str!(
    "../java/class_count_to_use_import_on_demand/respects_threshold.out.java"
);
const WILDCARD_PRESENT: &str =
    include_str!("../java/class_count_to_use_import_on_demand/wildcard_present.java");
const WILDCARD_PRESENT_OUT: &str = include_str!(
    "../java/class_count_to_use_import_on_demand/wildcard_present.out.java"
);
const CONFLICTING_NAMES: &str =
    include_str!("../java/class_count_to_use_import_on_demand/conflicting_names.java");
const CONFLICTING_NAMES_OUT: &str = include_str!(
    "../java/class_count_to_use_import_on_demand/conflicting_names.out.java"
);
const LOCAL_CONFLICT: &str =
    include_str!("../java/class_count_to_use_import_on_demand/local_conflict.java");
const LOCAL_CONFLICT_OUT: &str =
    include_str!("../java/class_count_to_use_import_on_demand/local_conflict.out.java");
const STATIC_NEVER_MERGED: &str =
    include_str!("../java/class_count_to_use_import_on_demand/static_never_merged.java");
const STATIC_NEVER_MERGED_OUT: &str = include_str!(
    "../java/class_count_to_use_import_on_demand/static_never_merged.out.java"
);
const MERGED_POSITION: &str =
    include_str!("../java/class_count_to_use_import_on_demand/merged_position.java");
const MERGED_POSITION_OUT: &str =
    include_str!("../java/class_count_to_use_import_on_demand/merged_position.out.java");
const HIGH_THRESHOLD: &str =
    include_str!("../java/class_count_to_use_import_on_demand/high_threshold.java");
const HIGH_THRESHOLD_OUT: &str =
    include_str!("../java/class_count_to_use_import_on_demand/high_threshold.out.java");

#[test]
fn merges_past_threshold() {
    let style = style(|s| s.class_count_to_use_import_on_demand = 2);
    assert_eq!(
        format_with(MERGES_PAST_THRESHOLD, &style),
        MERGES_PAST_THRESHOLD_OUT
    );
}

#[test]
fn respects_threshold() {
    // Exactly threshold imports -> no merge.
    let style = style(|s| s.class_count_to_use_import_on_demand = 3);
    assert_eq!(format_with(RESPECTS_THRESHOLD, &style), RESPECTS_THRESHOLD_OUT);
}

#[test]
fn wildcard_already_present_disables_merging() {
    // Merging could change name resolution when a wildcard import is already in
    // the file, so the formatter leaves the section alone.
    let style = style(|s| s.class_count_to_use_import_on_demand = 2);
    assert_eq!(format_with(WILDCARD_PRESENT, &style), WILDCARD_PRESENT_OUT);
}

#[test]
fn conflicting_simple_names_disable_merging() {
    // `C` is imported from two packages; collapsing a.one would change which C
    // is bound, so a.one is left untouched.
    let style = style(|s| s.class_count_to_use_import_on_demand = 2);
    assert_eq!(format_with(CONFLICTING_NAMES, &style), CONFLICTING_NAMES_OUT);
}

#[test]
fn local_type_with_same_name_disables_merging() {
    let style = style(|s| s.class_count_to_use_import_on_demand = 2);
    assert_eq!(format_with(LOCAL_CONFLICT, &style), LOCAL_CONFLICT_OUT);
}

#[test]
fn static_imports_are_never_merged() {
    let style = style(|s| s.class_count_to_use_import_on_demand = 1);
    assert_eq!(format_with(STATIC_NEVER_MERGED, &style), STATIC_NEVER_MERGED_OUT);
}

#[test]
fn merged_wildcard_keeps_position_and_java_grouping() {
    let style = style(|s| s.class_count_to_use_import_on_demand = 2);
    assert_eq!(format_with(MERGED_POSITION, &style), MERGED_POSITION_OUT);
}

#[test]
fn high_threshold_keeps_single_imports() {
    // The default threshold is high enough that a.one's four imports never merge.
    assert_eq!(format(HIGH_THRESHOLD), HIGH_THRESHOLD_OUT);
}
