---
type: Changelog
title: Changelog
description: Shipped changes to java-formatter, newest first.
tags: [dev, changelog]
---

# Changelog

## 2026-09-05

- **The align-when-multiline options are honoured (R29, align-multiline-options)**:
  the eighteen alignment options — `ALIGN_MULTILINE_PARAMETERS` (default true),
  `ALIGN_MULTILINE_PARAMETERS_IN_CALLS`, `ALIGN_MULTILINE_RESOURCES` (default
  true), `ALIGN_MULTILINE_FOR` (default true), `ALIGN_MULTILINE_BINARY_OPERATION`,
  `ALIGN_MULTILINE_ASSIGNMENT`, `ALIGN_MULTILINE_TERNARY_OPERATION`,
  `ALIGN_MULTILINE_THROWS_LIST`, `ALIGN_THROWS_KEYWORD`,
  `ALIGN_MULTILINE_EXTENDS_LIST`, `ALIGN_MULTILINE_METHOD_BRACKETS`,
  `ALIGN_MULTILINE_PARENTHESIZED_EXPRESSION`,
  `ALIGN_MULTILINE_ARRAY_INITIALIZER_EXPRESSION`, `ALIGN_MULTILINE_CHAINED_METHODS`,
  `ALIGN_GROUP_FIELD_DECLARATIONS`, `ALIGN_CONSECUTIVE_VARIABLE_DECLARATIONS`,
  `ALIGN_CONSECUTIVE_ASSIGNMENTS` and `ALIGN_SUBSEQUENT_SIMPLE_METHODS` —
  previously ignored (R7) and marked ❌ in the docs/settings/common.md
  Alignment table, are now parsed into `JavaStyle` (eighteen `bool` fields with
  the IntelliJ built-in defaults, plus eighteen `OptionDef` entries, GUI group
  `Alignment`, `Section::CodeStyleJava`) and applied in the engine: a shared
  space-based `align_prefix` helper (the shipped `ALIGN_MULTILINE_RECORDS`
  model) replaces the fixed continuation prefix of a wrapped construct's
  continuation lines with spaces to the first element's column. The declaration
  parameter list / call-argument / try-with-resources / `throws` /
  `extends`-`implements` list layouts keep their first element on the header
  line after `(` / the keyword and pad the remaining element lines under it;
  a wrapped `for` header's cond / update (and enhanced `for` value) pad under
  its first slot; binary / ternary / parenthesized-expression continuation
  lines pad under the first operand / condition / `(`; chained-call link lines
  pad under the first link's dot; a wrapped assignment's right-hand side pads
  to the column right after the operator (its continuation column flows into
  nested non-aligning expressions); `ALIGN_MULTILINE_METHOD_BRACKETS` puts a
  wrapped declaration's own-line `)` under its `(` and `ALIGN_THROWS_KEYWORD`
  puts a wrapped `throws` keyword at its natural header column; and the four
  columnar options pad runs of output-adjacent members / statements — fields,
  one-line methods, local variable declarations and assignment statements with
  no blank line and no comment between them — so the declared names / method
  names / operators share one column. Because the three default-true options
  engage on absent schemes, thirteen wrapped-layout goldens were re-baselined
  to the aligned shape (the `(lparen-stays, rparen-alone)` and wrapped-`for`
  layouts, which previously carried a continuation-indent artifact); every
  other existing golden is byte-identical. Covered by eighteen new per-option
  golden test files under `tests/options/` (one per XML option, each asserting
  the on / off values plus the absent-option default and a self-golden
  idempotency fixture), fixtures under `tests/java/<option>/`; the suite grew
  from 455 to 527 tests, all green (`cargo test --workspace`). No IntelliJ
  installation was available to cross-check the goldens; the layouts follow the
  request's two canonical shapes (first element on the header line with the
  rest aligned under it; own-line elements keep their shared column) and the
  pinned shapes are called out in the option files.

