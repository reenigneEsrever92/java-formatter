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
| `SOFT_MARGINS`                           | Right margin used for line-length decisions (wins over `RIGHT_MARGIN` when both are set)                            |
| `RIGHT_MARGIN`                           | Hard right margin driving line-length decisions when `SOFT_MARGINS` is absent                                            |
| `LINE_SEPARATOR`                         | Line separator emitted at every line end — system default, LF, CRLF or CR                                                  |
| `CLASS_BRACE_STYLE`                      | Brace placement for class / interface / enum / record bodies                                                                    |
| `METHOD_BRACE_STYLE`                     | Brace placement for method, constructor and compact-constructor bodies                                                          |
| `IF_BRACE_FORCE`                         | Force braces around brace-less `if` / `else` bodies                                                                              |
| `FOR_BRACE_FORCE`                        | Force braces around brace-less `for` / enhanced-`for` bodies                                                                     |
| `WHILE_BRACE_FORCE`                      | Force braces around brace-less `while` bodies                                                                                    |
| `DOWHILE_BRACE_FORCE`                    | Force braces around brace-less `do … while` bodies                                                                               |
| `CALL_PARAMETERS_WRAP`                   | Wrapping of method-call argument lists                                                                                          |
| `CALL_PARAMETERS_LPAREN_ON_NEXT_LINE`    | Whether a wrapped call's `(` goes on its own line                                                                               |
| `CALL_PARAMETERS_RPAREN_ON_NEXT_LINE`    | Whether a wrapped call's `)` goes on its own line                                                                               |
| `METHOD_PARAMETERS_WRAP`                 | Wrapping of method / constructor parameter lists                                                                                |
| `METHOD_PARAMETERS_LPAREN_ON_NEXT_LINE`  | Whether a wrapped declaration's `(` goes on its own line                                                                        |
| `METHOD_PARAMETERS_RPAREN_ON_NEXT_LINE`  | Whether a wrapped declaration's `)` goes on its own line                                                                        |
| `METHOD_CALL_CHAIN_WRAP`                 | Wrapping of chained method calls                                                                                                |
| `ASSIGNMENT_WRAP`                        | Wrapping of assignment statements and variable / field initialisers                                                             |
| `BINARY_OPERATION_WRAP`                  | Wrapping of binary expressions at their operators                                                                               |
| `WRAP_LONG_LINES`                        | Hard-wrap lines longer than the right margin at the last whitespace boundary (literals and comments are never split)            |
| `KEEP_LINE_BREAKS`                       | Keep a construct's existing line breaks (its canonical wrapped layout) instead of joining it onto one line                        |
| `KEEP_SIMPLE_BLOCKS_IN_ONE_LINE`         | Keep one-statement blocks of `if` / `else` / `for` / `while` / `do`, `try` / `catch` / `finally` and `synchronized` on one line |
| `KEEP_SIMPLE_METHODS_IN_ONE_LINE`        | Keep single-statement method / constructor bodies on one line                                                                   |
| `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE`        | Keep single-statement lambda bodies on one line                                                                                 |
| `ANNOTATION_PARAMETER_WRAP`              | Wrapping of annotation argument lists                                                                                           |
| `RECORD_COMPONENTS_WRAP`                 | Wrapping of record component lists                                                                                              |
| `NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER` | Layout of a wrapped record header                                                                                               |
| `ALIGN_MULTILINE_RECORDS`                | Whether wrapped record components align under the first component                                                               |
| `CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND`    | Collapse single-type imports of one package into `pkg.*` above this count                                                       |
| `KEEP_BLANK_LINES_IN_CODE`               | Max blank lines kept inside code (between statements)                                                                          |
| `KEEP_BLANK_LINES_IN_DECLARATIONS`       | Max blank lines kept between class members                                                                                     |
| `KEEP_BLANK_LINES_BETWEEN_PACKAGE_DECLARATION_AND_HEADER` | Max blank lines kept between a file-header comment and the package declaration                    |
| `KEEP_BLANK_LINES_BEFORE_RBRACE`         | Max blank lines kept before a closing `}`                                                                                      |
| `BLANK_LINES_BEFORE_PACKAGE`             | Min blank lines before the package declaration                                                                                 |
| `BLANK_LINES_AFTER_PACKAGE`              | Min blank lines after the package declaration                                                                                  |
| `BLANK_LINES_BEFORE_IMPORTS`             | Min blank lines before the import section                                                                                      |
| `BLANK_LINES_AFTER_IMPORTS`              | Min blank lines after the import section                                                                                       |
| `BLANK_LINES_AROUND_CLASS`               | Min blank lines around class / interface declarations (nested and top-level)                                                   |
| `BLANK_LINES_AROUND_FIELD`               | Min blank lines around fields                                                                                                  |
| `BLANK_LINES_AROUND_METHOD`              | Min blank lines around methods and constructors                                                                                |
| `BLANK_LINES_BEFORE_METHOD_BODY`         | Min blank lines at the start of a method / constructor body                                                                    |
| `BLANK_LINES_AROUND_FIELD_IN_INTERFACE`  | Min blank lines around fields declared in interfaces                                                                           |
| `BLANK_LINES_AROUND_METHOD_IN_INTERFACE` | Min blank lines around methods declared in interfaces                                                                          |
| `BLANK_LINES_AFTER_CLASS_HEADER`         | Min blank lines after a class header, before its first member                                                                  |
| `BLANK_LINES_AFTER_ANONYMOUS_CLASS_HEADER` | Min blank lines after an anonymous class header                                                                              |
| `BLANK_LINES_BEFORE_CLASS_END`           | Min blank lines before a class's closing brace                                                                                 |
| `BLANK_LINES_AROUND_INITIALIZER`         | Min blank lines around instance / static initializer blocks                                                                    |
| `BLANK_LINES_AROUND_FIELD_WITH_ANNOTATIONS` | Min blank lines around annotated fields                                                                                      |
| `INDENT_SIZE`                            | Indentation width                                                                                                               |
| `CONTINUATION_INDENT_SIZE`               | Continuation-indent width                                                                                                       |
| `TAB_SIZE`                               | Width of a tab in columns; drives tab output and column arithmetic                                                              |
| `USE_TAB_CHARACTER`                      | Emit indentation as tab characters (tab-stop model; unset means spaces)                                                         |
| `SPACE_AROUND_ASSIGNMENT_OPERATORS`      | Space around `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`, `>>>=`                                          |
| `SPACE_AROUND_LOGICAL_OPERATORS`         | Space around `&&` and `\|\|`                                                                                                  |
| `SPACE_AROUND_EQUALITY_OPERATORS`        | Space around `==` and `!=`                                                                                                      |
| `SPACE_AROUND_RELATIONAL_OPERATORS`      | Space around `<`, `>`, `<=` and `>=`                                                                                            |
| `SPACE_AROUND_BITWISE_OPERATORS`         | Space around `&`, `\|` and `^`                                                                                                 |
| `SPACE_AROUND_ADDITIVE_OPERATORS`        | Space around `+` and `-`                                                                                                        |
| `SPACE_AROUND_MULTIPLICATIVE_OPERATORS`  | Space around `*`, `/` and `%`                                                                                                   |
| `SPACE_AROUND_SHIFT_OPERATORS`           | Space around `<<`, `>>` and `>>>`                                                                                               |
| `SPACE_AROUND_UNARY_OPERATOR`            | Space between a unary operator (`!`, `~`, unary `+` / `-`, `++`, `--`) and its operand (default: no space)                      |
| `SPACE_AROUND_LAMBDA_ARROW`              | Space around the lambda arrow `->`                                                                                              |
| `SPACE_AROUND_METHOD_REF_DBL_COLON`      | Space around the method-reference separator `::` (default: no space)                                                            |
| `SPACE_AFTER_TYPE_CAST`                  | Space between `(Type)` and the cast value (default: spaced, `(int) x`)                                                          |
| `SPACE_AFTER_COMMA`                      | Space after `,` in declarations, calls, arrays, annotations, record components, lambda parameters and type lists                |
| `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS`    | Space after `,` in generic type arguments (default: spaced, `Map<String, Integer>`)                                             |
| `SPACE_BEFORE_COMMA`                     | Space before `,` (default: none, `f(a, b)`)                                                                                    |
| `SPACE_AFTER_SEMICOLON`                  | Space after `;` inside a `for` header (default: spaced)                                                                        |
| `SPACE_BEFORE_SEMICOLON`                 | Space before `;` inside a `for` header (default: none)                                                                         |
| `SPACE_BEFORE_QUEST`                     | Space before `?` in a ternary (default: spaced)                                                                                |
| `SPACE_AFTER_QUEST`                      | Space after `?` in a ternary (default: spaced)                                                                                 |
| `SPACE_BEFORE_COLON`                     | Space before `:` in a ternary (default: spaced)                                                                                |
| `SPACE_AFTER_COLON`                      | Space after `:` in a ternary and an enhanced-`for` header (default: spaced)                                                    |
| `SPACE_BEFORE_TYPE_PARAMETER_LIST`       | Space between a class / interface / record name and its type-parameter list (default: none, `class Foo<T>`)                     |
| `SPACE_BEFORE_COLON_IN_FOREACH`          | Space before the colon in an enhanced-`for` header (default: spaced, `for (T t : xs)`)                                          |
| `SPACE_WITHIN_PARENTHESES`               | Space inside plain parentheses `( expr )` (default: none)                                                                          |
| `SPACE_WITHIN_METHOD_CALL_PARENTHESES`   | Space inside method-call parentheses `f( args )` (default: none)                                                                  |
| `SPACE_WITHIN_EMPTY_METHOD_CALL_PARENTHESES` | Space inside empty method-call parentheses `f( )` (default: none)                                                             |
| `SPACE_WITHIN_METHOD_PARENTHESES`        | Space inside method / constructor parameter parentheses `void f( params )` (default: none)                                        |
| `SPACE_WITHIN_EMPTY_METHOD_PARENTHESES`  | Space inside empty method / constructor parameter parentheses `void f( )` (default: none)                                        |
| `SPACE_WITHIN_IF_PARENTHESES`            | Space inside `if` conditions `if( cond )` (default: none)                                                                          |
| `SPACE_WITHIN_WHILE_PARENTHESES`         | Space inside `while` / `do … while` conditions `while( cond )` (default: none)                                                    |
| `SPACE_WITHIN_FOR_PARENTHESES`           | Space inside classic and enhanced `for` headers `for( … )` (default: none)                                                        |
| `SPACE_WITHIN_TRY_PARENTHESES`           | Space inside try-with-resources parentheses `try( resource )` (default: none)                                                     |
| `SPACE_WITHIN_CATCH_PARENTHESES`         | Space inside `catch` parentheses `catch( exc )` (default: none)                                                                    |
| `SPACE_WITHIN_SWITCH_PARENTHESES`        | Space inside `switch` conditions `switch( expr )` (default: none)                                                                 |
| `SPACE_WITHIN_SYNCHRONIZED_PARENTHESES`  | Space inside `synchronized` lock parentheses `synchronized( expr )` (default: none)                                               |
| `SPACE_WITHIN_CAST_PARENTHESES`          | Space inside cast parentheses `( Type ) expr` (default: none)                                                                      |
| `SPACE_WITHIN_BRACKETS`                  | Space inside `[ expr ]` array-access brackets (default: none)                                                                      |
| `SPACE_WITHIN_BRACES`                    | Space inside empty code-block / body braces `{ }` (default: none)                                                                  |
| `SPACE_WITHIN_ARRAY_INITIALIZER_BRACES`  | Space inside non-empty array-initialiser braces `{ 1, 3, 5 }` (default: none)                                                     |
| `SPACE_WITHIN_EMPTY_ARRAY_INITIALIZER_BRACES` | Space inside empty array-initialiser braces `{ }` (default: none)                                                            |
| `SPACE_WITHIN_ANNOTATION_PARENTHESES`    | Space inside annotation argument parentheses `@Anno( args )` (default: none)                                                       |
| `SPACE_BEFORE_METHOD_CALL_PARENTHESES`   | Space before method-call parentheses `f (x)` (default: none, `f(x)`)                                                                 |
| `SPACE_BEFORE_METHOD_PARENTHESES`        | Space before method / constructor declaration parentheses `void f (int p)` (default: none)                                          |
| `SPACE_BEFORE_IF_PARENTHESES`            | Space between `if` and its condition `if (...)`. `else if` chains share the toggle (default: spaced)                                |
| `SPACE_BEFORE_WHILE_PARENTHESES`         | Space between `while` and its condition, including a do-statement's trailing `while` (default: spaced)                              |
| `SPACE_BEFORE_FOR_PARENTHESES`           | Space between `for` and its header, classic and enhanced (default: spaced)                                                          |
| `SPACE_BEFORE_TRY_PARENTHESES`           | Space between `try` and its resource list (default: spaced)                                                                        |
| `SPACE_BEFORE_CATCH_PARENTHESES`         | Space between `catch` and its parameter (default: spaced)                                                                          |
| `SPACE_BEFORE_SWITCH_PARENTHESES`        | Space between `switch` and its selector (default: spaced)                                                                          |
| `SPACE_BEFORE_SYNCHRONIZED_PARENTHESES`  | Space between `synchronized` and its lock expression (default: spaced)                                                              |
| `SPACE_BEFORE_ANOTATION_PARAMETER_LIST`  | Space between an annotation name and its parameter list `@Anno (...)`. XML name spelled as in IntelliJ sources (default: none)       |
| `SPACE_BEFORE_CLASS_LBRACE`              | Space before the opening brace of class / interface / enum / record and anonymous-class bodies (default: spaced)                    |
| `SPACE_BEFORE_METHOD_LBRACE`             | Space before the opening brace of method and constructor bodies (default: spaced)                                                   |
| `SPACE_BEFORE_IF_LBRACE`                 | Space before the opening brace of an `if` body (default: spaced)                                                                    |
| `SPACE_BEFORE_ELSE_LBRACE`               | Space between `else` and its body's opening brace (default: spaced)                                                                 |
| `SPACE_BEFORE_WHILE_LBRACE`              | Space before the opening brace of a `while` body (default: spaced)                                                                  |
| `SPACE_BEFORE_FOR_LBRACE`                | Space before the opening brace of a `for` / enhanced-`for` body (default: spaced)                                                    |
| `SPACE_BEFORE_DO_LBRACE`                 | Space between `do` and its body's opening brace (default: spaced)                                                                   |
| `SPACE_BEFORE_SWITCH_LBRACE`             | Space before the opening brace of a `switch` body (default: spaced)                                                                 |
| `SPACE_BEFORE_TRY_LBRACE`                | Space before the opening brace of a `try` body, plain and with-resources (default: spaced)                                          |
| `SPACE_BEFORE_CATCH_LBRACE`              | Space before the opening brace of a `catch` body (default: spaced)                                                                  |
| `SPACE_BEFORE_FINALLY_LBRACE`            | Space between `finally` and its body's opening brace (default: spaced)                                                               |
| `SPACE_BEFORE_SYNCHRONIZED_LBRACE`       | Space before the opening brace of a `synchronized` body (default: spaced)                                                            |
| `SPACE_BEFORE_ARRAY_INITIALIZER_LBRACE`  | Space between the dimensions of `new T[]` and its initializer `new int[] {` (default: none, `new int[]{1, 2, 3}`)                    |
| `SPACE_BEFORE_ANNOTATION_ARRAY_INITIALIZER_LBRACE` | Space between an annotation's `(` and a bare array-initializer argument `@SuppressWarnings( {…)` (default: none)           |
| `SPACE_BEFORE_ELSE_KEYWORD`              | Space between `}` and the `else` keyword of an if-chain (default: spaced)                                                           |
| `SPACE_BEFORE_WHILE_KEYWORD`             | Space between `}` and the trailing `while` of a do-statement (default: spaced)                                                       |
| `SPACE_BEFORE_CATCH_KEYWORD`             | Space between `}` and the `catch` keyword of a try-statement (default: spaced)                                                       |
| `SPACE_BEFORE_FINALLY_KEYWORD`           | Space between `}` and the `finally` keyword of a try-statement (default: spaced)                                                     |

