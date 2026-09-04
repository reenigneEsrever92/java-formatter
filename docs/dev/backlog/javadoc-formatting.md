---
type: ChangeRequest
kind: feature
title: Format Javadoc per the JD_* javadoc options
description: Implement the javadoc formatting options so doc comments are laid out per the scheme.
state: planned
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

The whole "Javadoc" table in `docs/settings/java.md` — `ENABLE_JAVADOC_FORMATTING`, `CLASS_NAMES_IN_JAVADOC` and the `JD_*` options (`JD_ALIGN_PARAM_COMMENTS`, `JD_ALIGN_EXCEPTION_COMMENTS`, `JD_ADD_BLANK_AFTER_PARM_COMMENTS`, `JD_ADD_BLANK_AFTER_RETURN`, `JD_ADD_BLANK_AFTER_DESCRIPTION`, `JD_P_AT_EMPTY_LINES`, `JD_KEEP_INVALID_TAGS`, `JD_KEEP_EMPTY_LINES`, `JD_DO_NOT_WRAP_ONE_LINE_COMMENTS`, `JD_USE_THROWS_NOT_EXCEPTION`, `JD_KEEP_EMPTY_PARAMETER`, `JD_KEEP_EMPTY_EXCEPTION`, `JD_KEEP_EMPTY_RETURN`, `JD_LEADING_ASTERISKS_ARE_ENABLED`, `JD_PRESERVE_LINE_FEEDS`, `JD_PARAM_DESCRIPTION_ON_NEW_LINE`, `JD_INDENT_ON_CONTINUATION`) is all ❌. Comments today are never rewritten — block comments are echoed verbatim by the formatter (R4; the README never-corrupt contract) — so none of the javadoc options apply and a scheme that sets them is only partially honoured (safely ignored, R7). This family is deliberately one request because it needs a javadoc parsing/layout engine rather than a tweak at an existing construct.

# Proposal

Parse the javadoc options into `config::JavaStyle` via the `OPTIONS` registry in `crates/core/src/config.rs` (the `JD_*` XML names as written by the IDE, per the docs/settings note; `CLASS_NAMES_IN_JAVADOC` as an int with values `1`–`3`) and add a javadoc layout pass in `crates/core/src/formatter.rs` that runs when `ENABLE_JAVADOC_FORMATTING` is on: align `@param` / `@throws` descriptions per the `JD_ALIGN_*` options, add blank lines per the `JD_ADD_BLANK_*` options, keep or drop empty tags and lines per the `JD_KEEP_*` options, place `<p>` and leading asterisks per `JD_P_AT_EMPTY_LINES` / `JD_LEADING_ASTERISKS_ARE_ENABLED`, normalise `@exception` → `@throws` per `JD_USE_THROWS_NOT_EXCEPTION`, and wrap/indent per the remaining options. `JD_PRESERVE_LINE_FEEDS` and `JD_KEEP_EMPTY_LINES` govern how conservative the rewrite is; the initial slice may restrict itself to a safe subset — only reformat javadoc whose tags parse cleanly and echo everything else verbatim (R4) — with the full `JD_*` list above as the target, all within this one request.

Docs touched: on delivery the implementation flips the javadoc rows in `docs/settings/java.md` (❌ → ✅ for the applied subset, with a recorded-divergence note for the gating default), updates the README (comments-preserved-verbatim note, honoured-options table), adds a new requirement row to `docs/requirements.md`, and appends `docs/dev/changelog.md`.

# Decisions

