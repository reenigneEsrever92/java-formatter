---
type: ChangeRequest
kind: feature
title: Honour the builder-method wrapping options
description: Implement BUILDER_METHODS and KEEP_BUILDER_METHODS_INDENTS for chained builder calls.
state: done
priority: low
tags: [dev, formatter]
owner: maintainer
verified:
  by: maintainer
  at: 2026-09-06T00:00:00Z
---

# Problem

`METHOD_CALL_CHAIN_WRAP` already ships for generic chains (README honoured-options table, `JavaStyle.method_call_chain_wrap`), but the builder-method rows in docs/settings/common.md "Builder method calls" are ❌: `BUILDER_METHODS` (string, comma-separated method names, default `""`) and `KEEP_BUILDER_METHODS_INDENTS` (bool, default `false`).
A scheme that names specific methods gets no builder treatment for their chains, and because the registry's `OptionValue` has no String variant yet the name list cannot even be parsed into `JavaStyle`.
Every chain is formatted as one generic family, so schemes that set these options are only partially honoured — both are safely ignored today (R7).

# Proposal

Add the parsed `BUILDER_METHODS` list and `keep_builder_methods_indents: bool` (default `false`) to `JavaStyle` via `OptionDef` entries in the `OPTIONS` registry in crates/core/src/config.rs — this family introduces `OptionValue::String` (the comma-separated value is split into the `JavaStyle` field; absent → `""`) and the registry/serialize path must round-trip it — then apply them in crates/core/src/formatter.rs at chained-call rendering: chains whose calls match the listed names get the builder treatment for wrapping and indentation (composing with the shipped `METHOD_CALL_CHAIN_WRAP`), and with `KEEP_BUILDER_METHODS_INDENTS` the wrapped continuation lines keep the chain's indentation instead of stepping at the continuation indent.

Docs touched: `docs/settings/common.md` "Builder method calls" marks flipped ❌→✅, the README honoured-options table and formatting-behaviour notes, `docs/requirements.md` (a new requirement row), and `docs/dev/changelog.md` on delivery.

# Decisions

1. **One family, one request.** Only the two listed options ship; the sibling chain options belong to other requests — `WRAP_SEMICOLON_AFTER_CALL_CHAIN` to wrapping-expressions-and-statements and `CHAINED_CALL_INDENT` to the indentation request — and the remaining ❌ rows stay unimplemented and safely ignored (R7).
2. **Defaults.** The empty list and `false` are the IntelliJ built-ins from the table, so default and absent schemes format every chain exactly as `METHOD_CALL_CHAIN_WRAP` does today — byte-identical output and existing goldens stay green.
3. **Semantics.** R5 holds — the builder treatment only inserts line breaks at `.` boundaries; the call sequence and arguments are unchanged. R4 echoes unmodelled chain shapes verbatim; R6 is pinned by re-formatting the new goldens.
4. **Registry.** `OptionValue` currently carries only Bool/UInt/Wrap/Brace scalars; the String variant for `BUILDER_METHODS` (default `""`) must round-trip parse(serialize(style)) == style for both the empty default and a non-empty comma-separated list.

# Acceptance criteria

- `tests/options/builder_methods.rs` (fixtures under `tests/java/builder_methods/`) asserts that a chain of calls named in the list and overflowing the margin gets the builder-treatment golden, while the same fixture with the option absent follows the plain `METHOD_CALL_CHAIN_WRAP` layout.
- `tests/options/keep_builder_methods_indents.rs` asserts `true` vs `false`/absent differ only in the continuation-line indentation of a wrapped builder chain.
- parse(serialize(style)) == style for a non-empty `BUILDER_METHODS` list (`value="a,b"` round-trips) and for the `""` default; default-scheme output is unchanged and the whole suite stays green (`cargo test`).
- docs/settings marks are flipped ❌→✅; the README, `docs/requirements.md` and `docs/dev/changelog.md` are updated with the implementation; the new goldens are idempotent (R6).

