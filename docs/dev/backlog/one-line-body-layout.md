---
type: ChangeRequest
kind: feature
title: Keep simple classes and multi-expression statements on one line; lay out one-line block bodies
description: Implement the remaining keep-in-one-line options plus the block-body spacing/new-line options.
state: proposed
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

The remaining keep-in-one-line rows of docs/settings/common.md "Keep in one line" are ❌ — `KEEP_SIMPLE_CLASSES_IN_ONE_LINE` and `KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE` — as are the two one-line-block presentation toggles of docs/settings/java.md "Miscellaneous spacing & blank lines", `SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT` and `NEW_LINE_WHEN_BODY_IS_PRESENTED`; all are valid IntelliJ options java-formatter neither parses nor applies, so schemes setting them are only partially honoured and the options are ignored (R7). Today simple class bodies always render multi-line and multi-expression statements never collapse, while `KEEP_SIMPLE_BLOCKS`/`METHODS`/`LAMBDAS_IN_ONE_LINE` already ship (R12 extended blocks to `try`/`catch`/`synchronized`), and one-line block bodies use a fixed `{ … }` layout with no padding or new-line control.

# Proposal

Parse `KEEP_SIMPLE_CLASSES_IN_ONE_LINE` and `KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE` (JAVA `codeStyleSettings` block) and `SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT` and `NEW_LINE_WHEN_BODY_IS_PRESENTED` (`<JavaCodeStyleSettings>`) into `JavaStyle` via the OPTIONS registry in crates/core/src/config.rs — `OptionDef` entries with the IntelliJ built-in defaults (`false` for all four; absent → default) as `OptionValue::Bool`. Apply them in crates/core/src/formatter.rs: keep a simple class body on one line when every member is simple and the whole fits the margin (`KEEP_SIMPLE_CLASSES_IN_ONE_LINE`), keep multiple expressions of a statement on one line (`KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE`), pad a one-line non-empty block with spaces (`SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT`) and put a one-line body on a new line (`NEW_LINE_WHEN_BODY_IS_PRESENTED`), composing with the shipped keep-simple family.

Docs touched: `docs/settings/common.md` and `docs/settings/java.md` (❌ → ✅ for these rows), `README.md` (honoured-options table rows and formatting-behaviour notes), `docs/requirements.md` (a new requirement row) and `docs/dev/changelog.md` on completion.

# Decisions

1. **One family, one request.** Only the four listed options are added; other keep-in-one-line and spacing options still ❌ stay unimplemented and are ignored safely (R7).
2. **Defaults.** IntelliJ built-in defaults (`false`); absent → default, so default/absent schemes keep today's multi-line class layout and fixed one-line block layout byte-identical and existing goldens stay green.
3. **Semantics.** R5: collapsing a class body to one line or padding / re-placing a one-line block changes only whitespace, never tokens; unmodelled bodies stay verbatim (R4); new goldens pin R6 idempotency.
4. **Scope split.** The two common-block options complete the shipped keep-simple family (classes, multi-expression statements); the two Java-block options are presentation toggles for one-line block bodies (`{ … }` padding, new-line placement) that compose with it.

# Acceptance criteria

- Golden fixture + test file per option under crates/core/tests/options/ covering the option on and off, plus an absent-option default case.
- With `KEEP_SIMPLE_CLASSES_IN_ONE_LINE` a simple class body stays on one line; with `KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE` multi-expression statements are not split; the block-body toggles pad and re-place one-line bodies as configured.
- Default-scheme output unchanged; whole suite green (`cargo test`).
- `docs/settings` marks flipped (❌ → ✅); README honoured-options table / behaviour notes and `docs/requirements.md` updated.
- New goldens idempotent: formatting the output again is a no-op.
