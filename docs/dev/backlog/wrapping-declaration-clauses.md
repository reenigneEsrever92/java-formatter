---
type: ChangeRequest
kind: feature
title: Wrap resource lists, extends/implements and throws lists per their *_WRAP options
description: Implement RESOURCE_LIST_WRAP, EXTENDS_LIST_WRAP, THROWS_LIST_WRAP and related clause-layout sub-options.
state: planned
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

The clause-layout rows of docs/settings/common.md "Wrapping & braces" are all ❌ — `RESOURCE_LIST_WRAP`, `EXTENDS_LIST_WRAP`, `THROWS_LIST_WRAP` with their keyword/paren placement bools and `PREFER_PARAMETERS_WRAP` — valid IntelliJ options java-formatter neither parses nor applies, so schemes setting them are only partially honoured and the options are ignored (R7). Today try-with-resources clauses, `extends`/`implements` lists and `throws` lists render on one line regardless of the margin — the README records they "are preserved." — while method parameters and call arguments already wrap per their options (`METHOD_PARAMETERS_WRAP`, `CALL_PARAMETERS_WRAP` with their `LPAREN`/`RPAREN`-on-next-line bools), so these clause lists are the remaining unwrapped list constructs.

# Proposal

Parse `PREFER_PARAMETERS_WRAP`, `RESOURCE_LIST_WRAP`, `RESOURCE_LIST_LPAREN_ON_NEXT_LINE`, `RESOURCE_LIST_RPAREN_ON_NEXT_LINE`, `EXTENDS_LIST_WRAP`, `EXTENDS_KEYWORD_WRAP`, `THROWS_LIST_WRAP` and `THROWS_KEYWORD_WRAP` into `JavaStyle` via the OPTIONS registry in crates/core/src/config.rs — `OptionDef` entries in the JAVA `codeStyleSettings` block with the IntelliJ built-in defaults from the tables (all `0`/`false`; absent → default), the `*_WRAP` entries reusing the existing `WrapStyle` mapping, the bools as `OptionValue::Bool`. Apply them in crates/core/src/formatter.rs at the constructs they govern: when a resource list, `extends`/`implements` list or `throws` list exceeds the margin (or per wrap-always), break it into one clause per continuation line; the `*_KEYWORD_WRAP`/`*_ON_NEXT_LINE` bools put the keyword or paren on its own line only when the list actually wraps; `PREFER_PARAMETERS_WRAP` favours the parameter list over other wrap points.

Docs touched: `docs/settings/common.md` (❌ → ✅ for these rows), `README.md` (honoured-options table rows + removal of the "…clauses are preserved" limitation), `docs/requirements.md` (a new requirement row) and `docs/dev/changelog.md` on completion.

# Decisions

1. **One family, one request.** Only the listed clause-layout options are added; other `*_WRAP` options still ❌ stay unimplemented and are ignored safely (R7).
2. **Defaults.** IntelliJ built-in defaults (`0`/`false`) per the tables; absent → default, so default/absent schemes keep today's single-line, preserved-clause output byte-identical and existing goldens stay green.
3. **Semantics.** R5: wrapping inserts only newlines and continuation indentation at clause boundaries, never reorders tokens; unmodelled clause shapes stay verbatim (R4); new goldens pin R6 idempotency.
4. **Encodings.** The `*_WRAP` options share the wrap codes `0`/`1`/`2`/`5` already mapped by `WrapStyle`; the keyword/paren-on-next-line bools affect only constructs that actually wrap.

# Acceptance criteria

- Golden fixture + test file per option under crates/core/tests/options/ at the interesting values (wrap codes `0`/`1`/`2`/`5`, both placement-bool states) plus an absent-option default case.
- Long resource / `extends`/`implements` / `throws` lists wrap within the margin under wrap-if-long and always under wrap-always, with `*_KEYWORD_WRAP`/`*_ON_NEXT_LINE` placement honoured on wrapped output.
- Default-scheme output unchanged; whole suite green (`cargo test`).
- `docs/settings/common.md` marks flipped (❌ → ✅); README honoured-options table / behaviour notes and `docs/requirements.md` updated.
- New goldens idempotent: formatting the wrapped output again is a no-op.

# Implementation plan

## Approach

Two sides, mirroring the earlier `binary-expression-wrapping` change: option
plumbing in `crates/core/src/config.rs`, then layout in
`crates/core/src/formatter.rs`, followed by per-option golden tests and the
doc pass. No other crate changes: the GUI and `parse`/`serialize` are driven
by the `OPTIONS` registry, so new `OptionDef` entries are picked up
automatically.

