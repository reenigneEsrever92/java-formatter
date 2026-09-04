//! CATCH_ON_NEW_LINE — putting each `catch` clause of a `try` statement on
//! its own line.
//!
//! With the option on (IntelliJ default off), every `catch` keyword starts a
//! fresh line at the statement indent instead of joining the previous body's
//! closing `}`. A trailing `finally` clause is not affected (it follows
//! FINALLY_ON_NEW_LINE). Absent schemes fall back to the default, keeping the
//! clauses on their bodies' closing lines.
//!
//! Fixtures live under tests/java/catch_on_new_line/.

use super::common::*;

const TRY_CATCH_FINALLY: &str = include_str!("../java/catch_on_new_line/try_catch_finally.java");
const TRY_CATCH_FINALLY_ON_OUT: &str =
    include_str!("../java/catch_on_new_line/try_catch_finally_on.out.java");
const TRY_CATCH_FINALLY_DEFAULT_OUT: &str =
    include_str!("../java/catch_on_new_line/try_catch_finally_default.out.java");

#[test]
fn catch_on_new_line_starts_each_catch_clause_on_its_own_line() {
    let style = style(|s| s.catch_on_new_line = true);
    assert_eq!(
        format_with(TRY_CATCH_FINALLY, &style),
        TRY_CATCH_FINALLY_ON_OUT
    );
}

#[test]
fn absent_option_defaults_to_inline_catch_clauses() {
    assert_eq!(format(TRY_CATCH_FINALLY), TRY_CATCH_FINALLY_DEFAULT_OUT);
}
