---
type: Reference
title: Java-specific formatting settings
description: Every Java-only IntelliJ code style option — the <JavaCodeStyleSettings> block — with defaults, encodings, and java-formatter support status.
tags: [java, formatter, settings, reference]
status: active
---

# Java-specific formatting settings

The `<JavaCodeStyleSettings>` block of an IntelliJ code scheme holds the
options that exist only for Java. They live in IntelliJ's
`JavaCodeStyleSettings` class. Most are _formatting_ options (imports,
records, annotations, javadoc, spacing); a few are code-generation or
intention options that IntelliJ stores in the same block and that do not
affect reformatting.

Support marks follow the legend on the [section index](index.md#support-legend)
(✅ implemented, ◐ parsed but not fully applied, ❌ unimplemented formatter
option, n/a not a formatter concern).

## Naming

Prefix / suffix patterns used by code generation (not by the formatter).

| Option                       | Type   | Default | Effect                                           | Support |
| ---------------------------- | ------ | ------- | ------------------------------------------------ | ------- |
| `FIELD_NAME_PREFIX`          | string | `""`    | Prefix for non-static field names.               | n/a     |
| `STATIC_FIELD_NAME_PREFIX`   | string | `""`    | Prefix for static field names.                   | n/a     |
| `PARAMETER_NAME_PREFIX`      | string | `""`    | Prefix for parameter names.                      | n/a     |
| `LOCAL_VARIABLE_NAME_PREFIX` | string | `""`    | Prefix for local variable names.                 | n/a     |
| `TEST_NAME_PREFIX`           | string | `""`    | Prefix for test method names.                    | n/a     |
| `SUBCLASS_NAME_PREFIX`       | string | `""`    | Prefix for subclass names.                       | n/a     |
| `FIELD_NAME_SUFFIX`          | string | `""`    | Suffix for non-static field names.               | n/a     |
| `STATIC_FIELD_NAME_SUFFIX`   | string | `""`    | Suffix for static field names.                   | n/a     |
| `PARAMETER_NAME_SUFFIX`      | string | `""`    | Suffix for parameter names.                      | n/a     |
| `LOCAL_VARIABLE_NAME_SUFFIX` | string | `""`    | Suffix for local variable names.                 | n/a     |
| `TEST_NAME_SUFFIX`           | string | `Test`  | Suffix for test method names.                    | n/a     |
| `SUBCLASS_NAME_SUFFIX`       | string | `Impl`  | Suffix for subclass names.                       | n/a     |
| `PREFER_LONGER_NAMES`        | bool   | `true`  | Prefer longer, more descriptive generated names. | n/a     |

## Code generation

These are applied when IntelliJ _generates_ code, not when it reformats.

| Option                                     | Type   | Default  | Values                                                         | Effect                                                                                             | Support |
| ------------------------------------------ | ------ | -------- | -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- | ------- |
| `VISIBILITY`                               | string | `public` | `public`, `package`, `protected`, `private`, `EscalateVisible` | Visibility for generated members.                                                                  | n/a     |
| `GENERATE_FINAL_LOCALS`                    | bool   | `false`  | `true` / `false`                                               | Declare generated local variables `final`.                                                         | n/a     |
| `GENERATE_FINAL_PARAMETERS`                | bool   | `false`  | `true` / `false`                                               | Declare generated parameters `final`.                                                              | n/a     |
| `USE_EXTERNAL_ANNOTATIONS`                 | bool   | `false`  | `true` / `false`                                               | Attach external annotations.                                                                       | n/a     |
| `GENERATE_USE_TYPE_ANNOTATION_BEFORE_TYPE` | bool   | `true`   | `true` / `false`                                               | Put type-use annotations before the type.                                                          | n/a     |
| `INSERT_OVERRIDE_ANNOTATION`               | bool   | `true`   | `true` / `false`                                               | Insert `@Override` on overridden methods.                                                          | n/a     |
| `REPEAT_SYNCHRONIZED`                      | bool   | `true`   | `true` / `false`                                               | Repeat `synchronized` in overridden methods.                                                       | n/a     |
| `REPEAT_ANNOTATIONS`                       | list   | `[]`     | `<ANNO name="…"/>` entries                                     | Annotations repeated on overridden methods.                                                        | n/a     |
| `REPLACE_INSTANCEOF`                       | bool   | `false`  | `true` / `false`                                               | Replace `instanceof` chains with pattern matching (deprecated, use `REPLACE_INSTANCEOF_AND_CAST`). | n/a     |
| `REPLACE_CAST`                             | bool   | `false`  | `true` / `false`                                               | Replace cast patterns (deprecated, use `REPLACE_INSTANCEOF_AND_CAST`).                             | n/a     |
| `REPLACE_INSTANCEOF_AND_CAST`              | bool   | `false`  | `true` / `false`                                               | Replace `instanceof` + cast with pattern matching.                                                 | n/a     |
| `REPLACE_NULL_CHECK`                       | bool   | `true`   | `true` / `false`                                               | Replace `!= null` checks with `Objects.isNull`-style idioms in generated code.                     | n/a     |
| `REPLACE_SUM`                              | bool   | `true`   | `true` / `false`                                               | Replace sum-type lambdas with method references in generated code.                                 | n/a     |

## Imports

Formatting-relevant import options (import merging is the only one currently
applied, and only partially). The rows marked n/a are auto-import or
code-generation settings, not formatter concerns.

| Option                                            | Type  | Default                           | Values                                   | Effect                                                                                | Support |
| ------------------------------------------------- | ----- | --------------------------------- | ---------------------------------------- | ------------------------------------------------------------------------------------- | ------- |
| `USE_SINGLE_CLASS_IMPORTS`                        | bool  | `true`                            | `true` / `false`                         | Use single-class imports instead of on-demand (`*`) imports where possible.           | ❌      |
| `INSERT_INNER_CLASS_IMPORTS`                      | bool  | `false`                           | `true` / `false`                         | Add imports for inner classes.                                                        | n/a     |
| `CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND`             | int   | `5`                               | `0`–`n`                                  | Merge a package's single-type imports into `pkg.*` when the count reaches this value. | ✅      |
| `NAMES_COUNT_TO_USE_IMPORT_ON_DEMAND`             | int   | `3`                               | `0`–`n`                                  | Merge static members' imports into `pkg.*` at this count.                             | ❌      |
| `PACKAGES_TO_USE_IMPORT_ON_DEMAND`                | table | `java.awt`, `javax.swing`         | [import-table XML](#import-table-format) | Packages whose imports are always merged into `pkg.*`.                                | ❌      |
| `IMPORT_LAYOUT_TABLE`                             | table | [default layout](#default-layout) | [import-table XML](#import-table-format) | Ordering and grouping of import sections.                                             | ❌      |
| `LAYOUT_STATIC_IMPORTS_SEPARATELY`                | bool  | `true`                            | `true` / `false`                         | Keep static imports in their own section.                                             | ❌      |
| `LAYOUT_ON_DEMAND_IMPORT_FROM_SAME_PACKAGE_FIRST` | bool  | `true`                            | `true` / `false`                         | Sort same-package on-demand imports first.                                            | ❌      |
| `PRESERVE_MODULE_IMPORTS`                         | bool  | `true`                            | `true` / `false`                         | Keep module (`import module …`) imports on reformat.                                  | ❌      |
| `DELETE_UNUSED_MODULE_IMPORTS`                    | bool  | `false`                           | `true` / `false`                         | Remove unused module imports on reformat.                                             | ❌      |
| `KEEP_BLANK_LINES_BETWEEN_IMPORTS`                | bool  | `false`                           | `true` / `false`                         | Preserve blank lines between imports on reformat.                                     | ❌      |
| `USE_FQ_CLASS_NAMES`                              | bool  | `false`                           | `true` / `false`                         | Use fully-qualified names instead of imports.                                         | n/a     |
| `DO_NOT_IMPORT_INNER`                             | list  | `[]`                              | `<CLASS name="…"/>` entries              | Inner classes that are never auto-imported.                                           | n/a     |

### Import-table format

`PACKAGES_TO_USE_IMPORT_ON_DEMAND` and `IMPORT_LAYOUT_TABLE` serialize as an
`<option>` whose `<value>` child holds the entries:

```xml
<option name="IMPORT_LAYOUT_TABLE">
  <value>
    <package name="" withSubpackages="true" static="false" />
    <emptyLine />
    <package name="javax" withSubpackages="true" static="false" />
    <package name="java" withSubpackages="true" static="false" />
    <emptyLine />
    <package name="" withSubpackages="true" static="true" />
  </value>
</option>
```

- `<package>` — a package entry: `name` (empty = wildcard slot, see below),
  `withSubpackages` (`true` = `pkg.*` covers subpackages), `static`
  (`true` = static imports), `module="true"` for the module-imports slot.
- `<emptyLine />` — a blank line between import groups.
- The layout table has reserved entries: the first `<package name=""
module="true">` slot for module imports, an empty-name non-static slot for
  _all other imports_, and an empty-name static slot for _all other static
  imports_.

### Default layout

IntelliJ's built-in default import layout is:

```text
<all module imports>         (module="true")
<all other imports>          (empty-name, non-static)
<empty line>
javax.*                      (withSubpackages)
java.*                       (withSubpackages)
<empty line>
<all other static imports>   (empty-name, static)
```

## Records

| Option                                    | Type | Default              | Effect                                                     | Support |
| ----------------------------------------- | ---- | -------------------- | ---------------------------------------------------------- | ------- |
| `RECORD_COMPONENTS_WRAP`                  | int  | `1` (wrap as needed) | Wrapping of record component lists.                        | ✅      |
| `ALIGN_MULTILINE_RECORDS`                 | bool | `true`               | Align wrapped record components under the first component. | ✅      |
| `NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER`  | bool | `false`              | Put `(` of a wrapped record header on its own line.        | ✅      |
| `RPAREN_ON_NEW_LINE_IN_RECORD_HEADER`     | bool | `false`              | Put `)` of a wrapped record header on its own line.        | ❌      |
| `SPACE_WITHIN_RECORD_HEADER`              | bool | `false`              | `record R( String s )` vs `record R(String s)`.            | ❌      |
| `ANNOTATION_NEW_LINE_IN_RECORD_COMPONENT` | bool | `false`              | Put a component's annotation on a new line.                | ❌      |

> **Divergence (recorded, not fixed):** the table lists `RECORD_COMPONENTS_WRAP`
> with IntelliJ's built-in default `1` (wrap as needed), but
> `JavaStyle::default()` ships `DoNotWrap` (`0`). Changing the default would
> alter formatting behaviour, so the divergence is recorded here rather than
> fixed; a scheme that sets `RECORD_COMPONENTS_WRAP` explicitly parses
> identically in both.

## Annotations

| Option                                             | Type | Default | Effect                                                | Support |
| -------------------------------------------------- | ---- | ------- | ----------------------------------------------------- | ------- |
| `ANNOTATION_PARAMETER_WRAP`                        | int  | `0`     | Wrapping of annotation argument lists.                | ✅      |
| `ENUM_FIELD_ANNOTATION_WRAP`                       | int  | `0`     | Put annotations on enum constants on their own lines. | ❌      |
| `ALIGN_MULTILINE_ANNOTATION_PARAMETERS`            | bool | `false` | Align wrapped annotation parameters.                  | ❌      |
| `NEW_LINE_AFTER_LPAREN_IN_ANNOTATION`              | bool | `false` | Put `(` of a wrapped annotation on its own line.      | ❌      |
| `RPAREN_ON_NEW_LINE_IN_ANNOTATION`                 | bool | `false` | Put `)` of a wrapped annotation on its own line.      | ❌      |
| `SPACE_AROUND_ANNOTATION_EQ`                       | bool | `true`  | Spaces around `=` in annotation arguments.            | ❌      |
| `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION`              | bool | `false` | Do not wrap after a single annotation on a field.     | ❌      |
| `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION_IN_PARAMETER` | bool | `false` | Do not wrap after a single annotation on a parameter. | ❌      |

Annotation _placement_ on declarations (`METHOD_ANNOTATION_WRAP` etc.) lives in
the common block — see [Annotations](common.md#annotations).

## Type arguments and parameters

| Option                                                 | Type | Default | Effect                                                   | Support |
| ------------------------------------------------------ | ---- | ------- | -------------------------------------------------------- | ------- |
| `SPACES_WITHIN_ANGLE_BRACKETS`                         | bool | `false` | `< T >` vs `<T>`.                                        | ❌      |
| `SPACE_AFTER_CLOSING_ANGLE_BRACKET_IN_TYPE_ARGUMENT`   | bool | `false` | Space after `>` in type arguments.                       | ❌      |
| `SPACE_BEFORE_OPENING_ANGLE_BRACKET_IN_TYPE_PARAMETER` | bool | `false` | Space before `<` in type parameters.                     | ❌      |
| `SPACE_AROUND_TYPE_BOUNDS_IN_TYPE_PARAMETERS`          | bool | `true`  | Spaces around `extends` / `&` bounds in type parameters. | ❌      |

## Text blocks

| Option                                             | Type | Default | Effect                                                         | Support |
| -------------------------------------------------- | ---- | ------- | -------------------------------------------------------------- | ------- |
| `ALIGN_MULTILINE_TEXT_BLOCKS`                      | bool | `false` | Align the opening delimiter of multiline text blocks.          | ❌      |
| `STRIP_WHITESPACE_FROM_BLANK_LINES_IN_TEXT_BLOCKS` | bool | `false` | Strip trailing whitespace from blank lines inside text blocks. | ❌      |

## Deconstruction patterns (Java 21)

| Option                                            | Type | Default              | Effect                                      | Support |
| ------------------------------------------------- | ---- | -------------------- | ------------------------------------------- | ------- |
| `DECONSTRUCTION_LIST_WRAP`                        | int  | `1` (wrap as needed) | Wrapping of record-pattern component lists. | ❌      |
| `ALIGN_MULTILINE_DECONSTRUCTION_LIST_COMPONENTS`  | bool | `true`               | Align wrapped pattern components.           | ❌      |
| `NEW_LINE_AFTER_LPAREN_IN_DECONSTRUCTION_PATTERN` | bool | `true`               | `(` of a wrapped pattern on its own line.   | ❌      |
| `RPAREN_ON_NEW_LINE_IN_DECONSTRUCTION_PATTERN`    | bool | `true`               | `)` of a wrapped pattern on its own line.   | ❌      |
| `SPACE_WITHIN_DECONSTRUCTION_LIST`                | bool | `false`              | `case A( int x )` vs `case A(int x)`.       | ❌      |
| `SPACE_BEFORE_DECONSTRUCTION_LIST`                | bool | `false`              | `case A (int x)` vs `case A(int x)`.        | ❌      |

## Multi-catch

| Option                       | Type | Default              | Effect                                     | Support |
| ---------------------------- | ---- | -------------------- | ------------------------------------------ | ------- |
| `MULTI_CATCH_TYPES_WRAP`     | int  | `1` (wrap as needed) | Wrapping of `catch (A \| B e)` type lists. | ❌      |
| `ALIGN_TYPES_IN_MULTI_CATCH` | bool | `true`               | Align wrapped multi-catch types.           | ❌      |

## Miscellaneous spacing & blank lines

| Option                                            | Type | Default | Effect                                                                                 | Support |
| ------------------------------------------------- | ---- | ------- | -------------------------------------------------------------------------------------- | ------- |
| `SPACE_BEFORE_COLON_IN_FOREACH`                   | bool | `true`  | `for (T t : xs)` vs `for (T t: xs)`.                                                   | ❌      |
| `SPACE_INSIDE_ONE_LINE_ENUM_BRACES`               | bool | `false` | `enum E { A, B }` vs `enum E {A, B}`.                                                  | ❌      |
| `SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT` | bool | `false` | Spaces inside `{ … }` of a non-empty one-line block when `SPACE_WITHIN_BRACES` is off. | ❌      |
| `NEW_LINE_WHEN_BODY_IS_PRESENTED`                 | bool | `false` | Put the body of a one-line block on a new line.                                        | ❌      |
| `BLANK_LINES_AROUND_INITIALIZER`                  | int  | `1`     | Blank lines around instance / static initializer blocks.                               | ❌      |
| `BLANK_LINES_AROUND_FIELD_WITH_ANNOTATIONS`       | int  | `0`     | Blank lines around annotated fields.                                                   | ❌      |
| `BLANK_LINES_BETWEEN_RECORD_COMPONENTS`           | int  | `0`     | Blank lines between record components.                                                 | ❌      |
| `WRAP_SEMICOLON_AFTER_CALL_CHAIN`                 | bool | `false` | Put `;` of a wrapped chained call on its own line.                                     | ❌      |

## Javadoc

IntelliJ's javadoc options carry `JD_` option names in the scheme file (some
have newer lowercase aliases inside IntelliJ, but schemes written by the IDE
use the `JD_` names).

| Option                             | Type | Default | Effect                                                                                   | Support |
| ---------------------------------- | ---- | ------- | ---------------------------------------------------------------------------------------- | ------- |
| `ENABLE_JAVADOC_FORMATTING`        | bool | `true`  | Reformat javadoc comments at all.                                                        | ❌      |
| `CLASS_NAMES_IN_JAVADOC`           | int  | `1`     | `1` fully qualify if not imported, `2` always fully qualify, `3` shorten and add import. | ❌      |
| `JD_ALIGN_PARAM_COMMENTS`          | bool | `true`  | Align `@param` descriptions in a column.                                                 | ❌      |
| `JD_ALIGN_EXCEPTION_COMMENTS`      | bool | `true`  | Align `@throws` / `@exception` descriptions in a column.                                 | ❌      |
| `JD_ADD_BLANK_AFTER_PARM_COMMENTS` | bool | `false` | Blank line after the `@param` block.                                                     | ❌      |
| `JD_ADD_BLANK_AFTER_RETURN`        | bool | `false` | Blank line after the `@return` tag.                                                      | ❌      |
| `JD_ADD_BLANK_AFTER_DESCRIPTION`   | bool | `true`  | Blank line after the description paragraph.                                              | ❌      |
| `JD_P_AT_EMPTY_LINES`              | bool | `true`  | Put `<p>` on empty lines.                                                                | ❌      |
| `JD_KEEP_INVALID_TAGS`             | bool | `true`  | Keep invalid / unknown tags.                                                             | ❌      |
| `JD_KEEP_EMPTY_LINES`              | bool | `true`  | Keep empty lines inside javadoc.                                                         | ❌      |
| `JD_DO_NOT_WRAP_ONE_LINE_COMMENTS` | bool | `false` | Do not wrap one-line javadoc comments.                                                   | ❌      |
| `JD_USE_THROWS_NOT_EXCEPTION`      | bool | `true`  | Use `@throws` rather than `@exception`.                                                  | ❌      |
| `JD_KEEP_EMPTY_PARAMETER`          | bool | `true`  | Keep empty `@param` tags.                                                                | ❌      |
| `JD_KEEP_EMPTY_EXCEPTION`          | bool | `true`  | Keep empty `@throws` / `@exception` tags.                                                | ❌      |
| `JD_KEEP_EMPTY_RETURN`             | bool | `true`  | Keep empty `@return` tags.                                                               | ❌      |
| `JD_LEADING_ASTERISKS_ARE_ENABLED` | bool | `true`  | Render javadoc with leading `*` on every line.                                           | ❌      |
| `JD_PRESERVE_LINE_FEEDS`           | bool | `false` | Preserve line breaks inside javadoc.                                                     | ❌      |
| `JD_PARAM_DESCRIPTION_ON_NEW_LINE` | bool | `false` | Put `@param` descriptions on a new line.                                                 | ❌      |
| `JD_INDENT_ON_CONTINUATION`        | bool | `false` | Indent javadoc continuation lines.                                                       | ❌      |
