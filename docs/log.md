# Update log

## 2026-09-03

- **Documentation**: Added the settings reference under `docs/settings/` — a
  section index (scheme anatomy, value encodings, support legend) plus two
  exhaustive option tables: common settings (`<codeStyleSettings
language="JAVA">`) and Java-specific settings (`<JavaCodeStyleSettings>`).
  Grounded in the IntelliJ IDEA Community sources (`CommonCodeStyleSettings`,
  `JavaCodeStyleSettings`, `CodeStyleSettings`, `SoftMargins`,
  `PackageEntryTable`, verified against current master and the 2019.3 tag),
  listing defaults, encodings, and java-formatter support per option, and
  linking the new section from `docs/index.md`. Also records two documented
  gaps in the tool's value-encoding mapping (wrap codes `2`/`3` and brace
  codes `3`/`4`/`5`) that differ from IntelliJ's own encodings.

## 2026-09-02

- **Implement**: Delivered the parse-error-detection change request (R15) —
  parse diagnostics exposed through a new `format_java_diagnosed` entry point
  and surfaced as stderr warnings by the CLI while best-effort output is still
  emitted (exit 0). Added the `tests/parse_errors.rs` suite and the
  `tests/java/errors/syntax_error.java` fixture; documented the contract in
  the README and moved R15 to the delivered baseline in `requirements.md`;
  appended a changelog entry and marked the request done.

- **Planning**: Appended an `# Implementation plan` (approach + ordered,
  verifiable steps mapped to the acceptance criteria) to each of the six
  backlog change requests — parse-error detection, binary-expression
  wrapping, switch formatting, one-line try/catch/synchronized blocks, tab
  indentation, and generic type-argument spacing — and moved them from
  `proposed` to `planned` in their front matter and in the backlog index.
  Plans are grounded in the current `src/config.rs`, `src/formatter.rs`, and
  `tests/` structure.

## 2026-09-02

- **Creation**: Initialized the project as an OKF bundle — discovered the
  project kind, users, use cases, and technologies; recorded the requirement
  analysis in `requirements.md` (delivered baseline R1–R9 and deferred
  R10–R15); and seeded the backlog with the initial change requests for the
  deferred requirements (parse-error detection, binary-expression wrapping,
  switch formatting, one-line try/catch/synchronized blocks, tab
  indentation, and generic type-argument spacing). The bundle records an
  already-shipped implementation (crate `java-formatter` v0.1.0) rather than
  a greenfield project.