- **Ternary, assert, for-header, array-initialiser and chain wrapping are
  honoured (R27, wrapping-expressions-and-statements)**: the seventeen
  expression / statement wrapping options — `WRAP_FIRST_METHOD_IN_CALL_CHAIN`,
  `PARENTHESES_EXPRESSION_LPAREN_WRAP` / `PARENTHESES_EXPRESSION_RPAREN_WRAP`,
  `BINARY_OPERATION_SIGN_ON_NEXT_LINE`, `TERNARY_OPERATION_WRAP` (default do
  not wrap) / `TERNARY_OPERATION_SIGNS_ON_NEXT_LINE`,
  `PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE`, `ASSERT_STATEMENT_WRAP` (default do not
  wrap) / `ASSERT_STATEMENT_COLON_ON_NEXT_LINE`, `FOR_STATEMENT_WRAP` (default
  do not wrap) / `FOR_STATEMENT_LPAREN_ON_NEXT_LINE` /
  `FOR_STATEMENT_RPAREN_ON_NEXT_LINE`, `ARRAY_INITIALIZER_WRAP` (default do
  not wrap) / `ARRAY_INITIALIZER_LBRACE_ON_NEXT_LINE` /
  `ARRAY_INITIALIZER_RBRACE_ON_NEXT_LINE`, `MODIFIER_LIST_WRAP` and
  `WRAP_SEMICOLON_AFTER_CALL_CHAIN` (in the `JavaCodeStyleSettings` block) —
  previously ignored (R7) and marked ❌ in the docs/settings tables, are now
  parsed into `JavaStyle` (four `WrapStyle` fields and thirteen `bool` fields
  with the IntelliJ built-in defaults, plus seventeen `OptionDef` entries in
  the `OPTIONS` registry) and applied in the engine: `Fmt::ternary` wraps per
  `TERNARY_OPERATION_WRAP` at `?` / `:` with `TERNARY_OPERATION_SIGNS_ON_NEXT_LINE`
  steering the signs between the operator-end default and the
  signs-on-continuation layout (code `5` chop-down recursing into nested
  ternary sides via `Fmt::ternary_operand`, mirroring `binary_operand`);
  `Fmt::assert_stmt` wraps at the expression and after the `:` per
  `ASSERT_STATEMENT_WRAP` with `ASSERT_STATEMENT_COLON_ON_NEXT_LINE`;
  `Fmt::for_stmt` / `Fmt::enhanced_for` re-render the header from its
  init/condition/update fields and break it at the semicolons / `:` per
  `FOR_STATEMENT_WRAP` (the verbatim raw-header path stays for do-not-wrap),
  honouring the two paren-on-next-line bools; `Fmt::array_init` wraps one
  element per line per `ARRAY_INITIALIZER_WRAP` with the brace bools placing
  `{` / `}` on their own lines (the default keeps both braces at the end of
  their lines, and the `=` / `[` joins drop their separator when the brace
  moves to its own line so no trailing whitespace appears); `Fmt::mods_tail`
  breaks after the modifier list at the eight `modifiers()` call sites under
  `MODIFIER_LIST_WRAP`; `Fmt::fmt_chain` puts the first link on a
  continuation line under `WRAP_FIRST_METHOD_IN_CALL_CHAIN` (with an empty
  receiver there is nothing to wrap after, so the first link stays); the
  `expression_statement` arm puts the `;` of a wrapped chained statement on
  its own line under `WRAP_SEMICOLON_AFTER_CALL_CHAIN`; `Fmt::assign_expr`
  moves the operator to the start of the continuation line under
  `PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE`; and the `parenthesized_expression`
  arm puts the parens on their own lines under `PARENTHESES_EXPRESSION_LPAREN/RPAREN_WRAP`
  when the inner expression wraps. Layout / whitespace only (R5); absent
  options keep the defaults and default output stays byte-identical except
  the re-baselined wrapped `binary_operation_wrap` goldens (whose operator
  is now at the end of the line, the faithful false state of
  `BINARY_OPERATION_SIGN_ON_NEXT_LINE`), and every new golden was
  re-formatted under its own style and confirmed byte-identical (R6) —
  including explicit self-goldens for the wrapped families. Covered by
  seventeen new per-option golden test files under `tests/options/`
  (alphabetically wired in `tests/options.rs`), each asserting its
  interesting values plus the absent-option default, with fixtures under
  `tests/java/<option>/`; the suite grew from 391 to 455 tests, all green
  (`cargo test --workspace`). No IntelliJ installation was available to
  cross-check the goldens; the layouts follow the request's decisions and
  the existing call-parameter / binary wrap conventions.

- **Resource, extends / implements and throws lists wrap per their `*_WRAP`
  options (R26, wrapping-declaration-clauses)**: the eight clause-layout
  options — `RESOURCE_LIST_WRAP` (default do not wrap),
  `RESOURCE_LIST_LPAREN_ON_NEXT_LINE` / `RESOURCE_LIST_RPAREN_ON_NEXT_LINE`
  (default false), `EXTENDS_LIST_WRAP` (default do not wrap),
  `EXTENDS_KEYWORD_WRAP` (default false, a boolean in real schemes),
  `THROWS_LIST_WRAP` (default do not wrap), `THROWS_KEYWORD_WRAP` (default
  false, likewise boolean) and `PREFER_PARAMETERS_WRAP` (default false) —
  previously ignored (R7) and marked ❌ in the docs/settings/common.md
  "Wrapping & braces" tables (whose keyword rows claimed `int`/`0`), are now
  parsed into `JavaStyle` (three `WrapStyle` fields and five `bool` fields
  with the IntelliJ built-in defaults, plus eight contiguous `OptionDef`
  entries in the `OPTIONS` registry after `BINARY_OPERATION_WRAP` under the
  JAVA `codeStyleSettings` block, GUI group "Wrapping") and applied in the
  engine: a shared `Fmt::clause_list` helper lays out `throws` lists
  (`method_decl`, `constructor_decl`) and the `extends` / `implements` lists
  of class / interface / enum / record headers (`Fmt::append_type_clause`
  over the clause `type_list`), breaking one element per continuation line
  at `cont(indent)` when the flat clause overflows under wrap-if-long /
  chop-down (`1` / `5` — identical for these atomic elements) or always
  under wrap-always (`2`), with `*_KEYWORD_WRAP` moving the keyword to its
  own continuation line and single-element lists never splitting;
  `Fmt::resource_list` renders try-with-resources paren lists canonically
  (flat `(r1; r2)` when they fit, else one resource per line at
  `ind(indent + 1)` mirroring the call-parameter paren layout, honouring
  the two paren-on-next-line bools), falling back to the verbatim echo for
  spec shapes with comments or other unmodelled children (R4); and
  `Fmt::method_inv` honours `PREFER_PARAMETERS_WRAP` by wrapping the tail
  call's overflowing arguments before breaking its chain. Layout /
  whitespace only (R5); absent options keep the defaults and default
  output stays byte-identical (no existing golden changed), and every new
  golden was re-formatted under its own style and confirmed byte-identical
  (R6) — including explicit self-goldens for the three wrapped families
  (extends, throws, resources). Covered by eight new per-option golden test
  files under `tests/options/` (`extends_keyword_wrap.rs`,
  `extends_list_wrap.rs`, `prefer_parameters_wrap.rs`,
  `resource_list_lparen_on_next_line.rs`, `resource_list_rparen_on_next_line.rs`,
  `resource_list_wrap.rs`, `throws_keyword_wrap.rs`, `throws_list_wrap.rs`),
  each asserting its interesting values plus the absent-option default, with
  fixtures under `tests/java/<option>/`; the suite grew from 362 to 391
  tests, all green (`cargo test --workspace`). No IntelliJ installation was
  available to cross-check the goldens; the layouts follow the request's
  decisions and the existing call-parameter wrap conventions.