Wrapping values use IntelliJ's integer codes: `0` = do not wrap, `1` = wrap if
long, `2` = wrap always, `5` = chop down if long. Brace-forcing values
(`*_BRACE_FORCE`) use IntelliJ's force codes: `0` = do not force, `1` = force
braces when the body spans multiple lines, `3` = always force braces.

### Formatting behaviour notes

- Blank lines follow the scheme's blank-line policy: `KEEP_BLANK_LINES_*` caps
  how many pre-existing blank lines between two constructs are preserved, and
  the `BLANK_LINES_*` minimums insert the configured number around package,
  imports, class header/end, fields, methods, initializer blocks and interface
  members. Within the import section, one blank line separates the
  `java.*` / `javax.*` group from the other imports (an import-layout
  convention, independent of the blank-line options).
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
- The `*_BRACE_FORCE` options force braces around brace-less statement bodies
  (`IF_BRACE_FORCE`, `FOR_BRACE_FORCE` — covering both the classic and the
  enhanced `for` — `WHILE_BRACE_FORCE`, `DOWHILE_BRACE_FORCE`): force code `3`
  always wraps a brace-less body in `{ … }` with the statement indented one
  level, code `1` does so only when the body already spans multiple lines, and
  code `0` leaves the body as it is. Braces are only ever added, never
  stripped, so reformatting braced output is a no-op.
