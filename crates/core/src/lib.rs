//! Java source-code formatter driven by IntelliJ codestyle settings.
//!
//! The library exposes two pieces:
//!
//! * [`config`] — parses IntelliJ `<code_scheme>` XML into a [`config::JavaStyle`].
//! * [`formatter`] — formats Java source according to a [`config::JavaStyle`].
//! * [`highlight`] — tree-sitter-driven syntax-highlighting spans for Java source.
//!
//! The `java-formatter-cli` binary (in the `crates/cli` workspace member) is
//! a thin CLI around these two modules.

pub mod config;
pub mod formatter;
pub mod highlight;
