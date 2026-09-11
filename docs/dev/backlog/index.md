# Backlog

Change requests that define the work ahead on java-formatter. Each entry is a
concept document with `type: ChangeRequest` in this directory; the workflow
moves them `proposed` → `planned` → `in-progress` → `done`.

## Change requests

| State    | Priority | Kind        | Change request                                                                                                                           |
| -------- | -------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| proposed | medium   | feature     | [Format a directory in parallel across CPU cores](parallel-directory-formatting.md)                                                      |
| done     | high     | bug         | [Comments in comma-separated lists get a list separator, corrupt the construct, or vanish](comments-in-comma-separated-lists.md)         |
| done     | low      | improvement | [Show a progress bar when formatting a directory](directory-progress-bar.md)                                                             |
| done     | high     | bug         | [Array creation new T[expr] is rendered with doubled brackets (new T[[expr]])](array-creation-double-brackets.md)                        |
| done     | medium   | feature     | [Format all Java files in a directory in place (-d/--dir, -r/--recursive)](directory-formatting.md)                                      |
| done     | high     | bug         | [Prefer breaking a call chain over wrapping its parameters when PREFER_PARAMETERS_WRAP is off](prefer-chain-wrap-over-parameter-wrap.md) |
| done     | high     | bug         | [Place the around-member blank lines before a member's leading comment](blank-line-before-member-comment.md)                             |
| done     | high     | bug         | [Wrap chain-link arguments and fix the lparen-stays / rparen-alone indentation](chain-link-args-and-lparen-layout.md)                    |
| done     | medium   | improvement | [Structure the GUI config options into ordered, collapsible sections](gui-option-structure.md)                                           |
| done     | high     | bug         | [METHOD_CALL_CHAIN_WRAP does not wrap a chain nested in an argument](nested-chain-argument-wrap.md)                                      |
| done     | medium   | feature     | [GUI editor polish: right-margin guide, scrollable panes, tree-sitter highlighting](gui-editor-polish.md)                                |
| done     | medium   | refactor    | [Restructure the test suite so each option gets a dedicated test](per-option-test-suite.md)                                              |
| done     | high     | feature     | [egui codestyle editor (crates/gui)](egui-codestyle-editor.md)                                                                           |
| done     | high     | improvement | [Detect parse errors and warn, still emitting best-effort output](parse-error-detection.md)                                              |
| done     | medium   | refactor    | [Split the crate into a core/cli/gui workspace](workspace-split.md)                                                                      |
| done     | medium   | feature     | [Wrap binary expressions per BINARY_OPERATION_WRAP](binary-expression-wrapping.md)                                                       |
| done     | medium   | feature     | [Format switch statements and switch expressions](switch-formatting.md)                                                                  |
| done     | low      | feature     | [Keep simple try/catch/finally and synchronized bodies on one line](one-line-try-catch-blocks.md)                                        |
| done     | low      | feature     | [Emit tab indentation per USE_TAB_CHARACTER / TAB_SIZE](tab-indentation.md)                                                              |
| done     | low      | improvement | [Normalise spacing around generic type arguments](generic-type-argument-spacing.md)                                                      |
| done     | high     | feature     | [Honour the blank-line policy options (KEEP_BLANK_LINES and BLANK_LINES families)](blank-line-policy.md)                                 |
| done     | high     | feature     | [Honour the spacing-around-operators options](spaces-around-operators.md)                                                                |
| done     | high     | feature     | [Honour the spacing-around-separators options](spaces-around-separators.md)                                                              |
| done     | high     | feature     | [Honour the within-parentheses/brackets/braces spacing options](spaces-within-parentheses-brackets-braces.md)                            |
| done     | high     | feature     | [Honour the before-parentheses/braces/keywords spacing options](spaces-before-keywords-and-parens.md)                                    |
| done     | high     | feature     | [Force braces on statement bodies per the *_BRACE_FORCE options](force-braces.md)                                                        |
| done     | medium   | feature     | [Honour the right margin, line separator and hard line-wrapping options](line-length-and-line-endings.md)                                |
| done     | medium   | feature     | [Honour the comment layout options](comment-layout.md)                                                                                   |
| done     | medium   | feature     | [Honour clause-keyword and brace-less control-statement layout options](clause-keyword-layout.md)                                        |
| done     | medium   | feature     | [Honour the switch/case indentation and wrapping options](switch-case-layout.md)                                                         |
| done     | medium   | feature     | [Honour the align-when-multiline options](align-multiline-options.md)                                                                    |
| done     | medium   | feature     | [Wrap resource lists, extends/implements and throws lists per their *_WRAP options](wrapping-declaration-clauses.md)                     |
| done     | medium   | feature     | [Wrap the remaining expressions, statements and initialisers per their *_WRAP options](wrapping-expressions-and-statements.md)           |
| done     | medium   | feature     | [Keep simple classes and multi-expression statements on one line; lay out one-line block bodies](one-line-body-layout.md)                |
| done     | medium   | feature     | [Honour the annotation placement and annotation-body layout options](annotation-layout.md)                                               |
| done     | medium   | feature     | [Honour the remaining indentation options (labels, smart tabs, relative indents, per-construct indents)](indentation-details.md)         |
| done     | medium   | feature     | [Honour the remaining record-header layout options](record-header-layout.md)                                                             |
| done     | medium   | feature     | [Order and group imports per the import layout options](import-ordering-and-layout.md)                                                   |
| done     | medium   | feature     | [Extend import-on-demand merging per the on-demand import options](import-on-demand-extensions.md)                                       |
| done     | medium   | feature     | [Format Javadoc per the JD_* javadoc options](javadoc-formatting.md)                                                                     |
| done     | low      | feature     | [Layout enum constant lists and enum spacing per the enum options](enum-layout.md)                                                       |
| done     | low      | feature     | [Honour the builder-method wrapping options](builder-method-layout.md)                                                                   |
| done     | low      | feature     | [Honour the type-argument and type-parameter spacing options](type-argument-spacing-options.md)                                          |
| done     | low      | feature     | [Honour the text-block layout and multi-catch wrapping options](text-blocks-and-multi-catch.md)                                          |
| done     | low      | feature     | [Honour the deconstruction-pattern layout options (Java 21)](deconstruction-pattern-layout.md)                                           |
| done     | medium   | feature     | [Set up a GitHub Actions CI pipeline (fmt, clippy, tests on an OS matrix)](github-ci-pipeline.md)                                        |
