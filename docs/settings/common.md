---
type: Reference
title: Common Java formatting settings
description: Every common IntelliJ code style option as applied to Java — the <codeStyleSettings language="JAVA"> block — with defaults, encodings, and java-formatter support status.
tags: [java, formatter, settings, reference]
status: active
---

# Common Java formatting settings

The `<codeStyleSettings language="JAVA">` block of an IntelliJ code scheme
holds the _common_ code style options — the same option set IntelliJ exposes
for other languages, as applied to Java. They live in IntelliJ's
`CommonCodeStyleSettings` class. The nested `<indentOptions>` child holds the
indentation settings.

Support marks follow the legend on the [section index](index.md#support-legend)
(✅ implemented, ◐ parsed but not fully applied, ❌ unimplemented formatter
option, n/a not a formatter concern).

## Root-level options

Options that appear as direct children of `<code_scheme>` (before any
`<JavaCodeStyleSettings>` / `<codeStyleSettings>` block). java-formatter reads
`SOFT_MARGINS`, `RIGHT_MARGIN` and `LINE_SEPARATOR` here; the rest are global
scheme options.

| Option                         | Type   | Default          | Values                                          | Effect                                                                                                                     | Support                                    |
| ------------------------------ | ------ | ---------------- | ----------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| `SOFT_MARGINS`                 | string | empty            | comma-separated integers, e.g. `100,120`        | Right margin(s) used for line-length decisions; java-formatter reads this for its line-length limit.                       | ✅ (first value used as the single margin; wins over `RIGHT_MARGIN` when both are set) |
| `RIGHT_MARGIN`                 | int    | `120`            | `≥ 0`                                           | Hard right margin; drives the line-length decisions when `SOFT_MARGINS` is absent.                                         | ✅                                        |
| `LINE_SEPARATOR`               | string | system default   | `&#10;` (LF), `&#13;&#10;` (CRLF), `&#13;` (CR) | Line separator emitted at every line end, including the final newline.                                                     | ✅                                        |
| `FORMATTER_TAGS_ENABLED`       | bool   | `true`           | `true` / `false`                                | Honour `// @formatter:off` / `// @formatter:on` comment tags.                                                              | n/a                                        |
| `FORMATTER_TAGS_ACCEPT_REGEXP` | bool   | `false`          | `true` / `false`                                | Treat the formatter tags as regular expressions.                                                                           | n/a                                        |
| `FORMATTER_ON_TAG`             | string | `@formatter:on`  | any                                             | Text of the "formatter on" tag.                                                                                            | n/a                                        |
| `FORMATTER_OFF_TAG`            | string | `@formatter:off` | any                                             | Text of the "formatter off" tag.                                                                                           | n/a                                        |
| `AUTODETECT_INDENTS`           | bool   | `true`           | `true` / `false`                                | Detect indentation from file contents instead of the configured indent.                                                    | n/a                                        |
| `OTHER_INDENT_OPTIONS`         | block  | —                | nested `<option>` list                          | Legacy global indentation block (pre-2018); the same keys as `<indentOptions>`.                                            | n/a                                        |

## General & comments

Options controlling comment layout and line-break retention.

| Option                                  | Type | Default | Values           | Effect                                                                         | Support |
| --------------------------------------- | ---- | ------- | ---------------- | ------------------------------------------------------------------------------ | ------- |
| `LINE_COMMENT_AT_FIRST_COLUMN`          | bool | `true`  | `true` / `false` | Keep `//` line comments in the first column (no indent).                       | ✅      |
| `BLOCK_COMMENT_AT_FIRST_COLUMN`         | bool | `true`  | `true` / `false` | Keep `/* */` block comments in the first column.                               | ✅      |
| `LINE_COMMENT_ADD_SPACE`                | bool | `false` | `true` / `false` | Insert a space after `//` when commenting / uncommenting lines.                | n/a     |
| `BLOCK_COMMENT_ADD_SPACE`               | bool | `false` | `true` / `false` | Insert a space after `/*` and before `*/`.                                     | n/a     |
| `LINE_COMMENT_ADD_SPACE_ON_REFORMAT`    | bool | `false` | `true` / `false` | Add the space after `//` on reformat.                                          | ✅      |
| `LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION` | bool | `false` | `true` / `false` | Add the space inside `// noinspection` suppression comments.                   | ✅      |
| `DOCUMENTATION_LINE_COMMENT_PREFERRED`  | bool | `false` | `true` / `false` | Prefer documentation line comments where the language supports them.           | n/a     |
| `KEEP_LINE_BREAKS`                      | bool | `true`  | `true` / `false` | Keep existing line breaks in the code: a construct whose source spans rows keeps its canonical wrapped layout.               | ✅      |
| `KEEP_FIRST_COLUMN_COMMENT`             | bool | `true`  | `true` / `false` | Keep comments that start in the first column at the first column.              | ✅      |
| `KEEP_CONTROL_STATEMENT_IN_ONE_LINE`    | bool | `true`  | `true` / `false` | Keep `if (…) …;` / `while (…) …;` / `for (…) …;` (without braces) on one line. | ✅ (source-driven: a same-line body stays, an own-line body keeps its line) |
| `WRAP_COMMENTS`                         | bool | `false` | `true` / `false` | Wrap long comments to the right margin.                                        | ✅      |
| `WRAP_LONG_LINES`                       | bool | `false` | `true` / `false` | Wrap lines longer than the right margin (hard wrap) at the last whitespace boundary; literals and comments are never split. | ✅      |

## Blank lines

`KEEP_BLANK_LINES_*` cap how many existing blank lines are preserved;
`BLANK_LINES_*` are minimums inserted around constructs.

| Option                                                    | Type | Default | Values  | Effect                                                                     | Support |
| --------------------------------------------------------- | ---- | ------- | ------- | -------------------------------------------------------------------------- | ------- |
| `KEEP_BLANK_LINES_IN_CODE`                                | int  | `2`     | `0`–`n` | Max blank lines kept inside code (statement level).                        | ✅      |
| `KEEP_BLANK_LINES_IN_DECLARATIONS`                        | int  | `2`     | `0`–`n` | Max blank lines kept between declarations.                                 | ✅      |
| `KEEP_BLANK_LINES_BETWEEN_PACKAGE_DECLARATION_AND_HEADER` | int  | `2`     | `0`–`n` | Max blank lines between the package declaration and a file header comment. | ✅      |
| `KEEP_BLANK_LINES_BEFORE_RBRACE`                          | int  | `2`     | `0`–`n` | Max blank lines kept before a closing `}`.                                 | ✅      |
| `BLANK_LINES_BEFORE_PACKAGE`                              | int  | `0`     | `0`–`n` | Min blank lines before the package declaration.                            | ✅      |
| `BLANK_LINES_AFTER_PACKAGE`                               | int  | `1`     | `0`–`n` | Min blank lines after the package declaration.                             | ✅      |
| `BLANK_LINES_BEFORE_IMPORTS`                              | int  | `1`     | `0`–`n` | Min blank lines before the import section.                                 | ✅      |
| `BLANK_LINES_AFTER_IMPORTS`                               | int  | `1`     | `0`–`n` | Min blank lines after the import section.                                  | ✅      |
| `BLANK_LINES_AROUND_CLASS`                                | int  | `1`     | `0`–`n` | Min blank lines around class / interface declarations.                     | ✅      |
| `BLANK_LINES_AROUND_FIELD`                                | int  | `0`     | `0`–`n` | Min blank lines around fields.                                             | ✅      |
| `BLANK_LINES_AROUND_METHOD`                               | int  | `1`     | `0`–`n` | Min blank lines around methods.                                            | ✅      |
| `BLANK_LINES_BEFORE_METHOD_BODY`                          | int  | `0`     | `0`–`n` | Min blank lines before a method body.                                      | ✅      |
| `BLANK_LINES_AROUND_FIELD_IN_INTERFACE`                   | int  | `0`     | `0`–`n` | Min blank lines around fields declared in interfaces.                      | ✅      |
| `BLANK_LINES_AROUND_METHOD_IN_INTERFACE`                  | int  | `1`     | `0`–`n` | Min blank lines around methods declared in interfaces.                     | ✅      |
| `BLANK_LINES_AFTER_CLASS_HEADER`                          | int  | `0`     | `0`–`n` | Min blank lines after the class header / before the first member.          | ✅      |
| `BLANK_LINES_AFTER_ANONYMOUS_CLASS_HEADER`                | int  | `0`     | `0`–`n` | Min blank lines after an anonymous class header.                           | ✅      |
| `BLANK_LINES_BEFORE_CLASS_END`                            | int  | `0`     | `0`–`n` | Min blank lines before the class closing brace.                            | ✅      |

## Braces & indentation

| Option                                  | Type | Default           | Values                              | Effect                                                                   | Support                         |
| --------------------------------------- | ---- | ----------------- | ----------------------------------- | ------------------------------------------------------------------------ | ------------------------------- |
| `BRACE_STYLE`                           | int  | `1` (end of line) | [brace codes](index.md#brace-codes) | Brace placement for statements / other blocks not covered below.         | ✅ (as the "other" brace style) |
| `CLASS_BRACE_STYLE`                     | int  | `1` (end of line) | [brace codes](index.md#brace-codes) | Brace placement for class / interface / enum / record bodies.            | ✅                              |
| `METHOD_BRACE_STYLE`                    | int  | `1` (end of line) | [brace codes](index.md#brace-codes) | Brace placement for method, constructor, and compact-constructor bodies. | ✅                              |
| `LAMBDA_BRACE_STYLE`                    | int  | `1` (end of line) | [brace codes](index.md#brace-codes) | Brace placement for lambda bodies.                                       | ✅ (block-bodied lambdas)       |
| `DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS` | bool | `false`           | `true` / `false`                    | Do not indent members of a top-level class.                              | ❌                              |
| `ELSE_ON_NEW_LINE`                      | bool | `false`           | `true` / `false`                    | `} else {` → `}\nelse {`.                                                | ✅ (if / else-if chains)        |
| `WHILE_ON_NEW_LINE`                     | bool | `false`           | `true` / `false`                    | `} while (…)` → `}\nwhile (…)`.                                          | ✅ (do-while tail)              |
| `CATCH_ON_NEW_LINE`                     | bool | `false`           | `true` / `false`                    | `} catch (…)` → `}\ncatch (…)`.                                          | ✅                              |
| `FINALLY_ON_NEW_LINE`                   | bool | `false`           | `true` / `false`                    | `} finally` → `}\nfinally`.                                              | ✅                              |
| `SPECIAL_ELSE_IF_TREATMENT`             | bool | `true`            | `true` / `false`                    | Keep `else if` as one construct instead of nested `else { if … }`.       | ✅                              |
| `INDENT_CASE_FROM_SWITCH`               | bool | `true`            | `true` / `false`                    | Indent `case` labels from the `switch`.                                  | ✅                              |
| `CASE_STATEMENT_ON_NEW_LINE`            | bool | `true`            | `true` / `false`                    | Put the statement after a `case` label on a new line.                    | ✅                              |
| `INDENT_BREAK_FROM_CASE`                | bool | `true`            | `true` / `false`                    | Indent `break` / `continue` / `return` one level from the `case` label.  | ✅                              |

## Alignment

"Align when multiline" options: when a construct is wrapped, subsequent lines
align under the first element instead of using the continuation indent.

| Option                                         | Type | Default | Effect                                                                       | Support |
| ---------------------------------------------- | ---- | ------- | ---------------------------------------------------------------------------- | ------- |
| `ALIGN_MULTILINE_PARAMETERS`                   | bool | `true`  | Align wrapped method parameter declarations.                                 | ✅      |
| `ALIGN_MULTILINE_PARAMETERS_IN_CALLS`          | bool | `false` | Align wrapped method-call arguments.                                         | ✅      |
| `ALIGN_MULTILINE_RESOURCES`                    | bool | `true`  | Align wrapped try-with-resources clauses.                                    | ✅      |
| `ALIGN_MULTILINE_FOR`                          | bool | `true`  | Align wrapped `for` header parts.                                            | ✅      |
| `ALIGN_MULTILINE_BINARY_OPERATION`             | bool | `false` | Align wrapped binary expression operands.                                    | ✅      |
| `ALIGN_MULTILINE_ASSIGNMENT`                   | bool | `false` | Align wrapped assignment right-hand sides.                                   | ✅      |
| `ALIGN_MULTILINE_TERNARY_OPERATION`            | bool | `false` | Align wrapped ternary expression operands.                                   | ✅      |
| `ALIGN_MULTILINE_THROWS_LIST`                  | bool | `false` | Align wrapped `throws` list entries.                                         | ✅      |
| `ALIGN_THROWS_KEYWORD`                         | bool | `false` | Align the `throws` keyword itself.                                           | ✅      |
| `ALIGN_MULTILINE_EXTENDS_LIST`                 | bool | `false` | Align wrapped `extends` / `implements` list entries.                         | ✅      |
| `ALIGN_MULTILINE_METHOD_BRACKETS`              | bool | `false` | Align method declaration parentheses (`(` / `)`) when wrapped.               | ✅      |
| `ALIGN_MULTILINE_PARENTHESIZED_EXPRESSION`     | bool | `false` | Align wrapped parenthesized expressions.                                     | ✅      |
| `ALIGN_MULTILINE_ARRAY_INITIALIZER_EXPRESSION` | bool | `false` | Align wrapped array initializer entries.                                     | ✅      |
| `ALIGN_MULTILINE_CHAINED_METHODS`              | bool | `false` | Align the dots of a wrapped chained call under the first call.               | ✅      |
| `ALIGN_GROUP_FIELD_DECLARATIONS`               | bool | `false` | Align consecutive field / variable declarations and initialisers in columns. | ✅      |
| `ALIGN_CONSECUTIVE_VARIABLE_DECLARATIONS`      | bool | `false` | Align consecutive local variable declarations.                               | ✅      |
| `ALIGN_CONSECUTIVE_ASSIGNMENTS`                | bool | `false` | Align consecutive assignment statements.                                     | ✅      |
| `ALIGN_SUBSEQUENT_SIMPLE_METHODS`              | bool | `false` | Align consecutive one-line methods.                                          | ✅      |

## Spaces

### Around operators

| Option                                  | Type | Default | Operators                                  | Support                        |
| --------------------------------------- | ---- | ------- | ------------------------------------------ | ------------------------------ |
| `SPACE_AROUND_ASSIGNMENT_OPERATORS`     | bool | `true`  | `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, ` | =`, `^=`, `<<=`, `>>=`, `>>>=` | ✅  |
| `SPACE_AROUND_LOGICAL_OPERATORS`        | bool | `true`  | `&&`, `\|\|`                               | ✅                             |
| `SPACE_AROUND_EQUALITY_OPERATORS`       | bool | `true`  | `==`, `!=`                                 | ✅                             |
| `SPACE_AROUND_RELATIONAL_OPERATORS`     | bool | `true`  | `<`, `>`, `<=`, `>=`                       | ✅                             |
| `SPACE_AROUND_BITWISE_OPERATORS`        | bool | `true`  | `&`, `\|`, `^`                             | ✅                             |
| `SPACE_AROUND_ADDITIVE_OPERATORS`       | bool | `true`  | `+`, `-`                                   | ✅                             |
| `SPACE_AROUND_MULTIPLICATIVE_OPERATORS` | bool | `true`  | `*`, `/`, `%`                              | ✅                             |
| `SPACE_AROUND_SHIFT_OPERATORS`          | bool | `true`  | `<<`, `>>`, `>>>`                          | ✅                             |
| `SPACE_AROUND_UNARY_OPERATOR`           | bool | `false` | `!`, `~`, unary `+` / `-`, `++`, `--`      | ✅                             |
| `SPACE_AROUND_LAMBDA_ARROW`             | bool | `true`  | `->`                                       | ✅                             |
| `SPACE_AROUND_METHOD_REF_DBL_COLON`     | bool | `false` | `::`                                       | ✅                             |
| `SPACE_AFTER_TYPE_CAST`                 | bool | `true`  | `(Type) expr`                              | ✅                             |

### After / before separators

| Option                                | Type | Default | Effect                                                                   | Support |
| ------------------------------------- | ---- | ------- | ------------------------------------------------------------------------ | ------- |
| `SPACE_AFTER_COMMA`                   | bool | `true`  | Space after `,` (declarations, calls, arrays).                           | ✅      |
| `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS` | bool | `true`  | Space after `,` in generic type arguments.                               | ✅      |
| `SPACE_BEFORE_COMMA`                  | bool | `false` | Space before `,`.                                                        | ✅      |
| `SPACE_AFTER_SEMICOLON`               | bool | `true`  | Space after `;` inside a `for` header.                                   | ✅      |
| `SPACE_BEFORE_SEMICOLON`              | bool | `false` | Space before `;` inside a `for` header.                                  | ✅      |
| `SPACE_BEFORE_QUEST`                  | bool | `true`  | Space before `?` in a ternary expression.                                | ✅      |
| `SPACE_AFTER_QUEST`                   | bool | `true`  | Space after `?` in a ternary expression.                                 | ✅      |
| `SPACE_BEFORE_COLON`                  | bool | `true`  | Space before `:` in a ternary expression.                                | ✅      |
| `SPACE_AFTER_COLON`                   | bool | `true`  | Space after `:` (ternary, `for-each`).                                   | ✅      |
| `SPACE_BEFORE_TYPE_PARAMETER_LIST`    | bool | `false` | Space between a class / method name and its type-parameter list (`<…>`). | ✅      |

### Within parentheses, brackets, braces

| Option                                        | Type | Default | Applies to                    | Support |
| --------------------------------------------- | ---- | ------- | ----------------------------- | ------- |
| `SPACE_WITHIN_PARENTHESES`                    | bool | `false` | Any parentheses `( expr )`.   | ✅      |
| `SPACE_WITHIN_METHOD_CALL_PARENTHESES`        | bool | `false` | `f( args )`.                  | ✅      |
| `SPACE_WITHIN_EMPTY_METHOD_CALL_PARENTHESES`  | bool | `false` | `f( )` vs `f()`.              | ✅      |
| `SPACE_WITHIN_METHOD_PARENTHESES`             | bool | `false` | `void f( params )`.           | ✅      |
| `SPACE_WITHIN_EMPTY_METHOD_PARENTHESES`       | bool | `false` | `void f( )` vs `void f()`.    | ✅      |
| `SPACE_WITHIN_IF_PARENTHESES`                 | bool | `false` | `if( cond )`.                 | ✅      |
| `SPACE_WITHIN_WHILE_PARENTHESES`              | bool | `false` | `while( cond )`.              | ✅      |
| `SPACE_WITHIN_FOR_PARENTHESES`                | bool | `false` | `for( … )`.                   | ✅      |
| `SPACE_WITHIN_TRY_PARENTHESES`                | bool | `false` | `try( resource )`.            | ✅      |
| `SPACE_WITHIN_CATCH_PARENTHESES`              | bool | `false` | `catch( exc )`.               | ✅      |
| `SPACE_WITHIN_SWITCH_PARENTHESES`             | bool | `false` | `switch( expr )`.             | ✅      |
| `SPACE_WITHIN_SYNCHRONIZED_PARENTHESES`       | bool | `false` | `synchronized( expr )`.       | ✅      |
| `SPACE_WITHIN_CAST_PARENTHESES`               | bool | `false` | `( Type ) expr`.              | ✅      |
| `SPACE_WITHIN_BRACKETS`                       | bool | `false` | `[ expr ]` in array indexing. | ✅      |
| `SPACE_WITHIN_BRACES`                         | bool | `false` | `{ … }` code blocks.          | ✅      |
| `SPACE_WITHIN_ARRAY_INITIALIZER_BRACES`       | bool | `false` | `{ 1, 3, 5 }`.                | ✅      |
| `SPACE_WITHIN_EMPTY_ARRAY_INITIALIZER_BRACES` | bool | `false` | `{ }` vs `{}`.                | ✅      |
| `SPACE_WITHIN_ANNOTATION_PARENTHESES`         | bool | `false` | `@Anno( args )`.              | ✅      |

### Before parentheses / braces / keywords

| Option                                             | Type | Default | Effect                                               | Support |
| -------------------------------------------------- | ---- | ------- | ---------------------------------------------------- | ------- |
| `SPACE_BEFORE_METHOD_CALL_PARENTHESES`             | bool | `false` | `f (x)` vs `f(x)`.                                   | ✅      |
| `SPACE_BEFORE_METHOD_PARENTHESES`                  | bool | `false` | `void f (int p)` vs `void f(int p)`.                 | ✅      |
| `SPACE_BEFORE_IF_PARENTHESES`                      | bool | `true`  | `if (...)`.                                          | ✅      |
| `SPACE_BEFORE_WHILE_PARENTHESES`                   | bool | `true`  | `while (...)`.                                       | ✅      |
| `SPACE_BEFORE_FOR_PARENTHESES`                     | bool | `true`  | `for (...)`.                                         | ✅      |
| `SPACE_BEFORE_TRY_PARENTHESES`                     | bool | `true`  | `try (...)`.                                         | ✅      |
| `SPACE_BEFORE_CATCH_PARENTHESES`                   | bool | `true`  | `catch (...)`.                                       | ✅      |
| `SPACE_BEFORE_SWITCH_PARENTHESES`                  | bool | `true`  | `switch (...)`.                                      | ✅      |
| `SPACE_BEFORE_SYNCHRONIZED_PARENTHESES`            | bool | `true`  | `synchronized (...)`.                                | ✅      |
| `SPACE_BEFORE_ANOTATION_PARAMETER_LIST`            | bool | `false` | `@Anno (...)`. (Name spelled as in IntelliJ source.) | ✅      |
| `SPACE_BEFORE_CLASS_LBRACE`                        | bool | `true`  | `class A {`.                                         | ✅      |
| `SPACE_BEFORE_METHOD_LBRACE`                       | bool | `true`  | `void f() {`.                                        | ✅      |
| `SPACE_BEFORE_IF_LBRACE`                           | bool | `true`  | `if (…) {`.                                          | ✅      |
| `SPACE_BEFORE_ELSE_LBRACE`                         | bool | `true`  | `else {`.                                            | ✅      |
| `SPACE_BEFORE_WHILE_LBRACE`                        | bool | `true`  | `while (…) {`.                                       | ✅      |
| `SPACE_BEFORE_FOR_LBRACE`                          | bool | `true`  | `for (…) {`.                                         | ✅      |
| `SPACE_BEFORE_DO_LBRACE`                           | bool | `true`  | `do {`.                                              | ✅      |
| `SPACE_BEFORE_SWITCH_LBRACE`                       | bool | `true`  | `switch (…) {`.                                      | ✅      |
| `SPACE_BEFORE_TRY_LBRACE`                          | bool | `true`  | `try {`.                                             | ✅      |
| `SPACE_BEFORE_CATCH_LBRACE`                        | bool | `true`  | `catch (…) {`.                                       | ✅      |
| `SPACE_BEFORE_FINALLY_LBRACE`                      | bool | `true`  | `finally {`.                                         | ✅      |
| `SPACE_BEFORE_SYNCHRONIZED_LBRACE`                 | bool | `true`  | `synchronized (…) {`.                                | ✅      |
| `SPACE_BEFORE_ARRAY_INITIALIZER_LBRACE`            | bool | `false` | `new int[] {`.                                       | ✅      |
| `SPACE_BEFORE_ANNOTATION_ARRAY_INITIALIZER_LBRACE` | bool | `false` | `@SuppressWarnings( {…)`.                            | ✅      |
| `SPACE_BEFORE_ELSE_KEYWORD`                        | bool | `true`  | `} else {`.                                          | ✅      |
| `SPACE_BEFORE_WHILE_KEYWORD`                       | bool | `true`  | `} while (…)`.                                       | ✅      |
| `SPACE_BEFORE_CATCH_KEYWORD`                       | bool | `true`  | `} catch (…)`.                                       | ✅      |
| `SPACE_BEFORE_FINALLY_KEYWORD`                     | bool | `true`  | `} finally`.                                         | ✅      |

## Wrapping & braces

All `*_WRAP` options use the [wrap codes](index.md#wrap-codes).

### Parameters and arguments

| Option                                  | Type | Default | Effect                                                         | Support |
| --------------------------------------- | ---- | ------- | -------------------------------------------------------------- | ------- |
| `CALL_PARAMETERS_WRAP`                  | int  | `0`     | Wrapping of method-call argument lists.                        | ✅      |
| `CALL_PARAMETERS_LPAREN_ON_NEXT_LINE`   | bool | `false` | Put `(` of a wrapped call on its own line.                     | ✅      |
| `CALL_PARAMETERS_RPAREN_ON_NEXT_LINE`   | bool | `false` | Put `)` of a wrapped call on its own line.                     | ✅      |
| `PREFER_PARAMETERS_WRAP`                | bool | `false` | Prefer wrapping the parameter list over other wrapping points. | ✅      |
| `METHOD_PARAMETERS_WRAP`                | int  | `0`     | Wrapping of method / constructor parameter lists.              | ✅      |
| `METHOD_PARAMETERS_LPAREN_ON_NEXT_LINE` | bool | `false` | Put `(` of a wrapped declaration on its own line.              | ✅      |
| `METHOD_PARAMETERS_RPAREN_ON_NEXT_LINE` | bool | `false` | Put `)` of a wrapped declaration on its own line.              | ✅      |
| `RESOURCE_LIST_WRAP`                    | int  | `0`     | Wrapping of try-with-resources clauses.                        | ✅      |
| `RESOURCE_LIST_LPAREN_ON_NEXT_LINE`     | bool | `false` | `(` of a wrapped resource list on its own line.                | ✅      |
| `RESOURCE_LIST_RPAREN_ON_NEXT_LINE`     | bool | `false` | `)` of a wrapped resource list on its own line.                | ✅      |

### Extends / implements / throws

| Option                 | Type | Default | Effect                                                   | Support |
| ---------------------- | ---- | ------- | -------------------------------------------------------- | ------- |
| `EXTENDS_LIST_WRAP`    | int  | `0`     | Wrapping of `extends` / `implements` lists.              | ✅      |
| `EXTENDS_KEYWORD_WRAP` | bool | `false` | Put the `extends` / `implements` keyword on its own line when wrapping. | ✅      |
| `THROWS_LIST_WRAP`     | int  | `0`     | Wrapping of `throws` lists.                              | ✅      |
| `THROWS_KEYWORD_WRAP`  | bool | `false` | Put the `throws` keyword on its own line when wrapping.  | ✅      |

### Expressions and statements

| Option                                  | Type | Default | Effect                                                             | Support |
| --------------------------------------- | ---- | ------- | ------------------------------------------------------------------ | ------- |
| `METHOD_CALL_CHAIN_WRAP`                | int  | `0`     | Wrapping of chained method calls.                                  | ✅      |
| `WRAP_FIRST_METHOD_IN_CALL_CHAIN`       | bool | `false` | Wrap after the first call in a chain as well.                      | ✅      |
| `PARENTHESES_EXPRESSION_LPAREN_WRAP`    | bool | `false` | `(` of a wrapped parenthesized expression on its own line.         | ✅      |
| `PARENTHESES_EXPRESSION_RPAREN_WRAP`    | bool | `false` | `)` of a wrapped parenthesized expression on its own line.         | ✅      |
| `BINARY_OPERATION_WRAP`                 | int  | `0`     | Wrapping of binary expressions at their operators.                 | ✅      |
| `BINARY_OPERATION_SIGN_ON_NEXT_LINE`    | bool | `false` | Put the operator at the start of the continuation line.            | ✅      |
| `TERNARY_OPERATION_WRAP`                | int  | `0`     | Wrapping of ternary (`?:`) expressions.                            | ✅      |
| `TERNARY_OPERATION_SIGNS_ON_NEXT_LINE`  | bool | `false` | Put `?` / `:` at the start of continuation lines.                  | ✅      |
| `ASSIGNMENT_WRAP`                       | int  | `0`     | Wrapping of assignments and variable / field initialisers.         | ✅      |
| `PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE`    | bool | `false` | Put the assignment operator at the start of the continuation line. | ✅      |
| `ASSERT_STATEMENT_WRAP`                 | int  | `0`     | Wrapping of `assert` statements.                                   | ✅      |
| `ASSERT_STATEMENT_COLON_ON_NEXT_LINE`   | bool | `false` | Put the `:` of an `assert` on the next line.                       | ✅      |
| `SWITCH_EXPRESSIONS_WRAP`               | int  | `1`     | Wrapping of `switch` expressions used as values.                   | ✅      |
| `FOR_STATEMENT_WRAP`                    | int  | `0`     | Wrapping of `for` headers.                                         | ✅      |
| `FOR_STATEMENT_LPAREN_ON_NEXT_LINE`     | bool | `false` | `(` of a wrapped `for` on its own line.                            | ✅      |
| `FOR_STATEMENT_RPAREN_ON_NEXT_LINE`     | bool | `false` | `)` of a wrapped `for` on its own line.                            | ✅      |
| `ARRAY_INITIALIZER_WRAP`                | int  | `0`     | Wrapping of array initializer lists.                               | ✅      |
| `ARRAY_INITIALIZER_LBRACE_ON_NEXT_LINE` | bool | `false` | `{` of a wrapped array initializer on its own line.                | ✅      |
| `ARRAY_INITIALIZER_RBRACE_ON_NEXT_LINE` | bool | `false` | `}` of a wrapped array initializer on its own line.                | ✅      |
| `MODIFIER_LIST_WRAP`                    | bool | `false` | Wrap after the modifier / annotation list of a declaration.        | ✅      |

### Keep in one line

| Option                                  | Type | Default | Effect                                                                              | Support |
| --------------------------------------- | ---- | ------- | ----------------------------------------------------------------------------------- | ------- |
| `KEEP_SIMPLE_BLOCKS_IN_ONE_LINE`        | bool | `false` | Keep single-statement blocks of `if` / `else` / `for` / `while` / `do` on one line. | ✅      |
| `KEEP_SIMPLE_METHODS_IN_ONE_LINE`       | bool | `false` | Keep single-statement method / constructor bodies on one line.                      | ✅      |
| `KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE`       | bool | `false` | Keep single-statement lambda bodies on one line.                                    | ✅      |
| `KEEP_SIMPLE_CLASSES_IN_ONE_LINE`       | bool | `false` | Keep simple class bodies on one line.                                               | ✅      |
| `KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE` | bool | `false` | Keep multiple expressions (e.g. in a `for` header) on one line.                     | ✅      |

## Force braces

| Option                | Type | Default | Values                                    | Effect                                   | Support |
| --------------------- | ---- | ------- | ----------------------------------------- | ---------------------------------------- | ------- |
| `IF_BRACE_FORCE`      | int  | `0`     | [force codes](index.md#force-brace-codes) | Force braces around `if` / `else` bodies.                     | ✅      |
| `FOR_BRACE_FORCE`     | int  | `0`     | [force codes](index.md#force-brace-codes) | Force braces around `for` / enhanced-`for` bodies.            | ✅      |
| `WHILE_BRACE_FORCE`   | int  | `0`     | [force codes](index.md#force-brace-codes) | Force braces around `while` bodies.                           | ✅      |
| `DOWHILE_BRACE_FORCE` | int  | `0`     | [force codes](index.md#force-brace-codes) | Force braces around `do … while` bodies.                      | ✅      |

## Annotations

The `*_ANNOTATION_WRAP` options use the [wrap codes](index.md#wrap-codes) and
control whether an annotation is placed on its own line before a declaration.

| Option                      | Type | Default           | Effect                                                | Support |
| --------------------------- | ---- | ----------------- | ----------------------------------------------------- | ------- |
| `METHOD_ANNOTATION_WRAP`    | int  | `2` (wrap always) | Put a method's annotations on separate lines.         | ✅      |
| `CLASS_ANNOTATION_WRAP`     | int  | `2` (wrap always) | Put a class's annotations on separate lines.          | ✅      |
| `FIELD_ANNOTATION_WRAP`     | int  | `2` (wrap always) | Put a field's annotations on separate lines.          | ✅      |
| `PARAMETER_ANNOTATION_WRAP` | int  | `0`               | Put a parameter's annotations on separate lines.      | ✅      |
| `VARIABLE_ANNOTATION_WRAP`  | int  | `0`               | Put a local variable's annotations on separate lines. | ✅      |

(Annotation _argument_ wrapping is `ANNOTATION_PARAMETER_WRAP`, which lives in
[`JavaCodeStyleSettings`](java.md#annotations).)

## Enums

| Option                | Type | Default | Effect                           | Support |
| --------------------- | ---- | ------- | -------------------------------- | ------- |
| `ENUM_CONSTANTS_WRAP` | int  | `0`     | Wrapping of enum constant lists. | ❌      |

## Builder method calls

| Option                         | Type   | Default | Effect                                                                            | Support |
| ------------------------------ | ------ | ------- | --------------------------------------------------------------------------------- | ------- |
| `BUILDER_METHODS`              | string | `""`    | Comma-separated method names treated as builder calls for wrapping / indentation. | ❌      |
| `KEEP_BUILDER_METHODS_INDENTS` | bool   | `false` | Keep indentation of builder-method chains.                                        | ❌      |

## Rearranger & typing

| Option                 | Type | Default | Values                            | Effect                                              | Support |
| ---------------------- | ---- | ------- | --------------------------------- | --------------------------------------------------- | ------- |
| `FORCE_REARRANGE_MODE` | int  | `0`     | `0` dialog, `1` always, `2` never | Whether to apply the arrangement rules on reformat. | n/a     |
| `WRAP_ON_TYPING`       | int  | `-1`    | `-1` default, `0` no, `1` yes     | Wrap lines while typing past the margin.            | n/a     |

## Indent options

The `<indentOptions>` child of `<codeStyleSettings language="JAVA">`:

| Option                          | Type | Default | Effect                                                                                                | Support                                                     |
| ------------------------------- | ---- | ------- | ----------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| `INDENT_SIZE`                   | int  | `4`     | Indentation width in spaces.                                                                          | ✅                                                          |
| `CONTINUATION_INDENT_SIZE`      | int  | `8`     | Continuation-line indent width in spaces.                                                             | ✅                                                          |
| `TAB_SIZE`                      | int  | `4`     | Width a tab is displayed / counted as.                                                                | ✅ (tab-stop output model — see [R13](../requirements.md))  |
| `USE_TAB_CHARACTER`             | bool | `false` | Indent with tab characters instead of spaces.                                                         | ✅ (tab indentation output — see [R13](../requirements.md)) |
| `SMART_TABS`                    | bool | `false` | Use tabs only where they align to tab stops.                                                          | ❌                                                          |
| `LABEL_INDENT_SIZE`             | int  | `0`     | Indent for `label:` statements.                                                                       | ❌                                                          |
| `LABEL_INDENT_ABSOLUTE`         | bool | `false` | Indent labels by `LABEL_INDENT_SIZE` regardless of nesting.                                           | ❌                                                          |
| `USE_RELATIVE_INDENTS`          | bool | `false` | Use relative indentation for continuation lines.                                                      | ❌                                                          |
| `KEEP_INDENTS_ON_EMPTY_LINES`   | bool | `false` | Keep the indent on empty lines.                                                                       | ❌                                                          |
| `DECLARATION_PARAMETER_INDENT`  | int  | `-1`    | Per-construct continuation indent for declaration parameters (`-1` = use `CONTINUATION_INDENT_SIZE`). | ❌                                                          |
| `GENERIC_TYPE_PARAMETER_INDENT` | int  | `-1`    | Per-construct continuation indent for generic type parameters.                                        | ❌                                                          |
| `CALL_PARAMETER_INDENT`         | int  | `-1`    | Per-construct continuation indent for call arguments.                                                 | ❌                                                          |
| `CHAINED_CALL_INDENT`           | int  | `-1`    | Per-construct continuation indent for chained calls.                                                  | ❌                                                          |
| `ARRAY_ELEMENT_INDENT`          | int  | `-1`    | Per-construct continuation indent for array elements.                                                 | ❌                                                          |