**Configuration** (`crates/core/src/config.rs`). Add eight fields to
`JavaStyle` (constructed only via `Default`, so no literal-site changes):
`resource_list_wrap`, `extends_list_wrap`, `throws_list_wrap` as `WrapStyle`
(default `DoNotWrap`) and `resource_list_lparen_on_next_line`,
`resource_list_rparen_on_next_line`, `extends_keyword_wrap`,
`throws_keyword_wrap`, `prefer_parameters_wrap` as `bool` (default `false`)
— IntelliJ's built-in defaults per the tables, so absent → default keeps
current output. Add eight contiguous `OptionDef` entries to `OPTIONS` right
after `BINARY_OPERATION_WRAP` (`Section::CodeStyleJava`, group `"Wrapping"`
so the GUI shows one block): the three `*_WRAP` entries reuse
`OptionValue::Wrap` / the `WrapStyle` code mapping (0/1/2/5), the five bools
use `OptionValue::Bool` (`true`/`false` literals). `EXTENDS_KEYWORD_WRAP` and
`THROWS_KEYWORD_WRAP` are booleans in real schemes — the docs rows that
currently label them `int`/`0` are corrected in the doc step.

**Rendering** (`crates/core/src/formatter.rs`). R5 holds by construction:
wrapping inserts only newlines + continuation indentation at clause-element
boundaries in source order; anything whose CST shape is not recognised falls
back to today's verbatim echo (R4); the absent/`DoNotWrap` paths stay
byte-identical, so existing goldens keep passing. Column arithmetic follows
the existing `method_decl` pattern: the column where a clause begins is
`c + self.col_after(0, out)` (cursor on the current physical line, so headers
whose annotations or wrapped parameters already contain newlines measure
correctly).

