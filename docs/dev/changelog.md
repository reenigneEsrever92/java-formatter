---
type: Changelog
title: Changelog
description: Shipped changes to java-formatter, newest first.
tags: [dev, changelog]
---

# Changelog

## 2026-09-03

- **The per-option test suite is now pure golden pairs (per-option-test-suite)**:
  every test formats a `.java` fixture under a specific style and compares
  byte-exact to a `*.out.java` golden next to it, so each option's
  input→output transformation is visible at a glance; inline source strings
  and partial `assert_contains` checks are gone from `crates/core/tests/options/`.
  Tests that were not option related were removed for now (the topic suites
  `config`, `generics`, `idempotency`, `methods`, `parse_errors`, `records`,
  `switch`, `types` and their fixtures, plus idempotency/config-parsing/
  throws-preservation checks inside option files), and the now-unused
  `assert_contains` / `assert_not_contains` / `assert_idempotent` helpers were
  dropped from `tests/common/mod.rs`. The suite is 84 golden tests across the
  25 option files, all green (`cargo test`).

- **The desktop GUI ships with an option registry and IntelliJ-correct value
  encodings (egui-codestyle-editor)**: `crates/gui` is now an egui (eframe)
  codestyle editor instead of a stub — it renders every supported option from
  core's new declarative `OPTIONS` registry with the right control per type
  (bool → checkbox, `u32` → drag value, wrap/brace → labeled combo of the
  IntelliJ meaning), shows a live formatting preview, opens schemes via a
  native file chooser (`rfd`) or drag-and-drop, and saves a minimal
  `<code_scheme>` with only the options that differ from the IntelliJ
  defaults. To make the GUI trustworthy, core gained the registry
  (`Section` / `OptionValue` / `OptionDef` in `crates/core/src/config.rs`)
  as the single source of truth, `parse_codestyle` is now registry-driven,
  a new `serialize_codestyle(style) -> String` writes minimal schemes, and
  the `WrapStyle` / `BraceStyle` integer mappings were corrected to
  IntelliJ's codes (wrap `2` = wrap always, `5` = chop down if long; brace
  `1` = end of line, `3` = next line shifted, `4` = next line shifted 2,
  `5` = next line if wrapped). README wrap/brace tables, the
  `docs/settings` Caveats and the mapping tests were updated in the same
  change; `parse(serialize(style)) == style` is covered by new round-trip
  tests. The backlog's v1 "no `rfd`" decision was revised at the user's
  request — opening uses a native file chooser.

- **The single crate was split into a core/cli/gui Cargo workspace
  (workspace-split)**: the repository is now a virtual workspace under
  `crates/` — `crates/core` (`java-formatter-core`, the formatting library:
  config + formatter modules) with the integration suites, `tests/java/`
  fixtures and Criterion bench moved beside it, `crates/cli`
  (`java-formatter-cli`, the CLI whose binary keeps the `java-formatter` name
  so every documented usage stays valid) and `crates/gui`
  (`java-formatter-gui`, a stub binary so the three-crate structure exists;
  the egui editor is a separate change request). The root `Cargo.toml` is now
  a virtual workspace manifest sharing versions via
  `[workspace.dependencies]`; `codestyle.xml` stays at the root and is reached
  from core's moved tests/benches via adjusted relative `include_str!` paths.
  No formatting, CLI surface, or test behaviour changed — the moved suite
  passes unchanged and `cargo bench` runs from `crates/core/benches/`.
  `examples/` was empty (no `tree_dump.rs` present), so nothing moved there.

- **Generic type-argument spacing is normalised (R14,
  generic-type-argument-spacing)**: type text is no longer echoed verbatim
  from the source at each type site but rendered from the syntax tree with
  canonical IntelliJ spacing — no space inside angle brackets, no space
  before a comma, one space after a comma, and no stray spaces around nested
  brackets (`List< String >` → `List<String>`, `Map<String ,Integer>` →
  `Map<String, Integer>`, `Foo<Bar<Baz > >` → `Foo<Bar<Baz>>`). A small
  type renderer (`flat_type`, plus `flat_type_args`, `flat_type_params`,
  `flat_type_param`, `flat_type_bound`, `flat_type_list` and
  `flat_dimensions` in src/formatter.rs) handles `type_identifier`,
  `scoped_type_identifier`, `generic_type`, arrays, primitives, wildcards
  (`? extends T` / `? super T`) and annotated types, and is routed through
  every verbatim type read: local-variable/field/parameter/spread/enhanced-for
  types, casts and `instanceof` right-hand types, class `extends`/`implements`
  and interface `extends` lists, invocation/`new` `type_arguments`, and
  declaration `type_parameters` (classes, interfaces, records, methods,
  constructors). Unmodelled shapes fall back to the verbatim echo (R4); the
  change is whitespace-only (R5), so correctly spaced input is byte-identical
  and every existing golden stays green, with idempotency verified on the new
  fixture. Covered by the new `tests/generics.rs` suite and the
  `tests/java/types/generic_spacing.java` + `.out.java` golden (field/local/
  param/cast/extends/implements/throws/type-param/wildcard/array/new/
  invocation sites with irregular spacing).