- **Gated subsystem, one request.** `ENABLE_JAVADOC_FORMATTING` gates the whole engine, and the other options only shape the rewrite; delivering the engine is a single change request because splitting the `JD_*` options would leave the layout half-configured.
- **Opt-in gate, never-corrupt default.** `ENABLE_JAVADOC_FORMATTING`'s IntelliJ default is `true`, but comments are never rewritten today (R4) and default/absent schemes must stay byte-identical (existing goldens green) — so, mirroring the recorded `RECORD_COMPONENTS_WRAP` divergence in `docs/settings/java.md`, `JavaStyle::default()` ships the gate off (absent → javadoc stays verbatim; a scheme that sets the option explicitly, `true` or `false`, is honoured exactly). The remaining `JD_*` entries carry their table defaults (`JD_ALIGN_PARAM_COMMENTS` / `JD_ALIGN_EXCEPTION_COMMENTS` `true`, `JD_KEEP_*` `true`, `JD_LEADING_ASTERISKS_ARE_ENABLED` `true`, `JD_PRESERVE_LINE_FEEDS` `false`, …).
- **Safe subset first.** Only javadoc whose structure parses cleanly is reformatted; malformed, unusual, or one-line javadoc beyond the subset is echoed byte-for-byte (R4). Reformatting never drops or reorders prose/tags (R5), and `JD_PRESERVE_LINE_FEEDS` / `JD_KEEP_EMPTY_LINES` / `JD_KEEP_INVALID_TAGS` make the conservative knobs explicit, so output is idempotent (R6).
- **One family, one request.** Unlisted javadoc behaviour stays unimplemented and is ignored safely (R7).

# Acceptance criteria

