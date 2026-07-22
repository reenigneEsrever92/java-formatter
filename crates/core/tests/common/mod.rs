//! Shared helpers for the integration test suite.
//! Each integration test only uses a subset of these helpers, so silence the
//! resulting dead-code warnings per test crate.
#![allow(dead_code)]

use java_formatter_core::config::JavaStyle;
use java_formatter_core::formatter;

/// A pristine default style (IntelliJ built-in defaults).
pub fn default_style() -> JavaStyle {
    JavaStyle::default()
}

/// Build a style by tweaking [`default_style`] via `configure`.
pub fn style(configure: impl FnOnce(&mut JavaStyle)) -> JavaStyle {
    let mut s = default_style();
    configure(&mut s);
    s
}

/// Format `src` with the default style.
pub fn format(src: &str) -> String {
    formatter::format_java(src, &default_style())
}

/// Format `src` with an explicit style.
pub fn format_with(src: &str, style: &JavaStyle) -> String {
    formatter::format_java(src, style)
}