- **`throws` lists** (`method_decl` L799-807, `constructor_decl` L841-849):
  after the parameters, build the flat clause ` throws A, B, C` (identical to
today's text) and let a small shared helper `clause_list(keyword, items,
keyword_wrap, wrap, indent, cur_col)` decide: `DoNotWrap` → flat; otherwise
wrap when `items.len() > 1` and (`WrapAlways` or the flat clause does not fit
from `cur_col`). Wrapped layout: keyword stays on the header line
(`) throws A,`) or, when `throws_keyword_wrap`, moves to its own line at
`self.cont(indent)` (`\n<cont>throws A,`); subsequent exceptions each go on
a `\n<cont>` line. `WrapIfLong` and `ChopDownIfLong` produce the same layout
for these atomic list elements (an element cannot be split further); the
code-5 golden may equal the code-1 golden — record that in the file header.
Single-element lists never split.
- **`extends`/`implements` lists** (`class_decl` L485-488, `iface_decl`
  L514-521, `enum_decl` L544-547, `record_decl` L622-625): same helper, with
  keyword `implements` or `extends`, wrap code `extends_list_wrap`, keyword
  placement `extends_keyword_wrap` (governs both keywords), and items taken
  per-type from the clause's `type_list` via the existing `flat_type`/by-kind
  discovery (fall back to the current `flat_type_list` echo when no
  `type_list` is found). The single-supertype `extends Base` of a class is
  not a list and stays untouched; an interface's `extends A, B` list and
  class/enum/record `implements A, B` lists are what wrap.
- **try-with-resources** (`try_stmt` L1707-1714; `try_one_line` untouched):
  keep the verbatim resource-spec echo when `resource_list_wrap` is
  `DoNotWrap` (byte-identical default). Otherwise render the paren list
  canonically — flat `(r1; r2)` when it fits from the `(` column
  (`c + 4`), else one resource per line mirroring `args_wrapped`'s four
  `(lparen_nl, rparen_nl)` branches with `;` separators and resources at
  `ind(indent + 1)`. Each resource's text is flattened via `normalise_ws`;
  specs containing comments or other unmodelled children fall back to the
  verbatim echo (R4). Confirm the `resource_specification` CST shape
  (resource child kinds, `;` as anonymous separators) with a throwaway
  scratch test first, as `switch-formatting` did, and delete it.
- **`prefer_parameters_wrap`** (`method_inv` L2065-2082): when set and
  `call_parameters_wrap != DoNotWrap`, try the argument-list wrap
  (`inv_wrapped`) *before* the chain wrap; fall through to the chain when the
  arguments stay flat. Default (absent → `false`) keeps today's order
  byte-identical. `assign_expr` needs no change: if the RHS call's arguments
  overflow they already wrap internally, and if they fit the whole line fits.

**Tests** follow `.agents/AGENTS.md` strictly: one golden-pair test file per
option at `crates/core/tests/options/<XML_OPTION>.rs` (wired by
`tests/options.rs`, alphabetically), fixtures under `tests/java/<option>/`,
no inline Java, no `parse_codestyle` tests, no `assert_idempotent` helper.
AC5 (idempotency) is pinned by stable self-goldens: an input identical to the
wrapped output with an identical `*.out.java` — formatting already-wrapped
output is asserted a no-op through an ordinary golden pair.

**Docs.** `docs/settings/common.md` flips the eight rows to ✅ (and corrects
the `EXTENDS_KEYWORD_WRAP`/`THROWS_KEYWORD_WRAP` Type/Default cells to
`bool`/`false`); `README.md` gains the eight honoured-options rows, drops the
"throws / extends / implements clauses are preserved" limitation bullet
(L115-116) in favour of a behaviour note on clause-list wrapping;
`docs/requirements.md` gains requirement R16 (the next free number after
R15); `docs/dev/changelog.md` gets the ship entry `(R16,
wrapping-declaration-clauses)`.

## Steps

- [ ] `crates/core/src/config.rs`: add the eight `JavaStyle` fields (grouped
      comment) + `Default` values; add the eight contiguous `OptionDef`
      entries after `BINARY_OPERATION_WRAP` (Section::CodeStyleJava, group
      "Wrapping", 3 × `Wrap` default `DoNotWrap`, 5 × `Bool` default
      `false`). Verify: `cargo build` and a quick round-trip that an absent
      option parses to the default (mapping, R7).
- [ ] Confirm the CST shapes with a temporary scratch test (print node kinds
      and children of a `resource_specification`, of a class/interface
      `interfaces`/`extends_interfaces` clause and of a `throws` clause),
      record the node/field names in the implementation comments, delete the
      scratch test (AC2 groundwork; `switch-formatting` precedent).
- [ ] Implement the `throws` clause layout in `method_decl` and
      `constructor_decl` via the shared `clause_list` helper honouring
      `throws_list_wrap` and `throws_keyword_wrap`; `DoNotWrap`/absent output
      must be byte-identical to today (AC2 throws, AC3).
- [ ] Implement the `extends`/`implements` clause layout in `class_decl`,
      `iface_decl`, `enum_decl` and `record_decl` via the same helper
      honouring `extends_list_wrap` and `extends_keyword_wrap` (AC2
      extends/implements, AC3).
- [ ] Implement the try-with-resources resource-list layout in `try_stmt`
      (verbatim echo under `DoNotWrap`; canonical flat or wrapped paren list
      per `resource_list_wrap` + the two paren bools), falling back to the
      verbatim echo for unmodelled spec shapes (AC2 resource, AC3, R4/R5).
- [ ] Implement `prefer_parameters_wrap` in `method_inv` (argument-list wrap
      attempted before the chain wrap when set and params can wrap); default
      order and output unchanged (AC2, AC3).
- [ ] Add fixtures and golden-pair test files for each of the eight options
      (AGENTS layout, wired alphabetically in `tests/options.rs`): wrap-code
      tests at `0`/`1`/`2`/`5` for the three `*_WRAP` options plus a
      default/absent golden (e.g. a long resource spec preserved verbatim by
      `format(INPUT)`); both placement-bool states for the five bool options
      (the `*_ON_NEXT_LINE` files pin the sibling bool on/off to isolate the
      behaviour, as `call_parameters_*` files do); a `prefer_parameters_wrap`
      fixture whose overflowing call is the tail of a chain so `true` and
      `false` diverge; and one stable self-golden per wrapped family
      (extends/throws/resources) whose input already matches the wrapped
      output — reformatting it is a no-op (AC1, AC5).
- [ ] Run `cargo test` (whole workspace); every pre-existing golden must stay
      green unchanged (AC3) and the new fixtures must pass; regenerate any
      golden only after eyeballing it for correctness (AC2).
- [ ] If an IntelliJ installation is available, format the wrapped fixtures
      there and align the goldens (paren/keyword placement, continuation
      indent); record the outcome in the changelog entry (fidelity check,
      `binary-expression-wrapping` precedent).
- [ ] Docs pass + final suite: update `docs/settings/common.md` (eight rows
      ❌ → ✅; keyword rows to `bool`/`false`), `README.md` (honoured-options
      rows; replace the "throws / extends / implements clauses are preserved"
      limitation with a clause-wrapping behaviour note),
      `docs/requirements.md` (add requirement R16 and extend the milestone
      delivered list), then run `cargo test` once more and append the
      `(R16, wrapping-declaration-clauses)` entry to `docs/dev/changelog.md`
      (AC4, whole suite green).