- Method `throws` clauses, constructor type parameters and `extends` /
  `implements` clauses are preserved.
- Spacing inside generic type-argument lists is normalised rather than copied
  from the source: no space inside the angle brackets, no space before a
  comma, and no stray spaces around nested brackets (`List< String >` and
  `Map<String ,Integer>` become `List<String>` and `Map<String, Integer>`,
  nested `Foo<Bar<Baz > >` becomes `Foo<Bar<Baz>>`). The space after each
  comma follows `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS` (default: one space;
  off renders `Map<String,Integer>`).
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
- Spacing around operators follows the `SPACE_AROUND_*` toggles: one space
  each side when on, none when off. The assignment, logical, equality,
  relational, bitwise, additive, multiplicative, shift and lambda-arrow
  options default to on; `SPACE_AROUND_UNARY_OPERATOR` and
  `SPACE_AROUND_METHOD_REF_DBL_COLON` default to off (`-a`, `i++`, `A::new`)
  and `SPACE_AFTER_TYPE_CAST` defaults to on — a type cast is rendered as
  `(int) x` by default and `(int)x` with the option off. Spacing applies
  inside wrapped binary expressions (the continuation line joins the operand
  to the operator when the toggle is off) and to one-line lambdas as well as
  flat expressions. Separator spacing is likewise toggleable: commas
  (`SPACE_AFTER_COMMA` / `SPACE_BEFORE_COMMA`, and
  `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS` for type arguments), the `for`-header
  semicolons (`SPACE_AFTER_SEMICOLON` / `SPACE_BEFORE_SEMICOLON`), the
  ternary `?` (`SPACE_BEFORE_QUEST` / `SPACE_AFTER_QUEST`) and `:`
  (`SPACE_BEFORE_COLON` / `SPACE_AFTER_COLON`), the enhanced-`for` colon
  (`SPACE_BEFORE_COLON_IN_FOREACH`), and the class / interface / record
  name-to-type-parameter gap (`SPACE_BEFORE_TYPE_PARAMETER_LIST`) all follow
  their toggles; `instanceof`, annotation element-value `=` and switch `->`
  arrows stay always spaced regardless.