# Implementation plan

## Approach

Two sides, as in the binary-expression-wrapping CR: configuration/registry in
`crates/core/src/config.rs`, rendering in `crates/core/src/formatter.rs`, then
goldens and docs. The sibling chain options stay out of scope per Decision 1
(`WRAP_SEMICOLON_AFTER_CALL_CHAIN` → wrapping-expressions-and-statements,
`CHAINED_CALL_INDENT` → the indentation request, `WRAP_FIRST_METHOD_IN_CALL_CHAIN`
→ the wrapping CR).

**Configuration (`crates/core/src/config.rs`).** `JavaStyle` (L105-150) gains two
fields under a new `// --- builder method chains ---` group after
`binary_operation_wrap` (L136): `pub builder_methods: Vec<String>` (default
`Vec::new()`) — the comma-separated XML value is split into this field (trimmed,
empties dropped; absent → `""` → empty vec) — and
`pub keep_builder_methods_indents: bool` (default `false`). `JavaStyle` is
constructed only via `Default` (L152-182), so no literal-site changes are
needed. IntelliJ ground truth (the JAVA `codeStyleSettings` block, verified in
`CommonCodeStyleSettings`): both options are plain scalar entries with defaults
`""` / `false`, and `BUILDER_METHODS` is matched by splitting on `,` and
trimming into exact method names.

`OptionValue` (L202-208) grows `String(String)`. `String` is not `Copy`, so
`Copy` is dropped from the derive (keep `Debug, Clone, PartialEq, Eq`;
`WrapStyle`/`BraceStyle` stay `Copy`). Contained fallout:

- `parse_codestyle` (L696-719) matches `def.default` — moving out of a
  `&OptionDef` works today only because of `Copy` — so switch to `match
  &def.default` (deref the scalar defaults) and add the `String` arm reading a
  new `OptionMap::get_string` helper (beside `get_bool`/`get_wrap`, L629-664).
- `serialize_codestyle` (L740-761) compares `value == def.default` — becomes
  `&value == &def.default` — and the new arm emits the raw string value.

Two `OptionDef` entries are added to the `OPTIONS` registry after
`METHOD_CALL_CHAIN_WRAP` (L421-433), both `Section::CodeStyleJava` (the
`<codeStyleSettings language="JAVA">` block where common.md's "Builder method
calls" table lives): `BUILDER_METHODS` (default `OptionValue::String(String::new())`,
group "Builder methods", `get` joins the vec with `,`, `set` splits/trims) and
`KEEP_BUILDER_METHODS_INDENTS` (default `Bool(false)`). The round-trip property
holds for both the `""` default (empty vec → `""` equals the default → not
serialized) and a non-empty list (`value="a,b"` parses → serializes → parses
back to the same vec).

`crates/gui/src/main.rs` `option_row` (L133-180) matches `&mut OptionValue`
exhaustively, so a `String` arm (a single-line `TextEdit` over the comma-separated
value) must be added for the workspace to compile; the registry-driven options
panel then renders the new option with no other GUI change.

**Rendering (`crates/core/src/formatter.rs`).** Today `method_inv` (L2065)
returns the flat invocation when it fits; otherwise, when `method_call_chain_wrap
!= DoNotWrap`, `collect_chain` (L2181) splits the overflowing chain into `(base,
links)` and `fmt_chain` (L2223) emits the base with the first link attached and
each later link on its own continuation line at `cont(indent)`; `args_wrapped`
(L2144) reuses the same `fmt_chain` for a single argument that is a long chain.
That stays the generic layout. On top of it (composing with
`METHOD_CALL_CHAIN_WRAP` — chains that fit, and chains under `DoNotWrap`, never
reach the new branch):

