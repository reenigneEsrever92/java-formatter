# Java formatting settings — reference

This section is an exhaustive reference of **every setting IntelliJ IDEA offers
for Java code formatting**, in the exact form they appear in an IntelliJ
`<code_scheme>` document (`.idea/codeStyles/Project.xml`, or a scheme exported
with _Settings → Editor → Code Style → … → Scheme → Export_). The
[java-formatter](../../README.md) CLI reads such a scheme via `--style` and
applies the settings it implements; the pages here document the full IntelliJ
surface so the tool's coverage can be compared at a glance.

The reference is grounded in the IntelliJ IDEA Community sources
(`platform/code-style-api` and `java/java-frontback-impl`, verified against
current `master` and the 2019.3 tag, where the encodings are unchanged):

| Page                              | Contents                                                                                                                                    |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| [Common settings](common.md)      | The `<codeStyleSettings language="JAVA">` block: indentation, spaces, wrapping, braces, blank lines, alignment — `CommonCodeStyleSettings`. |
| [Java-specific settings](java.md) | The `<JavaCodeStyleSettings>` block: imports, javadoc, records, annotations, naming, code generation — `JavaCodeStyleSettings`.             |

## Scheme anatomy

A scheme is a single `<code_scheme>` root element. Only the parts that apply
to Java are listed; schemes may also carry blocks for other languages
(`JSCodeStyleSettings`, `<codeStyleSettings language="JavaScript">`, …),
which java-formatter ignores.

```xml
<code_scheme name="Project" version="173">
  <!-- 1. root-level options -->
  <option name="SOFT_MARGINS" value="120" />
  <option name="RIGHT_MARGIN" value="120" />
  <option name="LINE_SEPARATOR" value="&#10;" />
  <option name="FORMATTER_TAGS_ENABLED" value="true" />

  <!-- 2. Java-specific options -->
  <JavaCodeStyleSettings>
    <option name="CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND" value="999" />
  </JavaCodeStyleSettings>

  <!-- 3. common (language) options -->
  <codeStyleSettings language="JAVA">
    <option name="CLASS_BRACE_STYLE" value="2" />
    <option name="CALL_PARAMETERS_WRAP" value="5" />
    <indentOptions>
      <option name="INDENT_SIZE" value="4" />
      <option name="CONTINUATION_INDENT_SIZE" value="8" />
      <option name="TAB_SIZE" value="4" />
      <option name="USE_TAB_CHARACTER" value="false" />
    </indentOptions>
  </codeStyleSettings>
</code_scheme>
```