- Spacing inside parentheses, brackets and braces follows the `SPACE_WITHIN_*`
  toggles, all off by default: when one is on, a single space is inserted just
  inside the delimiter pair it names — `( expr )` for
  `SPACE_WITHIN_PARENTHESES`, `f( args )` for
  `SPACE_WITHIN_METHOD_CALL_PARENTHESES`, `void f( params )` for
  `SPACE_WITHIN_METHOD_PARENTHESES`, the `if` / `while` / `do … while` / `for` /
  `try` / `catch` / `switch` / `synchronized` conditions and headers each
  governed by their own option, `( Type ) expr` for
  `SPACE_WITHIN_CAST_PARENTHESES`, `a[ 0 ]` for `SPACE_WITHIN_BRACKETS`, `{ … }`
  for `SPACE_WITHIN_BRACES`, `{ 1, 3, 5 }` for
  `SPACE_WITHIN_ARRAY_INITIALIZER_BRACES` and `@Anno( args )` for
  `SPACE_WITHIN_ANNOTATION_PARENTHESES`. The empty variants are independent:
  `SPACE_WITHIN_EMPTY_METHOD_CALL_PARENTHESES` renders `f( )`,
  `SPACE_WITHIN_EMPTY_METHOD_PARENTHESES` renders `void f( )`, and
  `SPACE_WITHIN_EMPTY_ARRAY_INITIALIZER_BRACES` renders `{ }`; a bare `@A()`
  stays tight. Wrapped (multi-line) parameter and argument lists keep their
  line breaks — no space is inserted next to a newline, so no trailing
  whitespace is produced — and single-line `{ … }` bodies already carry one
  inner space and are unchanged. The change is whitespace-only (R5) and
  re-formatting padded output reproduces it (R6).
