# java-formatter — documentation

Documentation bundle for **java-formatter**, a CLI that formats Java source
code according to IntelliJ IDEA code style schemes (`codestyle.xml`), written
in Rust.

## Documents

- [Overview](overview.md) — what the project is and what it does.
- [Requirements](requirements.md) — users, use cases, technology choices, and
  the prioritized requirement list.
- [Front matter](frontmatter.md) — the schema every concept document follows.

## Settings reference

- [Settings index](settings/index.md) — scheme anatomy, value encodings, and
  the support-status legend.
- [Common settings](settings/common.md) — the `<codeStyleSettings language="JAVA">`
  block: indentation, spaces, wrapping, braces, blank lines, alignment.
- [Java-specific settings](settings/java.md) — the `<JavaCodeStyleSettings>`
  block: imports, javadoc, records, annotations, naming, code generation.

## Development

- [Development index](dev/index.md) — entry point to the development docs.
- [Changelog](dev/changelog.md) — shipped changes, newest first.
- [Backlog](dev/backlog/index.md) — proposed and planned change requests.

## Log

- [Update log](log.md) — history of this bundle's own evolution.

For usage-level detail (CLI flags, supported scheme options, formatting
behaviour), see the repository [README](../README.md).
