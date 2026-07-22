---
type: Overview
title: java-formatter
description: A CLI that formats Java source according to IntelliJ IDEA code style schemes.
tags: [java, formatter, cli]
status: active
---

# Overview

**java-formatter** is a command-line tool that formats Java source code
according to the code style rules declared in an IntelliJ IDEA
`codestyle.xml` scheme (for example `.idea/codeStyles/Project.xml`). It parses
Java with tree-sitter-java and pretty-prints the syntax tree following a
`config::JavaStyle` configuration; when no style file is given it uses the
IntelliJ built-in defaults.

Teams that standardise on an IntelliJ scheme can apply the same style outside
the IDE — in editors, scripts, or on machines without IntelliJ — and get
output that matches what the IDE would produce. The repository is a Cargo
workspace: a small formatting library (`java-formatter-core`, split into a
config module for scheme parsing and a formatter module for tree-based
pretty-printing), a thin CLI over it (`java-formatter-cli`, whose binary is
named `java-formatter`), and a desktop GUI crate (`java-formatter-gui`, an
egui codestyle editor built on the core) — all under `crates/`.

Correctness is the project's first value: the formatted output must be
semantically equivalent to the input — only formatting changes — and
constructs the formatter does not model are preserved verbatim from the
source rather than re-synthesised.

See the [requirements analysis](requirements.md) for the users, use cases,
and technology decisions, and the [development section](dev/index.md) for the
work ahead.