- The gap *before* a parenthesis, brace or clause keyword follows the
  `SPACE_BEFORE_*` toggles, one space when on, none when off. The
  keyword-to-paren toggles (`SPACE_BEFORE_IF_PARENTHESES`,
  `SPACE_BEFORE_WHILE_PARENTHESES` — covering the do-statement's trailing
  `while` too — `SPACE_BEFORE_FOR_PARENTHESES`, `SPACE_BEFORE_TRY_PARENTHESES`,
  `SPACE_BEFORE_CATCH_PARENTHESES`, `SPACE_BEFORE_SWITCH_PARENTHESES`,
  `SPACE_BEFORE_SYNCHRONIZED_PARENTHESES`) and the keyword-to-`{` toggles
  (`SPACE_BEFORE_CLASS_LBRACE`, `SPACE_BEFORE_METHOD_LBRACE`,
  `SPACE_BEFORE_IF_LBRACE`, `SPACE_BEFORE_ELSE_LBRACE`,
  `SPACE_BEFORE_WHILE_LBRACE`, `SPACE_BEFORE_FOR_LBRACE`,
  `SPACE_BEFORE_DO_LBRACE`, `SPACE_BEFORE_SWITCH_LBRACE`,
  `SPACE_BEFORE_TRY_LBRACE`, `SPACE_BEFORE_CATCH_LBRACE`,
  `SPACE_BEFORE_FINALLY_LBRACE`, `SPACE_BEFORE_SYNCHRONIZED_LBRACE`) default to
  on (`if (x) {`, `} else {`); `SPACE_BEFORE_METHOD_CALL_PARENTHESES` (calls,
  constructor calls and chains), `SPACE_BEFORE_METHOD_PARENTHESES`,
  `SPACE_BEFORE_ANOTATION_PARAMETER_LIST`, `SPACE_BEFORE_ARRAY_INITIALIZER_LBRACE`
  and `SPACE_BEFORE_ANNOTATION_ARRAY_INITIALIZER_LBRACE` default to off
  (`f(x)`, `void f(int p)`, `@Anno(...)`, `new int[]{1, 2, 3}`). The `}`-to-
  keyword toggles (`SPACE_BEFORE_ELSE_KEYWORD`, `SPACE_BEFORE_WHILE_KEYWORD`,
  `SPACE_BEFORE_CATCH_KEYWORD`, `SPACE_BEFORE_FINALLY_KEYWORD`) are independent
  of the brace toggles, so `} else {`, `}else {`, `}else{` are all reachable.
  The `for` / `try` headers are rebuilt from source bytes, so their
  `for (` / `try (` gap is pinned to the toggle rather than copied from the
  input. The change is whitespace-only (R5) and inserting or removing one
  space is idempotent (R6).