- Fixtures + test files under `crates/core/tests/options/` for the applied subset (e.g. `javadoc_formatting.rs` covering `ENABLE_JAVADOC_FORMATTING` plus each exercised `JD_*` option at its interesting bool/int values): a cleanly parsed javadoc is laid out per the alignment/blank-line/keep defaults, and a messy or invalid-tag javadoc echoes byte-for-byte verbatim.
- With `ENABLE_JAVADOC_FORMATTING` absent (or `false`) javadoc output is byte-identical to today, and explicit-`true` schemes engage the engine.
- The whole suite stays green (`cargo test`) and the new goldens are idempotent.
- The applied `docs/settings/java.md` rows flip to ✅ (with the gate's divergence note), and README / `docs/requirements.md` / `docs/dev/changelog.md` are updated.

# Implementation plan

## Approach

**Configuration (crates/core/src/config.rs).** Add nineteen fields to `JavaStyle`
(L105-150) under a `// --- javadoc ---` block: `enable_javadoc_formatting:
bool`, `class_names_in_javadoc: u32`, and the seventeen `JD_*` bools. `Default`
(L152-182) ships `enable_javadoc_formatting: false` (the recorded gate
divergence — see below) and the table defaults for the rest
(`JD_ALIGN_PARAM_COMMENTS`, `JD_ALIGN_EXCEPTION_COMMENTS`,
`JD_ADD_BLANK_AFTER_DESCRIPTION`, `JD_P_AT_EMPTY_LINES`, `JD_KEEP_INVALID_TAGS`,
`JD_KEEP_EMPTY_LINES`, `JD_USE_THROWS_NOT_EXCEPTION`, `JD_KEEP_EMPTY_PARAMETER`,
`JD_KEEP_EMPTY_EXCEPTION`, `JD_KEEP_EMPTY_RETURN`,
`JD_LEADING_ASTERISKS_ARE_ENABLED` `true`; `JD_ADD_BLANK_AFTER_PARM_COMMENTS`,
`JD_ADD_BLANK_AFTER_RETURN`, `JD_DO_NOT_WRAP_ONE_LINE_COMMENTS`,
`JD_PRESERVE_LINE_FEEDS`, `JD_PARAM_DESCRIPTION_ON_NEW_LINE`,
`JD_INDENT_ON_CONTINUATION` `false`; `CLASS_NAMES_IN_JAVADOC` `1`). Register
each as an `OptionDef` in `OPTIONS` (L232-567) with
`section: Section::JavaCodeStyle` (the javadoc options live in the
`<JavaCodeStyleSettings>` block alongside the record options, per the
docs/settings page) and `group: "Javadoc"`; all are `OptionValue::Bool` except
`CLASS_NAMES_IN_JAVADOC` (`OptionValue::UInt`). `parse_codestyle` /
`serialize_codestyle` and the GUI are registry-driven, so the rows are parsed,
written and rendered automatically.

**Gate divergence.** The table's IntelliJ default for `ENABLE_JAVADOC_FORMATTING`
is `true`, but comments are never rewritten today (R4) and default/absent
schemes must stay byte-identical (existing goldens green). Mirroring the
recorded `RECORD_COMPONENTS_WRAP` divergence in docs/settings/java.md
(L136-141), `JavaStyle::default()` ships the gate `false` and the divergence is
recorded on delivery; a scheme that sets the gate explicitly (`true` or
`false`) is honoured exactly. `CLASS_NAMES_IN_JAVADOC` parses (int 1–3) but its
type-reference rewriting (`{@link …}` / `@see` fully-qualify/shorten + import
management) is not implemented by this request and stays safely ignored (R7);
its docs/settings row keeps ❌ with a parsed-not-applied note.

**Javadoc layout engine (crates/core/src/formatter.rs).** Today every comment
emit site echoes node text verbatim: header comments in `Fmt::program`
(L267-269), class members in `Fmt::class_member` (L760), in-block extras in
`Fmt::block` (L1269-1270), the `line_comment` / `block_comment` arm of
`Fmt::stmt` (L1423), the stray-node fallback in `Fmt::switch_stmt`
(L1837-1838) and the `is_extra` shortcut in `Fmt::expr` (L1993-1994). Add one
helper — `fn javadoc(&self, node: Node, indent: usize) -> Option<String>` —
returning the fully rendered comment (every line prefixed with
`self.ind(indent)`) when the node is a `block_comment` whose text starts with
`/**` and the gate is on, and `None` otherwise (the caller keeps the verbatim
echo). Route all six sites through it; the three call sites that currently
prefix `self.ind(...)` themselves (class_body L722, block L1267, switch_stmt
L1837) skip that prefix for the javadoc branch so the helper's own per-line
indent is not doubled.

The helper has three phases:

1. **Parse + clean check.** Strip the `/**` opener and the trailing `*/`, split
   the body into lines, and drop the leading whitespace plus optional `*` from
   each. A comment is cleanly parseable only when every non-blank line carries
   the `*` prefix, the `*/` terminator is alone on its final line, and each
   tag is well-formed (`@param` / `@throws` / `@exception` followed by a name,
   `@return` bare, other `@tag` lines free text). Anything else — missing `*`
   prefixes, embedded `*/`, malformed tags — is echoed byte-for-byte (R4),
   keeping the messy case verbatim.
2. **Structure.** Split the body into the description block (lines before the
   first tag) and an ordered list of tag blocks (`@param name …`,
   `@throws Type …`, `@return …`, `@exception Type …`, plus unknown tags like
   `@see` / `@since` / `@author`). Inline `{@code …}` / `{@link …}` text is
   preserved verbatim as prose (no type rewriting).
3. **Layout.** Re-emit the description and tags per the options: description
   line breaks kept per `JD_PRESERVE_LINE_FEEDS` or merged per paragraph;
   empty description lines kept per `JD_KEEP_EMPTY_LINES` and rendered as
   `<p>` per `JD_P_AT_EMPTY_LINES`; a blank line after the description per
   `JD_ADD_BLANK_AFTER_DESCRIPTION`; `@param` descriptions aligned to a shared
   column per `JD_ALIGN_PARAM_COMMENTS` (a blank line after the param block
   per `JD_ADD_BLANK_AFTER_PARM_COMMENTS`, the description on its own line per
   `JD_PARAM_DESCRIPTION_ON_NEW_LINE`); `@throws` / `@exception` aligned per
   `JD_ALIGN_EXCEPTION_COMMENTS` and normalised to `@throws` per
   `JD_USE_THROWS_NOT_EXCEPTION`; a blank line after `@return` per
   `JD_ADD_BLANK_AFTER_RETURN`; empty tags dropped per `JD_KEEP_EMPTY_PARAMETER`
   / `JD_KEEP_EMPTY_EXCEPTION` / `JD_KEEP_EMPTY_RETURN`; unknown tags dropped
   per `JD_KEEP_INVALID_TAGS`; continuation lines indented per
   `JD_INDENT_ON_CONTINUATION`; the per-line leading `*` per
   `JD_LEADING_ASTERISKS_ARE_ENABLED`; and one-line javadoc (`/** … */`) kept
   on one line per `JD_DO_NOT_WRAP_ONE_LINE_COMMENTS` or expanded to the
   multi-line form when false. Exact rendering — the alignment column,
   blank-line placement, `<p>` lines, the expanded one-line form — is pinned
   by the goldens and cross-checked against IntelliJ when available.

The rewrite is whitespace/layout only (R5): prose and tag text are preserved,
never reordered or dropped; unparseable shapes keep the verbatim echo (R4);
formatted javadoc re-parses to the same layout, so output is idempotent (R6).

**Interaction with comment-layout.** The sibling CR `comment-layout.md`
(already `planned`) owns the comment-level options — `LINE_COMMENT_AT_FIRST_COLUMN`,
`BLOCK_COMMENT_AT_FIRST_COLUMN`, `LINE_COMMENT_ADD_SPACE_*`,
`KEEP_FIRST_COLUMN_COMMENT`, `WRAP_COMMENTS`. They are **not** part of this
request and the two CRs are deliberately disjoint: javadoc owns the `/** … */`
interior layout, comment-layout owns the outer comment's column / `//`-space /
wrapping. Both touch the same six emit sites, so the implementer coordinates
the routing — the javadoc branch plugs into the shared comment helper if
comment-layout lands first (javadoc detection before column placement), or the
sites are routed here first and comment-layout slots in later; either order
lands cleanly because each helper fully renders its own lines.

**Tests.** One file `crates/core/tests/options/javadoc_formatting.rs`, wired
via `#[path]` in `tests/options.rs`, with fixtures under
`tests/java/javadoc_formatting/` — the CR acceptance criterion names this
single-file organisation explicitly, and since the `JD_*` options are knobs of
one engine (not independent options) the per-option-file convention does not
apply here. Per the AGENTS.md hard rules: golden pairs only (`format_with` /
`format` via the `common` helpers), no inline Java strings, no
`parse_codestyle` tests, no `assert_*` helpers; input and golden share a stem.
A shared clean fixture (a class/method javadoc with a multi-paragraph
description, several `@param`s of differing name length, `@return`, `@throws`,
`@exception`, an empty tag and an unknown tag) is formatted at the defaults
and at each `JD_*` knob's interesting value; a messy fixture (irregular `*`
prefixes, malformed tag) asserts the byte-for-byte echo; gate-absent and
gate-false checks assert byte-identical output.