- **Detection.** A chain is a builder chain when `style.builder_methods` is
  non-empty and every collected link's method name (`Link.name`, L157-161) is in
  the list — IntelliJ's "the whole chain consists of builder methods" rule with
  exact split/trimmed names. Helper `fn is_builder_chain(&self, links:
  &[Link<'s>]) -> bool`.
- **Builder wrapping.** When a builder chain wraps, the base ends its own line
  and **every** `.call()` — including the first — goes on its own line at the
  same per-chain indent. This is the observable BUILDER_METHODS effect at the
  option's default `keep=false`: the wrapped layout of a named chain differs
  from the generic chain (which keeps the first call on the base line), so
  AC1's builder-vs-absent golden pair is distinct without the second option.
  With an empty base the first link still opens the first line as today.
- **Builder indentation.** The wrapped `.call()` lines sit at `cont(indent)`
  when `KEEP_BUILDER_METHODS_INDENTS` is `false`, and at `ind(indent)` — the
  indentation of the chain's own line, i.e. keeping the chain's indentation
  instead of stepping a continuation indent — when it is `true` (AC2). Both are
  the engine's fixed indent strings (`ind`/`cont`, L170-180); no column
  alignment is involved.

`fmt_chain` gains a builder flag (or equivalent parameter) threaded from its two
call sites (`method_inv` L2076 and `args_wrapped` L2148), computed there with
`is_builder_chain`. Default/absent schemes (empty list, `false`) never take the
new branch, so every existing golden stays byte-identical (Decision 2); the
treatment only inserts whitespace at `.` boundaries (R5), unmodelled chain
shapes keep the verbatim / `fmt_chain` fallbacks (R4), and re-formatting the new
goldens is stable (R6).

The exact break/indent convention is pinned by the new goldens below; if an
IntelliJ installation is available to the implementer, format the fixture chain
there with the two options set and adjust the goldens (and this note) if the
observed layout differs — recording the outcome in the changelog, as
binary-expression-wrapping did for operator placement.

**Tests.** Per the `.agents/AGENTS.md` hard rules: two golden-pair modules at
`crates/core/tests/options/<XML_OPTION>.rs` (`builder_methods.rs`,
`keep_builder_methods_indents.rs`) with doc header `//! <XML_OPTION> — …` plus
`//! Fixtures live under tests/java/<option>/.`, opening `use super::common::*;`,
fixtures under `tests/java/<option>/` referenced by `include_str!`, wired
alphabetically into `tests/options.rs`. Styles are built with `style()` /
`format_with` only. Because AGENTS.md bans committed `parse_codestyle` tests,
the AC3 round-trip is verified by hand during the config step (as the
import-on-demand-extensions plan does); the committed tests pin the observable
formatting, idempotency and unchanged default-scheme output.

**Docs.** Flip the two "Builder method calls" rows in `docs/settings/common.md`
(L311-316) to ✅; add the two rows to the README honoured-options table (after
`METHOD_CALL_CHAIN_WRAP`, README L83) and a builder formatting-behaviour note
(plus the GUI control sentence, which gains the string control); add a
requirement row to `docs/requirements.md` at the next free number — R16 at the
earliest, but several sibling CRs also reserve R16, so take the first unused
slot at delivery — and extend the Milestones paragraph; append
`docs/dev/changelog.md` on delivery.

## Steps

- [x] config.rs: add `builder_methods: Vec<String>` and
      `keep_builder_methods_indents: bool` to `JavaStyle` (L136) with `Default`
      values (empty vec / `false`); `cargo check` (AC: config mapping, absent →
      default).
- [x] config.rs: add `OptionValue::String(String)` and drop `Copy` from the
      derive; switch `parse_codestyle` to match `&def.default` with a `String`
      arm via a new `OptionMap::get_string`, and `serialize_codestyle` to
      compare `&value == &def.default` with a `String` arm; add the two
      `OPTIONS` entries after `METHOD_CALL_CHAIN_WRAP`; verify by hand that
      `parse(serialize(style)) == style` for `value="a,b"` and the `""`
      default (per AGENTS no committed `parse_codestyle` test) (AC3
      round-trip).