- **Tab indentation is emitted per `USE_TAB_CHARACTER` / `TAB_SIZE` (R13,
  tab-indentation)**: indentation is now tab-aware instead of always
  space-based. With `USE_TAB_CHARACTER`, the indent builder emits one tab per
  full `TAB_SIZE` of width and spaces for the remainder (a tab-stop model
  matching IntelliJ, so `INDENT_SIZE == TAB_SIZE` yields exactly one tab per
  level), and alignment that needs exact columns stays space-based. Column
  arithmetic is routed through a new `col_after` helper (newline resets to 0,
  a tab advances to the next multiple of `TAB_SIZE`), so margin and wrap
  decisions use logical columns and a tab scheme breaks wrapped constructs at
  the same columns as the equivalent space scheme; the default space path is
  byte-identical to before, keeping every existing golden and the idempotency
  suite green. Covered by the new `tests/indent.rs` suite with the
  `tests/java/indent/` fixtures: a `tab_scheme.xml` (tab output at margin 40
  with binary/call wrapping), `tab_indent.java` + `tab_indent.out.java`
  golden (one tab per level, wrapped lines at the tab continuation), a
  logical-column equivalence check against the same settings without tabs,
  and idempotency of both the input and the tab-formatted golden.

- **Simple `try`/`catch`/`finally` and `synchronized` bodies go on one line**
  (R12, one-line-try-catch-blocks)**: `KEEP_SIMPLE_BLOCKS_IN_ONE_LINE` now
  applies to `try`/`catch`/`finally` and `synchronized` statements, not just
  `if`/`else`/`for`/`while`/`do`. When the option is set, the try body and
  every catch/finally body are tested via the existing `one_line_body`
  machinery and the whole statement collapses to one line only when each
  body is a single statement and the assembled form fits the margin;
  otherwise the multi-line layout is kept. `synchronized (lock) { s }`
  collapses the same way, and try-with-resources is included. The option-off
  multi-line path was also fixed where it diverged from the grammar: the
  catch parameter is read from the `catch_formal_parameter` child (the
  old `parameter` field lookup rendered `catch ()`), the `finally` body is
  the `finally_clause`'s plain `block` child (it was dropped entirely), and
  the try-with-resources `resource_specification` already includes its
  parentheses (no double parens). Covered by the new
  `tests/java/control/try_sync_one_line.java` fixture and seven new
  assertions in `tests/control_flow.rs` (collapse per clause, option-off
  multi-line regression, multi-statement bodies stay multi-line,
  next-line brace style blocks collapse, idempotent).

- **Switch statements and switch expressions are formatted (R11,
  switch-formatting)**: instead of echoing the original source text, a
  `switch` is laid out with the header on its own line, `case`/`default`
  labels indented one level and their statements a further level; colon and
  arrow (`case x ->`) forms are preserved and their bodies formatted by the
  existing statement machinery. A switch expression used as a value
  (assignment RHS, return value, argument) stays on one line when the whole
  construct fits the margin and falls back to the multi-line layout
  otherwise; in flat contexts that cannot contain newlines the one-line
  rendering is used, with the verbatim echo (R4) as the fallback for any
  unmodelled shape. tree-sitter-java 0.23 parses both switch statements and
  switch expressions as `switch_expression` nodes, so `stmt` now dispatches
  that kind to the layout (the old `switch_statement` arm was dead code).
  Covered by `tests/control_flow.rs` with the `tests/java/control/`
  `switch_basic.java` (canonical layout unchanged), `switch_messy.java` +
  `.out.java` golden (indentation normalised) and `switch_expression.java`
  (one-line collapse vs multi-line fallback, idempotent) fixtures.
  No IntelliJ installation was available to cross-check the golden; the
  label/body indentation follows IntelliJ's default switch layout.

## 2026-09-02

- **Binary expressions wrap per `BINARY_OPERATION_WRAP` (R10,
  binary-expression-wrapping)**: a long binary expression that exceeds the
  margin is broken at its top-level operators, one operand per line at the
  continuation indent with the operator at the start of the continuation
  line. Wrap codes map as documented: `0` never wraps, `1` wraps only when
  long, `2`/`5` chop down when long (also breaking a nested binary operand
  whose own line overflows), `3` wraps always. `JavaStyle` gained the
  `binary_operation_wrap` field (default `DoNotWrap`), parsed from the JAVA
  `codeStyleSettings` block; the default and do-not-wrap layouts are
  unchanged. Covered by the new `tests/binary.rs` suite with fixtures under
  `tests/java/binary/` (golden `long_sum.out.java` at a tight margin,
  do-not-wrap, chop-down, and wrap-always cases, all idempotent).
  No IntelliJ installation was available to cross-check the golden; the
  operator-placement convention follows the codebase's existing
  continuation style.

- **Parse errors are now reported (R15, parse-error-detection)**: invalid
  Java is surfaced instead of being silently formatted. `format_java_diagnosed`
  returns the formatted source plus up to ten top-most parse diagnostics
  (kind, 1-based line:column); the existing `format_java` delegates and is
  unchanged. The CLI prints each diagnostic as a `warning:` line on stderr and
  still writes best-effort output, exiting 0. The never-corrupt contract
  (unmodeled constructs are preserved verbatim) is now documented in the
  README and covered by the new `tests/parse_errors.rs` suite with the
  `tests/java/errors/syntax_error.java` fixture.

- **Baseline recorded**: Initialized the OKF bundle for the already-shipped
  implementation. The code in `src/` (CLI in `main.rs`, scheme parsing in
  `config.rs`, tree-sitter formatting engine in `formatter.rs`), the
  fixture-based integration suite in `tests/`, and the Criterion benchmarks in
  `benches/` existed before the bundle and are recorded here as the delivery
  of requirements R1–R9 (see [requirements.md](../requirements.md)). It ships
  as crate `java-formatter` v0.1.0 and honours the scheme options documented
  in the repository [README](../../README.md).
