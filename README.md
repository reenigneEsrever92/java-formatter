# java-formatter

[![CI](https://github.com/reenigneEsrever92/java-formatter/actions/workflows/ci.yml/badge.svg)](https://github.com/reenigneEsrever92/java-formatter/actions/workflows/ci.yml)

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
| `LAMBDA_BRACE_STYLE`                     | Brace placement for block lambda bodies                                                                                          |
| `IF_BRACE_FORCE`                         | Force braces around brace-less `if` / `else` bodies                                                                              |
| `FOR_BRACE_FORCE`                        | Force braces around brace-less `for` / enhanced-`for` bodies                                                                     |
| `WHILE_BRACE_FORCE`                      | Force braces around brace-less `while` bodies                                                                                    |
| `DOWHILE_BRACE_FORCE`                    | Force braces around brace-less `do … while` bodies                                                                               |
| `ELSE_ON_NEW_LINE`                       | Put the `else` of an if / else-if chain on a new line                                                                             |
| `WHILE_ON_NEW_LINE`                      | Put a `do … while`'s trailing `while (…) ;` on a new line                                                                         |
| `CATCH_ON_NEW_LINE`                      | Put each `catch` clause of a `try` on a new line                                                                                  |
| `FINALLY_ON_NEW_LINE`                    | Put the `finally` clause of a `try` on a new line                                                                                 |
| `SPECIAL_ELSE_IF_TREATMENT`              | Keep `else if` fused as one construct instead of nesting `else { if … }`                                                          |
| `INDENT_CASE_FROM_SWITCH`                | Indent `case` / `default` labels one level from the `switch`                                                                          |
| `CASE_STATEMENT_ON_NEW_LINE`             | Put the statement after a `case` / `default` label on a new line                                                                      |
| `INDENT_BREAK_FROM_CASE`                 | Indent `break` / `continue` / `return` one level from the `case` label                                                                |
| `CALL_PARAMETERS_WRAP`                   | Wrapping of method-call argument lists                                                                                          |
| `CALL_PARAMETERS_LPAREN_ON_NEXT_LINE`    | Whether a wrapped call's `(` goes on its own line                                                                               |
| `CALL_PARAMETERS_RPAREN_ON_NEXT_LINE`    | Whether a wrapped call's `)` goes on its own line                                                                               |
| `PREFER_PARAMETERS_WRAP`                 | Prefer wrapping the argument list of a chain's tail call over breaking the chain                                                |
| `METHOD_PARAMETERS_WRAP`                 | Wrapping of method / constructor parameter lists                                                                                |
| `METHOD_PARAMETERS_LPAREN_ON_NEXT_LINE`  | Whether a wrapped declaration's `(` goes on its own line                                                                        |
| `METHOD_PARAMETERS_RPAREN_ON_NEXT_LINE`  | Whether a wrapped declaration's `)` goes on its own line                                                                        |
| `RESOURCE_LIST_WRAP`                     | Wrapping of try-with-resources resource lists                                                                                   |
| `RESOURCE_LIST_LPAREN_ON_NEXT_LINE`      | Whether a wrapped resource list's `(` goes on its own line                                                                      |
| `RESOURCE_LIST_RPAREN_ON_NEXT_LINE`      | Whether a wrapped resource list's `)` goes on its own line                                                                      |
| `METHOD_CALL_CHAIN_WRAP`                 | Wrapping of chained method calls                                                                                                |
| `WRAP_FIRST_METHOD_IN_CALL_CHAIN`        | Whether the first link of a wrapped chain also goes on a continuation line                                                    |
| `WRAP_SEMICOLON_AFTER_CALL_CHAIN`        | Put the `;` of a wrapped chained call on its own line                                                                          |
| `ASSIGNMENT_WRAP`                        | Wrapping of assignment statements and variable / field initialisers                                                             |
| `PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE`     | Put the assignment operator at the start of the continuation line                                                               |
| `BINARY_OPERATION_WRAP`                  | Wrapping of binary expressions at their operators                                                                               |
| `BINARY_OPERATION_SIGN_ON_NEXT_LINE`     | Put a binary operator at the start of the continuation line                                                                     |
| `TERNARY_OPERATION_WRAP`                 | Wrapping of ternary (`?:`) expressions                                                                                          |
| `TERNARY_OPERATION_SIGNS_ON_NEXT_LINE`   | Put `?` / `:` of a wrapped ternary at the start of continuation lines                                                           |
| `ASSERT_STATEMENT_WRAP`                  | Wrapping of `assert` statements                                                                                                 |
| `ASSERT_STATEMENT_COLON_ON_NEXT_LINE`    | Put the `:` of a wrapped `assert` at the start of the next line                                                                 |
| `FOR_STATEMENT_WRAP`                     | Wrapping of `for` headers                                                                                                       |
| `FOR_STATEMENT_LPAREN_ON_NEXT_LINE`      | Whether a wrapped `for` header's `(` goes on its own line                                                                       |
| `FOR_STATEMENT_RPAREN_ON_NEXT_LINE`      | Whether a wrapped `for` header's `)` goes on its own line                                                                       |
| `ARRAY_INITIALIZER_WRAP`                 | Wrapping of array initializer lists                                                                                             |
| `ARRAY_INITIALIZER_LBRACE_ON_NEXT_LINE`  | Whether a wrapped array initializer's `{` goes on its own line                                                                  |
| `ARRAY_INITIALIZER_RBRACE_ON_NEXT_LINE`  | Whether a wrapped array initializer's `}` goes on its own line                                                                  |
| `MODIFIER_LIST_WRAP`                     | Wrap after the modifier / annotation list of a declaration                                                                      |
| `PARENTHESES_EXPRESSION_LPAREN_WRAP`     | Whether a wrapped parenthesized expression's `(` goes on its own line                                                           |
| `PARENTHESES_EXPRESSION_RPAREN_WRAP`     | Whether a wrapped parenthesized expression's `)` goes on its own line                                                           |
| `EXTENDS_LIST_WRAP`                      | Wrapping of `extends` / `implements` lists of type declarations                                                                 |
| `EXTENDS_KEYWORD_WRAP`                   | Whether a wrapped list's `extends` / `implements` keyword goes on its own line                                                   |
| `THROWS_LIST_WRAP`                       | Wrapping of method / constructor `throws` lists                                                                                 |
| `THROWS_KEYWORD_WRAP`                    | Whether a wrapped `throws` list's keyword goes on its own line                                                                  |
| `SWITCH_EXPRESSIONS_WRAP`                | Wrapping of switch expressions used as values                                                                                    |
| `WRAP_LONG_LINES`                        | Hard-wrap lines longer than the right margin at the last whitespace boundary (literals and comments are never split)            |
| `KEEP_LINE_BREAKS`                       | Keep a construct's existing line breaks (its canonical wrapped layout) instead of joining it onto one line                        |
| `LINE_COMMENT_AT_FIRST_COLUMN`           | Place `//` line comments at the first column (no indent)                                                                           |
| `BLOCK_COMMENT_AT_FIRST_COLUMN`          | Place `/* */` block comments at the first column                                                                                   |
| `KEEP_FIRST_COLUMN_COMMENT`              | Keep comments that start in the first column at the first column                                                                   |
| `LINE_COMMENT_ADD_SPACE_ON_REFORMAT`     | Insert the space after `//` on reformat                                                                                            |
| `LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION`  | Insert the space after `//` inside `//noinspection` suppression comments                                                           |
| `WRAP_COMMENTS`                          | Wrap long comments to the right margin                                                                                             |
| `KEEP_SIMPLE_BLOCKS_IN_ONE_LINE`         | Keep one-statement blocks of `if` / `else` / `for` / `while` / `do`, `try` / `catch` / `finally` and `synchronized` on one line |
| `KEEP_SIMPLE_METHODS_IN_ONE_LINE`        | Keep single-statement method / constructor bodies on one line                                                                   |
| `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE`        | Keep single-statement lambda bodies on one line                                                                                 |
| `KEEP_SIMPLE_CLASSES_IN_ONE_LINE`        | Keep simple class / interface / record bodies on one line                                                                     |
| `KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE`  | Keep multiple expressions (e.g. a classic `for` header's init / update lists) on one line                                      |
| `SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT` | Spaces inside `{ … }` of a non-empty one-line block when `SPACE_WITHIN_BRACES` is off (absent/false renders flush `{s}`) |
| `NEW_LINE_WHEN_BODY_IS_PRESENTED`        | Put the body of a one-line block on a new line below its statement head                                                          |
| `KEEP_CONTROL_STATEMENT_IN_ONE_LINE`     | Keep a brace-less control-statement body on the header's line when the source has it there                                      |
| `ANNOTATION_PARAMETER_WRAP`              | Wrapping of annotation argument lists                                                                                           |
| `METHOD_ANNOTATION_WRAP`                 | Placement of a method's annotations (own line vs inline with the declaration)                                                 |
| `CLASS_ANNOTATION_WRAP`                  | Placement of a class / interface / enum / record's annotations                                                                 |
| `FIELD_ANNOTATION_WRAP`                  | Placement of a field's annotations                                                                                             |
| `PARAMETER_ANNOTATION_WRAP`              | Placement of a parameter's annotations in the wrapped (per-line) parameter list                                               |
| `VARIABLE_ANNOTATION_WRAP`               | Placement of a local variable's annotations                                                                                    |
| `ENUM_FIELD_ANNOTATION_WRAP`             | Placement of enum constant annotations                                                                                         |
| `ALIGN_MULTILINE_ANNOTATION_PARAMETERS`  | Align wrapped annotation arguments under the first argument                                                                     |
| `NEW_LINE_AFTER_LPAREN_IN_ANNOTATION`    | Put `(` of a wrapped annotation argument list on its own line                                                                   |
| `RPAREN_ON_NEW_LINE_IN_ANNOTATION`       | Put `)` of a wrapped annotation argument list on its own line                                                                   |
| `SPACE_AROUND_ANNOTATION_EQ`             | Spaces around `=` in annotation arguments (default: spaced)                                                                     |
| `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION`    | Keep a lone field / method / class / local-variable annotation inline regardless of the wrap code                              |
| `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION_IN_PARAMETER` | Keep a lone parameter annotation inline in the wrapped parameter list                                              |
| `RECORD_COMPONENTS_WRAP`                 | Wrapping of record component lists                                                                                              |
| `NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER` | Layout of a wrapped record header                                                                                               |
| `ALIGN_MULTILINE_RECORDS`                | Whether wrapped record components align under the first component                                                               |
| `ALIGN_MULTILINE_PARAMETERS`             | Align wrapped method / constructor parameter lists under the first parameter (default on)                                      |
| `ALIGN_MULTILINE_PARAMETERS_IN_CALLS`    | Align wrapped method-call / `new` arguments under the first argument                                                            |
| `ALIGN_MULTILINE_RESOURCES`              | Align wrapped try-with-resources lists under the first resource (default on)                                                   |
| `ALIGN_MULTILINE_FOR`                    | Align the parts of a wrapped `for` header under its first slot (default on)                                                    |
| `ALIGN_MULTILINE_BINARY_OPERATION`       | Align wrapped binary operands under the first operand                                                                            |
| `ALIGN_MULTILINE_ASSIGNMENT`             | Align a wrapped assignment's right-hand side under the operator                                                                  |
| `ALIGN_MULTILINE_TERNARY_OPERATION`      | Align the `?` / `:` lines of a wrapped ternary under the condition                                                               |
| `ALIGN_MULTILINE_THROWS_LIST`            | Align wrapped `throws` list entries under the first exception                                                                    |
| `ALIGN_THROWS_KEYWORD`                   | Align a wrapped `throws` keyword at its natural header column                                                                     |
| `ALIGN_MULTILINE_EXTENDS_LIST`           | Align wrapped `extends` / `implements` entries under the first entry                                                             |
| `ALIGN_MULTILINE_METHOD_BRACKETS`        | Align a wrapped declaration's `)` under its `(`                                                                                   |
| `ALIGN_MULTILINE_PARENTHESIZED_EXPRESSION` | Align a wrapped parenthesized expression's continuation under the `(`                                                          |
| `ALIGN_MULTILINE_ARRAY_INITIALIZER_EXPRESSION` | Align a wrapped array initializer's entries under the first entry                                                         |
| `ALIGN_MULTILINE_CHAINED_METHODS`        | Align the dots of a wrapped chained call under the first call's dot                                                               |
| `ALIGN_GROUP_FIELD_DECLARATIONS`         | Align the declared names of output-adjacent fields in columns                                                                     |
| `ALIGN_CONSECUTIVE_VARIABLE_DECLARATIONS` | Align the declared names of consecutive local variables in columns                                                              |
| `ALIGN_CONSECUTIVE_ASSIGNMENTS`          | Align the operators of consecutive assignment statements in a column                                                              |
| `ALIGN_SUBSEQUENT_SIMPLE_METHODS`        | Align the names of output-adjacent one-line methods in columns                                                                    |
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
| `SMART_TABS`                             | With `USE_TAB_CHARACTER`, use tab characters only for indentation that lands exactly on a tab stop (off-stop indents stay spaces) |
| `LABEL_INDENT_SIZE`                      | Indent for `label:` lines (relative to the statement indent by default)                                                         |
| `LABEL_INDENT_ABSOLUTE`                  | Measure the label indent from the left margin regardless of nesting                                                              |
| `USE_RELATIVE_INDENTS`                   | With `USE_TAB_CHARACTER`, measure continuation indents from the construct's own indent level                                     |
| `KEEP_INDENTS_ON_EMPTY_LINES`            | Keep the block's inner indent on preserved blank lines                                                                          |
| `DECLARATION_PARAMETER_INDENT`           | Per-construct continuation indent for wrapped declaration parameters (`-1` = inherit)                                           |
| `GENERIC_TYPE_PARAMETER_INDENT`          | Per-construct continuation indent for generic type parameters (`-1` = inherit; inert — generic lists render flat)                |
| `CALL_PARAMETER_INDENT`                  | Per-construct continuation indent for wrapped call arguments (`-1` = inherit)                                                   |
| `CHAINED_CALL_INDENT`                    | Per-construct continuation indent for wrapped chained-call links (`-1` = inherit)                                               |
| `ARRAY_ELEMENT_INDENT`                   | Per-construct continuation indent for wrapped array elements (`-1` = inherit)                                                   |
| `DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS`  | Do not indent the members of a top-level class (they sit at the class declaration indent)                                        |
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
- Clause-keyword layout follows the scheme: `ELSE_ON_NEW_LINE` puts the `else`
  keyword (and each `else if` of a chain) on a fresh line at the statement
  indent, `WHILE_ON_NEW_LINE` does the same for a `do … while`'s trailing
  `while (…) ;`, and `CATCH_ON_NEW_LINE` / `FINALLY_ON_NEW_LINE` for a `try`'s
  `catch` / `finally` clauses. `SPECIAL_ELSE_IF_TREATMENT` (default on) keeps an
  `else if` chain fused; off, each level is rewritten as `else { if … }`.
  `LAMBDA_BRACE_STYLE` places a block lambda's `{` per its brace code,
  independently of `BRACE_STYLE`; the `NextLine` family puts the brace on its
  own line at the statement indent.
- `KEEP_CONTROL_STATEMENT_IN_ONE_LINE` (default on) is source-driven: a
  brace-less body (`if (x) foo();`, `while (go) step();`, `for (…) use(i);`,
  `do tick(); while (go);`) that the source already has on the header's line
  stays there, and a body on its own line keeps its own line. Off, every
  brace-less body is moved to its own line.
- The keep-simple one-liners (`KEEP_SIMPLE_BLOCKS_IN_ONE_LINE`,
  `KEEP_SIMPLE_METHODS_IN_ONE_LINE`, `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE` and
  `KEEP_SIMPLE_CLASSES_IN_ONE_LINE`) collapse a body only when it is simple
  and the whole construct fits the right margin; a simple class / interface /
  record body collapses when every member renders on one line (methods need
  `KEEP_SIMPLE_METHODS_IN_ONE_LINE`, comments / extras reject) — enums and
  anonymous classes are unaffected. A one-line non-empty block is rendered
  *flush* (`if (c) {use();}`) by default, matching IntelliJ's built-in
  `SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT = false`; the Java toggle
  adds the inner spaces (`if (c) { use(); }`). The other Java presentation
  toggle, `NEW_LINE_WHEN_BODY_IS_PRESENTED`, puts the collapsed block on its
  own line below the statement head at the head's indent
  (`if (c)` then `{ use(); }`). `KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE` keeps
  the multiple expressions of a statement — a classic `for` header's
  init/update clause lists, a multi-declarator field / local declaration —
  joined on one line; these lists are never split per expression, so the
  option's on / off / absent output is identical (it becomes load-bearing
  only if a per-expression wrap is ever added). The flat one-line bodies of
  call-argument lambdas and one-line switch values keep their pinned spaced
  `{ … }` layout regardless of these toggles.
- Resource lists and declaration clause lists wrap per the scheme's options:
  `RESOURCE_LIST_WRAP` (with `RESOURCE_LIST_LPAREN_ON_NEXT_LINE` /
  `RESOURCE_LIST_RPAREN_ON_NEXT_LINE`) breaks an over-margin
  try-with-resources list into one resource per line, and `EXTENDS_LIST_WRAP` /
  `THROWS_LIST_WRAP` break over-margin `extends` / `implements` and `throws`
  lists into one type per line at the continuation indent — with
  `EXTENDS_KEYWORD_WRAP` / `THROWS_KEYWORD_WRAP` moving the keyword to its own
  line. A class's single `extends Base` supertype is not a list and never
  wraps; code `5` (chop down) lays these atomic list elements out exactly like
  code `1`. Under the defaults (`0` = do not wrap) the clauses stay on one
  line, preserving today's output byte-for-byte, and with `PREFER_PARAMETERS_WRAP`
  an overflowing tail call's arguments wrap before its method-call chain
  breaks.
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
- Annotation placement on declarations follows the `*_ANNOTATION_WRAP` codes
  (`METHOD_ANNOTATION_WRAP`, `CLASS_ANNOTATION_WRAP`, `FIELD_ANNOTATION_WRAP`
  default to wrap always — one annotation per line; `PARAMETER_ANNOTATION_WRAP`
  and `VARIABLE_ANNOTATION_WRAP` default to do not wrap — inline `@A @B type`):
  code `0` joins all modifiers inline, code `2` places each annotation on its
  own line, and codes `1` / `5` keep the inline form unless the composed first
  line overflows the margin (then one annotation per line). A lone annotation
  can be kept inline under a wrap-always placement with
  `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION` (declarations, locals) and
  `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION_IN_PARAMETER` (parameters). Parameter
  annotations render inline unless the parameter list wraps, and a wrapped
  list breaks an annotated parameter so its annotations sit on their own lines
  above the type / name. Enum constant annotations follow
  `ENUM_FIELD_ANNOTATION_WRAP` (default do not wrap; the constant's remainder
  is preserved verbatim). Wrapped annotation argument lists honour
  `NEW_LINE_AFTER_LPAREN_IN_ANNOTATION` (first argument stays on the `(` line
  when off), `RPAREN_ON_NEW_LINE_IN_ANNOTATION` (`)` attaches to the last
  argument when off), `ALIGN_MULTILINE_ANNOTATION_PARAMETERS` (align under the
  first argument when on) and `SPACE_AROUND_ANNOTATION_EQ` (`key = value` when
  on, `key=value` off). Layout / whitespace only; the default style emits
  today's one-per-line shape for methods / classes / fields and the IntelliJ
  default reshaped expanded annotation argument lists (first argument on the
  `(` line, `)` attached).
- Wrapped binary expressions break at their top-level operators at the
  continuation indent (`BINARY_OPERATION_WRAP`): by default the operator ends
  the preceding line (`alpha() +`), and `BINARY_OPERATION_SIGN_ON_NEXT_LINE`
  moves it to the start of the continuation line (`+ beta()`). `5` (chop
  down) also breaks a nested binary operand whose own line overflows.
- Ternary expressions wrap per `TERNARY_OPERATION_WRAP` (codes `0` / `1` /
  `2` / `5`): the flat form stays until the expression overflows (or always
  under code `2`), then it breaks at `?` / `:` with the signs at the end of
  the preceding line by default or at the start of the continuation lines
  with `TERNARY_OPERATION_SIGNS_ON_NEXT_LINE`; code `5` also recurses into a
  nested ternary side whose own line overflows. `assert` statements wrap per
  `ASSERT_STATEMENT_WRAP` at the expression and after the `:`, with
  `ASSERT_STATEMENT_COLON_ON_NEXT_LINE` moving the `:` to the next line. `for`
  headers wrap per `FOR_STATEMENT_WRAP` — the classic header breaks at its
  semicolons, the enhanced header at its `:` — with
  `FOR_STATEMENT_LPAREN/RPAREN_ON_NEXT_LINE` putting the parens on their own
  lines. Array initializers wrap per `ARRAY_INITIALIZER_WRAP` one element per
  line, with `ARRAY_INITIALIZER_LBRACE/RBRACE_ON_NEXT_LINE` placing the braces
  on their own lines (by default `{` ends the preceding line and `}` ends the
  last element's line). All of these wrap codes share the binary semantics:
  `0` do not wrap, `1` wrap if long, `2` wrap always, `5` chop down if long.
  A wrapped parenthesized expression's `(` / `)` move to their own lines with
  `PARENTHESES_EXPRESSION_LPAREN/RPAREN_WRAP`, a wrapped assignment's operator
  moves to the next line with `PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE`, a wrapped
  chain's first link breaks after the receiver with
  `WRAP_FIRST_METHOD_IN_CALL_CHAIN`, its `;` moves to its own line with
  `WRAP_SEMICOLON_AFTER_CALL_CHAIN`, and `MODIFIER_LIST_WRAP` breaks a
  declaration after its modifier / annotation list. Layout / whitespace only;
  with none of these set, output matches today's one-line layouts.
- With `USE_TAB_CHARACTER`, indentation is emitted as tab characters using a
  tab-stop model: each full `TAB_SIZE` of indentation width becomes one tab,
  and any remainder becomes spaces — so with `INDENT_SIZE == TAB_SIZE` each
  level is exactly one tab, while continuation indents that are not a whole
  number of tabs (e.g. `TAB_SIZE` 4 with a continuation indent of 6) emit a
  tab plus trailing spaces. `SMART_TABS` restricts tab characters to
  indentation that lands exactly on a tab stop: an indent whose width is not
  a whole number of tabs is emitted as pure spaces (alignment and
  off-stop continuation indents stay space-based). Alignment that needs exact
  columns (e.g. record components aligned under the opening paren) stays
  space-based. Margin and wrap decisions always use logical columns — a tab
  counts as `TAB_SIZE` — so a tab scheme wraps exactly where the equivalent
  space scheme does. Without `USE_TAB_CHARACTER`, output is space-indented as
  before.
- The remaining indentation refinements apply on top: `LABEL_INDENT_SIZE`
  shifts `label:` lines from the statement indent (relative, the default) or,
  with `LABEL_INDENT_ABSOLUTE`, pins them to the width from the left margin
  regardless of nesting; `KEEP_INDENTS_ON_EMPTY_LINES` keeps the block's
  inner indent on preserved blank lines; `USE_RELATIVE_INDENTS` (with
  `USE_TAB_CHARACTER`) measures continuation indents from the construct's
  own indent level instead of adding the full continuation offset to the
  level columns; and the five per-construct continuations
  `DECLARATION_PARAMETER_INDENT`, `GENERIC_TYPE_PARAMETER_INDENT`,
  `CALL_PARAMETER_INDENT`, `CHAINED_CALL_INDENT` and `ARRAY_ELEMENT_INDENT`
  override the continuation indent of their construct kind only (an explicit
  width replaces the construct's continuation; `-1` = inherit, the built-in
  default, keeps today's layout). `GENERIC_TYPE_PARAMETER_INDENT` is parsed
  and round-trips but is otherwise inert today: generic parameter lists
  always render flat and never wrap. `DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS`
  places the members of a top-level class at the class declaration indent
  (nested classes keep the normal one-level indent). All of these are layout
  / whitespace-only; absent options keep the IntelliJ built-in defaults and
  today's byte-identical output.
- The align-when-multiline family replaces the continuation indent of a wrapped
  construct's continuation lines with spaces to the first element's column,
  exactly like the record-header alignment (`ALIGN_MULTILINE_RECORDS` aligns
  wrapped record components under the opening paren): with the toggle on, the
  wrapped parameter / argument / resource / `throws` / `extends`-`implements`
  list lines pad to the first element's column (the first element stays on the
  header line after `(` / the keyword); the parts of a wrapped `for` header pad
  to its first slot; wrapped binary / ternary / parenthesized-expression
  continuation lines pad to the first operand / condition / `(`; a wrapped
  chained call's link lines pad to the first link's dot; and a wrapped
  assignment's right-hand side pads to the column right after the operator.
  The columnar options (`ALIGN_GROUP_FIELD_DECLARATIONS`,
  `ALIGN_CONSECUTIVE_VARIABLE_DECLARATIONS`, `ALIGN_CONSECUTIVE_ASSIGNMENTS`,
  `ALIGN_SUBSEQUENT_SIMPLE_METHODS`) instead pad runs of output-adjacent class
  members / block statements — fields, one-line methods, local variable
  declarations and assignment statements with no blank line and no comment
  between them — so the declared names / method names / operators share one
  column. Three of the wrapped-list options default on
  (`ALIGN_MULTILINE_PARAMETERS`, `ALIGN_MULTILINE_RESOURCES`, `ALIGN_MULTILINE_FOR`);
  the rest default off. Alignment is space-based like the record model, so the
  option's output is byte-stable under `USE_TAB_CHARACTER` schemes too.
- `switch` statements are laid out with `case` / `default` labels indented one
  level and their statements a further level; colon and arrow (`case x ->`)
  forms are preserved. The layout follows the scheme's case options:
  `INDENT_CASE_FROM_SWITCH` (default on) puts the labels one level below the
  `switch` — off, they sit at the `switch` indent; `CASE_STATEMENT_ON_NEW_LINE`
  (default on) starts the statement after a label on a new line — off, the
  group's first single-line statement is joined onto the label's line; and
  `INDENT_BREAK_FROM_CASE` (default on) keeps `break` / `continue` / `return`
  one level from the label — off, they line up with the label.
- A switch expression used as a value (assignment, return, argument) stays on
  one line when it fits the margin and falls back to the same multi-line
  layout otherwise, per `SWITCH_EXPRESSIONS_WRAP` (default wrap if long, code
  `1`): code `0` keeps the one-line form whenever one exists, code `2` always
  uses the multi-line layout, and code `5` additionally breaks an overflowing
  nested switch expression in the body.
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
  whitespace is produced — and the empty-block and flat-context one-line
  braces (argument lambdas, one-line switches) keep their pinned layout. The
  change is whitespace-only (R5) and
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
- Comments follow the scheme's comment layout options: a comment whose source
  starts in the first column stays there under `KEEP_FIRST_COLUMN_COMMENT`
  (default on), and `LINE_COMMENT_AT_FIRST_COLUMN` /
  `BLOCK_COMMENT_AT_FIRST_COLUMN` (both default on) pin `//` and `/* */`
  comments to the first column — matching IntelliJ, the built-in defaults
  place comments at column 1 rather than the code indent. With those toggles
  off, comments are emitted at the indentation of the surrounding code.
  `LINE_COMMENT_ADD_SPACE_ON_REFORMAT` inserts the missing space after `//`
  of ordinary line comments; `//noinspection` suppression comments are
  governed separately by `LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION` (a space
  there would break the suppression). Under `WRAP_COMMENTS`, a single-line
  comment longer than the right margin is wrapped at word boundaries with the
  continuation lines repeating the comment's column prefix (`//` for line
  comments, aligned ` * ` text for block comments); multi-line block comments
  keep their source text verbatim. Comment text is never invented — only the
  indentation, the optional space after `//` and the line breaks change (R5),
  and re-formatting the output reproduces the layout (R6).

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

A GitHub Actions pipeline ([`.github/workflows/ci.yml`](.github/workflows/ci.yml))
verifies every push to `main` and every pull request: `cargo fmt --all -- --check`
and, on an `ubuntu` / `macos` / `windows` × stable Rust matrix,
`cargo clippy --workspace --lib --bins --tests -- -D warnings` and
`cargo test --workspace`. Benches are not built on CI.

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
