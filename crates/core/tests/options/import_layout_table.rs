//! IMPORT_LAYOUT_TABLE — ordering and grouping of the import section per the
//! table's `<package>` / `<emptyLine>` entries.
//! Fixtures live under tests/java/import_layout_table/.

use super::common::*;
use java_formatter_core::config::{ImportLayoutEntry, JavaStyle};

const JAVA_FIRST: &str = include_str!("../java/import_layout_table/java_first.java");
const JAVA_FIRST_OUT: &str = include_str!("../java/import_layout_table/java_first.out.java");
const JAVA_FIRST_SELF: &str = include_str!("../java/import_layout_table/java_first_self.java");
const JAVA_FIRST_SELF_OUT: &str =
    include_str!("../java/import_layout_table/java_first_self.out.java");
const EMPTY_LINE_CHANGES: &str =
    include_str!("../java/import_layout_table/empty_line_changes.java");
const EMPTY_LINE_CHANGES_OUT: &str =
    include_str!("../java/import_layout_table/empty_line_changes.out.java");
const ABSENT_TABLE: &str = include_str!("../java/import_layout_table/absent_table.java");
const ABSENT_TABLE_OUT: &str = include_str!("../java/import_layout_table/absent_table.out.java");

fn pkg(name: &str, with_subpackages: bool, is_static: bool) -> ImportLayoutEntry {
    ImportLayoutEntry::Package {
        name: name.to_string(),
        with_subpackages,
        is_static,
        is_module: false,
    }
}

/// A custom table that moves the `java.*` group to the head of the section and
/// keeps one `<emptyLine/>` between every pair of groups.
fn java_first_table() -> Vec<ImportLayoutEntry> {
    vec![
        pkg("java", true, false),
        ImportLayoutEntry::EmptyLine,
        pkg("", true, false),
        ImportLayoutEntry::EmptyLine,
        pkg("javax", true, false),
        ImportLayoutEntry::EmptyLine,
        pkg("", true, true),
    ]
}

/// A custom table that removes the `<emptyLine/>` between the catch-all and
/// `javax.*` groups and doubles the one before `java.*`.
fn empty_line_changes_table() -> Vec<ImportLayoutEntry> {
    vec![
        pkg("", true, false),
        pkg("javax", true, false),
        ImportLayoutEntry::EmptyLine,
        ImportLayoutEntry::EmptyLine,
        pkg("java", true, false),
        ImportLayoutEntry::EmptyLine,
        pkg("", true, true),
    ]
}

fn java_first_style() -> JavaStyle {
    style(|s| s.import_layout = java_first_table())
}

#[test]
fn custom_table_ordering_java_first_reorders_imports_and_blanks() {
    // `java.*` heads the section (before the catch-all and `javax.*`), each
    // group separated by the table's single `<emptyLine/>`.
    assert_eq!(format_with(JAVA_FIRST, &java_first_style()), JAVA_FIRST_OUT);
}

#[test]
fn added_and_removed_empty_lines_shift_the_group_gap() {
    // No `<emptyLine/>` between the catch-all and `javax.*` (adjacent lines);
    // two `<emptyLine/>`s before `java.*` (two blank lines).
    let style = style(|s| s.import_layout = empty_line_changes_table());
    assert_eq!(
        format_with(EMPTY_LINE_CHANGES, &style),
        EMPTY_LINE_CHANGES_OUT
    );
}

#[test]
fn absent_table_uses_the_built_in_layout() {
    // The default table groups the third-party imports, then a blank line,
    // then the `javax.*` / `java.*` groups in the built-in order.
    assert_eq!(format(ABSENT_TABLE), ABSENT_TABLE_OUT);
}

#[test]
fn reformatting_the_java_first_output_is_a_no_op() {
    assert_eq!(
        format_with(JAVA_FIRST_SELF, &java_first_style()),
        JAVA_FIRST_SELF_OUT
    );
}
