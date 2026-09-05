---
type: Requirements
title: Requirements
description: Users, use cases, technology choices, and the prioritized requirements of java-formatter.
tags: [java, formatter, cli]
status: active
---

# Requirements

Requirement analysis for **java-formatter**, recorded on 2026-09-02 when the
OKF bundle was initialized for the already-shipped implementation. Facts
about behaviour below are grounded in the repository `README.md`, the
workspace crates under `crates/` (`crates/core/src/`, `crates/core/tests/`,
`crates/core/benches/`), and `docs/`.

## Users

### U1 — Java developer (primary)

A Java developer who wants source formatted the way their team's IntelliJ
scheme dictates, without opening the IDE — formatting by hand from the
terminal, or through an editor/script that pipes source in and out.

- **Goal:** turn a `.java` file (or stdin) into the same output IntelliJ
  would produce for the given code style.
- **Expertise:** ordinary Java developer; does not need to know IntelliJ
  style-config internals beyond handing over the team's scheme file.
- **Cares about, in order:** (1) **correctness** — the formatted output must
  be semantically equivalent to the input; only formatting changes;
  (2) **fidelity** — output matches IntelliJ for the supported options;
  (3) **safety** — reformatting is a no-op, and code the tool does not model
  is never corrupted; (4) **speed** — fast enough for interactive use.

No secondary persona is recorded yet; CI / pre-commit usage is anticipated to
become one once the tool matures.

## Use cases

| #   | Use case                                                                                                                                                                                                               | Essential |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| UC1 | **Format a file with a team scheme** — given a `.java` file and an IntelliJ `codestyle.xml`, `java-formatter --style codestyle.xml Foo.java` prints the reformatted source to stdout and exits 0.                      | yes       |
| UC2 | **Format with built-in defaults** — given a `.java` file and no `--style`, the output follows the IntelliJ built-in default style.                                                                                     | yes       |
| UC3 | **Pipe stdin → stdout** — with no `FILE`, or with `-`, the tool reads Java source from stdin and writes the formatted result to stdout (editor and script usage).                                                      | yes       |
| UC4 | **Style-scheme robustness** — a missing, unreadable, or malformed scheme exits 1 with a clear message on stderr; options absent from the scheme fall back to defaults; non-Java and unimplemented options are ignored. | yes       |
| UC5 | **Safe best-effort formatting** — valid Java is always reformatted; constructs the formatter does not model are preserved verbatim from the source; formatting formatted output again is a no-op.                      | yes       |

Edge cases worth naming now: empty input (output is a single newline), input
that is not valid Java (see R15 — warned about on stderr, formatted
best-effort), style files whose only settings are for other languages, and
files that use constructs not yet modelled (see the deferred requirements).

## Technology choices

Recorded from the shipped implementation (the workspace `Cargo.toml` and the
crates under `crates/`). For each choice, the alternative considered and why
the chosen one won:

| Choice             | Recorded decision                                                                                                                                            | Alternatives considered                                     | Why the chosen one won                                                                                                                                                           |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Language / runtime | Rust (stable, edition 2021), compiled to a single binary                                                                                                     | Java (would need a JVM), Go                                 | The implementation already exists in Rust; a single static binary with fast startup suits a CLI formatter; no JVM to install.                                                    |
| CLI                | `clap` (derive)                                                                                                                                              | Manual `std::env` parsing, `lexopt`                         | Declarative argument definitions give usage/help text and `-h`/`--help` for free, and match the project's existing code.                                                         |
| Parsing            | `tree-sitter` + `tree-sitter-java` (CST, error-tolerant)                                                                                                     | Hand-written lexer/parser, `syn`-style token streams        | Tree-sitter produces a full CST with error nodes for invalid input, which lets the formatter preserve unknown constructs verbatim and recover per-node.                          |
| Scheme parsing     | `quick-xml` (serialize feature) + `serde`                                                                                                                    | Hand-rolled XML scanning, heavier XML crates                | An IntelliJ `<code_scheme>` is a flat, well-formed option store; `quick-xml` is small and fast, and deserialization keeps the mapping to `JavaStyle` declarative.                |
| Formatting engine  | Pretty-print of the tree-sitter CST per `JavaStyle`                                                                                                          | Token-based reformatting, invoking IntelliJ's own formatter | Node-structured printing is what lets the formatter honour per-construct options (braces, wrapping, indentation) and echo untouched what it does not model.                      |
| Tests              | Integration tests with a dedicated file per option, each test a golden pair of a `.java` fixture and a `*.out.java` expected output embedded at compile time | Unit-only tests, golden text files read at runtime          | Real fixtures exercise whole-file behaviour; compile-time embedding keeps tests self-contained; per-option golden pairs make each option's input→output correlation easy to see. |
| Benchmarks         | Criterion suite in `crates/core/benches/` (formatting throughput, scheme parsing)                                                                            | None shipped                                                | Provides a regression signal for R8.                                                                                                                                             |

