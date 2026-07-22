# java-formatter

A Java source code formatter that applies [IntelliJ IDEA code style](https://www.jetbrains.com/help/idea/code-style.html) rules declared in a `codestyle.xml` scheme (e.g. `.idea/codeStyles/Project.xml`).

It parses Java with [tree-sitter-java](https://github.com/tree-sitter/tree-sitter-java) and pretty-prints the syntax tree according to a [`JavaStyle`](crates/core/src/config.rs) configuration. When no style file is given, IntelliJ built-in defaults are used.

## Building

Requires Rust (stable) and Cargo.

```sh
cargo build --release
```

The binary is written to `target/release/java-formatter`.

## Usage

```text
Usage: java-formatter [OPTIONS] [FILE]

Arguments:
  [FILE]  Path to the Java source file to format. Reads from standard input
          when omitted or when '-' is given

Options:
  -s, --style <STYLE>  Path to an IntelliJ codestyle XML file
                       (e.g. .idea/codeStyles/Project.xml).
                       Defaults to IntelliJ built-in settings when omitted
  -h, --help           Print help
```

The formatted source is written to stdout.

### Examples

Format a file with the repository's `codestyle.xml`:

```sh
java-formatter --style codestyle.xml src/main/java/demo/Foo.java
```

Format a file with the default (IntelliJ built-in) style:

```sh
java-formatter Foo.java
```

Read from stdin (either by omitting the file, or by passing `-`):

```sh
java-formatter --style codestyle.xml < Foo.java
cat Foo.java | java-formatter - --style codestyle.xml
```

Update a file (format to a new file, then replace):

```sh
java-formatter --style codestyle.xml Foo.java > Foo.formatted.java
mv Foo.formatted.java Foo.java
```

## Style files

A style file is an IntelliJ `<code_scheme>` XML document. Only the settings that
apply to **Java** are read; blocks for other languages (`HTMLCodeStyleSettings`,
`JSCodeStyleSettings`, `<codeStyleSettings language="JavaScript">`, …) are
ignored, as are `<option>` elements the formatter does not implement.

The options currently honoured are:

| Option                                   | Effect                                                                                                                          |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `SOFT_MARGINS`                           | Right margin used for line-length decisions                                                                                     |
| `CLASS_BRACE_STYLE`                      | Brace placement for class / interface / enum / record bodies                                                                    |
| `METHOD_BRACE_STYLE`                     | Brace placement for method, constructor and compact-constructor bodies                                                          |
| `CALL_PARAMETERS_WRAP`                   | Wrapping of method-call argument lists                                                                                          |
| `CALL_PARAMETERS_LPAREN_ON_NEXT_LINE`    | Whether a wrapped call's `(` goes on its own line                                                                               |
| `CALL_PARAMETERS_RPAREN_ON_NEXT_LINE`    | Whether a wrapped call's `)` goes on its own line                                                                               |
| `METHOD_PARAMETERS_WRAP`                 | Wrapping of method / constructor parameter lists                                                                                |
| `METHOD_PARAMETERS_LPAREN_ON_NEXT_LINE`  | Whether a wrapped declaration's `(` goes on its own line                                                                        |
| `METHOD_PARAMETERS_RPAREN_ON_NEXT_LINE`  | Whether a wrapped declaration's `)` goes on its own line                                                                        |
| `METHOD_CALL_CHAIN_WRAP`                 | Wrapping of chained method calls                                                                                                |
| `ASSIGNMENT_WRAP`                        | Wrapping of assignment statements and variable / field initialisers                                                             |
| `BINARY_OPERATION_WRAP`                  | Wrapping of binary expressions at their operators                                                                               |
| `KEEP_SIMPLE_BLOCKS_IN_ONE_LINE`         | Keep one-statement blocks of `if` / `else` / `for` / `while` / `do`, `try` / `catch` / `finally` and `synchronized` on one line |
| `KEEP_SIMPLE_METHODS_IN_ONE_LINE`        | Keep single-statement method / constructor bodies on one line                                                                   |
| `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE`        | Keep single-statement lambda bodies on one line                                                                                 |
| `ANNOTATION_PARAMETER_WRAP`              | Wrapping of annotation argument lists                                                                                           |
| `RECORD_COMPONENTS_WRAP`                 | Wrapping of record component lists                                                                                              |
| `NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER` | Layout of a wrapped record header                                                                                               |
| `ALIGN_MULTILINE_RECORDS`                | Whether wrapped record components align under the first component                                                               |
| `CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND`    | Collapse single-type imports of one package into `pkg.*` above this count                                                       |
| `INDENT_SIZE`                            | Indentation width                                                                                                               |
| `CONTINUATION_INDENT_SIZE`               | Continuation-indent width                                                                                                       |
| `TAB_SIZE`                               | Width of a tab in columns; drives tab output and column arithmetic                                                              |
| `USE_TAB_CHARACTER`                      | Emit indentation as tab characters (tab-stop model; unset means spaces)                                                         |

Wrapping values use IntelliJ's integer codes: `0` = do not wrap, `1` = wrap if
long, `2` = wrap always, `5` = chop down if long.

### Formatting behaviour notes

- A blank line is inserted before the `java.*` / `javax.*` import group.
- Import-on-demand merging is conservative: it is skipped when the file already
  uses a wildcard import, when a simple name would become ambiguous (imported
  from another package), when a top-level type of the same name is declared in
  the file, and it never merges static imports.
- Conditions (`if`, `while`, `do`, `synchronized`) are rendered with exactly the
  parentheses that belong to them; no extra parentheses are added.
- With `KEEP_SIMPLE_BLOCKS_IN_ONE_LINE`, a one-statement `try` body collapses to
  one line along with its `catch` / `finally` clauses only when every body in
  the statement is simple and the whole statement fits the margin; otherwise
  the multi-line layout is used. `synchronized` bodies collapse the same way.
- Method `throws` clauses, constructor type parameters and `extends` /
  `implements` clauses are preserved.
- Spacing inside generic type-argument lists is normalised rather than copied
  from the source: no space inside the angle brackets, no space before a
  comma, one space after a comma, and no stray spaces around nested brackets
  (`List< String >` and `Map<String ,Integer>` become `List<String>` and
  `Map<String, Integer>`, nested `Foo<Bar<Baz > >` becomes `Foo<Bar<Baz>>`).
  The same canonical spacing is applied to declaration type-parameter lists
  (`<T extends Number & Serializable, U>`), wildcards (`? extends T` /
  `? super T`), and array dimensions; already canonical input is unchanged.
- Wrapped binary expressions break at their top-level operators: the operator
  goes at the start of the continuation line, at the continuation indent
  (`BINARY_OPERATION_WRAP`). `5` (chop down) also breaks a nested binary
  operand whose own line overflows.
- With `USE_TAB_CHARACTER`, indentation is emitted as tab characters using a
  tab-stop model: each full `TAB_SIZE` of indentation width becomes one tab,
  and any remainder becomes spaces — so with `INDENT_SIZE == TAB_SIZE` each
  level is exactly one tab, while continuation indents that are not a whole
  number of tabs (e.g. `TAB_SIZE` 4 with a continuation indent of 6) emit a
  tab plus trailing spaces. Alignment that needs exact columns (e.g. record
  components aligned under the opening paren) stays space-based. Margin and
  wrap decisions always use logical columns — a tab counts as `TAB_SIZE` — so
  a tab scheme wraps exactly where the equivalent space scheme does. Without
  `USE_TAB_CHARACTER`, output is space-indented as before.
- `switch` statements are laid out with `case` / `default` labels indented one
  level and their statements a further level; colon and arrow (`case x ->`)
  forms are preserved. A switch expression used as a value (assignment,
  return, argument) stays on one line when the whole construct fits the
  margin, and falls back to the same multi-line layout otherwise.
- Input that is not valid Java is reported, not silently formatted: parse
  errors and missing tokens are written to stderr as `warning:` lines
  (naming the construct and its line:column), while the best-effort formatted
  source is still written to stdout and the exit code stays 0. Anything the
  formatter does not model — valid or not — is preserved verbatim, never
  dropped or invented.

## Desktop GUI

`java-formatter-gui` is a desktop codestyle editor built with egui (eframe):

```sh
cargo run -p java-formatter-gui
```

It renders every option the formatter supports, grouped logically, with the
correct control per type (bool → checkbox, `u32` → drag value, wrap/brace →
labeled combo of the IntelliJ meaning):

- **New** resets the style to the IntelliJ built-in defaults.
- **Open…** opens a native file chooser (or drop a `codestyle.xml` anywhere in
  the window) and loads an existing scheme; the file is parsed with the same
  `parse_codestyle` the CLI uses.
- Editing any option immediately re-formats the preview pane, which formats
  the Java source in the editor with the current style.
- **Save** writes a minimal `<code_scheme>` via `serialize_codestyle` to the
  path field: only options that differ from the IntelliJ defaults are written,
  matching IntelliJ's own export convention, so the file stays small and
  remains semantically identical. A loaded IntelliJ `Project.xml` does **not**
  round-trip losslessly — other-language blocks and unknown options are
  dropped on save (a documented limitation; in-place XML editing is out of
  scope).

## Architecture

The repository is a Cargo workspace with three members under `crates/`:

```
crates/core/  java-formatter-core — the library: lib.rs (crate root),
              config.rs (parses IntelliJ <code_scheme> XML into a JavaStyle),
              formatter.rs (tree-sitter-based formatting engine). Its
              integration suite, fixtures and Criterion benchmarks live under
              crates/core/tests/ and crates/core/benches/
crates/cli/   java-formatter-cli — the CLI binary (java-formatter): file /
              stdin input, --style; built on java-formatter-core
crates/gui/   java-formatter-gui — the desktop codestyle editor (see the
              README's _Desktop GUI_ section): egui/eframe app rendering
              every option from core's OPTIONS registry with a live
              formatting preview; opens schemes via a file chooser or
              drag-and-drop and saves minimal codestyle files
```

The `codestyle.xml` sample stays at the workspace root.

## Testing

```sh
cargo test
```

The integration suite lives in [`crates/core/tests/`](crates/core/tests/).
Every supported code-style option has a dedicated test file under
[`crates/core/tests/options/`](crates/core/tests/options/), named after the
XML option it exercises (e.g. `assignment_wrap.rs` for `ASSIGNMENT_WRAP`,
`method_call_chain_wrap.rs` for `METHOD_CALL_CHAIN_WRAP`), wired together by
the `tests/options.rs` aggregator. Every test is a golden pair: it formats a
`.java` fixture under a specific style and compares the byte-exact output to a
`*.out.java` golden next to it (e.g. `tests/java/assignment_wrap/long_init_wrap_if_long.java`
→ `..._wrap_if_long.out.java`), so each option's input→output transformation
is visible at a glance. Fixtures are embedded into the tests at compile time.

## Benchmarking

A [Criterion](https://github.com/bheisler/criterion.rs) suite lives in
[`crates/core/benches/`](crates/core/benches/). It benchmarks:

- formatting the realistic kitchen-sink fixture,
- formatting synthetically generated files of 50 / 200 / 600 classes,
- parsing the project's `codestyle.xml`,

each formatting case run with both the default style and the `codestyle.xml`
style, reporting throughput in MiB/s.

```sh
cargo bench
```

The suite is configured to run quickly by default; pass standard criterion
flags to trade runtime for precision, e.g.
`cargo bench -- --sample-size 100`.

## Limitations

- Only Java is supported; IntelliJ settings for HTML, JavaScript, TypeScript
  and Vue in the scheme file are ignored.