- [x] crates/gui: add the `OptionValue::String` arm to `option_row`
      (single-line text edit over the comma-separated value) so the exhaustive
      match compiles; `cargo test` builds the workspace (AC: registry option
      editable).
- [x] formatter.rs: add `is_builder_chain` and teach `fmt_chain` the builder
      layout — wrapped builder chain breaks after the base (every `.call()` on
      its own line) and indents those lines at `cont(indent)` or `ind(indent)`
      per `keep_builder_methods_indents` — threading the flag from both call
      sites (`method_inv`, `args_wrapped`); run `cargo test` and confirm no
      existing golden changed (AC1 semantics, Decision 2, default output
      unchanged).
- [x] Add `tests/options/builder_methods.rs` + fixtures under
      `tests/java/builder_methods/`: the overflowing all-builder chain (names
      `setName,setAge,setCity,setZip,build`) under `WrapIfLong` at a tight
      margin gets the builder golden with the list set, the same fixture with
      the list absent reproduces the plain `METHOD_CALL_CHAIN_WRAP` golden, and
      a fits-within-the-margin case stays flat (AC1).
- [x] Add `tests/options/keep_builder_methods_indents.rs` + fixtures under
      `tests/java/keep_builder_methods_indents/`: with the list and wrap set,
      `true` vs `false`/absent differ only in the continuation-line indentation
      of the wrapped builder chain (AC2).
- [x] Wire both modules alphabetically into `tests/options.rs`; assert each new
      golden is idempotent by re-formatting the `*.out.java` with the same
      style (AC3, R6: whole suite green, goldens idempotent).
- [x] If an IntelliJ installation is available, format the fixture chain with
      both options there, align the goldens and the approach note if the
      observed layout differs; record the outcome in the changelog.
- [x] Docs + full suite: flip the two `docs/settings/common.md` "Builder method
      calls" rows to ✅; add the two rows to the README honoured-options table,
      a builder formatting-behaviour note, and the GUI string-control sentence;
      add the new requirement row to `docs/requirements.md` (next free number)
      and extend the Milestones paragraph; append the `docs/dev/changelog.md`
      entry; run `cargo test` and confirm the whole suite is green with
      default-scheme output unchanged (AC4).

Commit: not committed (worktree changes only — the repository is left for the
owner to commit).

## Delivery notes

The plan's Copy fallout was already resolved by the import CRs: `OptionValue`
has been non-`Copy` since `ImportLayout` / `Packages` were added, so adding
`OptionValue::String` needed no derive change — `parse_codestyle` already
matches on `&def.default` and `serialize_codestyle` already compares against
`(def.get)(&JavaStyle::default())`; only the `String` arms and the
`OptionMap::get_string`/`xml_attr_escape` wiring were added. The plan's
`method_inv` (L2076) is `method_inv_ac` in the current code; both actual `fmt_chain`
call sites (`method_inv_ac` and `args_wrapped`) thread the `is_builder_chain`
flag. `CHAINED_CALL_INDENT` composes with the builder continuation indent: with
`KEEP_BUILDER_METHODS_INDENTS` off the builder link lines use the same
`construct_ind`-wrapped continuation width as the generic chain (an explicit
width overrides); with it on the lines sit at `ind(indent)`, where no
continuation width applies — pinned by the goldens. Requirement row R38 was
added (R16 in the plan was stale; R37 was the last row).

No IntelliJ installation was available, so the break/indent convention (break
after the receiver, every `.call()` on its own line at the fixed indent
strings, no column alignment) is pinned by the new goldens rather than
cross-checked against the IDE, as recorded in the changelog. Default / absent
schemes (empty list, `false`) never take the builder branch, so no
pre-existing golden changed; the suite grew from 738 to 747 tests, all green.