Hard constraints: Java only; IntelliJ schemes only; no external services or
storage. No license file is shipped in the repository; licensing is
unrecorded and should be settled by the owner before distribution.

## Requirements

Tie-backs: U = user, UC = use case from above. Priorities use the backlog
scale (high / medium / low).

| #   | Requirement                                                                                                                                                                                                                       | Tie-back    | Type           | Priority                      |
| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- | -------------- | ----------------------------- |
| R1  | Format a `.java` file or stdin according to a given IntelliJ `codestyle.xml`.                                                                                                                                                     | U1, UC1/UC3 | functional     | high                          |
| R2  | Format according to IntelliJ built-in defaults when no `--style` is given.                                                                                                                                                        | U1, UC2     | functional     | high                          |
| R3  | Honour the documented scheme options on the constructs they govern so output matches IntelliJ (brace placement, wrapping codes, indentation, import merging, simple-block/simple-method/simple-lambda one-lining, record layout). | U1, UC1     | functional     | high                          |
| R4  | Never invent output: constructs the formatter does not model are preserved verbatim from the source, and conditions are rendered with exactly their own parentheses.                                                              | U1, UC5     | functional     | high                          |
| R5  | Semantic equivalence: the formatted output is semantically identical to the input — only formatting (whitespace, layout) changes.                                                                                                 | U1, UC1/UC5 | non-functional | high                          |
| R6  | Idempotency: formatting already-formatted output is a no-op.                                                                                                                                                                      | U1, UC5     | non-functional | high                          |
| R7  | Scheme robustness: missing/unreadable/malformed scheme → exit 1 with a clear stderr message; options absent from a scheme fall back to defaults; non-Java and unimplemented options are ignored without error.                    | U1, UC4     | non-functional | high                          |
| R8  | Performance: formatting throughput and scheme parsing are tracked by the Criterion suite; no hard target, but regressions must be visible.                                                                                        | U1, UC1/UC3 | non-functional | medium                        |
| R9  | Maintainability: a core/cli/gui Cargo workspace separating the formatting library from the CLI and the GUI, fixture-based integration tests with a dedicated test file per supported option, and this documentation bundle.       | dev         | non-functional | medium                        |
| R10 | Binary expressions wrap per `BINARY_OPERATION_WRAP` so a long right-hand side respects the margin.                                                                                                                                | U1, UC1     | functional     | medium (delivered 2026-09-02) |
| R11 | `switch` statements and switch expressions are formatted rather than emitted as their original source text.                                                                                                                       | U1, UC1     | functional     | medium (delivered 2026-09-03) |
| R12 | `try`/`catch`/`finally` and `synchronized` bodies honour `KEEP_SIMPLE_BLOCKS_IN_ONE_LINE`.                                                                                                                                        | U1, UC1     | functional     | low (delivered 2026-09-03)    |
| R13 | Tab indentation output for `USE_TAB_CHARACTER` / `TAB_SIZE`.                                                                                                                                                                      | U1, UC1     | functional     | low (delivered 2026-09-03)    |
| R14 | Normalise spacing around generic type arguments to the canonical IntelliJ form (no inner padding, one space after commas).                                                                                                        | U1, UC1     | functional     | low (delivered 2026-09-03)    |
| R15 | Invalid Java is detected: parse errors are reported as a warning on stderr while best-effort output is still emitted (exit 0), and the never-corrupt contract is documented.                                                      | U1, UC5     | non-functional | high (delivered 2026-09-02)   |
| R16 | Vertical spacing follows the scheme's blank-line policy: the `KEEP_BLANK_LINES_*` caps limit how many pre-existing blank lines are preserved and the `BLANK_LINES_*` minimums insert the configured blanks around package/import boundaries, class headers/ends, fields, methods, initializer blocks and interface members. | U1, UC1/UC2 | functional | high (delivered 2026-09-03) |
| R17 | Braces are forced on brace-less `if`/`else`, `for`/enhanced-`for`, `while` and `do … while` bodies per the `*_BRACE_FORCE` force codes (`0` do not force, `1` force when multiline, `3` always force), matching IntelliJ. | U1, UC1 | functional | high (delivered 2026-09-03) |
| R18 | Operator spacing follows the scheme's `SPACE_AROUND_*` toggles: assignment, logical, equality, relational, bitwise, additive, multiplicative, shift and lambda-arrow operators space per their own toggle (all default on), unary operators and the method-reference `::` stay space-less by default, and a type cast renders as `(int) x` by default (`SPACE_AFTER_TYPE_CAST`). | U1, UC1/UC2 | functional | high (delivered 2026-09-03) |
| R19 | Separator spacing follows the scheme's toggles: commas (`SPACE_AFTER_COMMA`, `SPACE_BEFORE_COMMA`, plus `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS` for generic type arguments), `for`-header semicolons (`SPACE_AFTER_SEMICOLON`, `SPACE_BEFORE_SEMICOLON`), the ternary `?` / `:` (`SPACE_BEFORE_QUEST`, `SPACE_AFTER_QUEST`, `SPACE_BEFORE_COLON`, `SPACE_AFTER_COLON`), the enhanced-`for` colon (`SPACE_BEFORE_COLON_IN_FOREACH`) and the class / interface / record name-to-type-parameter-list gap (`SPACE_BEFORE_TYPE_PARAMETER_LIST`). | U1, UC1/UC2 | functional | high (delivered 2026-09-03) |
| R20 | Padding inside parentheses, brackets and braces follows the scheme's `SPACE_WITHIN_*` toggles: each paren / bracket / brace kind spaces per its own toggle — plain parentheses, method-call and method-declaration parens with independent empty variants, `if` / `while` / `do … while` / `for` / `try` / `catch` / `switch` / `synchronized` conditions and headers, casts, array-index brackets, code-block and array-initialiser braces with an independent empty-array variant, and annotation argument parens — all defaulting to off. | U1, UC1/UC2 | functional | high (delivered 2026-09-03) |
| R21 | The gap before parentheses, braces and clause keywords follows the scheme's `SPACE_BEFORE_*` toggles: the keyword-to-paren gap of `if` / `while` (incl. the do-`while` tail) / `for` / `try` / `catch` / `switch` / `synchronized` headers, of method calls (incl. constructor calls and chains) and method / constructor declarations, and of annotation parameter lists; the keyword-to-brace gap of class-like bodies (incl. anonymous classes), method / constructor bodies, `if` / `else` / `while` / `for` / `do` / `switch` / `try` / `catch` / `finally` / `synchronized` bodies, array and annotation-array initialisers; and the `}`-to-keyword gap of `else` / `catch` / `finally` / do-`while` — the paren and brace toggles defaulting to on except method-call / method-declaration / annotation-parameter / array- and annotation-array-initialiser gaps which default to off, matching IntelliJ. | U1, UC1/UC2 | functional | high (delivered 2026-09-03) |
| R22 | Line length, line endings and line-break retention follow the scheme's root `RIGHT_MARGIN` / `LINE_SEPARATOR` options and the `WRAP_LONG_LINES` / `KEEP_LINE_BREAKS` toggles: the right margin (`SOFT_MARGINS` wins when a scheme sets both) drives the wrap decisions, the configured separator (LF / CRLF / CR) ends every line including the final newline, hard wrapping breaks an over-margin line at the last whitespace boundary at or before the margin without ever splitting a literal or comment, and a construct whose source spans multiple lines keeps its canonical wrapped layout unless `KEEP_LINE_BREAKS` is off (reflow). | U1, UC1/UC2 | functional | medium (delivered 2026-09-03) |
| R23 | Comment layout follows the scheme's comment options: `LINE_COMMENT_AT_FIRST_COLUMN` / `BLOCK_COMMENT_AT_FIRST_COLUMN` (both default true) pin `//` / `/* */` comments to the first column, `KEEP_FIRST_COLUMN_COMMENT` (default true) keeps source first-column comments there, `LINE_COMMENT_ADD_SPACE_ON_REFORMAT` inserts the missing space after `//` of ordinary line comments, `LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION` does the same inside `//noinspection` suppression comments only, and `WRAP_COMMENTS` wraps single-line comments longer than the right margin at word boundaries (continuation lines repeat the comment's column prefix) — whitespace/layout only. | U1, UC1/UC2 | functional | medium (delivered 2026-09-03) |
| R24 | Clause-keyword and brace-less control-statement layout follows the scheme's options: `ELSE_ON_NEW_LINE` / `WHILE_ON_NEW_LINE` / `CATCH_ON_NEW_LINE` / `FINALLY_ON_NEW_LINE` (default false) move the `else` of an if / else-if chain, the trailing `while` of a `do … while`, and a `try`'s `catch` / `finally` clauses to their own line, `SPECIAL_ELSE_IF_TREATMENT` (default true) keeps an `else if` chain fused instead of nesting `else { if … }`, `LAMBDA_BRACE_STYLE` (default end of line) places a block lambda body's brace per its brace code independently of `BRACE_STYLE`, and `KEEP_CONTROL_STATEMENT_IN_ONE_LINE` (default true) keeps a brace-less `if (…) …;` / `while (…) …;` / `for (…) …;` / `do … while` body on its header's line when the source has it there — whitespace/layout only. | U1, UC1/UC2 | functional | medium (delivered 2026-09-03) |
| R25 | Switch / case layout follows the scheme's case options: `INDENT_CASE_FROM_SWITCH` (default true) indents `case` / `default` labels one level from the `switch` — off, labels sit at the `switch` indent; `CASE_STATEMENT_ON_NEW_LINE` (default true) puts the statement after a label on a new line — off, the group's first single-line statement is joined onto the label's line; `INDENT_BREAK_FROM_CASE` (default true) indents `break` / `continue` / `return` one level from the label — off, they line up with the label; and `SWITCH_EXPRESSIONS_WRAP` (default wrap if long) governs when a switch expression used as a value stays on one line (`0` never wraps, `1` wraps when long, `2` always wraps, `5` chop-down also breaks an overflowing nested switch expression in the body) — whitespace/layout only. | U1, UC1/UC2 | functional | medium (delivered 2026-09-04) |
| R26 | Resource-list and declaration clause-list layout follows the scheme's wrapping options: `RESOURCE_LIST_WRAP` (default do not wrap) with `RESOURCE_LIST_LPAREN_ON_NEXT_LINE` / `RESOURCE_LIST_RPAREN_ON_NEXT_LINE` governs try-with-resources resource lists, `EXTENDS_LIST_WRAP` (default do not wrap) with `EXTENDS_KEYWORD_WRAP` (default false) governs the `extends` / `implements` lists of type declarations, `THROWS_LIST_WRAP` (default do not wrap) with `THROWS_KEYWORD_WRAP` (default false) governs method / constructor `throws` lists, and `PREFER_PARAMETERS_WRAP` (default false) prefers wrapping a chain tail call's overflowing argument list over breaking the chain — each list breaks one element per continuation line only when it overflows under wrap-if-long / chop-down (`1` / `5`, which share a layout for these atomic elements) or always under wrap-always (`2`), keyword / paren placement honours the bool sub-options on wrapped output only, a single-element list never splits, and absent options keep today's preserved one-line output — whitespace/layout only (R5, R6). | U1, UC1/UC2 | functional | medium (delivered 2026-09-05) |
| R27 | Expression / statement / initialiser wrapping follows the scheme's options: ternary expressions wrap per `TERNARY_OPERATION_WRAP` (`0` do not wrap, `1` wrap if long, `2` wrap always, `5` chop down — the latter also recursing into a nested ternary side), assert statements per `ASSERT_STATEMENT_WRAP` (at the expression and after the `:`), for headers per `FOR_STATEMENT_WRAP` (the classic header breaks at its semicolons, the enhanced at its `:`), and array initializers per `ARRAY_INITIALIZER_WRAP` (one element per line), each with its sign / keyword / paren / brace placement bools honoured on wrapped output only — `BINARY_OPERATION_SIGN_ON_NEXT_LINE`, `TERNARY_OPERATION_SIGNS_ON_NEXT_LINE`, `PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE`, `ASSERT_STATEMENT_COLON_ON_NEXT_LINE`, `FOR_STATEMENT_LPAREN/RPAREN_ON_NEXT_LINE`, `ARRAY_INITIALIZER_LBRACE/RBRACE_ON_NEXT_LINE` and `PARENTHESES_EXPRESSION_LPAREN/RPAREN_WRAP`; `MODIFIER_LIST_WRAP` breaks a declaration after its modifier / annotation list, `WRAP_FIRST_METHOD_IN_CALL_CHAIN` puts the first link of a wrapped chain on a continuation line, and `WRAP_SEMICOLON_AFTER_CALL_CHAIN` puts the `;` of a wrapped chained statement on its own line — all defaults keep the one-line layouts, and the changes are whitespace/layout only (R5, R6). | U1, UC1/UC2 | functional | medium (delivered 2026-09-05) |
| R28 | Continuous integration: a GitHub Actions pipeline (`.github/workflows/ci.yml`) verifies every push to `main` and every pull request with `cargo fmt --all -- --check`, `cargo clippy --workspace --lib --bins --tests -- -D warnings` and `cargo test --workspace` on an ubuntu/macos/windows × stable Rust matrix, so formatting drift, lint warnings and test regressions are caught automatically. | dev | non-functional | medium (delivered 2026-09-05) |
| R29 | Wrapped constructs align per the scheme's align-when-multiline options when the construct's wrap is active: `ALIGN_MULTILINE_PARAMETERS` (default true) / `ALIGN_MULTILINE_PARAMETERS_IN_CALLS` align wrapped declaration parameter lists and call / `new` arguments, `ALIGN_MULTILINE_RESOURCES` (default true) wrapped try-with-resources lists, `ALIGN_MULTILINE_FOR` (default true) wrapped `for` header parts, `ALIGN_MULTILINE_BINARY_OPERATION` wrapped binary operands, `ALIGN_MULTILINE_ASSIGNMENT` a wrapped assignment's right-hand side, `ALIGN_MULTILINE_TERNARY_OPERATION` ternary `?` / `:` lines, `ALIGN_MULTILINE_THROWS_LIST` / `ALIGN_THROWS_KEYWORD` a wrapped `throws` list and its keyword, `ALIGN_MULTILINE_EXTENDS_LIST` wrapped `extends` / `implements` lists, `ALIGN_MULTILINE_METHOD_BRACKETS` a wrapped declaration's parens, `ALIGN_MULTILINE_PARENTHESIZED_EXPRESSION` a wrapped parenthesized expression, `ALIGN_MULTILINE_ARRAY_INITIALIZER_EXPRESSION` wrapped array initializer entries, and `ALIGN_MULTILINE_CHAINED_METHODS` the dots of a wrapped chained call — each replacing the fixed continuation prefix with spaces to the first element's column (the record-header model); the columnar options `ALIGN_GROUP_FIELD_DECLARATIONS`, `ALIGN_CONSECUTIVE_VARIABLE_DECLARATIONS`, `ALIGN_CONSECUTIVE_ASSIGNMENTS` and `ALIGN_SUBSEQUENT_SIMPLE_METHODS` pad runs of output-adjacent fields / local variable declarations / assignment statements / one-line methods so names and operators share one column — whitespace/layout only (R5, R6), absent and default-false schemes keep today's output byte-for-byte. | U1, UC1/UC2 | functional | medium (delivered 2026-09-05) |
| R30 | The remaining keep-in-one-line options are honoured: `KEEP_SIMPLE_CLASSES_IN_ONE_LINE` collapses a simple class / interface / record body to one line when every member renders newline-free and the whole declaration fits the margin (members collapse recursively; enums and anonymous classes out of scope), `KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE` keeps the multiple expressions of a statement (a classic `for` header's init/update clause lists, a multi-declarator field / local declaration) joined — lists the engine never splits per expression, so on / off / absent output is identical — and the two Java one-line-body presentation toggles lay out a collapsed non-empty block flush `{s}` by default with `SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT` (default false, matching IntelliJ) adding the inner spaces `{ s }` and `NEW_LINE_WHEN_BODY_IS_PRESENTED` putting the one-line block on its own line below the statement head at the head's indent — whitespace/layout only (R5, R6), absent options keep the defaults, default-scheme goldens stay byte-identical, and flat one-line contexts (argument lambdas, one-line switches) keep their pinned layout. | U1, UC1/UC2 | functional | medium (delivered 2026-09-05) |
| R31 | Annotation placement and annotation-body layout follow the scheme's options: the five placement options `METHOD_ANNOTATION_WRAP` / `CLASS_ANNOTATION_WRAP` / `FIELD_ANNOTATION_WRAP` (default wrap always, code `2`) and `PARAMETER_ANNOTATION_WRAP` / `VARIABLE_ANNOTATION_WRAP` (default do not wrap, code `0`) place each annotation on its own line before the declaration (`2`), join them inline with the declaration (`0`), or keep the inline form unless the composed first line overflows the margin (`1` / `5` — identical at this granularity); enum-constant annotations follow `ENUM_FIELD_ANNOTATION_WRAP`; the two single-annotation exemptions `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION` (members / types / locals) and `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION_IN_PARAMETER` (parameters) keep a lone annotation inline regardless of the wrap code; and wrapped annotation argument lists lay out per `NEW_LINE_AFTER_LPAREN_IN_ANNOTATION` (first argument on the `(` line when false), `RPAREN_ON_NEW_LINE_IN_ANNOTATION` (`)` attaches to the last argument when false), `ALIGN_MULTILINE_ANNOTATION_PARAMETERS` (pad under the first argument when true) and `SPACE_AROUND_ANNOTATION_EQ` (`key = value` when true, `key=value` when false) — whitespace/layout only (R5, R6), absent options keep the IntelliJ built-in defaults, default-scheme goldens for the placement options stay byte-identical, and the `annotation_parameter_wrap` goldens that encoded the old hard-coded stacked expanded shape are re-baselined to the option-default shape idempotently. | U1, UC1/UC2 | functional | medium (delivered 2026-09-05) |
| R32 | The remaining indentation options are honoured: `DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS` (default false) places the members of a top-level class at the class declaration indent (nested classes keep the normal one-level indent); `LABEL_INDENT_SIZE` (default 0) / `LABEL_INDENT_ABSOLUTE` (default false) position `label:` lines at the statement indent plus the width (relative) or at the width from the left margin regardless of nesting (absolute); `KEEP_INDENTS_ON_EMPTY_LINES` (default false) keeps a block's inner indent on preserved blank lines; `SMART_TABS` (default false, with `USE_TAB_CHARACTER`) restricts tab characters to indentation that lands exactly on a tab stop with off-stop indents emitted as spaces; `USE_RELATIVE_INDENTS` (default false, with `USE_TAB_CHARACTER`) measures continuation indents from the construct's own indent level; and the five per-construct continuation widths `DECLARATION_PARAMETER_INDENT` / `GENERIC_TYPE_PARAMETER_INDENT` / `CALL_PARAMETER_INDENT` / `CHAINED_CALL_INDENT` / `ARRAY_ELEMENT_INDENT` (each default `-1` = inherit `CONTINUATION_INDENT_SIZE`) override the continuation indent of their construct kind only — `GENERIC_TYPE_PARAMETER_INDENT` is parsed but inert because generic parameter lists always render flat — using the new signed `OptionValue::Int` registry variant so `-1` and explicit widths round-trip through parse/serialize; whitespace/layout only (R5, R6), absent and default options keep today's output byte-for-byte. | U1, UC1/UC2 | functional | medium (delivered 2026-09-05) |
| R33 | The remaining record-header layout options are honoured: `RPAREN_ON_NEW_LINE_IN_RECORD_HEADER` (default false) puts the `)` of a wrapped record header on its own line at the record indent (the lparen-on layout already closes alone, so the option visibly moves the lparen-attached layout's glued `)`), `SPACE_WITHIN_RECORD_HEADER` (default false) inserts one space just inside each `(` / `)` that shares its line with a component — the flat `record R( String s )`, the lparen-attached `( first,` line and the glued `last )` — with a paren alone on its line unpadded and the margin decision / alignment column accounting for the pad, `ANNOTATION_NEW_LINE_IN_RECORD_COMPONENT` (default false) renders an own-line `formal_parameter` component with annotations as one line per annotation (tokens verbatim) plus the declaration core, keeping the first inline component of the lparen-attached layout, non-formal-parameter shapes and flat headers inline, and `BLANK_LINES_BETWEEN_RECORD_COMPONENTS` (default 0) inserts `n` bare blank lines between the components of a wrapped header (the `,\n` separator becomes `,` plus `n + 1` newlines; inert when the header is not wrapped) — whitespace/layout only (R5, R6), absent and default options keep today's record-header output byte-for-byte. | U1, UC1/UC2 | functional | medium (delivered 2026-09-05) |
| R34 | Import ordering and grouping follow the scheme's import options: `IMPORT_LAYOUT_TABLE` (default IntelliJ layout, java.md "Default layout") groups the import section per its `<package>` / `<emptyLine>` entries — a package matches by longest name, `withSubpackages` extends a match to the package's subpackages, the empty-name entries catch the remaining imports, the reserved module entry owns `import module …;` lines, groups emit in table order with one blank line per `<emptyLine/>` strictly between their positions, and group-internal order is preserved — `LAYOUT_STATIC_IMPORTS_SEPARATELY` (default true) keeps static imports in their own section (off, the entries' `static` attribute is ignored and static imports join the ordinary package sections), `LAYOUT_ON_DEMAND_IMPORT_FROM_SAME_PACKAGE_FIRST` (default true) moves the file's own-package on-demand import before its group, `KEEP_BLANK_LINES_BETWEEN_IMPORTS` (default false) preserves the source's blank lines inside a group, `PRESERVE_MODULE_IMPORTS` (default true) keeps `import module …;` lines at the module slot (a table without it puts them at the head of the section) with `DELETE_UNUSED_MODULE_IMPORTS` (default false) removing only clearly-unused ones (duplicates beyond the first) — whitespace/layout only (R5, R6), absent and default options keep today's import output byte-for-byte on the existing fixtures. | U1, UC1/UC2 | functional | medium (delivered 2026-09-05) |

### Resolved ambiguities

- **Invalid-Java behaviour (R15).** Decided with the user on 2026-09-02:
  parse errors must be _detected and warned about_, but the tool still emits
  best-effort output and exits 0 — best for editor and pipe usage. The
  safe-passthrough behaviour (R4) is documented as the contract in all cases.
- **Option parity.** "Implementing every IntelliJ Java option" was _removed_
  from the out-of-scope list: closing gaps toward IntelliJ parity is the
  ongoing direction, reached incrementally through the backlog. Unimplemented
  options remain safely ignored (R7) until then.

### Out of scope

- Languages other than Java, and IntelliJ settings for HTML, JavaScript,
  TypeScript, and Vue inside a scheme file.
- Style sources other than IntelliJ `<code_scheme>` XML (for example
  google-java-format or checkstyle configuration).
- An IntelliJ plugin or any IDE integration.

## Milestones

**Initial milestone — delivered baseline (R1–R9, R15 as of 2026-09-02).**
The smallest coherent slice that demonstrates the project's value is the
implementation that ships: it covers the primary user's essential use cases
UC1–UC5 and no more. It was delivered by the code present when the bundle was
recorded (crate `java-formatter` v0.1.0; see the [changelog](dev/changelog.md)),
since extended by R15 (parse-error reporting), R10 (binary-expression
wrapping per `BINARY_OPERATION_WRAP`), R11 (switch layout), R12
(simple `try`/`synchronized` bodies on one line), R13 (tab indentation
for `USE_TAB_CHARACTER` / `TAB_SIZE`), R14 (generic type-argument
spacing normalisation), R16 (blank-line policy), R17 (braces forced on
statement bodies per the `*_BRACE_FORCE` options) and R18 (operator spacing
per the `SPACE_AROUND_*` toggles). On 2026-09-03 the single
crate was restructured into the core/cli/gui workspace under `crates/`
(workspace-split) so the library can be a dependency target for the GUI and
other consumers without pulling in the CLI. The line-length / line-ending
requirements are delivered by R22 (right margin, line separator, hard
wrapping and line-break retention per `RIGHT_MARGIN` / `LINE_SEPARATOR` /
`WRAP_LONG_LINES` / `KEEP_LINE_BREAKS`).

The switch/case indentation and wrapping options are delivered by R25
(`INDENT_CASE_FROM_SWITCH`, `CASE_STATEMENT_ON_NEW_LINE`,
`INDENT_BREAK_FROM_CASE` and `SWITCH_EXPRESSIONS_WRAP` — switch-case-layout).

The clause-layout wrapping options are delivered by R26
(`RESOURCE_LIST_WRAP`, `RESOURCE_LIST_LPAREN_ON_NEXT_LINE`,
`RESOURCE_LIST_RPAREN_ON_NEXT_LINE`, `EXTENDS_LIST_WRAP`,
`EXTENDS_KEYWORD_WRAP`, `THROWS_LIST_WRAP`, `THROWS_KEYWORD_WRAP` and
`PREFER_PARAMETERS_WRAP` — wrapping-declaration-clauses).

The keep-in-one-line family and the one-line block-body presentation toggles
are delivered by R30 (`KEEP_SIMPLE_CLASSES_IN_ONE_LINE`,
`KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE`,
`SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT` and
`NEW_LINE_WHEN_BODY_IS_PRESENTED` — one-line-body-layout).

The annotation placement and annotation-body layout options are delivered by
R31 (`METHOD_ANNOTATION_WRAP`, `CLASS_ANNOTATION_WRAP`,
`FIELD_ANNOTATION_WRAP`, `PARAMETER_ANNOTATION_WRAP`,
`VARIABLE_ANNOTATION_WRAP`, `ENUM_FIELD_ANNOTATION_WRAP`,
`ALIGN_MULTILINE_ANNOTATION_PARAMETERS`,
`NEW_LINE_AFTER_LPAREN_IN_ANNOTATION`,
`RPAREN_ON_NEW_LINE_IN_ANNOTATION`, `SPACE_AROUND_ANNOTATION_EQ`,
`DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION` and
`DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION_IN_PARAMETER` — annotation-layout).

The remaining indentation options are delivered by R32
(`DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS`, `LABEL_INDENT_SIZE`,
`LABEL_INDENT_ABSOLUTE`, `KEEP_INDENTS_ON_EMPTY_LINES`, `SMART_TABS`,
`USE_RELATIVE_INDENTS` and the five per-construct continuation widths
`DECLARATION_PARAMETER_INDENT`, `GENERIC_TYPE_PARAMETER_INDENT`,
`CALL_PARAMETER_INDENT`, `CHAINED_CALL_INDENT`, `ARRAY_ELEMENT_INDENT` —
indentation-details).

The remaining record-header layout options are delivered by R33
(`RPAREN_ON_NEW_LINE_IN_RECORD_HEADER`, `SPACE_WITHIN_RECORD_HEADER`,
`ANNOTATION_NEW_LINE_IN_RECORD_COMPONENT` and
`BLANK_LINES_BETWEEN_RECORD_COMPONENTS` — record-header-layout).

Import ordering and grouping are delivered by R34 (`IMPORT_LAYOUT_TABLE`,
`LAYOUT_STATIC_IMPORTS_SEPARATELY`,
`LAYOUT_ON_DEMAND_IMPORT_FROM_SAME_PACKAGE_FIRST`,
`KEEP_BLANK_LINES_BETWEEN_IMPORTS`, `PRESERVE_MODULE_IMPORTS` and
`DELETE_UNUSED_MODULE_IMPORTS` — import-ordering-and-layout).

**Deferred (none).** Every deferred requirement is now delivered; the backlog
holds future work.

## Related documentation

The repository `README.md` remains the usage-level reference (CLI flags,
supported scheme options, formatting behaviour notes, limitations). This
document is the analysis of record; the [overview](overview.md) is the
project summary.