**Docs.** On delivery: flip the applied javadoc rows in docs/settings/java.md
(❌ → ✅ for the gate and the seventeen `JD_*` rows) with the recorded
divergence note on the gate default, and keep `CLASS_NAMES_IN_JAVADOC` at ❌
with a parsed-not-applied note; update the README (honoured-options table, the
comments-preserved-verbatim note, a javadoc behaviour note); add a requirement
row to docs/requirements.md (R16 is claimed by the comment-layout plan, so R17
unless that CR has already landed — coordinate the number); append the entry
to docs/dev/changelog.md.

## Steps

- [ ] config.rs: add the nineteen javadoc fields + `Default` values to
      `JavaStyle` (gate `false` per the divergence decision, the seventeen
      `JD_*` and `CLASS_NAMES_IN_JAVADOC` per the table) and register the
      nineteen `OptionDef`s in `OPTIONS` (`Section::JavaCodeStyle`,
      `group: "Javadoc"`, `OptionValue::Bool` except `CLASS_NAMES_IN_JAVADOC`
      as `OptionValue::UInt`) — AC1/AC2 config mapping; `cargo test` stays
      green (existing goldens untouched).
- [ ] formatter.rs: add the `javadoc` helper with the parse + clean check
      (delimiter stripping, per-line `*` prefix validation, tag-shape
      validation) returning `None` for non-javadoc / gate-off / unparseable
      nodes — AC1 messy-verbatim, AC2 gate-off byte-identical.