| #   | Block                                 | Holds                                                                                  | Documented on                             |
| --- | ------------------------------------- | -------------------------------------------------------------------------------------- | ----------------------------------------- |
| 1   | Root-level `<option>` children        | margins, line separator, formatter tags, autodetect indents                            | [common.md](common.md#root-level-options) |
| 2   | `<JavaCodeStyleSettings>`             | Java-only settings (imports, javadoc, records, annotations, …)                         | [java.md](java.md)                        |
| 3   | `<codeStyleSettings language="JAVA">` | settings shared across languages as applied to Java, plus the nested `<indentOptions>` | [common.md](common.md)                    |

`<option>` elements use `name` (the option's XML name, in `UPPER_SNAKE`) and
`value`. Missing options fall back to IntelliJ's built-in defaults (listed per
option in the tables).

A Java scheme may also carry an `<arrangement>` block inside
`<codeStyleSettings language="JAVA">` (member ordering / rearrangement
rules). That is a rearrangement feature rather than a formatting setting and
is out of scope here (java-formatter ignores it).

## Value encodings

Most numeric options use one of a few shared encodings. Wrapping options
(`*_WRAP`) take a **wrap code**; brace options (`*_BRACE_STYLE`) take a
**brace code**; brace-forcing options (`*_BRACE_FORCE`) take a **force code**.

### Wrap codes

| value | IntelliJ constant                                          | Meaning                                                                        |
| ----- | ---------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `0`   | `DO_NOT_WRAP`                                              | Do not wrap.                                                                   |
| `1`   | `WRAP_IF_LONG` (`WRAP_AS_NEEDED`)                          | Wrap only when the element exceeds the right margin.                           |
| `2`   | `WRAP_ALWAYS`                                              | Always wrap, regardless of length.                                             |
| `4`   | `WRAP_ON_EVERY_ITEM`                                       | Wrap at every element (bit flag; not exposed alone in the UI).                 |
| `5`   | `CHOP_DOWN_IF_LONG` = `WRAP_IF_LONG \| WRAP_ON_EVERY_ITEM` | Wrap if long, then chop down (wrap at every sub-element that still overflows). |

Only `0`, `1`, `2`, and `5` are selectable in the IntelliJ UI; `4` and `3`
(the `1\|2` combination) are not produced by the IDE but are valid field
values.

### Brace codes

| value | IntelliJ constant      | Meaning                                                                             |
| ----- | ---------------------- | ----------------------------------------------------------------------------------- |
| `1`   | `END_OF_LINE`          | Brace on the same line as the declaration (`class A {`).                            |
| `2`   | `NEXT_LINE`            | Brace on its own line (`\n{`).                                                      |
| `3`   | `NEXT_LINE_SHIFTED`    | Brace on its own line, indented one extra level.                                    |
| `4`   | `NEXT_LINE_SHIFTED2`   | Brace on its own line, indented to the body's indent.                               |
| `5`   | `NEXT_LINE_IF_WRAPPED` | Brace on the same line; moved to its own line only when the declaration is wrapped. |

### Force-brace codes

| value | IntelliJ constant           | Meaning                                                  |
| ----- | --------------------------- | -------------------------------------------------------- |
| `0`   | `DO_NOT_FORCE`              | Do not force braces.                                     |
| `1`   | `FORCE_BRACES_IF_MULTILINE` | Add braces when the statement body spans multiple lines. |
| `3`   | `FORCE_BRACES_ALWAYS`       | Always add braces.                                       |

### Other numeric encodings

| Option family                  | values                                                                                         |
| ------------------------------ | ---------------------------------------------------------------------------------------------- |
| `CLASS_NAMES_IN_JAVADOC`       | `1` fully qualify if not imported, `2` always fully qualify, `3` shorten names and add import. |
| `FORCE_REARRANGE_MODE`         | `0` rearrange according to dialog, `1` always rearrange, `2` never rearrange.                  |
| `WRAP_ON_TYPING`               | `-1` default, `0` no wrap, `1` wrap.                                                           |
| `VISIBILITY` (code generation) | `public`, `package`, `protected`, `private`, `EscalateVisible`.                                |

## Support legend

Each option table marks java-formatter's current support:

| Mark | Meaning                                                                                                                                                        |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ✅   | Implemented: parsed from the scheme and applied by the formatter.                                                                                              |
| ◐    | Parsed into the config, but not (fully) applied — see the note in the row.                                                                                     |
| ❌   | Not implemented: a valid formatter option not yet supported; covered by a [backlog](../dev/backlog/index.md) change request and safely ignored meanwhile (R7). |
| n/a  | Not a formatter concern: an IDE/editor or code-generation option IntelliJ stores in the same scheme block; deliberately never applied (safely ignored, R7).    |

## Caveats

java-formatter's `config.rs` decodes the wrap and brace codes exactly as
documented above (wrap `2` = wrap always, `5` = chop down if long; brace `1` =
end of line, `5` = next line if wrapped), so a scheme written by the GUI or
exported from IntelliJ means the same thing to both. Values IntelliJ never
writes on its own (`3` / `4` wrap codes, brace code `0`) fall back to the
safe defaults (`do not wrap`, `end of line`).

Everything marked ❌ or n/a below is ignored until supported: ❌ options are
tracked as [backlog](../dev/backlog/index.md) change requests and n/a options
are not formatter concerns. The formatter's contract is to leave unsupported
constructs untouched rather than guess (see the [overview](../overview.md) and
the README's _Formatting behaviour notes_).