- **The switch / case indentation and wrapping options are honoured (R25,
  switch-case-layout)**: the four layout options — `INDENT_CASE_FROM_SWITCH`
  (default true), `CASE_STATEMENT_ON_NEW_LINE` (default true),
  `INDENT_BREAK_FROM_CASE` (default true) and `SWITCH_EXPRESSIONS_WRAP`
  (default wrap if long) — previously ignored (R7) and marked ❌ in the
  docs/settings/common.md "Braces & indentation" and "Wrapping & braces →
  Expressions and statements" tables, are now parsed into `JavaStyle` (three
  `bool` fields and one `WrapStyle` field with the IntelliJ built-in defaults
  and four `OptionDef` entries in the `OPTIONS` registry — the flags under
  the "Braces" GUI group next to the clause rows,
  `SWITCH_EXPRESSIONS_WRAP` under "Wrapping", all in the JAVA
  `codeStyleSettings` block) and applied in the engine:
  `Fmt::switch_stmt` computes the label / statement indent levels from
  `INDENT_CASE_FROM_SWITCH` (labels one level below the `switch` when on,
  at the `switch` indent when off) and threads them through `switch_group`,
  `switch_rule` and the comment fallback; `Fmt::switch_group` joins the
  group's first single-line statement onto the label's line when
  `CASE_STATEMENT_ON_NEW_LINE` is off (following statements still start
  their own lines) and renders `break` / `continue` / `return` at the label
  level when `INDENT_BREAK_FROM_CASE` is off; and `Fmt::switch_expr` decides
  the one-line vs multi-line layout of a switch expression used as a value
  per `SWITCH_EXPRESSIONS_WRAP` (`0` do not wrap keeps the one-line form
  whenever one exists, `1` wrap if long is the shipped fits-based default,
  `2` wrap always, `5` chop down if long additionally breaks an overflowing
  nested switch expression in the body via `Fmt::switch_rule`), with
  statement-position switches and the `flat` echo untouched. Layout /
  whitespace only (R5), absent options keep the defaults, default and
  absent-scheme output stays byte-identical (no existing golden changed),
  and each new golden was re-formatted under its own style and confirmed
  byte-identical (R6). Covered by four new per-option golden test files
  under `tests/options/` (`indent_case_from_switch.rs`,
  `case_statement_on_new_line.rs`, `indent_break_from_case.rs`,
  `switch_expressions_wrap.rs`), each asserting its interesting values plus
  the absent-option default, with fixtures under `tests/java/<option>/`; the
  suite grew from 350 to 362 tests, all green (`cargo test --workspace`). No
  IntelliJ installation was available to cross-check the goldens; the case
  layouts and the chop-down behaviour follow the settings table and the
  request's decisions.

## 2026-09-03