- [ ] formatter.rs: implement the layout phase — description handling per
      `JD_PRESERVE_LINE_FEEDS` / `JD_KEEP_EMPTY_LINES` / `JD_P_AT_EMPTY_LINES`
      / `JD_ADD_BLANK_AFTER_DESCRIPTION`, tag layout per the `JD_ALIGN_*` /
      `JD_ADD_BLANK_AFTER_*` / `JD_KEEP_*` / `JD_USE_THROWS_NOT_EXCEPTION` /
      `JD_PARAM_DESCRIPTION_ON_NEW_LINE` / `JD_INDENT_ON_CONTINUATION`
      options, the leading `*` per `JD_LEADING_ASTERISKS_ARE_ENABLED`, and the
      one-line form per `JD_DO_NOT_WRAP_ONE_LINE_COMMENTS` — AC1.
- [ ] formatter.rs: route all six emit sites (program L267-269, class_member
      L760, block L1269-1270, stmt L1423, switch stray L1837-1838, expr extra
      L1993) through the helper, skipping the call-site `self.ind(...)` prefix
      for the javadoc branch at class_body L722 / block L1267 / switch_stmt
      L1837 — AC1, AC2.
- [ ] tests: `javadoc_formatting.rs` + fixtures under
      `tests/java/javadoc_formatting/`: the clean multi-tag fixture laid out
      at the engine defaults (golden), byte-identical with the gate absent
      (`format` on the default style) and with the gate explicitly `false`,
      and engaged with the gate `true` — AC1, AC2.
- [ ] tests: the per-knob scenarios on the same fixture family —
      `JD_ALIGN_PARAM_COMMENTS` / `JD_ALIGN_EXCEPTION_COMMENTS` off (no column
      alignment), `JD_ADD_BLANK_AFTER_PARM_COMMENTS` /
      `JD_ADD_BLANK_AFTER_RETURN` on (blank lines),
      `JD_ADD_BLANK_AFTER_DESCRIPTION` off, `JD_P_AT_EMPTY_LINES` off,
      `JD_KEEP_EMPTY_LINES` off, `JD_KEEP_INVALID_TAGS` off (unknown tag
      dropped), `JD_KEEP_EMPTY_PARAMETER` / `JD_KEEP_EMPTY_EXCEPTION` /
      `JD_KEEP_EMPTY_RETURN` off (empty tags dropped),
      `JD_USE_THROWS_NOT_EXCEPTION` off (`@exception` kept),
      `JD_LEADING_ASTERISKS_ARE_ENABLED` off,
      `JD_PARAM_DESCRIPTION_ON_NEW_LINE` on, `JD_INDENT_ON_CONTINUATION` on,
      `JD_PRESERVE_LINE_FEEDS` on, and a one-line javadoc under
      `JD_DO_NOT_WRAP_ONE_LINE_COMMENTS` true (kept one line) and false
      (expanded) — AC1.
- [ ] tests: the messy fixture (irregular `*` prefixes / malformed tag)
      asserting byte-for-byte verbatim echo with the gate on — AC1.
- [ ] Register the file in `tests/options.rs` and run `cargo test`: the full
      suite stays green, no existing golden changes, and each new golden is
      idempotent (formatting it again is a no-op) — AC3.
- [ ] Docs: flip the applied rows in docs/settings/java.md to ✅ (gate +
      seventeen `JD_*`) with the gate-divergence note, keep
      `CLASS_NAMES_IN_JAVADOC` at ❌ with a parsed-not-applied note; add the
      options to the README honoured-options table, update the
      comments-preserved-verbatim note and add a javadoc behaviour note; add
      the requirement row to docs/requirements.md (R16 claimed by
      comment-layout → R17 unless landed); append the changelog entry; then
      run `cargo test` once more for a final green suite — AC4, AC3.
- [ ] If an IntelliJ installation is available, format a representative
      javadoc fixture there and align the goldens (alignment column,
      blank-line / `<p>` placement, one-line expansion, continuation indent);
      record the outcome in the changelog.
