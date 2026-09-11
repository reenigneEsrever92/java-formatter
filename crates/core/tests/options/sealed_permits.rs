//! Sealed type `permits` clauses — preserved on class / interface headers.
//! Fixtures live under tests/java/sealed_permits/.
//!
//! The `permits` clause of a sealed class or sealed interface must survive
//! formatting (a regression guard: it used to be dropped). Under the default
//! style the clause renders flat ` permits A, B` on the header line, and
//! formatting already-canonical output is a no-op. The clause's wrap behaviour
//! under the extends-list options is covered by the extends_list_wrap /
//! extends_keyword_wrap / align_multiline_extends_list suites.

use super::common::*;

const INTERFACE: &str = include_str!("../java/sealed_permits/interface.java");
const INTERFACE_OUT: &str = include_str!("../java/sealed_permits/interface.out.java");
const CLASS: &str = include_str!("../java/sealed_permits/class.java");
const CLASS_OUT: &str = include_str!("../java/sealed_permits/class.out.java");
const COMBINED: &str = include_str!("../java/sealed_permits/combined.java");
const COMBINED_OUT: &str = include_str!("../java/sealed_permits/combined.out.java");
const NESTED: &str = include_str!("../java/sealed_permits/nested.java");
const NESTED_OUT: &str = include_str!("../java/sealed_permits/nested.out.java");
const SELF_GOLDEN: &str = include_str!("../java/sealed_permits/self_golden.java");
const SELF_GOLDEN_OUT: &str = include_str!("../java/sealed_permits/self_golden.out.java");

#[test]
fn sealed_interface_keeps_its_permits_clause() {
    assert_eq!(format(INTERFACE), INTERFACE_OUT);
}

#[test]
fn sealed_class_keeps_its_permits_clause() {
    assert_eq!(format(CLASS), CLASS_OUT);
}

#[test]
fn permits_survives_alongside_extends_implements_and_type_parameters() {
    assert_eq!(format(COMBINED), COMBINED_OUT);
}

#[test]
fn nested_sealed_type_keeps_its_permits_clause() {
    assert_eq!(format(NESTED), NESTED_OUT);
}

#[test]
fn reformatting_canonical_sealed_headers_is_a_no_op() {
    // A self-golden: already-canonical sealed headers format to themselves
    // under the default style (R6).
    assert_eq!(format(SELF_GOLDEN), SELF_GOLDEN_OUT);
}