- **The clause-keyword and brace-less control-statement layout options are
  honoured (R24, clause-keyword-layout)**: the seven layout options —
  `ELSE_ON_NEW_LINE`, `WHILE_ON_NEW_LINE`, `CATCH_ON_NEW_LINE` and
  `FINALLY_ON_NEW_LINE` (default false), `SPECIAL_ELSE_IF_TREATMENT` (default
  true), `LAMBDA_BRACE_STYLE` (default end of line) and
  `KEEP_CONTROL_STATEMENT_IN_ONE_LINE` (default true) — previously ignored
  (R7) and marked ❌ in the docs/settings/common.md "Braces & indentation" /
  "General & comments" tables, are now parsed into `JavaStyle` (six `bool`
  fields and one `BraceStyle` field with the IntelliJ built-in defaults and
  seven `OptionDef` entries in the `OPTIONS` registry, the flags under the
  "Braces" GUI group next to the existing brace rows and
  `KEEP_CONTROL_STATEMENT_IN_ONE_LINE` under "One-liners", all in the JAVA
  `codeStyleSettings` block) and applied in the engine: `Fmt::if_stmt` puts
  the alternative's `else` keyword on a fresh line at the statement indent
  under `ELSE_ON_NEW_LINE` (with the `if_one_line` collapse suppressed for
  any chain with an alternative) and, with `SPECIAL_ELSE_IF_TREATMENT` off,
  rewrites each fused `else if` level as an explicit `else { if … }` block
  (the braces group a single `if`, keeping R5 and R6); `Fmt::try_stmt` starts
  each `catch` / `finally` clause on a fresh line under `CATCH_ON_NEW_LINE` /
  `FINALLY_ON_NEW_LINE` (and the `try_one_line` collapse is gated on them);
  `Fmt::do_while` starts its trailing `while (…);` on a fresh line under
  `WHILE_ON_NEW_LINE` (collapse gated on the flag, own-line brace-less bodies
  keep their pinned tail); every brace-less control-statement body
  (`Fmt::stmt_as_block_or_inline` in if / else / for / enhanced-for / while
  and `Fmt::do_while`'s own arm) is kept on its header's line under
  `KEEP_CONTROL_STATEMENT_IN_ONE_LINE` when the source gap to the body holds
  no line break, and moved to its own line when the option is off; and
  `Fmt::lambda` renders block bodies through the lambda brace style — the
  `NextLine` family puts the `{` on its own line at the statement indent,
  independently of `BRACE_STYLE`, and the simple-lambda one-line collapse is
  gated on an inline-compatible brace style. Layout/whitespace only (R5),
  absent options keep the defaults, default and absent-scheme output stays
  byte-identical (no existing golden changed), and each new golden was
  re-formatted under its own style and confirmed byte-identical (R6).
  Covered by seven new per-option golden test files under `tests/options/`
  (`else_on_new_line.rs`, `while_on_new_line.rs`, `catch_on_new_line.rs`,
  `finally_on_new_line.rs`, `special_else_if_treatment.rs`,
  `keep_control_statement_in_one_line.rs`, `lambda_brace_style.rs`), each
  asserting the interesting values plus the absent-option default, with
  fixtures under `tests/java/<option>/`; the suite grew from 334 to 350
  tests, all green (`cargo test`). No IntelliJ installation was available to
  cross-check the goldens; the clause and brace layouts follow the settings
  table and the request's decisions.

- **The comment layout options are honoured (R23, comment-layout)**: the six
  comment options — `LINE_COMMENT_AT_FIRST_COLUMN` (default true),
  `BLOCK_COMMENT_AT_FIRST_COLUMN` (default true),
  `LINE_COMMENT_ADD_SPACE_ON_REFORMAT` (default false),
  `LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION` (default false),
  `KEEP_FIRST_COLUMN_COMMENT` (default true) and `WRAP_COMMENTS` (default
  false) — previously ignored (R7) and marked ❌ in the docs/settings/common.md
  "General & comments" table, are now parsed into `JavaStyle` (six `bool`
  fields with the IntelliJ built-in defaults from the table and six
  `OptionDef` entries in the `OPTIONS` registry under a new "Comments" GUI
  group, all in the JAVA `codeStyleSettings` block) and applied in the
  engine: every standalone-comment emit site (file-header comments in
  `Fmt::program`, comment members of class / interface / record / anonymous
  bodies (`Fmt::class_body`), enum-body declaration members (`Fmt::enum_body`)
  and the `Fmt::class_member` comment arm, in-block extras (`Fmt::block`), the
  `Fmt::stmt` comment arm and switch-block strays (`Fmt::switch_stmt` /
  `Fmt::switch_group`), and the inline extra shortcut in `Fmt::expr`) now
  routes through one shared `comment` helper that decides, in order: the
  column — a source first-column comment stays there when
  `KEEP_FIRST_COLUMN_COMMENT` is on, otherwise the per-kind `*_AT_FIRST_COLUMN`
  toggle pins the comment to column 1, else the contextual indent (the call
  sites drop their indent prefix for column-1 comments); the space after `//`
  — `LINE_COMMENT_ADD_SPACE_ON_REFORMAT` inserts one space after `//` of an
  ordinary line comment when absent, while `//noinspection` suppression
  comments are governed separately by `LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION`
  (a space there would break the suppression); and `WRAP_COMMENTS`
  word-wraps a single-line comment longer than the right margin, repeating
  the comment's column prefix on continuation lines (`//` for line comments,
  aligned ` * ` text for block comments) — multi-line block comments keep
  their source text verbatim (R4). Comment text is preserved: indentation,
  the optional space and line breaks only change (R5), and each new golden
  was re-formatted under its own style and confirmed byte-identical (R6).
  Default and absent-scheme output stays byte-identical for comment-free
  sources — the existing suite needed no golden regeneration — and the
  built-in defaults (true/true) place comments at column 1, matching
  IntelliJ. Covered by six new per-option golden test files under
  `tests/options/` (`line_comment_at_first_column.rs`,
  `block_comment_at_first_column.rs`, `keep_first_column_comment.rs`,
  `line_comment_add_space_on_reformat.rs`,
  `line_comment_add_space_in_suppression.rs`, `wrap_comments.rs`), each
  asserting both values plus the absent-option default, with fixtures under
  `tests/java/<option>/`; the suite grew from 314 to 334 tests, all green
  (`cargo test`). No IntelliJ installation was available to cross-check the
  goldens; the column / space / wrap layouts follow the settings table and
  the request's decisions.

