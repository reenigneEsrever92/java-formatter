//! WHILE_ON_NEW_LINE — putting the trailing `while` keyword of a
//! `do … while` statement on its own line.
//!
//! With the option on (IntelliJ default off), the `while (…) ;` tail that
//! follows the do body starts a fresh line at the statement indent, for both
//! braced and brace-less bodies. The `while` header of an ordinary `while`
//! loop is not affected. Absent schemes fall back to the default, keeping the
//! tail on the body's closing line.
//!
//! Fixtures live under tests/java/while_on_new_line/.

use super::common::*;

const DO_WHILE: &str = include_str!("../java/while_on_new_line/do_while.java");
const DO_WHILE_ON_OUT: &str = include_str!("../java/while_on_new_line/do_while_on.out.java");
const DO_WHILE_DEFAULT_OUT: &str =
    include_str!("../java/while_on_new_line/do_while_default.out.java");

#[test]
fn while_on_new_line_starts_the_do_while_tail_on_its_own_line() {
    let style = style(|s| s.while_on_new_line = true);
    assert_eq!(format_with(DO_WHILE, &style), DO_WHILE_ON_OUT);
}

#[test]
fn absent_option_defaults_to_inline_while_tail() {
    assert_eq!(format(DO_WHILE), DO_WHILE_DEFAULT_OUT);
}
