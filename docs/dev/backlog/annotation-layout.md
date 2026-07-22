---
type: ChangeRequest
kind: feature
title: Honour the annotation placement and annotation-body layout options
description: Implement annotation-on-separate-line placement and annotation parameter wrapping/alignment options.
state: proposed
priority: medium
tags: [dev, formatter]
owner: maintainer
---

# Problem

The annotation rows are ❌ in both settings docs — the placement options `METHOD_ANNOTATION_WRAP`, `CLASS_ANNOTATION_WRAP`, `FIELD_ANNOTATION_WRAP`, `PARAMETER_ANNOTATION_WRAP` and `VARIABLE_ANNOTATION_WRAP` in docs/settings/common.md "Annotations", and `ENUM_FIELD_ANNOTATION_WRAP`, `ALIGN_MULTILINE_ANNOTATION_PARAMETERS`, `NEW_LINE_AFTER_LPAREN_IN_ANNOTATION`, `RPAREN_ON_NEW_LINE_IN_ANNOTATION`, `SPACE_AROUND_ANNOTATION_EQ`, `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION` and `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION_IN_PARAMETER` in docs/settings/java.md "Annotations" — valid IntelliJ options java-formatter neither parses nor applies, so schemes setting them are only partially honoured and the options are ignored (R7). Today annotations are rendered inline with their declaration (the `modifiers` renderer joins them with spaces) regardless of the placement wrap code, and annotation argument lists are laid out without alignment, `(`/`)` placement, `=` spacing or single-annotation exemptions — `ANNOTATION_PARAMETER_WRAP` (argument-list wrapping) already ships and is the model for the Java-block options.

# Proposal

Parse the common-block placement options (`Section::CodeStyleJava`, built-in defaults `2`/`2`/`2`/`0`/`0` per the tables — wrap-always for methods, classes and fields — absent → default, reusing the existing `WrapStyle` mapping) and the Java-block options (`Section::JavaCodeStyle`, defaults `0`/`false`/`false`/`false`/`true`/`false`/`false`) into `JavaStyle` via the OPTIONS registry in crates/core/src/config.rs — `OptionDef` entries with `OptionValue::Wrap` or `OptionValue::Bool` accordingly. Apply them in crates/core/src/formatter.rs: place each annotation on its own line before the declaration per the placement wrap code, lay out annotation arguments per the Java-block toggles (alignment, `(`/`)` on their own lines, spaces around `=`), put enum-field annotations on their own lines per `ENUM_FIELD_ANNOTATION_WRAP`, and exempt a lone annotation from the line break per the two `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION` options.

Docs touched: `docs/settings/common.md` and `docs/settings/java.md` (❌ → ✅ for these rows), `README.md` (honoured-options table rows and formatting-behaviour notes), `docs/requirements.md` (a new requirement row) and `docs/dev/changelog.md` on completion.

# Decisions

1. **One family, one request.** Only the listed placement and body-layout options are added; other annotation/enum options still ❌ stay unimplemented and are ignored safely (R7).
2. **Defaults — a deliberate default-behaviour change.** IntelliJ's built-in defaults include wrap-always (`2`) for `METHOD_ANNOTATION_WRAP`, `CLASS_ANNOTATION_WRAP` and `FIELD_ANNOTATION_WRAP`, so default/absent schemes move those annotations onto their own lines; goldens touching annotated declarations are updated once in this CR, and all other output stays byte-identical (existing goldens otherwise stay green).
3. **Semantics — placement relocates layout tokens only.** R5: putting an annotation on its own line, or moving `(`/`)`/`=` in an argument list, inserts or relocates only whitespace and newlines around existing tokens, never changing the annotation or its declaration; unmodelled shapes stay verbatim (R4); updated goldens pin R6 idempotency.
4. **Encodings.** The `*_ANNOTATION_WRAP` placement options use the wrap codes `0`/`1`/`2`/`5` shared across the `*_WRAP` options; the Java-block options are plain bools, and the `(`/`)`-on-new-line bools affect only wrapped annotation argument lists.

# Acceptance criteria

- Golden fixture + test file per option under crates/core/tests/options/ at the interesting values (wrap codes `0`/`1`/`2`/`5` for the placement options, both bool states) plus an absent-option default case.
- Annotated methods, classes, fields, parameters, local variables and enum fields place annotations on separate lines per their wrap code; annotation arguments align / place `(`/`)` / space `=` per the Java-block toggles; single-annotation no-wrap exemptions are honoured.
- Whole suite green (`cargo test`); goldens changed by the built-in wrap-always defaults are updated deliberately and re-formatting them is a no-op (R6).
- `docs/settings` marks flipped (❌ → ✅); README honoured-options table / behaviour notes and `docs/requirements.md` updated.
- New goldens idempotent: formatting the output again is a no-op.