- **The line-length and line-ending options are honoured (R22,
  line-length-and-line-endings)**: the root-level `RIGHT_MARGIN` (default
  `120`) and `LINE_SEPARATOR` (default system) options and the JAVA-block
  `WRAP_LONG_LINES` (default false) / `KEEP_LINE_BREAKS` (default true)
  toggles — previously ignored (R7) and marked ❌ in docs/settings/common.md
  — are now parsed into `JavaStyle` (a new `LineSeparator` enum over
  System / LF / CRLF / CR with a resolve helper, a new
  `OptionValue::LineSep` registry variant serialised in the XML-escaped
  `&#10;` / `&#13;&#10;` / `&#13;` forms, three new fields, and four new
  `OptionDef` entries: `RIGHT_MARGIN` registered before `SOFT_MARGINS` so
  the latter keeps precedence when a scheme sets both, with absent options
  now skipped by `parse_codestyle` so the earlier shared-field option keeps
  its parsed value) and applied in the engine: a finalisation helper in
  `format_java_diagnosed` collapses verbatim CRLF echoes, trims to one
  trailing line end and substitutes the configured separator at every line
  end including the final newline (LF output byte-identical to before); a
  deterministic `WRAP_LONG_LINES` post-pass hard-wraps over-margin lines at
  the rightmost whitespace at or before the margin (continuations at the
  line's indent plus `CONTINUATION_INDENT_SIZE`), scanning each line
  string/char/comment/text-block aware so literals and comments are never
  split and comment-only lines are skipped; and the listed layout sites
  (call argument lists, declaration parameter lists, initialisers,
  binary/ternary spines, chains, annotation arguments, `new` arguments and
  array initialisers) render their canonical wrapped layout when
  `KEEP_LINE_BREAKS` is on and the construct's source carries a line break
  at its own join level, while `false` keeps the flatten-if-fits reflow.
  The GUI gained a `LineSep` combo arm (Margins group). Default and
  absent-scheme output stays byte-identical: the existing suite needed no
  golden regeneration, and each new golden was re-formatted under its own
  style to confirm idempotency (R6). The existing `SOFT_MARGINS` test
  module, misnamed `right_margin.rs`, was renamed to `soft_margins.rs`
  (fixtures moved to `tests/java/soft_margins/`). Covered by new per-option
  golden test files `right_margin.rs`, `line_separator.rs` (CRLF and CR
  goldens store the real separator bytes), `keep_line_breaks.rs` and
  `wrap_long_lines.rs`; the suite grew from 300 to 314 tests, all green
  (`cargo test`). No IntelliJ installation was available to cross-check the
  goldens; the wrap/keep goldens pin `KEEP_LINE_BREAKS` off so the hard-wrap
  round trip is deterministic.

- **The spacing-around-operators options are honoured (R18,
  spaces-around-operators)**: the `SPACE_AROUND_*` toggles — assignment,
  logical, equality, relational, bitwise, additive, multiplicative, shift,
  unary, lambda arrow and the method-reference `::`, plus `SPACE_AFTER_TYPE_CAST`
  — previously ignored (R7) and marked ❌ in docs/settings/common.md are now
  parsed into `JavaStyle` (twelve `bool` fields with the IntelliJ built-in
  defaults from the settings table, twelve `OptionDef` entries under a new
  "Spaces" GUI group in the `OPTIONS` registry) and applied in the engine: a
  small operator-classifying helper returns the separator (one space when the
  toggle is on, nothing when off) and every emission site is routed through it
  in both the structured `expr` renderer and the newline-free `flat` renderer —
  binary expressions (flat joins, the wrapped `BINARY_OPERATION_WRAP`
  continuation lines, and the chop-down recursion), assignments (statement
  form, flat form, the wrapped `assign_expr` path, and the `field_decl` /
  `local_var` declarator joins), unary arms plus the `update_expression`
  rebuild (`i ++` now canonicalises to `i++`), the lambda `->` separator with
  body-column bookkeeping derived from the emitted separator, the
  `method_reference` rebuild (`A::new`, on → `A :: new`; unexpected shapes and
  comment-bearing nodes fall back to the verbatim echo, R4) and the cast
  separator. The column constants that assumed the old canonical spacing (`c +
  ty.len() + 2`, `+ name.len() + 3`, `+ op.len() + 2`, `c + left.len() + 4`, `c
  + params.len() + 4`) were replaced by arithmetic over the separator actually
  emitted so margin/wrap decisions stay exact. One deliberate consequence:
  `SPACE_AFTER_TYPE_CAST` defaults `true`, so a cast now renders `(int) x`
  (matching IntelliJ) instead of the old `(int)x` — a fidelity fix, not a
  regression, since no existing golden contains a cast or a method reference.
  The change is whitespace-only (R5), inserting/removing one space is
  idempotent (R6), and default/absent schemes keep byte-identical goldens.
  Covered by twelve new per-option golden test files
  (`tests/options/space_around_*.rs`, `space_after_type_cast.rs`) each testing
  the option toggled away from its default plus the absent-option default;
  `space_around_additive_operators` also covers the wrapped long-sum (margin
  40 + `WrapIfLong` + off → glued `+beta()` continuation lines) and
  `space_around_lambda_arrow` one-line lambdas under
  `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE`; the suite grew from 160 to 186 tests, all
  green (`cargo test`). No IntelliJ installation was available to cross-check
  the cast / unary-on / `::`-on goldens; the defaults follow the settings
  table in docs/settings/common.md.

- **Braces are forced on statement bodies per the `*_BRACE_FORCE` options (R17,
  force-braces)**: `IF_BRACE_FORCE`, `FOR_BRACE_FORCE`, `WHILE_BRACE_FORCE` and
  `DOWHILE_BRACE_FORCE` — previously ignored (R7) and marked ❌ in
  docs/settings/common.md — are now parsed into `JavaStyle` (four `ForceStyle`
  fields, `OptionValue::Force`, a `get_force` decoder beside `get_wrap` /
  `get_brace`, and four `OptionDef` entries in the Braces group of the `OPTIONS`
  registry, so the GUI lists them as labeled combos like the brace styles) and
  applied in the engine: a brace-less single-statement body of the governed
  construct is wrapped in `{ … }` with the statement indented one level, using
  exactly the bytes a real block would render so a forced body and a braced
  source converge on identical canonical output. Force codes follow
  docs/settings/index.md: `3` always braces, `1` (force if multiline) braces
  only when the rendered body spans multiple lines (a brace-less nested
  statement body, e.g. `for (...) if (c) x();`), `0` and out-of-set values fall
  back to do-not-force and keep today's output byte-for-byte. `if` governs both
  its consequence and a brace-less `else` body; the classic and enhanced `for`
  both count as `for` bodies. Braces are only ever added, never stripped, so
  reformatting braced output stays a no-op (R6) and the insertion is
  whitespace-only (R5); default/absent schemes are unchanged and every existing
  golden stays green. Covered by four new per-option golden test files
  (`tests/options/if_brace_force.rs`, `for_brace_force.rs`,
  `while_brace_force.rs`, `dowhile_brace_force.rs`) at force codes `0`/`1`/`3`
  plus an absent-option default and an already-braced idempotency pair each;
  the suite grew from 140 to 160 tests, all green (`cargo test`). No IntelliJ
  installation was available to cross-check the goldens; the convention recorded
  here is that both the classic and the enhanced `for` are governed by
  `FOR_BRACE_FORCE`, `else` bodies by `IF_BRACE_FORCE`, and an unbraced
  `do` body keeps the engine's pre-existing brace-less layout (the closing
  `while` line carries the engine's canonical spacing) unless forced.

- **The blank-line policy options are honoured (R16, blank-line-policy)**: the
  `KEEP_BLANK_LINES_*` caps and `BLANK_LINES_*` minimums are parsed into
  `JavaStyle` (19 new fields; `OptionDef` entries in the `OPTIONS` registry
  under a new "Blank lines" GUI group — the common rows serialize under the
  JAVA `codeStyleSettings` block, `BLANK_LINES_AROUND_INITIALIZER` and
  `BLANK_LINES_AROUND_FIELD_WITH_ANNOTATIONS` under `<JavaCodeStyleSettings>`)
  and applied in the engine. Vertical spacing now follows one shared rule
  `emitted = max(min(existing, keep_cap), required_min)` at every gap:
  `KEEP_BLANK_LINES_IN_CODE` / `KEEP_BLANK_LINES_BEFORE_RBRACE` govern
  statement blocks, `BLANK_LINES_BEFORE_METHOD_BODY` leads method bodies,
  `Fmt::program` spaces package/import/type boundaries per the
  `BLANK_LINES_*`/keep options (the `java.*`/`javax.*` group separator stays an
  import-layout convention), and class/enum bodies space members by their
  per-kind around minimum (fields — annotated fields use the annotations
  variant — methods/constructors, nested types, initializers, the
  `*_IN_INTERFACE` variants, `AFTER_CLASS_HEADER`,
  `AFTER_ANONYMOUS_CLASS_HEADER`, `BEFORE_CLASS_END`). The new engine keeps
  source blank runs up to the configured caps, so fields are no longer forced
  apart and 2+ blank runs are preserved, and a previously unmodelled
  construct — anonymous class bodies — is now rendered (their `class_body`
  child is found by kind, since the grammar gives it no field name) so the
  anonymous-class-header option applies. Default output is unchanged wherever
  it already matched IntelliJ; the five `tab_indent` goldens that removed the
  blank line after a class header were regenerated (IntelliJ keeps it up to
  `KEEP_BLANK_LINES_IN_DECLARATIONS`). Covered by 19 new per-option test files
  (one per XML option, golden pairs under `tests/java/<option>/`, including an
  absent-option default check per file and idempotent goldens); the suite grew
  from 84 to 140 tests, all green (`cargo test`).

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

- **The spacing-around-separators options are honoured (R19,
  spaces-around-separators)**: the `SPACE_AFTER_COMMA`,
  `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS`, `SPACE_BEFORE_COMMA`,
  `SPACE_AFTER_SEMICOLON`, `SPACE_BEFORE_SEMICOLON`, `SPACE_BEFORE_QUEST`,
  `SPACE_AFTER_QUEST`, `SPACE_BEFORE_COLON`, `SPACE_AFTER_COLON`,
  `SPACE_BEFORE_TYPE_PARAMETER_LIST` and `SPACE_BEFORE_COLON_IN_FOREACH`
  options — previously ignored (R7) and marked ❌ in the docs/settings
  separator tables — are now parsed into `JavaStyle` (eleven `bool` fields
  with the IntelliJ built-in defaults from the settings tables:
  `SPACE_BEFORE_COMMA`, `SPACE_BEFORE_SEMICOLON` and
  `SPACE_BEFORE_TYPE_PARAMETER_LIST` default false, the rest true — equal to
  today's canonical output, so absent/default schemes keep byte-identical
  goldens; eleven `OptionDef` entries in a new separator-spacing group of the
  `OPTIONS` registry, ten in the JAVA `codeStyleSettings` block and
  `SPACE_BEFORE_COLON_IN_FOREACH` in `<JavaCodeStyleSettings>`) and applied
  in the engine: a `comma_sep(after)` helper builds the comma separator from
  `SPACE_BEFORE_COMMA` and the after toggle, and every single-line comma join
  is routed through it — calls (`flat_args`), declarations (`formal_params`,
  `flat_formal_params`, `field_decl`, `local_var` declarator lists),
  annotations (`flat_ann_args`), arrays (`flat_arr_init`), record components
  (`record_components`), lambda inferred parameters, `throws` lists and
  `implements`/`extends` type lists (`flat_type_list`) under
  `SPACE_AFTER_COMMA`, while `flat_type_args` uses
  `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS` (the `",\n"` wrapped joins stay as-is:
  a newline already replaces the after space). The `for` header keeps its
  raw echo but normalises the whitespace around each `;` per
  `SPACE_BEFORE_SEMICOLON` / `SPACE_AFTER_SEMICOLON` (never inserting a
  space before `)`) and falls back to rebuilding the header from the
  statement's init/condition/update children on awkward empty-slot edges
  (`for (;;)` stays compact). Ternary rendering (both the `ternary` function
  and the flat ternary arm) builds the `?` / `:` separators from
  `SPACE_BEFORE_QUEST` / `SPACE_AFTER_QUEST` and `SPACE_BEFORE_COLON` /
  `SPACE_AFTER_COLON` instead of the hard-coded `" ? "` / `" : "`; the
  enhanced-`for` colon takes its before space from
  `SPACE_BEFORE_COLON_IN_FOREACH` and its after space from `SPACE_AFTER_COLON`;
  and `SPACE_BEFORE_TYPE_PARAMETER_LIST` inserts the name→`<…>` gap in
  `class_decl` / `iface_decl` / `record_decl` (default off keeps
  `class Foo<T>`, generic method/constructor type-parameter lists are left
  alone). Deliberately unchanged (defaults already match today):
  statement-terminating `;`, switch `case`-label colons, the assert colon and
  enum-constant commas. The change is whitespace-only (R5), inserting or
  removing a single space is idempotent (R6 — verified by formatting each
  new golden under its own style), and unmodelled shapes are echoed verbatim
  (R4). Covered by eleven new per-option golden test files under
  `tests/options/` (each asserting the option toggled away from its default
  plus the absent-option default, fixtures under `tests/java/<option>/`); the
  suite grew from 186 to 208 tests, all green (`cargo test`).

- **The within-parentheses/brackets/braces spacing options are honoured (R20,
  spaces-within-parentheses-brackets-braces)**: the 18 `SPACE_WITHIN_*`
  options — `SPACE_WITHIN_PARENTHESES`, `SPACE_WITHIN_METHOD_CALL_PARENTHESES`,
  `SPACE_WITHIN_EMPTY_METHOD_CALL_PARENTHESES`, `SPACE_WITHIN_METHOD_PARENTHESES`,
  `SPACE_WITHIN_EMPTY_METHOD_PARENTHESES`, `SPACE_WITHIN_IF_PARENTHESES`,
  `SPACE_WITHIN_WHILE_PARENTHESES`, `SPACE_WITHIN_FOR_PARENTHESES`,
  `SPACE_WITHIN_TRY_PARENTHESES`, `SPACE_WITHIN_CATCH_PARENTHESES`,
  `SPACE_WITHIN_SWITCH_PARENTHESES`, `SPACE_WITHIN_SYNCHRONIZED_PARENTHESES`,
  `SPACE_WITHIN_CAST_PARENTHESES`, `SPACE_WITHIN_BRACKETS`, `SPACE_WITHIN_BRACES`,
  `SPACE_WITHIN_ARRAY_INITIALIZER_BRACES`,
  `SPACE_WITHIN_EMPTY_ARRAY_INITIALIZER_BRACES` and
  `SPACE_WITHIN_ANNOTATION_PARENTHESES` — previously ignored (R7) and marked ❌
  in the docs/settings/common.md "Within parentheses, brackets, braces" table
  are now parsed into `JavaStyle` (eighteen `bool` fields defaulting to
  `false` — equal to today's tight output, so absent/default schemes keep
  byte-identical goldens — and eighteen `OptionDef` entries in the `OPTIONS`
  registry under the existing "Spaces" GUI group, ten in the JAVA
  `codeStyleSettings` block) and applied in the engine: a `within(open, close,
  pad, inner)` helper (plus an empty-aware `within_opt` for the constructs
  with empty variants) rebuilds every structured paren/bracket/brace pair,
  padding one space per side when the toggle is on, keeping the pair bare for
  an empty inner unless the construct has an empty variant in the request, and
  leaving a side bare when its neighbour is a newline so wrapped layouts never
  gain trailing whitespace. Per-construct granularity is preserved by
  destructuring: keyword conditions (`if`, `while` / `do … while`, `switch`,
  `synchronized`) are rendered by `keyword_cond` / `flat_keyword_cond`, which
  unwrap the outer `parenthesized_expression` and rebuild the paren pair with
  the keyword's own toggle so plain `SPACE_WITHIN_PARENTHESES` does not leak
  into them (nested parentheses inside a condition still flow through
  `expr`/`flat` and pad as plain parentheses); the textual `for` header and
  try-with-resources list are padded at their outermost paren pair via an
  idempotent insertion (`pad_outer_parens` — a space is added only when the
  neighbour is not already a space, so a padded header reformats to itself);
  and the empty variants are independent (`f( )`, `void f( )`, `{ }`), while
  a bare `@A()` stays tight (no empty annotation variant in the request). The
  change is whitespace-only (R5) and idempotent (R6 — each new golden was
  re-formatted under its own style and confirmed byte-identical); unmodelled
  shapes are echoed verbatim (R4). Covered by eighteen new per-option golden
  test files under `tests/options/` (each asserting the option on → padded
  golden and the absent-option default → tight golden, fixtures under
  `tests/java/<option>/`); the suite grew from 208 to 244 tests, all green
  (`cargo test`). No IntelliJ installation was available to cross-check the
  goldens; the defaults follow the settings table in docs/settings/common.md.

- **The before-parentheses/braces/keywords spacing options are honoured (R21,
  spaces-before-keywords-and-parens)**: the 28 `SPACE_BEFORE_*` options of the
  "Before parentheses / braces / keywords" table — the keyword-to-paren gaps
  `SPACE_BEFORE_IF_PARENTHESES`, `SPACE_BEFORE_WHILE_PARENTHESES`,
  `SPACE_BEFORE_FOR_PARENTHESES`, `SPACE_BEFORE_TRY_PARENTHESES`,
  `SPACE_BEFORE_CATCH_PARENTHESES`, `SPACE_BEFORE_SWITCH_PARENTHESES` and
  `SPACE_BEFORE_SYNCHRONIZED_PARENTHESES`, the name-to-paren gaps
  `SPACE_BEFORE_METHOD_CALL_PARENTHESES`, `SPACE_BEFORE_METHOD_PARENTHESES`
  and `SPACE_BEFORE_ANOTATION_PARAMETER_LIST` (XML name spelled as in IntelliJ
  sources, typo included), the brace gaps `SPACE_BEFORE_CLASS_LBRACE`,
  `SPACE_BEFORE_METHOD_LBRACE`, `SPACE_BEFORE_IF_LBRACE`,
  `SPACE_BEFORE_ELSE_LBRACE`, `SPACE_BEFORE_WHILE_LBRACE`,
  `SPACE_BEFORE_FOR_LBRACE`, `SPACE_BEFORE_DO_LBRACE`,
  `SPACE_BEFORE_SWITCH_LBRACE`, `SPACE_BEFORE_TRY_LBRACE`,
  `SPACE_BEFORE_CATCH_LBRACE`, `SPACE_BEFORE_FINALLY_LBRACE`,
  `SPACE_BEFORE_SYNCHRONIZED_LBRACE`, `SPACE_BEFORE_ARRAY_INITIALIZER_LBRACE`
  and `SPACE_BEFORE_ANNOTATION_ARRAY_INITIALIZER_LBRACE`, and the `}`-to-keyword
  gaps `SPACE_BEFORE_ELSE_KEYWORD`, `SPACE_BEFORE_WHILE_KEYWORD`,
  `SPACE_BEFORE_CATCH_KEYWORD` and `SPACE_BEFORE_FINALLY_KEYWORD` — previously
  ignored (R7) and marked ❌ in the docs/settings/common.md "Before
  parentheses / braces / keywords" table, are now parsed into `JavaStyle`
  (twenty-eight `bool` fields and twenty-eight `OptionDef` entries in the
  `OPTIONS` registry under the existing "Spaces" GUI group, all in the JAVA
  `codeStyleSettings` block) and applied in the engine. Defaults follow the
  table exactly: the clause-keyword paren and brace toggles plus the four
  keyword toggles default to `true` — equal to today's canonical gap — while
  `SPACE_BEFORE_METHOD_CALL_PARENTHESES`, `SPACE_BEFORE_METHOD_PARENTHESES`,
  `SPACE_BEFORE_ANOTATION_PARAMETER_LIST`,
  `SPACE_BEFORE_ARRAY_INITIALIZER_LBRACE` and
  `SPACE_BEFORE_ANNOTATION_ARRAY_INITIALIZER_LBRACE` default to `false`, so
  absent/default schemes keep byte-identical goldens except for one
  deliberate tightening: `array_creation` / `flat_arr_creation` always
  printed `new int[] {…}` (a space) whereas the option defaults `false`, so
  default output for that construct is now `new int[]{…}` (the IntelliJ
  built-in); the two `space_within_array_initializer_braces` goldens that
  contained the spaced form were regenerated, and no other existing golden
  changed. A tiny `sp(bool)` helper returns the gap (`" "` / `""`) and every
  emission site routes its single gap through it across all three rendering
  paths (the multi-line emitter, the keep-simple one-line candidates and the
  flat emitter used for inline contexts and margin checks): the name→`(`
  joins of method calls (`flat_inv`, `inv_wrapped`, `fmt_chain`, `new_expr`
  constructor calls), method/constructor declarations, and annotations
  (`annotation` / `annotation_expanded` / `flat_annotation` — raw-echoed
  annotation positions are untouched, R4); the keyword→`(` joins of `if` /
  `while` / `do … while` / `switch` / `synchronized` conditions and of the
  textual `for` / `try` headers (the classic-`for` header is rebuilt from
  source bytes and its `for`↔`(` gap pinned to the toggle, and the
  try-with-resources list is joined with the toggle rather than a fixed
  space); the `{` joins of class-like bodies (`with_brace`, plus the
  anonymous-class join in `new_expr`), method/constructor bodies
  (`brace_before_body` and the one-line push in `method_body`), the statement
  bodies of `if` / `else` / `while` / `for` / `enhanced-for` / `do` / `switch` /
  `try` / `catch` / `finally` / `synchronized` (`stmt_as_block_or_inline`
  gained an `lbrace` parameter; the one-line candidate formats follow suit),
  and the array / annotation-array initialisers (`array_creation` /
  `flat_arr_creation` and the annotation `(`→`{` join via a new `ann_parens`
  helper); and the `}`→keyword joins of `else`, `catch`, `finally` and the
  do-`while` tail combine with the paren/brace toggles to produce the
  `} else {`, `}else{`, `} while(x);` variants. The change is whitespace-only
  (R5) and inserting/removing one space is idempotent (R6 — every new golden
  was re-formatted under its own style and confirmed byte-identical);
  unmodelled shapes are echoed verbatim (R4). Covered by twenty-eight new
  per-option golden test files under `tests/options/` (each asserting the
  option toggled away from its default plus the absent-option default,
  fixtures under `tests/java/<option>/`); the suite grew from 244 to 300
  tests, all green (`cargo test`). No IntelliJ installation was available to
  cross-check the goldens; the defaults follow the settings table in
  docs/settings/common.md and the constructor-call and anonymous-class joins
  are mapped to the method-call-parentheses and class-lbrace toggles
  respectively.

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
