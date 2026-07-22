# AGENTS.md

Cargo workspace: `crates/core` (java-formatter-core: `config.rs` parses
IntelliJ `<code_scheme>` XML via the `OPTIONS` registry, `formatter.rs` is the
tree-sitter engine), `crates/cli`, `crates/gui`.

## Testing conventions (hard rules)

- Every test is a **golden pair**: input `x.java` + byte-exact `x.out.java`,
  asserted `assert_eq!(format_with(INPUT, &style), GOLDEN)` (or `format(INPUT)`
  for the default style). No exceptions.
- One test file per option: `crates/core/tests/options/<XML_OPTION>.rs`, wired
  by `tests/options.rs` via `#[path = "options/<name>.rs"] mod <name>;`.
  Option files start with `use super::common::*;` (never `mod common;`).
- Fixtures in `tests/java/<option>/`, referenced relative to the option file
  (`include_str!("../java/<option>/<scenario>.java")`); input and golden share
  a stem so input→output is visible at a glance. Doc header per file:
  `//! <XML_OPTION> — ...` + `//! Fixtures live under tests/java/<option>/.`
- No inline Java strings, no `assert_contains`/`assert_not_contains`/
  `assert_idempotent` (only `default_style`, `style`, `format`, `format_with`
  exist in `tests/common/mod.rs`), no `parse_codestyle` tests.
- Only option-related tests: don't add topic suites (throws, structural
  layout, parse errors, idempotency, kitchen sink, config XML).

## Workflow

- Changes go through `docs/dev/backlog/` change requests; append to
  `docs/dev/changelog.md` and update README/`docs/` when behaviour changes.
- Leave work uncommitted unless asked.
