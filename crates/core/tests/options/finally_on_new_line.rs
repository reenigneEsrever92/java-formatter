//! FINALLY_ON_NEW_LINE — putting the `finally` clause of a `try` statement on
//! its own line.
//!
//! With the option on (IntelliJ default off), the `finally` keyword starts a
//! fresh line at the statement indent instead of joining the previous body's
//! closing `}`. `catch` clauses are not affected (they follow
//! CATCH_ON_NEW_LINE). Absent schemes fall back to the default, keeping the
//! clause on its body's closing line.
//!
//! Fixtures live under tests/java/finally_on_new_line/.

use super::common::*;

const TRY_FINALLY: &str = include_str!("../java/finally_on_new_line/try_finally.java");
const TRY_FINALLY_ON_OUT: &str =
    include_str!("../java/finally_on_new_line/try_finally_on.out.java");
const TRY_FINALLY_DEFAULT_OUT: &str =
    include_str!("../java/finally_on_new_line/try_finally_default.out.java");

#[test]
fn finally_on_new_line_starts_the_finally_clause_on_its_own_line() {
    let style = style(|s| s.finally_on_new_line = true);
    assert_eq!(format_with(TRY_FINALLY, &style), TRY_FINALLY_ON_OUT);
}

#[test]
fn absent_option_defaults_to_inline_finally_clause() {
    assert_eq!(format(TRY_FINALLY), TRY_FINALLY_DEFAULT_OUT);
}
