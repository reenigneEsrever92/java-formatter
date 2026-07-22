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
for `USE_TAB_CHARACTER` / `TAB_SIZE`) and R14 (generic type-argument
spacing normalisation). On 2026-09-03 the single crate was restructured into
the core/cli/gui workspace under `crates/` (workspace-split) so the library
can be a dependency target for the GUI and other consumers without pulling in
the CLI.

**Deferred (none).** Every deferred requirement is now delivered; the backlog
holds future work.

## Related documentation

The repository `README.md` remains the usage-level reference (CLI flags,
supported scheme options, formatting behaviour notes, limitations). This
document is the analysis of record; the [overview](overview.md) is the
project summary.
