//! NAMES_COUNT_TO_USE_IMPORT_ON_DEMAND — merging of one owner's static member
//! imports into `import static pkg.Owner.*;` above this count.
//! Fixtures live under tests/java/names_count_to_use_import_on_demand/.

use super::common::*;

const BELOW_THRESHOLD: &str =
    include_str!("../java/names_count_to_use_import_on_demand/below_threshold.java");
const BELOW_THRESHOLD_OUT: &str =
    include_str!("../java/names_count_to_use_import_on_demand/below_threshold.out.java");
const COLLAPSE_ABOVE_COUNT: &str =
    include_str!("../java/names_count_to_use_import_on_demand/collapse_above_count.java");
const COLLAPSE_ABOVE_COUNT_OUT: &str =
    include_str!("../java/names_count_to_use_import_on_demand/collapse_above_count.out.java");
const RESPECTS_THRESHOLD: &str =
    include_str!("../java/names_count_to_use_import_on_demand/respects_threshold.java");
const RESPECTS_THRESHOLD_OUT: &str =
    include_str!("../java/names_count_to_use_import_on_demand/respects_threshold.out.java");
const WILDCARD_PRESENT: &str =
    include_str!("../java/names_count_to_use_import_on_demand/wildcard_present.java");
const WILDCARD_PRESENT_OUT: &str =
    include_str!("../java/names_count_to_use_import_on_demand/wildcard_present.out.java");
const AMBIGUOUS_NAMES: &str =
    include_str!("../java/names_count_to_use_import_on_demand/ambiguous_names.java");
const AMBIGUOUS_NAMES_OUT: &str =
    include_str!("../java/names_count_to_use_import_on_demand/ambiguous_names.out.java");
const LOCAL_CONFLICT: &str =
    include_str!("../java/names_count_to_use_import_on_demand/local_conflict.java");
const LOCAL_CONFLICT_OUT: &str =
    include_str!("../java/names_count_to_use_import_on_demand/local_conflict.out.java");
const SELF: &str =
    include_str!("../java/names_count_to_use_import_on_demand/collapse_above_count_self.java");
const SELF_OUT: &str =
    include_str!("../java/names_count_to_use_import_on_demand/collapse_above_count_self.out.java");

#[test]
fn member_imports_collapse_above_the_names_count() {
    // Four members of a.one.Methods with the count lowered to two: the group
    // collapses into `import static a.one.Methods.*;` at the first member's
    // position.
    let style = style(|s| s.names_count_to_use_import_on_demand = 2);
    assert_eq!(
        format_with(COLLAPSE_ABOVE_COUNT, &style),
        COLLAPSE_ABOVE_COUNT_OUT
    );
}

#[test]
fn exactly_at_the_names_count_no_merge_happens() {
    // Merging needs the group size to *exceed* the count: four members with the
    // count at four keep their single member imports.
    let style = style(|s| s.names_count_to_use_import_on_demand = 4);
    assert_eq!(
        format_with(RESPECTS_THRESHOLD, &style),
        RESPECTS_THRESHOLD_OUT
    );
}

#[test]
fn static_members_below_the_default_count_stay_single() {
    // The built-in default names count is three, so a three-member group is not
    // collapsed; the old claim that static imports are never merged (R3) is
    // obsolete.
    assert_eq!(format(BELOW_THRESHOLD), BELOW_THRESHOLD_OUT);
}

#[test]
fn wildcard_present_disables_static_merging() {
    // A wildcard static import is present, so the redundant member imports are
    // left untouched (merging could change name resolution).
    let style = style(|s| s.names_count_to_use_import_on_demand = 1);
    assert_eq!(format_with(WILDCARD_PRESENT, &style), WILDCARD_PRESENT_OUT);
}

#[test]
fn same_member_name_from_two_owners_disables_merging() {
    // `run` is a member of both a.one.Methods and b.two.Other; collapsing
    // either owner could hand name precedence to the other, so both groups are
    // left untouched.
    let style = style(|s| s.names_count_to_use_import_on_demand = 1);
    assert_eq!(format_with(AMBIGUOUS_NAMES, &style), AMBIGUOUS_NAMES_OUT);
}

#[test]
fn local_type_with_same_name_disables_merging() {
    let style = style(|s| s.names_count_to_use_import_on_demand = 1);
    assert_eq!(format_with(LOCAL_CONFLICT, &style), LOCAL_CONFLICT_OUT);
}

#[test]
fn reformatting_the_collapsed_output_is_a_no_op() {
    let style = style(|s| s.names_count_to_use_import_on_demand = 2);
    assert_eq!(format_with(SELF, &style), SELF_OUT);
}
