# Backlog

Change requests that define the work ahead on java-formatter. Each entry is a
concept document with `type: ChangeRequest` in this directory; the workflow
moves them `proposed` → `planned` → `in-progress` → `done`.

## Change requests

| State    | Priority | Kind        | Change request                                                                                                                   |
| -------- | -------- | ----------- | -------------------------------------------------------------------------------------------------------------------------------- |
| done     | medium   | refactor    | [Restructure the test suite so each option gets a dedicated test](per-option-test-suite.md)                                      |
| done     | high     | feature     | [egui codestyle editor (crates/gui)](egui-codestyle-editor.md)                                                                   |
| done     | high     | improvement | [Detect parse errors and warn, still emitting best-effort output](parse-error-detection.md)                                      |
| done     | medium   | refactor    | [Split the crate into a core/cli/gui workspace](workspace-split.md)                                                              |
| done     | medium   | feature     | [Wrap binary expressions per BINARY_OPERATION_WRAP](binary-expression-wrapping.md)                                               |
| done     | medium   | feature     | [Format switch statements and switch expressions](switch-formatting.md)                                                          |
| done     | low      | feature     | [Keep simple try/catch/finally and synchronized bodies on one line](one-line-try-catch-blocks.md)                                |
| done     | low      | feature     | [Emit tab indentation per USE_TAB_CHARACTER / TAB_SIZE](tab-indentation.md)                                                      |
| done     | low      | improvement | [Normalise spacing around generic type arguments](generic-type-argument-spacing.md)                                              |
| proposed | high     | feature     | [Honour the blank-line policy options (KEEP_BLANK_LINES and BLANK_LINES families)](blank-line-policy.md)                         |
| proposed | high     | feature     | [Honour the spacing-around-operators options](spaces-around-operators.md)                                                        |
| proposed | high     | feature     | [Honour the spacing-around-separators options](spaces-around-separators.md)                                                      |
| proposed | high     | feature     | [Honour the within-parentheses/brackets/braces spacing options](spaces-within-parentheses-brackets-braces.md)                    |
| proposed | high     | feature     | [Honour the before-parentheses/braces/keywords spacing options](spaces-before-keywords-and-parens.md)                            |
| proposed | high     | feature     | [Force braces on statement bodies per the *_BRACE_FORCE options](force-braces.md)                                                |
| proposed | medium   | feature     | [Honour the right margin, line separator and hard line-wrapping options](line-length-and-line-endings.md)                        |
| proposed | medium   | feature     | [Honour the comment layout options](comment-layout.md)                                                                           |
| proposed | medium   | feature     | [Honour clause-keyword and brace-less control-statement layout options](clause-keyword-layout.md)                                |
| proposed | medium   | feature     | [Honour the switch/case indentation and wrapping options](switch-case-layout.md)                                                 |
| proposed | medium   | feature     | [Honour the align-when-multiline options](align-multiline-options.md)                                                            |
| proposed | medium   | feature     | [Wrap resource lists, extends/implements and throws lists per their *_WRAP options](wrapping-declaration-clauses.md)             |
| proposed | medium   | feature     | [Wrap the remaining expressions, statements and initialisers per their *_WRAP options](wrapping-expressions-and-statements.md)   |
| proposed | medium   | feature     | [Keep simple classes and multi-expression statements on one line; lay out one-line block bodies](one-line-body-layout.md)        |
| proposed | medium   | feature     | [Honour the annotation placement and annotation-body layout options](annotation-layout.md)                                       |
| proposed | medium   | feature     | [Honour the remaining indentation options (labels, smart tabs, relative indents, per-construct indents)](indentation-details.md) |
| proposed | medium   | feature     | [Honour the remaining record-header layout options](record-header-layout.md)                                                     |
| proposed | medium   | feature     | [Order and group imports per the import layout options](import-ordering-and-layout.md)                                           |
| proposed | medium   | feature     | [Extend import-on-demand merging per the on-demand import options](import-on-demand-extensions.md)                               |
| proposed | medium   | feature     | [Format Javadoc per the JD_* javadoc options](javadoc-formatting.md)                                                             |
| proposed | low      | feature     | [Layout enum constant lists and enum spacing per the enum options](enum-layout.md)                                               |
| proposed | low      | feature     | [Honour the builder-method wrapping options](builder-method-layout.md)                                                           |
| proposed | low      | feature     | [Honour the type-argument and type-parameter spacing options](type-argument-spacing-options.md)                                  |
| proposed | low      | feature     | [Honour the text-block layout and multi-catch wrapping options](text-blocks-and-multi-catch.md)                                  |
| proposed | low      | feature     | [Honour the deconstruction-pattern layout options (Java 21)](deconstruction-pattern-layout.md)                                   |
| done     | medium   | feature     | [Set up a GitHub Actions CI pipeline (fmt, clippy, tests on an OS matrix)](github-ci-pipeline.md)                               |