- Line endings follow the root-level `LINE_SEPARATOR` option — `&#10;` (LF),
  `&#13;&#10;` (CRLF) or `&#13;` (CR); the default (`System`) emits the
  platform's own separator (`\n` on Unix). The configured separator is
  applied at every line end, including the final newline, and the whole
  reformat stays idempotent (R6). The line-length limit is set by the
  root-level `SOFT_MARGINS` (first value) when present and otherwise by
  `RIGHT_MARGIN`; when a scheme sets both, `SOFT_MARGINS` wins.
- With `KEEP_LINE_BREAKS` (default on), a construct whose source spans
  multiple lines keeps its canonical wrapped layout — one argument /
  parameter / operand / chain link / array element per line at the
  continuation indent — even when the flat form fits the margin; with the
  option off (or a joined source) the flatten-if-fits behaviour reflows the
  code. The retained breaks land at the canonical wrap boundaries (the
  engine re-renders rather than re-indenting source lines), keeping the
  output deterministic and idempotent; the opt-in `KEEP_SIMPLE_*` one-liner
  collapses and fixed structural layouts (blocks, bodies) win over line-break
  retention.
- With `WRAP_LONG_LINES`, a line longer than the right margin is hard-wrapped
  at the rightmost whitespace boundary at or before the margin, continuing at
  the line's indent plus `CONTINUATION_INDENT_SIZE`. The pass never splits a
  string / char literal or a comment, and comment-only lines are left alone
  (`WRAP_COMMENTS` governs those); an over-long line with no safe boundary (a
  long string literal or single token) stays over-long. The wrap points are a
  pure function of the flat text, so re-formatting reproduces them (R6).

## Desktop GUI

`java-formatter-gui` is a desktop codestyle editor built with egui (eframe):

```sh
cargo run -p java-formatter-gui
```

It renders every option the formatter supports, grouped logically, with the
correct control per type (bool → checkbox, `u32` → drag value, wrap/brace/force
→ labeled combo of the IntelliJ meaning):

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
