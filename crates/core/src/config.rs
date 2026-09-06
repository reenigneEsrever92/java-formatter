//! Parser and writer for IntelliJ IDEA codestyle XML files.
//!
//! Reads `<code_scheme>` XML (e.g. `.idea/codeStyles/Project.xml`) into a
//! [`JavaStyle`] with all relevant formatting settings, and writes one back
//! via [`serialize_codestyle`]. Every supported option is declared once in
//! the [`OPTIONS`] registry, which drives both parsing and serialization so
//! the two can never diverge.

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Public enums
// ---------------------------------------------------------------------------

/// Maps IntelliJ wrap-mode integer constants to a readable enum.
///
/// | value | meaning |
/// |-------|---------|
/// | 0     | DoNotWrap |
/// | 1     | WrapIfLong |
/// | 2     | WrapAlways |
/// | 5     | ChopDownIfLong |
///
/// Values `3` and `4` are bit combinations IntelliJ never writes alone and
/// fall back to [`WrapStyle::DoNotWrap`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapStyle {
    DoNotWrap,
    WrapIfLong,
    WrapAlways,
    ChopDownIfLong,
}

impl WrapStyle {
    fn from_int(v: u32) -> Self {
        match v {
            1 => WrapStyle::WrapIfLong,
            2 => WrapStyle::WrapAlways,
            5 => WrapStyle::ChopDownIfLong,
            _ => WrapStyle::DoNotWrap,
        }
    }

    fn to_int(self) -> u32 {
        match self {
            WrapStyle::DoNotWrap => 0,
            WrapStyle::WrapIfLong => 1,
            WrapStyle::WrapAlways => 2,
            WrapStyle::ChopDownIfLong => 5,
        }
    }
}

/// Maps IntelliJ brace-style integer constants to a readable enum.
///
/// | value | meaning |
/// |-------|---------|
/// | 0 / 1 | EndOfLine |
/// | 2     | NextLine |
/// | 3     | NextLineShifted |
/// | 4     | NextLineShifted2 |
/// | 5     | NextLineIfWrapped |
///
/// IntelliJ never writes `0`; unknown values fall back to
/// [`BraceStyle::EndOfLine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BraceStyle {
    EndOfLine,
    NextLine,
    NextLineShifted,
    NextLineShifted2,
    NextLineIfWrapped,
}

impl BraceStyle {
    fn from_int(v: u32) -> Self {
        match v {
            2 => BraceStyle::NextLine,
            3 => BraceStyle::NextLineShifted,
            4 => BraceStyle::NextLineShifted2,
            5 => BraceStyle::NextLineIfWrapped,
            _ => BraceStyle::EndOfLine, // 0, 1, and unknown values
        }
    }

    fn to_int(self) -> u32 {
        match self {
            BraceStyle::EndOfLine => 1,
            BraceStyle::NextLine => 2,
            BraceStyle::NextLineShifted => 3,
            BraceStyle::NextLineShifted2 => 4,
            BraceStyle::NextLineIfWrapped => 5,
        }
    }
}

/// Maps IntelliJ force-brace integer constants to a readable enum.
///
/// | value | meaning |
/// |-------|---------|
/// | 0     | DoNotForce |
/// | 1     | ForceIfMultiline |
/// | 3     | ForceAlways |
///
/// The force-brace codes come from `docs/settings/index.md`; unknown values
/// fall back to [`ForceStyle::DoNotForce`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceStyle {
    DoNotForce,
    ForceIfMultiline,
    ForceAlways,
}

impl ForceStyle {
    fn from_int(v: u32) -> Self {
        match v {
            1 => ForceStyle::ForceIfMultiline,
            3 => ForceStyle::ForceAlways,
            _ => ForceStyle::DoNotForce, // 0 and out-of-set values
        }
    }

    fn to_int(self) -> u32 {
        match self {
            ForceStyle::DoNotForce => 0,
            ForceStyle::ForceIfMultiline => 1,
            ForceStyle::ForceAlways => 3,
        }
    }
}

/// Maps the root-level `LINE_SEPARATOR` option's value to a readable enum.
///
/// IntelliJ stores the separator as an escaped character sequence in the XML
/// attribute (`&#10;` for LF, `&#13;&#10;` for CRLF, `&#13;` for CR), which
/// quick-xml decodes to the real `\n` / `\r\n` / `\r` when reading.
/// [`LineSeparator::System`] is the absence-of-option default: emit the
/// platform's own separator (IntelliJ's "system default").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineSeparator {
    /// The platform's native separator (`\n` on Unix, `\r\n` on Windows).
    System,
    /// LF — `\n` (`&#10;` in XML).
    Lf,
    /// CRLF — `\r\n` (`&#13;&#10;` in XML).
    Crlf,
    /// CR — `\r` (`&#13;` in XML).
    Cr,
}

impl LineSeparator {
    /// The actual character sequence this separator denotes. `System` resolves
    /// to the platform separator — `\n` on the test hosts, `\r\n` on Windows.
    pub fn resolve(self) -> &'static str {
        match self {
            LineSeparator::System => {
                if cfg!(windows) {
                    "\r\n"
                } else {
                    "\n"
                }
            }
            LineSeparator::Lf => "\n",
            LineSeparator::Crlf => "\r\n",
            LineSeparator::Cr => "\r",
        }
    }

    /// The XML-escaped attribute value IntelliJ writes for this separator;
    /// `System` (the default) is never serialised. A raw newline inside an XML
    /// attribute would be normalised to a space by parsers, so the escaped
    /// forms are written instead.
    fn to_xml(self) -> Option<&'static str> {
        match self {
            LineSeparator::System => None,
            LineSeparator::Lf => Some("&#10;"),
            LineSeparator::Crlf => Some("&#13;&#10;"),
            LineSeparator::Cr => Some("&#13;"),
        }
    }

    /// Decode a quick-xml-decoded attribute value (already the real character
    /// sequence `\n` / `\r\n` / `\r`) back into a separator; `None` for
    /// anything else.
    fn from_str(v: &str) -> Option<Self> {
        match v {
            "\n" => Some(LineSeparator::Lf),
            "\r\n" => Some(LineSeparator::Crlf),
            "\r" => Some(LineSeparator::Cr),
            _ => None,
        }
    }
}

/// One entry of the import-layout table ([`JavaStyle::import_layout`]), in
/// table order. The table drives the ordering and grouping of the import
/// section: each [`ImportLayoutEntry::Package`] names the imports that land in
/// its group, and an [`ImportLayoutEntry::EmptyLine`] inserts one blank line
/// between the groups around it. Serialized as the nested `<package>` /
/// `<emptyLine>` children of the option's `<value>` (java.md "Import-table
/// format").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportLayoutEntry {
    /// A blank line between the surrounding import groups (`<emptyLine />`).
    EmptyLine,
    /// A package group (`<package name=… withSubpackages=… static=… />`).
    Package {
        /// The package prefix; empty = the "all other imports" catch-all slot.
        name: String,
        /// `withSubpackages="true"`: also matches the package's subpackages.
        with_subpackages: bool,
        /// `static="true"`: matches static imports (when
        /// `LAYOUT_STATIC_IMPORTS_SEPARATELY` is on).
        is_static: bool,
        /// `module="true"`: the reserved slot for `import module …;` lines.
        is_module: bool,
    },
}

// ---------------------------------------------------------------------------
// Public output struct
// ---------------------------------------------------------------------------

/// Resolved Java formatting settings derived from an IntelliJ codestyle XML.
///
/// All fields fall back to IntelliJ defaults when absent from the XML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaStyle {
    // --- indentation ---
    pub indent_size: u32,
    pub continuation_indent_size: u32,
    pub tab_size: u32,
    pub use_tab_character: bool,
    pub smart_tabs: bool,
    pub label_indent_size: u32,
    pub label_indent_absolute: bool,
    pub use_relative_indents: bool,
    pub keep_indents_on_empty_lines: bool,
    pub do_not_indent_top_level_class_members: bool,
    // Per-construct continuation-indent widths, each defaulting to `-1`
    // (= "inherit": use `CONTINUATION_INDENT_SIZE`).
    pub declaration_parameter_indent: i32,
    pub generic_type_parameter_indent: i32,
    pub call_parameter_indent: i32,
    pub chained_call_indent: i32,
    pub array_element_indent: i32,

    // --- line length ---
    pub right_margin: u32,
    pub line_separator: LineSeparator,
    pub wrap_long_lines: bool,
    pub keep_line_breaks: bool,

    // --- comments ---
    pub line_comment_at_first_column: bool,
    pub block_comment_at_first_column: bool,
    pub line_comment_add_space_on_reformat: bool,
    pub line_comment_add_space_in_suppression: bool,
    pub keep_first_column_comment: bool,
    pub wrap_comments: bool,

    // --- javadoc ---
    pub enable_javadoc_formatting: bool,
    pub class_names_in_javadoc: u32,
    pub jd_align_param_comments: bool,
    pub jd_align_exception_comments: bool,
    pub jd_add_blank_after_parm_comments: bool,
    pub jd_add_blank_after_return: bool,
    pub jd_add_blank_after_description: bool,
    pub jd_p_at_empty_lines: bool,
    pub jd_keep_invalid_tags: bool,
    pub jd_keep_empty_lines: bool,
    pub jd_do_not_wrap_one_line_comments: bool,
    pub jd_use_throws_not_exception: bool,
    pub jd_keep_empty_parameter: bool,
    pub jd_keep_empty_exception: bool,
    pub jd_keep_empty_return: bool,
    pub jd_leading_asterisks_are_enabled: bool,
    pub jd_preserve_line_feeds: bool,
    pub jd_param_description_on_new_line: bool,
    pub jd_indent_on_continuation: bool,

    // --- brace styles ---
    pub class_brace_style: BraceStyle,
    pub method_brace_style: BraceStyle,
    pub other_brace_style: BraceStyle,
    pub lambda_brace_style: BraceStyle,

    // --- forced braces on statement bodies ---
    pub if_brace_force: ForceStyle,
    pub for_brace_force: ForceStyle,
    pub while_brace_force: ForceStyle,
    pub dowhile_brace_force: ForceStyle,

    // --- clause keywords on their own lines ---
    pub else_on_new_line: bool,
    pub while_on_new_line: bool,
    pub catch_on_new_line: bool,
    pub finally_on_new_line: bool,
    pub special_else_if_treatment: bool,

    // --- call-site parameter wrapping ---
    pub call_parameters_wrap: WrapStyle,
    pub call_parameters_lparen_on_next_line: bool,
    pub call_parameters_rparen_on_next_line: bool,

    // --- method declaration parameter wrapping ---
    pub method_parameters_wrap: WrapStyle,
    pub method_parameters_lparen_on_next_line: bool,
    pub method_parameters_rparen_on_next_line: bool,

    // --- chain / annotation / assignment wrapping ---
    pub method_call_chain_wrap: WrapStyle,
    pub wrap_first_method_in_call_chain: bool,
    pub annotation_parameter_wrap: WrapStyle,
    pub assignment_wrap: WrapStyle,
    pub place_assignment_sign_on_next_line: bool,
    pub parentheses_expression_lparen_wrap: bool,
    pub parentheses_expression_rparen_wrap: bool,
    pub modifier_list_wrap: bool,
    pub wrap_semicolon_after_call_chain: bool,

    // --- builder method chains ---
    pub builder_methods: Vec<String>,
    pub keep_builder_methods_indents: bool,

    // --- annotation placement (CodeStyleJava) ---
    pub method_annotation_wrap: WrapStyle,
    pub class_annotation_wrap: WrapStyle,
    pub field_annotation_wrap: WrapStyle,
    pub parameter_annotation_wrap: WrapStyle,
    pub variable_annotation_wrap: WrapStyle,

    // --- binary expression wrapping ---
    pub binary_operation_wrap: WrapStyle,
    pub binary_operation_sign_on_next_line: bool,

    // --- ternary / assert / for / array-initialiser wrapping ---
    pub ternary_operation_wrap: WrapStyle,
    pub ternary_operation_signs_on_next_line: bool,
    pub assert_statement_wrap: WrapStyle,
    pub assert_statement_colon_on_next_line: bool,
    pub for_statement_wrap: WrapStyle,
    pub for_statement_lparen_on_next_line: bool,
    pub for_statement_rparen_on_next_line: bool,
    pub array_initializer_wrap: WrapStyle,
    pub array_initializer_lbrace_on_next_line: bool,
    pub array_initializer_rbrace_on_next_line: bool,

    // --- declaration clause wrapping (resources / extends-implements / throws) ---
    pub resource_list_wrap: WrapStyle,
    pub resource_list_lparen_on_next_line: bool,
    pub resource_list_rparen_on_next_line: bool,
    pub extends_list_wrap: WrapStyle,
    pub extends_keyword_wrap: bool,
    pub throws_list_wrap: WrapStyle,
    pub throws_keyword_wrap: bool,
    pub prefer_parameters_wrap: bool,

    // --- alignment when multiline (ALIGN_*) ---
    pub align_multiline_parameters: bool,
    pub align_multiline_parameters_in_calls: bool,
    pub align_multiline_resources: bool,
    pub align_multiline_for: bool,
    pub align_multiline_binary_operation: bool,
    pub align_multiline_assignment: bool,
    pub align_multiline_ternary_operation: bool,
    pub align_multiline_throws_list: bool,
    pub align_throws_keyword: bool,
    pub align_multiline_extends_list: bool,
    pub align_multiline_method_brackets: bool,
    pub align_multiline_parenthesized_expression: bool,
    pub align_multiline_array_initializer_expression: bool,
    pub align_multiline_chained_methods: bool,
    pub align_group_field_declarations: bool,
    pub align_consecutive_variable_declarations: bool,
    pub align_consecutive_assignments: bool,
    pub align_subsequent_simple_methods: bool,

    // --- one-liners ---
    pub keep_simple_blocks_in_one_line: bool,
    pub keep_simple_methods_in_one_line: bool,
    pub keep_simple_lambdas_in_one_line: bool,
    pub keep_simple_classes_in_one_line: bool,
    pub keep_multiple_expressions_in_one_line: bool,
    pub keep_control_statement_in_one_line: bool,

    // --- one-line block body presentation (JavaCodeStyleSettings) ---
    pub spaces_inside_block_braces_when_body_is_present: bool,
    pub new_line_when_body_is_presented: bool,

    // --- switch / case layout ---
    pub indent_case_from_switch: bool,
    pub case_statement_on_new_line: bool,
    pub indent_break_from_case: bool,
    pub switch_expressions_wrap: WrapStyle,

    // --- record-specific (JavaCodeStyleSettings) ---
    pub record_components_wrap: WrapStyle,
    pub align_multiline_records: bool,
    pub new_line_after_lparen_in_record_header: bool,
    pub rparen_on_new_line_in_record_header: bool,
    pub space_within_record_header: bool,
    pub annotation_new_line_in_record_component: bool,
    pub blank_lines_between_record_components: u32,

    // --- enum layout (CodeStyleJava "Enums" + JavaCodeStyleSettings) ---
    pub enum_constants_wrap: WrapStyle,
    pub space_inside_one_line_enum_braces: bool,

    // --- annotation body layout (JavaCodeStyleSettings) ---
    pub enum_field_annotation_wrap: WrapStyle,
    pub align_multiline_annotation_parameters: bool,
    pub new_line_after_lparen_in_annotation: bool,
    pub rparen_on_new_line_in_annotation: bool,
    pub space_around_annotation_eq: bool,
    pub do_not_wrap_after_single_annotation: bool,
    pub do_not_wrap_after_single_annotation_in_parameter: bool,

    // --- imports (JavaCodeStyleSettings) ---
    pub class_count_to_use_import_on_demand: u32,
    /// Static member imports of one owner collapse into
    /// `import static pkg.Owner.*;` when their count exceeds this (java.md
    /// NAMES_COUNT_TO_USE_IMPORT_ON_DEMAND).
    pub names_count_to_use_import_on_demand: u32,
    /// Package prefixes whose single-type imports always merge into `pkg.*`
    /// on demand, regardless of count (java.md
    /// PACKAGES_TO_USE_IMPORT_ON_DEMAND). Stored as bare package prefixes;
    /// the `.*` suffix is an XML-boundary concern.
    pub packages_to_use_import_on_demand: Vec<String>,
    /// Prefer single-class imports over on-demand (`pkg.*`) imports where
    /// possible; off, every eligible non-static group merges (java.md
    /// USE_SINGLE_CLASS_IMPORTS).
    pub use_single_class_imports: bool,
    /// Ordered import-layout table (see [`ImportLayoutEntry`]); the built-in
    /// default is java.md's "Default layout".
    pub import_layout: Vec<ImportLayoutEntry>,
    pub layout_static_imports_separately: bool,
    pub layout_on_demand_import_from_same_package_first: bool,
    pub keep_blank_lines_between_imports: bool,
    pub preserve_module_imports: bool,
    pub delete_unused_module_imports: bool,

    // --- blank lines: KEEP_BLANK_LINES_* caps + BLANK_LINES_* minimums ---
    pub keep_blank_lines_in_code: u32,
    pub keep_blank_lines_in_declarations: u32,
    pub keep_blank_lines_between_package_declaration_and_header: u32,
    pub keep_blank_lines_before_rbrace: u32,
    pub blank_lines_before_package: u32,
    pub blank_lines_after_package: u32,
    pub blank_lines_before_imports: u32,
    pub blank_lines_after_imports: u32,
    pub blank_lines_around_class: u32,
    pub blank_lines_around_field: u32,
    pub blank_lines_around_method: u32,
    pub blank_lines_before_method_body: u32,
    pub blank_lines_around_field_in_interface: u32,
    pub blank_lines_around_method_in_interface: u32,
    pub blank_lines_after_class_header: u32,
    pub blank_lines_after_anonymous_class_header: u32,
    pub blank_lines_before_class_end: u32,

    // --- blank lines: Java-specific minimums (JavaCodeStyleSettings) ---
    pub blank_lines_around_initializer: u32,
    pub blank_lines_around_field_with_annotations: u32,

    // --- operator spacing ---
    pub space_around_assignment_operators: bool,
    pub space_around_logical_operators: bool,
    pub space_around_equality_operators: bool,
    pub space_around_relational_operators: bool,
    pub space_around_bitwise_operators: bool,
    pub space_around_additive_operators: bool,
    pub space_around_multiplicative_operators: bool,
    pub space_around_shift_operators: bool,
    pub space_around_unary_operator: bool,
    pub space_around_lambda_arrow: bool,
    pub space_around_method_ref_dbl_colon: bool,
    pub space_after_type_cast: bool,

    // --- separator spacing ---
    pub space_after_comma: bool,
    pub space_after_comma_in_type_arguments: bool,
    pub space_before_comma: bool,
    pub space_after_semicolon: bool,
    pub space_before_semicolon: bool,
    pub space_before_quest: bool,
    pub space_after_quest: bool,
    pub space_before_colon: bool,
    pub space_after_colon: bool,
    pub space_before_type_parameter_list: bool,
    pub space_before_colon_in_foreach: bool,

    // --- generic type spacing ---
    pub spaces_within_angle_brackets: bool,
    pub space_after_closing_angle_bracket_in_type_argument: bool,
    pub space_before_opening_angle_bracket_in_type_parameter: bool,
    pub space_around_type_bounds_in_type_parameters: bool,

    // --- spacing within parens / brackets / braces ---
    pub space_within_parentheses: bool,
    pub space_within_method_call_parentheses: bool,
    pub space_within_empty_method_call_parentheses: bool,
    pub space_within_method_parentheses: bool,
    pub space_within_empty_method_parentheses: bool,
    pub space_within_if_parentheses: bool,
    pub space_within_while_parentheses: bool,
    pub space_within_for_parentheses: bool,
    pub space_within_try_parentheses: bool,
    pub space_within_catch_parentheses: bool,
    pub space_within_switch_parentheses: bool,
    pub space_within_synchronized_parentheses: bool,
    pub space_within_cast_parentheses: bool,
    pub space_within_brackets: bool,
    pub space_within_braces: bool,
    pub space_within_array_initializer_braces: bool,
    pub space_within_empty_array_initializer_braces: bool,
    pub space_within_annotation_parentheses: bool,

    // --- spacing before parentheses / braces / keywords ---
    pub space_before_method_call_parentheses: bool,
    pub space_before_method_parentheses: bool,
    pub space_before_if_parentheses: bool,
    pub space_before_while_parentheses: bool,
    pub space_before_for_parentheses: bool,
    pub space_before_try_parentheses: bool,
    pub space_before_catch_parentheses: bool,
    pub space_before_switch_parentheses: bool,
    pub space_before_synchronized_parentheses: bool,
    // XML name spelled as in IntelliJ sources, typo included.
    pub space_before_anotation_parameter_list: bool,
    pub space_before_class_lbrace: bool,
    pub space_before_method_lbrace: bool,
    pub space_before_if_lbrace: bool,
    pub space_before_else_lbrace: bool,
    pub space_before_while_lbrace: bool,
    pub space_before_for_lbrace: bool,
    pub space_before_do_lbrace: bool,
    pub space_before_switch_lbrace: bool,
    pub space_before_try_lbrace: bool,
    pub space_before_catch_lbrace: bool,
    pub space_before_finally_lbrace: bool,
    pub space_before_synchronized_lbrace: bool,
    pub space_before_array_initializer_lbrace: bool,
    pub space_before_annotation_array_initializer_lbrace: bool,
    pub space_before_else_keyword: bool,
    pub space_before_while_keyword: bool,
    pub space_before_catch_keyword: bool,
    pub space_before_finally_keyword: bool,
}

impl JavaStyle {
    /// IntelliJ's built-in import layout (java.md "Default layout"): the
    /// reserved module-imports slot, the empty-name non-static catch-all, a
    /// blank line, `javax.*`, `java.*` (each `withSubpackages`), a blank line,
    /// then the empty-name static catch-all. The single construction site for
    /// the built-in table, so [`JavaStyle::default`] and the registry's
    /// default reference can never diverge.
    pub fn builtin_import_layout() -> Vec<ImportLayoutEntry> {
        vec![
            ImportLayoutEntry::Package {
                name: String::new(),
                with_subpackages: true,
                is_static: false,
                is_module: true,
            },
            ImportLayoutEntry::Package {
                name: String::new(),
                with_subpackages: true,
                is_static: false,
                is_module: false,
            },
            ImportLayoutEntry::EmptyLine,
            ImportLayoutEntry::Package {
                name: "javax".to_string(),
                with_subpackages: true,
                is_static: false,
                is_module: false,
            },
            ImportLayoutEntry::Package {
                name: "java".to_string(),
                with_subpackages: true,
                is_static: false,
                is_module: false,
            },
            ImportLayoutEntry::EmptyLine,
            ImportLayoutEntry::Package {
                name: String::new(),
                with_subpackages: true,
                is_static: true,
                is_module: false,
            },
        ]
    }
}

impl Default for JavaStyle {
    fn default() -> Self {
        JavaStyle {
            indent_size: 4,
            continuation_indent_size: 8,
            tab_size: 4,
            use_tab_character: false,
            smart_tabs: false,
            label_indent_size: 0,
            label_indent_absolute: false,
            use_relative_indents: false,
            keep_indents_on_empty_lines: false,
            do_not_indent_top_level_class_members: false,
            declaration_parameter_indent: -1,
            generic_type_parameter_indent: -1,
            call_parameter_indent: -1,
            chained_call_indent: -1,
            array_element_indent: -1,
            right_margin: 120,
            line_separator: LineSeparator::System,
            wrap_long_lines: false,
            keep_line_breaks: true,
            line_comment_at_first_column: true,
            block_comment_at_first_column: true,
            line_comment_add_space_on_reformat: false,
            line_comment_add_space_in_suppression: false,
            keep_first_column_comment: true,
            wrap_comments: false,
            enable_javadoc_formatting: false,
            class_names_in_javadoc: 1,
            jd_align_param_comments: true,
            jd_align_exception_comments: true,
            jd_add_blank_after_parm_comments: false,
            jd_add_blank_after_return: false,
            jd_add_blank_after_description: true,
            jd_p_at_empty_lines: true,
            jd_keep_invalid_tags: true,
            jd_keep_empty_lines: true,
            jd_do_not_wrap_one_line_comments: false,
            jd_use_throws_not_exception: true,
            jd_keep_empty_parameter: true,
            jd_keep_empty_exception: true,
            jd_keep_empty_return: true,
            jd_leading_asterisks_are_enabled: true,
            jd_preserve_line_feeds: false,
            jd_param_description_on_new_line: false,
            jd_indent_on_continuation: false,
            class_brace_style: BraceStyle::EndOfLine,
            method_brace_style: BraceStyle::EndOfLine,
            other_brace_style: BraceStyle::EndOfLine,
            lambda_brace_style: BraceStyle::EndOfLine,
            if_brace_force: ForceStyle::DoNotForce,
            for_brace_force: ForceStyle::DoNotForce,
            while_brace_force: ForceStyle::DoNotForce,
            dowhile_brace_force: ForceStyle::DoNotForce,
            else_on_new_line: false,
            while_on_new_line: false,
            catch_on_new_line: false,
            finally_on_new_line: false,
            special_else_if_treatment: true,
            call_parameters_wrap: WrapStyle::DoNotWrap,
            call_parameters_lparen_on_next_line: false,
            call_parameters_rparen_on_next_line: false,
            method_parameters_wrap: WrapStyle::DoNotWrap,
            method_parameters_lparen_on_next_line: false,
            method_parameters_rparen_on_next_line: false,
            method_call_chain_wrap: WrapStyle::DoNotWrap,
            wrap_first_method_in_call_chain: false,
            annotation_parameter_wrap: WrapStyle::DoNotWrap,
            assignment_wrap: WrapStyle::DoNotWrap,
            place_assignment_sign_on_next_line: false,
            parentheses_expression_lparen_wrap: false,
            parentheses_expression_rparen_wrap: false,
            modifier_list_wrap: false,
            wrap_semicolon_after_call_chain: false,
            builder_methods: Vec::new(),
            keep_builder_methods_indents: false,
            method_annotation_wrap: WrapStyle::WrapAlways,
            class_annotation_wrap: WrapStyle::WrapAlways,
            field_annotation_wrap: WrapStyle::WrapAlways,
            parameter_annotation_wrap: WrapStyle::DoNotWrap,
            variable_annotation_wrap: WrapStyle::DoNotWrap,
            binary_operation_wrap: WrapStyle::DoNotWrap,
            binary_operation_sign_on_next_line: false,
            ternary_operation_wrap: WrapStyle::DoNotWrap,
            ternary_operation_signs_on_next_line: false,
            assert_statement_wrap: WrapStyle::DoNotWrap,
            assert_statement_colon_on_next_line: false,
            for_statement_wrap: WrapStyle::DoNotWrap,
            for_statement_lparen_on_next_line: false,
            for_statement_rparen_on_next_line: false,
            array_initializer_wrap: WrapStyle::DoNotWrap,
            array_initializer_lbrace_on_next_line: false,
            array_initializer_rbrace_on_next_line: false,
            resource_list_wrap: WrapStyle::DoNotWrap,
            resource_list_lparen_on_next_line: false,
            resource_list_rparen_on_next_line: false,
            extends_list_wrap: WrapStyle::DoNotWrap,
            extends_keyword_wrap: false,
            throws_list_wrap: WrapStyle::DoNotWrap,
            throws_keyword_wrap: false,
            prefer_parameters_wrap: false,
            align_multiline_parameters: true,
            align_multiline_parameters_in_calls: false,
            align_multiline_resources: true,
            align_multiline_for: true,
            align_multiline_binary_operation: false,
            align_multiline_assignment: false,
            align_multiline_ternary_operation: false,
            align_multiline_throws_list: false,
            align_throws_keyword: false,
            align_multiline_extends_list: false,
            align_multiline_method_brackets: false,
            align_multiline_parenthesized_expression: false,
            align_multiline_array_initializer_expression: false,
            align_multiline_chained_methods: false,
            align_group_field_declarations: false,
            align_consecutive_variable_declarations: false,
            align_consecutive_assignments: false,
            align_subsequent_simple_methods: false,
            keep_simple_blocks_in_one_line: false,
            keep_simple_methods_in_one_line: false,
            keep_simple_lambdas_in_one_line: false,
            keep_simple_classes_in_one_line: false,
            keep_multiple_expressions_in_one_line: false,
            keep_control_statement_in_one_line: true,
            spaces_inside_block_braces_when_body_is_present: false,
            new_line_when_body_is_presented: false,
            indent_case_from_switch: true,
            case_statement_on_new_line: true,
            indent_break_from_case: true,
            switch_expressions_wrap: WrapStyle::WrapIfLong,
            record_components_wrap: WrapStyle::DoNotWrap,
            align_multiline_records: true,
            new_line_after_lparen_in_record_header: false,
            rparen_on_new_line_in_record_header: false,
            space_within_record_header: false,
            annotation_new_line_in_record_component: false,
            blank_lines_between_record_components: 0,
            enum_constants_wrap: WrapStyle::DoNotWrap,
            space_inside_one_line_enum_braces: false,
            enum_field_annotation_wrap: WrapStyle::DoNotWrap,
            align_multiline_annotation_parameters: false,
            new_line_after_lparen_in_annotation: false,
            rparen_on_new_line_in_annotation: false,
            space_around_annotation_eq: true,
            do_not_wrap_after_single_annotation: false,
            do_not_wrap_after_single_annotation_in_parameter: false,
            class_count_to_use_import_on_demand: 5,
            names_count_to_use_import_on_demand: 3,
            packages_to_use_import_on_demand: vec![
                "java.awt".to_string(),
                "javax.swing".to_string(),
            ],
            use_single_class_imports: true,
            import_layout: JavaStyle::builtin_import_layout(),
            layout_static_imports_separately: true,
            layout_on_demand_import_from_same_package_first: true,
            keep_blank_lines_between_imports: false,
            preserve_module_imports: true,
            delete_unused_module_imports: false,
            keep_blank_lines_in_code: 2,
            keep_blank_lines_in_declarations: 2,
            keep_blank_lines_between_package_declaration_and_header: 2,
            keep_blank_lines_before_rbrace: 2,
            blank_lines_before_package: 0,
            blank_lines_after_package: 1,
            blank_lines_before_imports: 1,
            blank_lines_after_imports: 1,
            blank_lines_around_class: 1,
            blank_lines_around_field: 0,
            blank_lines_around_method: 1,
            blank_lines_before_method_body: 0,
            blank_lines_around_field_in_interface: 0,
            blank_lines_around_method_in_interface: 1,
            blank_lines_after_class_header: 0,
            blank_lines_after_anonymous_class_header: 0,
            blank_lines_before_class_end: 0,
            blank_lines_around_initializer: 1,
            blank_lines_around_field_with_annotations: 0,
            space_around_assignment_operators: true,
            space_around_logical_operators: true,
            space_around_equality_operators: true,
            space_around_relational_operators: true,
            space_around_bitwise_operators: true,
            space_around_additive_operators: true,
            space_around_multiplicative_operators: true,
            space_around_shift_operators: true,
            space_around_unary_operator: false,
            space_around_lambda_arrow: true,
            space_around_method_ref_dbl_colon: false,
            space_after_type_cast: true,
            space_after_comma: true,
            space_after_comma_in_type_arguments: true,
            space_before_comma: false,
            space_after_semicolon: true,
            space_before_semicolon: false,
            space_before_quest: true,
            space_after_quest: true,
            space_before_colon: true,
            space_after_colon: true,
            space_before_type_parameter_list: false,
            space_before_colon_in_foreach: true,
            spaces_within_angle_brackets: false,
            space_after_closing_angle_bracket_in_type_argument: false,
            space_before_opening_angle_bracket_in_type_parameter: false,
            space_around_type_bounds_in_type_parameters: true,
            space_within_parentheses: false,
            space_within_method_call_parentheses: false,
            space_within_empty_method_call_parentheses: false,
            space_within_method_parentheses: false,
            space_within_empty_method_parentheses: false,
            space_within_if_parentheses: false,
            space_within_while_parentheses: false,
            space_within_for_parentheses: false,
            space_within_try_parentheses: false,
            space_within_catch_parentheses: false,
            space_within_switch_parentheses: false,
            space_within_synchronized_parentheses: false,
            space_within_cast_parentheses: false,
            space_within_brackets: false,
            space_within_braces: false,
            space_within_array_initializer_braces: false,
            space_within_empty_array_initializer_braces: false,
            space_within_annotation_parentheses: false,
            space_before_method_call_parentheses: false,
            space_before_method_parentheses: false,
            space_before_if_parentheses: true,
            space_before_while_parentheses: true,
            space_before_for_parentheses: true,
            space_before_try_parentheses: true,
            space_before_catch_parentheses: true,
            space_before_switch_parentheses: true,
            space_before_synchronized_parentheses: true,
            space_before_anotation_parameter_list: false,
            space_before_class_lbrace: true,
            space_before_method_lbrace: true,
            space_before_if_lbrace: true,
            space_before_else_lbrace: true,
            space_before_while_lbrace: true,
            space_before_for_lbrace: true,
            space_before_do_lbrace: true,
            space_before_switch_lbrace: true,
            space_before_try_lbrace: true,
            space_before_catch_lbrace: true,
            space_before_finally_lbrace: true,
            space_before_synchronized_lbrace: true,
            space_before_array_initializer_lbrace: false,
            space_before_annotation_array_initializer_lbrace: false,
            space_before_else_keyword: true,
            space_before_while_keyword: true,
            space_before_catch_keyword: true,
            space_before_finally_keyword: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Option registry — single source of truth for parsing, writing and the GUI
// ---------------------------------------------------------------------------

/// The scheme section an option lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// A root-level `<option>` child of `<code_scheme>`.
    Root,
    /// Inside the `<JavaCodeStyleSettings>` block.
    JavaCodeStyle,
    /// Inside `<codeStyleSettings language="JAVA">`.
    CodeStyleJava,
    /// Inside the `<indentOptions>` nested in the JAVA `codeStyleSettings` block.
    IndentOptions,
}

/// The value of a supported option, typed per the option's kind.
///
/// Not `Copy`: the [`OptionValue::String`] variant owns a `String`, and the
/// list-typed [`OptionValue::ImportLayout`] and [`OptionValue::Packages`]
/// variants hold a `Vec`. The registry's
/// [`OptionDef::default`] for those variants is an empty type tag whose real
/// value lives in [`JavaStyle::default`] (see `OptionDef::default`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionValue {
    Bool(bool),
    UInt(u32),
    /// A signed integer option. The five per-construct indent widths default
    /// to `-1` = "inherit" (use `CONTINUATION_INDENT_SIZE`), which the
    /// unsigned [`OptionValue::UInt`] cannot represent.
    Int(i32),
    Wrap(WrapStyle),
    Brace(BraceStyle),
    Force(ForceStyle),
    LineSep(LineSeparator),
    /// The ordered import-layout table: the nested `<package>` / `<emptyLine>`
    /// entries of the option's `<value>` (java.md "Import-table format").
    /// Round-trips through parse/serialize only for the nested-`<value>` XML
    /// form; the registry default is `Vec::new()` (a type tag — see
    /// [`OptionDef::default`]).
    ImportLayout(Vec<ImportLayoutEntry>),
    /// The always-on-demand package list of
    /// `PACKAGES_TO_USE_IMPORT_ON_DEMAND`, stored as bare package prefixes
    /// (the `.*` suffix is an XML-boundary concern — stripped on parse,
    /// appended on serialize). The registry default is `Vec::new()` (a type
    /// tag — see [`OptionDef::default`]).
    Packages(Vec<String>),
    /// A raw string option (e.g. `BUILDER_METHODS`' comma-separated method
    /// names). The registry default is the empty string — a real default,
    /// not a type tag.
    String(String),
}

/// Declarative description of one supported code style option — the single
/// source of truth shared by [`parse_codestyle`], [`serialize_codestyle`] and
/// the GUI. For the scalar variants each entry's `default` equals the
/// corresponding [`JavaStyle::default`] value so the serialize/parse
/// round-trip is exact. The list-typed
/// [`OptionValue::ImportLayout`] and [`OptionValue::Packages`] cannot hold
/// their real default in a `static` literal, so their `default` is an empty
/// `Vec::new()` *type tag* whose real value lives in [`JavaStyle::default`];
/// [`serialize_codestyle`] therefore
/// compares every option against `(def.get)(&JavaStyle::default())` (identical
/// to the literal default for scalars), and `parse_codestyle` matches on the
/// tag only to select the ImportLayout / Packages arm.
pub struct OptionDef {
    /// The XML `name` attribute, e.g. `"CLASS_BRACE_STYLE"`.
    pub xml_name: &'static str,
    /// The scheme section the option lives in.
    pub section: Section,
    /// The IntelliJ default value (see the struct doc for list-typed options).
    pub default: OptionValue,
    /// GUI display group, e.g. `"Braces"`.
    pub group: &'static str,
    /// Human-readable description, shown in the GUI.
    pub description: &'static str,
    /// Reads the option's current value from a style.
    pub get: fn(&JavaStyle) -> OptionValue,
    /// Writes a value back into a style (ignores a mismatched [`OptionValue`] variant).
    pub set: fn(&mut JavaStyle, OptionValue),
}

/// Every code style option java-formatter supports, in display order.
pub static OPTIONS: &[OptionDef] = &[
    // --- Indentation ---
    OptionDef {
        xml_name: "INDENT_SIZE",
        section: Section::IndentOptions,
        default: OptionValue::UInt(4),
        group: "Indentation",
        description: "Indentation width in spaces.",
        get: |s| OptionValue::UInt(s.indent_size),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.indent_size = n;
            }
        },
    },
    OptionDef {
        xml_name: "CONTINUATION_INDENT_SIZE",
        section: Section::IndentOptions,
        default: OptionValue::UInt(8),
        group: "Indentation",
        description: "Continuation-line indent width in spaces.",
        get: |s| OptionValue::UInt(s.continuation_indent_size),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.continuation_indent_size = n;
            }
        },
    },
    OptionDef {
        xml_name: "TAB_SIZE",
        section: Section::IndentOptions,
        default: OptionValue::UInt(4),
        group: "Indentation",
        description: "Width a tab is displayed and counted as.",
        get: |s| OptionValue::UInt(s.tab_size),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.tab_size = n;
            }
        },
    },
    OptionDef {
        xml_name: "USE_TAB_CHARACTER",
        section: Section::IndentOptions,
        default: OptionValue::Bool(false),
        group: "Indentation",
        description: "Indent with tab characters instead of spaces.",
        get: |s| OptionValue::Bool(s.use_tab_character),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.use_tab_character = b;
            }
        },
    },
    OptionDef {
        xml_name: "SMART_TABS",
        section: Section::IndentOptions,
        default: OptionValue::Bool(false),
        group: "Indentation",
        description: "Use tab characters only for indentation that lands exactly on tab stops; other indents use spaces.",
        get: |s| OptionValue::Bool(s.smart_tabs),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.smart_tabs = b;
            }
        },
    },
    OptionDef {
        xml_name: "LABEL_INDENT_SIZE",
        section: Section::IndentOptions,
        default: OptionValue::UInt(0),
        group: "Indentation",
        description: "Indent for `label:` statements.",
        get: |s| OptionValue::UInt(s.label_indent_size),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.label_indent_size = n;
            }
        },
    },
    OptionDef {
        xml_name: "LABEL_INDENT_ABSOLUTE",
        section: Section::IndentOptions,
        default: OptionValue::Bool(false),
        group: "Indentation",
        description: "Indent labels by LABEL_INDENT_SIZE from the margin regardless of nesting.",
        get: |s| OptionValue::Bool(s.label_indent_absolute),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.label_indent_absolute = b;
            }
        },
    },
    OptionDef {
        xml_name: "USE_RELATIVE_INDENTS",
        section: Section::IndentOptions,
        default: OptionValue::Bool(false),
        group: "Indentation",
        description: "Measure continuation indents relative to the construct's own indent level.",
        get: |s| OptionValue::Bool(s.use_relative_indents),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.use_relative_indents = b;
            }
        },
    },
    OptionDef {
        xml_name: "KEEP_INDENTS_ON_EMPTY_LINES",
        section: Section::IndentOptions,
        default: OptionValue::Bool(false),
        group: "Indentation",
        description: "Keep the block's inner indent on preserved blank lines.",
        get: |s| OptionValue::Bool(s.keep_indents_on_empty_lines),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.keep_indents_on_empty_lines = b;
            }
        },
    },
    OptionDef {
        xml_name: "DECLARATION_PARAMETER_INDENT",
        section: Section::IndentOptions,
        default: OptionValue::Int(-1),
        group: "Indentation",
        description: "Per-construct continuation indent for declaration parameters (-1 = use CONTINUATION_INDENT_SIZE).",
        get: |s| OptionValue::Int(s.declaration_parameter_indent),
        set: |s, v| {
            if let OptionValue::Int(n) = v {
                s.declaration_parameter_indent = n;
            }
        },
    },
    OptionDef {
        xml_name: "GENERIC_TYPE_PARAMETER_INDENT",
        section: Section::IndentOptions,
        default: OptionValue::Int(-1),
        group: "Indentation",
        description: "Per-construct continuation indent for generic type parameters (-1 = use CONTINUATION_INDENT_SIZE).",
        get: |s| OptionValue::Int(s.generic_type_parameter_indent),
        set: |s, v| {
            if let OptionValue::Int(n) = v {
                s.generic_type_parameter_indent = n;
            }
        },
    },
    OptionDef {
        xml_name: "CALL_PARAMETER_INDENT",
        section: Section::IndentOptions,
        default: OptionValue::Int(-1),
        group: "Indentation",
        description: "Per-construct continuation indent for call arguments (-1 = use CONTINUATION_INDENT_SIZE).",
        get: |s| OptionValue::Int(s.call_parameter_indent),
        set: |s, v| {
            if let OptionValue::Int(n) = v {
                s.call_parameter_indent = n;
            }
        },
    },
    OptionDef {
        xml_name: "CHAINED_CALL_INDENT",
        section: Section::IndentOptions,
        default: OptionValue::Int(-1),
        group: "Indentation",
        description: "Per-construct continuation indent for chained calls (-1 = use CONTINUATION_INDENT_SIZE).",
        get: |s| OptionValue::Int(s.chained_call_indent),
        set: |s, v| {
            if let OptionValue::Int(n) = v {
                s.chained_call_indent = n;
            }
        },
    },
    OptionDef {
        xml_name: "ARRAY_ELEMENT_INDENT",
        section: Section::IndentOptions,
        default: OptionValue::Int(-1),
        group: "Indentation",
        description: "Per-construct continuation indent for array elements (-1 = use CONTINUATION_INDENT_SIZE).",
        get: |s| OptionValue::Int(s.array_element_indent),
        set: |s, v| {
            if let OptionValue::Int(n) = v {
                s.array_element_indent = n;
            }
        },
    },
    // --- Margins ---
    OptionDef {
        xml_name: "RIGHT_MARGIN",
        section: Section::Root,
        default: OptionValue::UInt(120),
        group: "Margins",
        description: "Hard right margin used for line-length decisions when SOFT_MARGINS is absent.",
        get: |s| OptionValue::UInt(s.right_margin),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.right_margin = n;
            }
        },
    },
    OptionDef {
        xml_name: "LINE_SEPARATOR",
        section: Section::Root,
        default: OptionValue::LineSep(LineSeparator::System),
        group: "Margins",
        description: "Line separator emitted at every line end (System / LF / CRLF / CR).",
        get: |s| OptionValue::LineSep(s.line_separator),
        set: |s, v| {
            if let OptionValue::LineSep(sep) = v {
                s.line_separator = sep;
            }
        },
    },
    OptionDef {
        xml_name: "SOFT_MARGINS",
        section: Section::Root,
        default: OptionValue::UInt(120),
        group: "Margins",
        description: "Right margin used for line-length decisions.",
        get: |s| OptionValue::UInt(s.right_margin),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.right_margin = n;
            }
        },
    },
    // --- Braces ---
    OptionDef {
        xml_name: "CLASS_BRACE_STYLE",
        section: Section::CodeStyleJava,
        default: OptionValue::Brace(BraceStyle::EndOfLine),
        group: "Braces",
        description: "Brace placement for class / interface / enum / record bodies.",
        get: |s| OptionValue::Brace(s.class_brace_style),
        set: |s, v| {
            if let OptionValue::Brace(b) = v {
                s.class_brace_style = b;
            }
        },
    },
    OptionDef {
        xml_name: "METHOD_BRACE_STYLE",
        section: Section::CodeStyleJava,
        default: OptionValue::Brace(BraceStyle::EndOfLine),
        group: "Braces",
        description: "Brace placement for method, constructor and compact-constructor bodies.",
        get: |s| OptionValue::Brace(s.method_brace_style),
        set: |s, v| {
            if let OptionValue::Brace(b) = v {
                s.method_brace_style = b;
            }
        },
    },
    OptionDef {
        xml_name: "BRACE_STYLE",
        section: Section::CodeStyleJava,
        default: OptionValue::Brace(BraceStyle::EndOfLine),
        group: "Braces",
        description: "Brace placement for statements and other blocks.",
        get: |s| OptionValue::Brace(s.other_brace_style),
        set: |s, v| {
            if let OptionValue::Brace(b) = v {
                s.other_brace_style = b;
            }
        },
    },
    OptionDef {
        xml_name: "IF_BRACE_FORCE",
        section: Section::CodeStyleJava,
        default: OptionValue::Force(ForceStyle::DoNotForce),
        group: "Braces",
        description: "Force braces around if / else statement bodies (0 do not force, 1 force when multiline, 3 always force).",
        get: |s| OptionValue::Force(s.if_brace_force),
        set: |s, v| {
            if let OptionValue::Force(f) = v {
                s.if_brace_force = f;
            }
        },
    },
    OptionDef {
        xml_name: "FOR_BRACE_FORCE",
        section: Section::CodeStyleJava,
        default: OptionValue::Force(ForceStyle::DoNotForce),
        group: "Braces",
        description: "Force braces around for / enhanced-for statement bodies (0 do not force, 1 force when multiline, 3 always force).",
        get: |s| OptionValue::Force(s.for_brace_force),
        set: |s, v| {
            if let OptionValue::Force(f) = v {
                s.for_brace_force = f;
            }
        },
    },
    OptionDef {
        xml_name: "WHILE_BRACE_FORCE",
        section: Section::CodeStyleJava,
        default: OptionValue::Force(ForceStyle::DoNotForce),
        group: "Braces",
        description: "Force braces around while statement bodies (0 do not force, 1 force when multiline, 3 always force).",
        get: |s| OptionValue::Force(s.while_brace_force),
        set: |s, v| {
            if let OptionValue::Force(f) = v {
                s.while_brace_force = f;
            }
        },
    },
    OptionDef {
        xml_name: "DOWHILE_BRACE_FORCE",
        section: Section::CodeStyleJava,
        default: OptionValue::Force(ForceStyle::DoNotForce),
        group: "Braces",
        description: "Force braces around do / while statement bodies (0 do not force, 1 force when multiline, 3 always force).",
        get: |s| OptionValue::Force(s.dowhile_brace_force),
        set: |s, v| {
            if let OptionValue::Force(f) = v {
                s.dowhile_brace_force = f;
            }
        },
    },
    OptionDef {
        xml_name: "LAMBDA_BRACE_STYLE",
        section: Section::CodeStyleJava,
        default: OptionValue::Brace(BraceStyle::EndOfLine),
        group: "Braces",
        description: "Brace placement for lambda bodies.",
        get: |s| OptionValue::Brace(s.lambda_brace_style),
        set: |s, v| {
            if let OptionValue::Brace(b) = v {
                s.lambda_brace_style = b;
            }
        },
    },
    OptionDef {
        xml_name: "ELSE_ON_NEW_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Braces",
        description: "Put the else keyword of an if / else-if chain on a new line after the closing brace.",
        get: |s| OptionValue::Bool(s.else_on_new_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.else_on_new_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "WHILE_ON_NEW_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Braces",
        description: "Put the trailing while keyword of a do / while statement on a new line after the body.",
        get: |s| OptionValue::Bool(s.while_on_new_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.while_on_new_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "CATCH_ON_NEW_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Braces",
        description: "Put each catch clause of a try statement on a new line after the previous body.",
        get: |s| OptionValue::Bool(s.catch_on_new_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.catch_on_new_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "FINALLY_ON_NEW_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Braces",
        description: "Put the finally clause of a try statement on a new line after the previous body.",
        get: |s| OptionValue::Bool(s.finally_on_new_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.finally_on_new_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPECIAL_ELSE_IF_TREATMENT",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Braces",
        description: "Keep an else-if chain fused as `else if` instead of nesting `else { if … }`.",
        get: |s| OptionValue::Bool(s.special_else_if_treatment),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.special_else_if_treatment = b;
            }
        },
    },
    OptionDef {
        xml_name: "DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Braces",
        description: "Do not indent the members of a top-level class (they sit at the class declaration indent).",
        get: |s| OptionValue::Bool(s.do_not_indent_top_level_class_members),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.do_not_indent_top_level_class_members = b;
            }
        },
    },
    OptionDef {
        xml_name: "INDENT_CASE_FROM_SWITCH",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Braces",
        description: "Indent case / default labels one level from the switch.",
        get: |s| OptionValue::Bool(s.indent_case_from_switch),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.indent_case_from_switch = b;
            }
        },
    },
    OptionDef {
        xml_name: "CASE_STATEMENT_ON_NEW_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Braces",
        description: "Put the statement after a case label on a new line.",
        get: |s| OptionValue::Bool(s.case_statement_on_new_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.case_statement_on_new_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "INDENT_BREAK_FROM_CASE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Braces",
        description: "Indent break / continue / return statements one level from the case label.",
        get: |s| OptionValue::Bool(s.indent_break_from_case),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.indent_break_from_case = b;
            }
        },
    },
    // --- Call-site parameter wrapping ---
    OptionDef {
        xml_name: "CALL_PARAMETERS_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Call parameters",
        description: "Wrapping of method-call argument lists.",
        get: |s| OptionValue::Wrap(s.call_parameters_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.call_parameters_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "CALL_PARAMETERS_LPAREN_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Call parameters",
        description: "Put the '(' of a wrapped call on its own line.",
        get: |s| OptionValue::Bool(s.call_parameters_lparen_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.call_parameters_lparen_on_next_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "CALL_PARAMETERS_RPAREN_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Call parameters",
        description: "Put the ')' of a wrapped call on its own line.",
        get: |s| OptionValue::Bool(s.call_parameters_rparen_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.call_parameters_rparen_on_next_line = b;
            }
        },
    },
    // --- Method declaration parameter wrapping ---
    OptionDef {
        xml_name: "METHOD_PARAMETERS_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Method parameters",
        description: "Wrapping of method / constructor parameter lists.",
        get: |s| OptionValue::Wrap(s.method_parameters_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.method_parameters_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "METHOD_PARAMETERS_LPAREN_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Method parameters",
        description: "Put the '(' of a wrapped declaration on its own line.",
        get: |s| OptionValue::Bool(s.method_parameters_lparen_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.method_parameters_lparen_on_next_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "METHOD_PARAMETERS_RPAREN_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Method parameters",
        description: "Put the ')' of a wrapped declaration on its own line.",
        get: |s| OptionValue::Bool(s.method_parameters_rparen_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.method_parameters_rparen_on_next_line = b;
            }
        },
    },
    // --- Chain / annotation / assignment / binary wrapping ---
    OptionDef {
        xml_name: "METHOD_CALL_CHAIN_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Wrapping",
        description: "Wrapping of chained method calls.",
        get: |s| OptionValue::Wrap(s.method_call_chain_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.method_call_chain_wrap = w;
            }
        },
    },
    // --- Builder method chains ---
    OptionDef {
        xml_name: "BUILDER_METHODS",
        section: Section::CodeStyleJava,
        default: OptionValue::String(String::new()),
        group: "Builder methods",
        description: "Comma-separated method names whose chains are treated as builder calls for wrapping / indentation.",
        get: |s| OptionValue::String(s.builder_methods.join(",")),
        set: |s, v| {
            if let OptionValue::String(x) = v {
                s.builder_methods = x
                    .split(',')
                    .map(str::trim)
                    .filter(|m| !m.is_empty())
                    .map(str::to_string)
                    .collect();
            }
        },
    },
    OptionDef {
        xml_name: "KEEP_BUILDER_METHODS_INDENTS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Builder methods",
        description: "Keep the indentation of builder-method chains instead of stepping the continuation indent.",
        get: |s| OptionValue::Bool(s.keep_builder_methods_indents),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.keep_builder_methods_indents = b;
            }
        },
    },
    OptionDef {
        xml_name: "ASSIGNMENT_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Wrapping",
        description: "Wrapping of assignments and variable / field initialisers.",
        get: |s| OptionValue::Wrap(s.assignment_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.assignment_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "BINARY_OPERATION_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Wrapping",
        description: "Wrapping of binary expressions at their operators.",
        get: |s| OptionValue::Wrap(s.binary_operation_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.binary_operation_wrap = w;
            }
        },
    },
    // --- Expression / statement / declaration wrapping ---
    OptionDef {
        xml_name: "WRAP_FIRST_METHOD_IN_CALL_CHAIN",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Wrap after the first call in a chain as well.",
        get: |s| OptionValue::Bool(s.wrap_first_method_in_call_chain),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.wrap_first_method_in_call_chain = b;
            }
        },
    },
    OptionDef {
        xml_name: "PARENTHESES_EXPRESSION_LPAREN_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the '(' of a wrapped parenthesized expression on its own line.",
        get: |s| OptionValue::Bool(s.parentheses_expression_lparen_wrap),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.parentheses_expression_lparen_wrap = b;
            }
        },
    },
    OptionDef {
        xml_name: "PARENTHESES_EXPRESSION_RPAREN_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the ')' of a wrapped parenthesized expression on its own line.",
        get: |s| OptionValue::Bool(s.parentheses_expression_rparen_wrap),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.parentheses_expression_rparen_wrap = b;
            }
        },
    },
    OptionDef {
        xml_name: "BINARY_OPERATION_SIGN_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the operator at the start of the continuation line.",
        get: |s| OptionValue::Bool(s.binary_operation_sign_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.binary_operation_sign_on_next_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "TERNARY_OPERATION_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Wrapping",
        description: "Wrapping of ternary (?:) expressions.",
        get: |s| OptionValue::Wrap(s.ternary_operation_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.ternary_operation_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "TERNARY_OPERATION_SIGNS_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the '?' and ':' of a wrapped ternary at the start of continuation lines.",
        get: |s| OptionValue::Bool(s.ternary_operation_signs_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.ternary_operation_signs_on_next_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the assignment operator at the start of the continuation line.",
        get: |s| OptionValue::Bool(s.place_assignment_sign_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.place_assignment_sign_on_next_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "ASSERT_STATEMENT_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Wrapping",
        description: "Wrapping of assert statements.",
        get: |s| OptionValue::Wrap(s.assert_statement_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.assert_statement_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "ASSERT_STATEMENT_COLON_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the ':' of an assert statement on the next line when wrapped.",
        get: |s| OptionValue::Bool(s.assert_statement_colon_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.assert_statement_colon_on_next_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "FOR_STATEMENT_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Wrapping",
        description: "Wrapping of for headers.",
        get: |s| OptionValue::Wrap(s.for_statement_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.for_statement_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "FOR_STATEMENT_LPAREN_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the '(' of a wrapped for on its own line.",
        get: |s| OptionValue::Bool(s.for_statement_lparen_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.for_statement_lparen_on_next_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "FOR_STATEMENT_RPAREN_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the ')' of a wrapped for on its own line.",
        get: |s| OptionValue::Bool(s.for_statement_rparen_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.for_statement_rparen_on_next_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "ARRAY_INITIALIZER_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Wrapping",
        description: "Wrapping of array initializer lists.",
        get: |s| OptionValue::Wrap(s.array_initializer_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.array_initializer_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "ARRAY_INITIALIZER_LBRACE_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the '{' of a wrapped array initializer on its own line.",
        get: |s| OptionValue::Bool(s.array_initializer_lbrace_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.array_initializer_lbrace_on_next_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "ARRAY_INITIALIZER_RBRACE_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the '}' of a wrapped array initializer on its own line.",
        get: |s| OptionValue::Bool(s.array_initializer_rbrace_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.array_initializer_rbrace_on_next_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "MODIFIER_LIST_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Wrap after the modifier / annotation list of a declaration.",
        get: |s| OptionValue::Bool(s.modifier_list_wrap),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.modifier_list_wrap = b;
            }
        },
    },
    OptionDef {
        xml_name: "METHOD_ANNOTATION_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::WrapAlways),
        group: "Wrapping",
        description: "Put a method's annotations on separate lines.",
        get: |s| OptionValue::Wrap(s.method_annotation_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.method_annotation_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "CLASS_ANNOTATION_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::WrapAlways),
        group: "Wrapping",
        description: "Put a class's annotations on separate lines.",
        get: |s| OptionValue::Wrap(s.class_annotation_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.class_annotation_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "FIELD_ANNOTATION_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::WrapAlways),
        group: "Wrapping",
        description: "Put a field's annotations on separate lines.",
        get: |s| OptionValue::Wrap(s.field_annotation_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.field_annotation_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "PARAMETER_ANNOTATION_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Wrapping",
        description: "Put a parameter's annotations on separate lines.",
        get: |s| OptionValue::Wrap(s.parameter_annotation_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.parameter_annotation_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "VARIABLE_ANNOTATION_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Wrapping",
        description: "Put a local variable's annotations on separate lines.",
        get: |s| OptionValue::Wrap(s.variable_annotation_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.variable_annotation_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "ENUM_CONSTANTS_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Enums",
        description: "Wrapping of enum constant lists.",
        get: |s| OptionValue::Wrap(s.enum_constants_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.enum_constants_wrap = w;
            }
        },
    },
    // --- Declaration clause wrapping (resource / extends-implements / throws lists) ---
    OptionDef {
        xml_name: "RESOURCE_LIST_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Wrapping",
        description: "Wrapping of try-with-resources resource lists.",
        get: |s| OptionValue::Wrap(s.resource_list_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.resource_list_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "RESOURCE_LIST_LPAREN_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the '(' of a wrapped resource list on its own line.",
        get: |s| OptionValue::Bool(s.resource_list_lparen_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.resource_list_lparen_on_next_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "RESOURCE_LIST_RPAREN_ON_NEXT_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the ')' of a wrapped resource list on its own line.",
        get: |s| OptionValue::Bool(s.resource_list_rparen_on_next_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.resource_list_rparen_on_next_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "EXTENDS_LIST_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Wrapping",
        description: "Wrapping of extends / implements lists of type declarations.",
        get: |s| OptionValue::Wrap(s.extends_list_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.extends_list_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "EXTENDS_KEYWORD_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the extends / implements keyword on its own line when the list wraps.",
        get: |s| OptionValue::Bool(s.extends_keyword_wrap),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.extends_keyword_wrap = b;
            }
        },
    },
    OptionDef {
        xml_name: "THROWS_LIST_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Wrapping",
        description: "Wrapping of method / constructor throws lists.",
        get: |s| OptionValue::Wrap(s.throws_list_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.throws_list_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "THROWS_KEYWORD_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the throws keyword on its own line when the list wraps.",
        get: |s| OptionValue::Bool(s.throws_keyword_wrap),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.throws_keyword_wrap = b;
            }
        },
    },
    OptionDef {
        xml_name: "PREFER_PARAMETERS_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Prefer wrapping a call's argument list over wrapping its method-call chain.",
        get: |s| OptionValue::Bool(s.prefer_parameters_wrap),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.prefer_parameters_wrap = b;
            }
        },
    },
    OptionDef {
        xml_name: "WRAP_LONG_LINES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Hard-wrap lines longer than the right margin at a whitespace boundary.",
        get: |s| OptionValue::Bool(s.wrap_long_lines),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.wrap_long_lines = b;
            }
        },
    },
    OptionDef {
        xml_name: "KEEP_LINE_BREAKS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Wrapping",
        description: "Keep a construct's existing line breaks instead of joining it onto one line.",
        get: |s| OptionValue::Bool(s.keep_line_breaks),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.keep_line_breaks = b;
            }
        },
    },
    OptionDef {
        xml_name: "SWITCH_EXPRESSIONS_WRAP",
        section: Section::CodeStyleJava,
        default: OptionValue::Wrap(WrapStyle::WrapIfLong),
        group: "Wrapping",
        description: "Wrapping of switch expressions used as values.",
        get: |s| OptionValue::Wrap(s.switch_expressions_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.switch_expressions_wrap = w;
            }
        },
    },
    // --- Alignment (align-when-multiline options) ---
    OptionDef {
        xml_name: "ALIGN_MULTILINE_PARAMETERS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Alignment",
        description: "Align wrapped method declaration parameters under the first parameter.",
        get: |s| OptionValue::Bool(s.align_multiline_parameters),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_parameters = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_PARAMETERS_IN_CALLS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align wrapped method-call arguments under the first argument.",
        get: |s| OptionValue::Bool(s.align_multiline_parameters_in_calls),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_parameters_in_calls = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_RESOURCES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Alignment",
        description: "Align wrapped try-with-resources clauses under the first resource.",
        get: |s| OptionValue::Bool(s.align_multiline_resources),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_resources = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_FOR",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Alignment",
        description: "Align wrapped for header parts under the first part.",
        get: |s| OptionValue::Bool(s.align_multiline_for),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_for = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_BINARY_OPERATION",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align wrapped binary expression operands under the first operand.",
        get: |s| OptionValue::Bool(s.align_multiline_binary_operation),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_binary_operation = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_ASSIGNMENT",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align a wrapped assignment's right-hand side under the assignment start.",
        get: |s| OptionValue::Bool(s.align_multiline_assignment),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_assignment = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_TERNARY_OPERATION",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align wrapped ternary operands under the condition.",
        get: |s| OptionValue::Bool(s.align_multiline_ternary_operation),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_ternary_operation = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_THROWS_LIST",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align wrapped throws list entries under the first exception.",
        get: |s| OptionValue::Bool(s.align_multiline_throws_list),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_throws_list = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_THROWS_KEYWORD",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align a wrapped throws clause's keyword with the exception column.",
        get: |s| OptionValue::Bool(s.align_throws_keyword),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_throws_keyword = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_EXTENDS_LIST",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align wrapped extends / implements list entries under the first type.",
        get: |s| OptionValue::Bool(s.align_multiline_extends_list),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_extends_list = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_METHOD_BRACKETS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align the closing paren of a wrapped declaration under its opening paren.",
        get: |s| OptionValue::Bool(s.align_multiline_method_brackets),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_method_brackets = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_PARENTHESIZED_EXPRESSION",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align a wrapped parenthesized expression's continuation under the '('.",
        get: |s| OptionValue::Bool(s.align_multiline_parenthesized_expression),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_parenthesized_expression = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_ARRAY_INITIALIZER_EXPRESSION",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align wrapped array initializer entries under the first entry.",
        get: |s| OptionValue::Bool(s.align_multiline_array_initializer_expression),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_array_initializer_expression = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_CHAINED_METHODS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align wrapped chained-call dots under the first call's dot.",
        get: |s| OptionValue::Bool(s.align_multiline_chained_methods),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_chained_methods = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_GROUP_FIELD_DECLARATIONS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align consecutive field declarations in columns.",
        get: |s| OptionValue::Bool(s.align_group_field_declarations),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_group_field_declarations = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_CONSECUTIVE_VARIABLE_DECLARATIONS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align consecutive local variable declarations in columns.",
        get: |s| OptionValue::Bool(s.align_consecutive_variable_declarations),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_consecutive_variable_declarations = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_CONSECUTIVE_ASSIGNMENTS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align consecutive assignment statements in columns.",
        get: |s| OptionValue::Bool(s.align_consecutive_assignments),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_consecutive_assignments = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_SUBSEQUENT_SIMPLE_METHODS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Alignment",
        description: "Align consecutive one-line methods' names in columns.",
        get: |s| OptionValue::Bool(s.align_subsequent_simple_methods),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_subsequent_simple_methods = b;
            }
        },
    },
    // --- Comments ---
    OptionDef {
        xml_name: "LINE_COMMENT_AT_FIRST_COLUMN",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Comments",
        description: "Place // line comments at the first column (no indent).",
        get: |s| OptionValue::Bool(s.line_comment_at_first_column),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.line_comment_at_first_column = b;
            }
        },
    },
    OptionDef {
        xml_name: "BLOCK_COMMENT_AT_FIRST_COLUMN",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Comments",
        description: "Place /* */ block comments at the first column.",
        get: |s| OptionValue::Bool(s.block_comment_at_first_column),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.block_comment_at_first_column = b;
            }
        },
    },
    OptionDef {
        xml_name: "LINE_COMMENT_ADD_SPACE_ON_REFORMAT",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Comments",
        description: "Add the space after // on reformat.",
        get: |s| OptionValue::Bool(s.line_comment_add_space_on_reformat),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.line_comment_add_space_on_reformat = b;
            }
        },
    },
    OptionDef {
        xml_name: "LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Comments",
        description: "Add the space inside //noinspection suppression comments.",
        get: |s| OptionValue::Bool(s.line_comment_add_space_in_suppression),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.line_comment_add_space_in_suppression = b;
            }
        },
    },
    OptionDef {
        xml_name: "KEEP_FIRST_COLUMN_COMMENT",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Comments",
        description: "Keep comments that start in the first column at the first column.",
        get: |s| OptionValue::Bool(s.keep_first_column_comment),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.keep_first_column_comment = b;
            }
        },
    },
    OptionDef {
        xml_name: "WRAP_COMMENTS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Comments",
        description: "Wrap long comments to the right margin.",
        get: |s| OptionValue::Bool(s.wrap_comments),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.wrap_comments = b;
            }
        },
    },
    // --- Javadoc (JavaCodeStyleSettings) ---
    OptionDef {
        xml_name: "ENABLE_JAVADOC_FORMATTING",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Javadoc",
        description: "Reformat javadoc comments at all.",
        get: |s| OptionValue::Bool(s.enable_javadoc_formatting),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.enable_javadoc_formatting = b;
            }
        },
    },
    OptionDef {
        xml_name: "CLASS_NAMES_IN_JAVADOC",
        section: Section::JavaCodeStyle,
        default: OptionValue::UInt(1),
        group: "Javadoc",
        description: "Class-name treatment inside javadoc (1 fully qualify if not imported, 2 always fully qualify, 3 shorten and add import).",
        get: |s| OptionValue::UInt(s.class_names_in_javadoc),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.class_names_in_javadoc = n;
            }
        },
    },
    OptionDef {
        xml_name: "JD_ALIGN_PARAM_COMMENTS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Javadoc",
        description: "Align @param descriptions in a column.",
        get: |s| OptionValue::Bool(s.jd_align_param_comments),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_align_param_comments = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_ALIGN_EXCEPTION_COMMENTS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Javadoc",
        description: "Align @throws / @exception descriptions in a column.",
        get: |s| OptionValue::Bool(s.jd_align_exception_comments),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_align_exception_comments = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_ADD_BLANK_AFTER_PARM_COMMENTS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Javadoc",
        description: "Blank line after the @param block.",
        get: |s| OptionValue::Bool(s.jd_add_blank_after_parm_comments),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_add_blank_after_parm_comments = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_ADD_BLANK_AFTER_RETURN",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Javadoc",
        description: "Blank line after the @return tag.",
        get: |s| OptionValue::Bool(s.jd_add_blank_after_return),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_add_blank_after_return = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_ADD_BLANK_AFTER_DESCRIPTION",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Javadoc",
        description: "Blank line after the description paragraph.",
        get: |s| OptionValue::Bool(s.jd_add_blank_after_description),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_add_blank_after_description = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_P_AT_EMPTY_LINES",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Javadoc",
        description: "Put <p> on empty lines.",
        get: |s| OptionValue::Bool(s.jd_p_at_empty_lines),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_p_at_empty_lines = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_KEEP_INVALID_TAGS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Javadoc",
        description: "Keep invalid / unknown tags.",
        get: |s| OptionValue::Bool(s.jd_keep_invalid_tags),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_keep_invalid_tags = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_KEEP_EMPTY_LINES",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Javadoc",
        description: "Keep empty lines inside javadoc.",
        get: |s| OptionValue::Bool(s.jd_keep_empty_lines),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_keep_empty_lines = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_DO_NOT_WRAP_ONE_LINE_COMMENTS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Javadoc",
        description: "Do not wrap one-line javadoc comments.",
        get: |s| OptionValue::Bool(s.jd_do_not_wrap_one_line_comments),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_do_not_wrap_one_line_comments = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_USE_THROWS_NOT_EXCEPTION",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Javadoc",
        description: "Use @throws rather than @exception.",
        get: |s| OptionValue::Bool(s.jd_use_throws_not_exception),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_use_throws_not_exception = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_KEEP_EMPTY_PARAMETER",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Javadoc",
        description: "Keep empty @param tags.",
        get: |s| OptionValue::Bool(s.jd_keep_empty_parameter),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_keep_empty_parameter = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_KEEP_EMPTY_EXCEPTION",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Javadoc",
        description: "Keep empty @throws / @exception tags.",
        get: |s| OptionValue::Bool(s.jd_keep_empty_exception),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_keep_empty_exception = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_KEEP_EMPTY_RETURN",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Javadoc",
        description: "Keep empty @return tags.",
        get: |s| OptionValue::Bool(s.jd_keep_empty_return),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_keep_empty_return = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_LEADING_ASTERISKS_ARE_ENABLED",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Javadoc",
        description: "Render javadoc with leading * on every line.",
        get: |s| OptionValue::Bool(s.jd_leading_asterisks_are_enabled),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_leading_asterisks_are_enabled = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_PRESERVE_LINE_FEEDS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Javadoc",
        description: "Preserve line breaks inside javadoc.",
        get: |s| OptionValue::Bool(s.jd_preserve_line_feeds),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_preserve_line_feeds = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_PARAM_DESCRIPTION_ON_NEW_LINE",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Javadoc",
        description: "Put @param descriptions on a new line.",
        get: |s| OptionValue::Bool(s.jd_param_description_on_new_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_param_description_on_new_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "JD_INDENT_ON_CONTINUATION",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Javadoc",
        description: "Indent javadoc continuation lines.",
        get: |s| OptionValue::Bool(s.jd_indent_on_continuation),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.jd_indent_on_continuation = b;
            }
        },
    },
    // --- One-liners ---
    OptionDef {
        xml_name: "KEEP_SIMPLE_BLOCKS_IN_ONE_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "One-liners",
        description: "Keep single-statement if/else/for/while/do, try/catch/finally and synchronized blocks on one line.",
        get: |s| OptionValue::Bool(s.keep_simple_blocks_in_one_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.keep_simple_blocks_in_one_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "KEEP_SIMPLE_METHODS_IN_ONE_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "One-liners",
        description: "Keep single-statement method / constructor bodies on one line.",
        get: |s| OptionValue::Bool(s.keep_simple_methods_in_one_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.keep_simple_methods_in_one_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "KEEP_SIMPLE_LAMBDAS_IN_ONE_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "One-liners",
        description: "Keep single-statement lambda bodies on one line.",
        get: |s| OptionValue::Bool(s.keep_simple_lambdas_in_one_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.keep_simple_lambdas_in_one_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "KEEP_SIMPLE_CLASSES_IN_ONE_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "One-liners",
        description: "Keep simple class / interface / record bodies on one line.",
        get: |s| OptionValue::Bool(s.keep_simple_classes_in_one_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.keep_simple_classes_in_one_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "One-liners",
        description: "Keep multiple expressions (e.g. in a `for` header) on one line.",
        get: |s| OptionValue::Bool(s.keep_multiple_expressions_in_one_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.keep_multiple_expressions_in_one_line = b;
            }
        },
    },
    OptionDef {
        xml_name: "KEEP_CONTROL_STATEMENT_IN_ONE_LINE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "One-liners",
        description: "Keep a brace-less control-statement body on the header's line when the source has it there.",
        get: |s| OptionValue::Bool(s.keep_control_statement_in_one_line),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.keep_control_statement_in_one_line = b;
            }
        },
    },
    // --- One-line block body presentation (JavaCodeStyleSettings) ---
    OptionDef {
        xml_name: "SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "One-liners",
        description: "Spaces inside { } of a non-empty one-line block when SPACE_WITHIN_BRACES is off (flush {s} when off).",
        get: |s| OptionValue::Bool(s.spaces_inside_block_braces_when_body_is_present),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.spaces_inside_block_braces_when_body_is_present = b;
            }
        },
    },
    OptionDef {
        xml_name: "NEW_LINE_WHEN_BODY_IS_PRESENTED",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "One-liners",
        description: "Put the body of a one-line block on a new line below its statement head.",
        get: |s| OptionValue::Bool(s.new_line_when_body_is_presented),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.new_line_when_body_is_presented = b;
            }
        },
    },
    // --- Records (JavaCodeStyleSettings) ---
    OptionDef {
        xml_name: "ANNOTATION_PARAMETER_WRAP",
        section: Section::JavaCodeStyle,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Records & annotations",
        description: "Wrapping of annotation argument lists.",
        get: |s| OptionValue::Wrap(s.annotation_parameter_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.annotation_parameter_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "ENUM_FIELD_ANNOTATION_WRAP",
        section: Section::JavaCodeStyle,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Records & annotations",
        description: "Put annotations on enum constants on their own lines.",
        get: |s| OptionValue::Wrap(s.enum_field_annotation_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.enum_field_annotation_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_INSIDE_ONE_LINE_ENUM_BRACES",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Enums",
        description: "Spaces inside the braces of a one-line enum body.",
        get: |s| OptionValue::Bool(s.space_inside_one_line_enum_braces),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_inside_one_line_enum_braces = b;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_ANNOTATION_PARAMETERS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Records & annotations",
        description: "Align wrapped annotation parameters under the first parameter.",
        get: |s| OptionValue::Bool(s.align_multiline_annotation_parameters),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_annotation_parameters = b;
            }
        },
    },
    OptionDef {
        xml_name: "NEW_LINE_AFTER_LPAREN_IN_ANNOTATION",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Records & annotations",
        description: "Put the '(' of a wrapped annotation on its own line.",
        get: |s| OptionValue::Bool(s.new_line_after_lparen_in_annotation),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.new_line_after_lparen_in_annotation = b;
            }
        },
    },
    OptionDef {
        xml_name: "RPAREN_ON_NEW_LINE_IN_ANNOTATION",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Records & annotations",
        description: "Put the ')' of a wrapped annotation on its own line.",
        get: |s| OptionValue::Bool(s.rparen_on_new_line_in_annotation),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.rparen_on_new_line_in_annotation = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AROUND_ANNOTATION_EQ",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Records & annotations",
        description: "Spaces around '=' in annotation arguments.",
        get: |s| OptionValue::Bool(s.space_around_annotation_eq),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_around_annotation_eq = b;
            }
        },
    },
    OptionDef {
        xml_name: "DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Records & annotations",
        description: "Do not wrap after a single annotation on a field.",
        get: |s| OptionValue::Bool(s.do_not_wrap_after_single_annotation),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.do_not_wrap_after_single_annotation = b;
            }
        },
    },
    OptionDef {
        xml_name: "DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION_IN_PARAMETER",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Records & annotations",
        description: "Do not wrap after a single annotation on a parameter.",
        get: |s| OptionValue::Bool(s.do_not_wrap_after_single_annotation_in_parameter),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.do_not_wrap_after_single_annotation_in_parameter = b;
            }
        },
    },
    OptionDef {
        xml_name: "RECORD_COMPONENTS_WRAP",
        section: Section::JavaCodeStyle,
        default: OptionValue::Wrap(WrapStyle::DoNotWrap),
        group: "Records & annotations",
        description: "Wrapping of record component lists.",
        get: |s| OptionValue::Wrap(s.record_components_wrap),
        set: |s, v| {
            if let OptionValue::Wrap(w) = v {
                s.record_components_wrap = w;
            }
        },
    },
    OptionDef {
        xml_name: "ALIGN_MULTILINE_RECORDS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Records & annotations",
        description: "Align wrapped record components under the first component.",
        get: |s| OptionValue::Bool(s.align_multiline_records),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.align_multiline_records = b;
            }
        },
    },
    OptionDef {
        xml_name: "NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Records & annotations",
        description: "Put the '(' of a wrapped record header on its own line.",
        get: |s| OptionValue::Bool(s.new_line_after_lparen_in_record_header),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.new_line_after_lparen_in_record_header = b;
            }
        },
    },
    OptionDef {
        xml_name: "RPAREN_ON_NEW_LINE_IN_RECORD_HEADER",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Records & annotations",
        description: "Put the ')' of a wrapped record header on its own line.",
        get: |s| OptionValue::Bool(s.rparen_on_new_line_in_record_header),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.rparen_on_new_line_in_record_header = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_RECORD_HEADER",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Records & annotations",
        description: "Put one space just inside the parens of a record header.",
        get: |s| OptionValue::Bool(s.space_within_record_header),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_record_header = b;
            }
        },
    },
    OptionDef {
        xml_name: "ANNOTATION_NEW_LINE_IN_RECORD_COMPONENT",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Records & annotations",
        description: "Put a wrapped record component's annotations on their own lines.",
        get: |s| OptionValue::Bool(s.annotation_new_line_in_record_component),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.annotation_new_line_in_record_component = b;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_BETWEEN_RECORD_COMPONENTS",
        section: Section::JavaCodeStyle,
        default: OptionValue::UInt(0),
        group: "Records & annotations",
        description: "Blank lines between the components of a wrapped record header.",
        get: |s| OptionValue::UInt(s.blank_lines_between_record_components),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_between_record_components = n;
            }
        },
    },
    // --- Imports (JavaCodeStyleSettings) ---
    OptionDef {
        xml_name: "CLASS_COUNT_TO_USE_IMPORT_ON_DEMAND",
        section: Section::JavaCodeStyle,
        default: OptionValue::UInt(5),
        group: "Imports",
        description: "Merge a package's single-type imports into pkg.* at this count.",
        get: |s| OptionValue::UInt(s.class_count_to_use_import_on_demand),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.class_count_to_use_import_on_demand = n;
            }
        },
    },
    OptionDef {
        xml_name: "NAMES_COUNT_TO_USE_IMPORT_ON_DEMAND",
        section: Section::JavaCodeStyle,
        default: OptionValue::UInt(3),
        group: "Imports",
        description: "Merge one owner's static member imports into static pkg.Owner.* above this count.",
        get: |s| OptionValue::UInt(s.names_count_to_use_import_on_demand),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.names_count_to_use_import_on_demand = n;
            }
        },
    },
    OptionDef {
        xml_name: "PACKAGES_TO_USE_IMPORT_ON_DEMAND",
        section: Section::JavaCodeStyle,
        default: OptionValue::Packages(Vec::new()),
        group: "Imports",
        description: "Packages whose single-type imports always merge into pkg.* on demand (nested list of pkg.* entries).",
        get: |s| OptionValue::Packages(s.packages_to_use_import_on_demand.clone()),
        set: |s, v| {
            if let OptionValue::Packages(p) = v {
                s.packages_to_use_import_on_demand = p;
            }
        },
    },
    OptionDef {
        xml_name: "USE_SINGLE_CLASS_IMPORTS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Imports",
        description: "Prefer single-class imports; off, every eligible package merges into pkg.* on demand.",
        get: |s| OptionValue::Bool(s.use_single_class_imports),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.use_single_class_imports = b;
            }
        },
    },
    OptionDef {
        xml_name: "IMPORT_LAYOUT_TABLE",
        section: Section::JavaCodeStyle,
        default: OptionValue::ImportLayout(Vec::new()),
        group: "Imports",
        description: "Ordering and grouping of the import section: <package> and <emptyLine> entries (java.md Import-table format).",
        get: |s| OptionValue::ImportLayout(s.import_layout.clone()),
        set: |s, v| {
            if let OptionValue::ImportLayout(l) = v {
                s.import_layout = l;
            }
        },
    },
    OptionDef {
        xml_name: "LAYOUT_STATIC_IMPORTS_SEPARATELY",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Imports",
        description: "Keep static imports in their own section (the table's static=\"true\" entries); off, they join the ordinary package sections.",
        get: |s| OptionValue::Bool(s.layout_static_imports_separately),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.layout_static_imports_separately = b;
            }
        },
    },
    OptionDef {
        xml_name: "LAYOUT_ON_DEMAND_IMPORT_FROM_SAME_PACKAGE_FIRST",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Imports",
        description: "Put the file's own-package on-demand (pkg.*) import before the other imports of its group.",
        get: |s| OptionValue::Bool(s.layout_on_demand_import_from_same_package_first),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.layout_on_demand_import_from_same_package_first = b;
            }
        },
    },
    OptionDef {
        xml_name: "PRESERVE_MODULE_IMPORTS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Imports",
        description: "Keep `import module …;` lines on reformat, placed at the layout table's module slot.",
        get: |s| OptionValue::Bool(s.preserve_module_imports),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.preserve_module_imports = b;
            }
        },
    },
    OptionDef {
        xml_name: "DELETE_UNUSED_MODULE_IMPORTS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Imports",
        description: "Remove clearly-unused module imports on reformat (conservative: duplicates beyond the first).",
        get: |s| OptionValue::Bool(s.delete_unused_module_imports),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.delete_unused_module_imports = b;
            }
        },
    },
    OptionDef {
        xml_name: "KEEP_BLANK_LINES_BETWEEN_IMPORTS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Imports",
        description: "Preserve source blank lines between the imports of one group on reformat.",
        get: |s| OptionValue::Bool(s.keep_blank_lines_between_imports),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.keep_blank_lines_between_imports = b;
            }
        },
    },
    // --- Blank lines: KEEP_BLANK_LINES_* caps (CodeStyleJava) ---
    OptionDef {
        xml_name: "KEEP_BLANK_LINES_IN_CODE",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(2),
        group: "Blank lines",
        description: "Max blank lines kept inside code (statement level).",
        get: |s| OptionValue::UInt(s.keep_blank_lines_in_code),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.keep_blank_lines_in_code = n;
            }
        },
    },
    OptionDef {
        xml_name: "KEEP_BLANK_LINES_IN_DECLARATIONS",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(2),
        group: "Blank lines",
        description: "Max blank lines kept between declarations.",
        get: |s| OptionValue::UInt(s.keep_blank_lines_in_declarations),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.keep_blank_lines_in_declarations = n;
            }
        },
    },
    OptionDef {
        xml_name: "KEEP_BLANK_LINES_BETWEEN_PACKAGE_DECLARATION_AND_HEADER",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(2),
        group: "Blank lines",
        description: "Max blank lines kept between the package declaration and a file header comment.",
        get: |s| OptionValue::UInt(s.keep_blank_lines_between_package_declaration_and_header),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.keep_blank_lines_between_package_declaration_and_header = n;
            }
        },
    },
    OptionDef {
        xml_name: "KEEP_BLANK_LINES_BEFORE_RBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(2),
        group: "Blank lines",
        description: "Max blank lines kept before a closing }.",
        get: |s| OptionValue::UInt(s.keep_blank_lines_before_rbrace),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.keep_blank_lines_before_rbrace = n;
            }
        },
    },
    // --- Blank lines: BLANK_LINES_* minimums (CodeStyleJava) ---
    OptionDef {
        xml_name: "BLANK_LINES_BEFORE_PACKAGE",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(0),
        group: "Blank lines",
        description: "Min blank lines before the package declaration.",
        get: |s| OptionValue::UInt(s.blank_lines_before_package),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_before_package = n;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_AFTER_PACKAGE",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(1),
        group: "Blank lines",
        description: "Min blank lines after the package declaration.",
        get: |s| OptionValue::UInt(s.blank_lines_after_package),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_after_package = n;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_BEFORE_IMPORTS",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(1),
        group: "Blank lines",
        description: "Min blank lines before the import section.",
        get: |s| OptionValue::UInt(s.blank_lines_before_imports),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_before_imports = n;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_AFTER_IMPORTS",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(1),
        group: "Blank lines",
        description: "Min blank lines after the import section.",
        get: |s| OptionValue::UInt(s.blank_lines_after_imports),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_after_imports = n;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_AROUND_CLASS",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(1),
        group: "Blank lines",
        description: "Min blank lines around class / interface declarations.",
        get: |s| OptionValue::UInt(s.blank_lines_around_class),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_around_class = n;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_AROUND_FIELD",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(0),
        group: "Blank lines",
        description: "Min blank lines around fields.",
        get: |s| OptionValue::UInt(s.blank_lines_around_field),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_around_field = n;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_AROUND_METHOD",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(1),
        group: "Blank lines",
        description: "Min blank lines around methods.",
        get: |s| OptionValue::UInt(s.blank_lines_around_method),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_around_method = n;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_BEFORE_METHOD_BODY",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(0),
        group: "Blank lines",
        description: "Min blank lines before a method body.",
        get: |s| OptionValue::UInt(s.blank_lines_before_method_body),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_before_method_body = n;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_AROUND_FIELD_IN_INTERFACE",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(0),
        group: "Blank lines",
        description: "Min blank lines around fields declared in interfaces.",
        get: |s| OptionValue::UInt(s.blank_lines_around_field_in_interface),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_around_field_in_interface = n;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_AROUND_METHOD_IN_INTERFACE",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(1),
        group: "Blank lines",
        description: "Min blank lines around methods declared in interfaces.",
        get: |s| OptionValue::UInt(s.blank_lines_around_method_in_interface),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_around_method_in_interface = n;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_AFTER_CLASS_HEADER",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(0),
        group: "Blank lines",
        description: "Min blank lines after the class header / before the first member.",
        get: |s| OptionValue::UInt(s.blank_lines_after_class_header),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_after_class_header = n;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_AFTER_ANONYMOUS_CLASS_HEADER",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(0),
        group: "Blank lines",
        description: "Min blank lines after an anonymous class header.",
        get: |s| OptionValue::UInt(s.blank_lines_after_anonymous_class_header),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_after_anonymous_class_header = n;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_BEFORE_CLASS_END",
        section: Section::CodeStyleJava,
        default: OptionValue::UInt(0),
        group: "Blank lines",
        description: "Min blank lines before the class closing brace.",
        get: |s| OptionValue::UInt(s.blank_lines_before_class_end),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_before_class_end = n;
            }
        },
    },
    // --- Blank lines: Java-specific minimums (JavaCodeStyleSettings) ---
    OptionDef {
        xml_name: "BLANK_LINES_AROUND_INITIALIZER",
        section: Section::JavaCodeStyle,
        default: OptionValue::UInt(1),
        group: "Blank lines",
        description: "Min blank lines around instance / static initializer blocks.",
        get: |s| OptionValue::UInt(s.blank_lines_around_initializer),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_around_initializer = n;
            }
        },
    },
    OptionDef {
        xml_name: "BLANK_LINES_AROUND_FIELD_WITH_ANNOTATIONS",
        section: Section::JavaCodeStyle,
        default: OptionValue::UInt(0),
        group: "Blank lines",
        description: "Min blank lines around annotated fields.",
        get: |s| OptionValue::UInt(s.blank_lines_around_field_with_annotations),
        set: |s, v| {
            if let OptionValue::UInt(n) = v {
                s.blank_lines_around_field_with_annotations = n;
            }
        },
    },
    OptionDef {
        xml_name: "WRAP_SEMICOLON_AFTER_CALL_CHAIN",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Wrapping",
        description: "Put the ';' of a wrapped chained call on its own line.",
        get: |s| OptionValue::Bool(s.wrap_semicolon_after_call_chain),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.wrap_semicolon_after_call_chain = b;
            }
        },
    },
    // --- Operator spacing (Spaces / Around operators) ---
    OptionDef {
        xml_name: "SPACE_AROUND_ASSIGNMENT_OPERATORS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space around assignment operators (=, +=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=, >>>=).",
        get: |s| OptionValue::Bool(s.space_around_assignment_operators),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_around_assignment_operators = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AROUND_LOGICAL_OPERATORS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space around logical operators (&&, ||).",
        get: |s| OptionValue::Bool(s.space_around_logical_operators),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_around_logical_operators = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AROUND_EQUALITY_OPERATORS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space around equality operators (==, !=).",
        get: |s| OptionValue::Bool(s.space_around_equality_operators),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_around_equality_operators = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AROUND_RELATIONAL_OPERATORS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space around relational operators (<, >, <=, >=).",
        get: |s| OptionValue::Bool(s.space_around_relational_operators),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_around_relational_operators = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AROUND_BITWISE_OPERATORS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space around bitwise operators (&, |, ^).",
        get: |s| OptionValue::Bool(s.space_around_bitwise_operators),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_around_bitwise_operators = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AROUND_ADDITIVE_OPERATORS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space around additive operators (+, -).",
        get: |s| OptionValue::Bool(s.space_around_additive_operators),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_around_additive_operators = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AROUND_MULTIPLICATIVE_OPERATORS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space around multiplicative operators (*, /, %).",
        get: |s| OptionValue::Bool(s.space_around_multiplicative_operators),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_around_multiplicative_operators = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AROUND_SHIFT_OPERATORS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space around shift operators (<<, >>, >>>).",
        get: |s| OptionValue::Bool(s.space_around_shift_operators),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_around_shift_operators = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AROUND_UNARY_OPERATOR",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space between a unary operator (!, ~, unary +/-, ++, --) and its operand.",
        get: |s| OptionValue::Bool(s.space_around_unary_operator),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_around_unary_operator = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AROUND_LAMBDA_ARROW",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space around the lambda arrow (->).",
        get: |s| OptionValue::Bool(s.space_around_lambda_arrow),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_around_lambda_arrow = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AROUND_METHOD_REF_DBL_COLON",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space around the method-reference separator (::).",
        get: |s| OptionValue::Bool(s.space_around_method_ref_dbl_colon),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_around_method_ref_dbl_colon = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AFTER_TYPE_CAST",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space after a type cast, between (Type) and the cast value.",
        get: |s| OptionValue::Bool(s.space_after_type_cast),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_after_type_cast = b;
            }
        },
    },
    // --- Separator spacing (Spaces / After & before separators) ---
    OptionDef {
        xml_name: "SPACE_AFTER_COMMA",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space after a comma (declarations, calls, arrays).",
        get: |s| OptionValue::Bool(s.space_after_comma),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_after_comma = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space after a comma in generic type arguments.",
        get: |s| OptionValue::Bool(s.space_after_comma_in_type_arguments),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_after_comma_in_type_arguments = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_COMMA",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space before a comma.",
        get: |s| OptionValue::Bool(s.space_before_comma),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_comma = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AFTER_SEMICOLON",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space after a semicolon inside a for header.",
        get: |s| OptionValue::Bool(s.space_after_semicolon),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_after_semicolon = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_SEMICOLON",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space before a semicolon inside a for header.",
        get: |s| OptionValue::Bool(s.space_before_semicolon),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_semicolon = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_QUEST",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space before a question mark in a ternary expression.",
        get: |s| OptionValue::Bool(s.space_before_quest),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_quest = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AFTER_QUEST",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space after a question mark in a ternary expression.",
        get: |s| OptionValue::Bool(s.space_after_quest),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_after_quest = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_COLON",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space before a colon in a ternary expression.",
        get: |s| OptionValue::Bool(s.space_before_colon),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_colon = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AFTER_COLON",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space after a colon.",
        get: |s| OptionValue::Bool(s.space_after_colon),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_after_colon = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_TYPE_PARAMETER_LIST",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space between a class / interface / record name and its type-parameter list.",
        get: |s| OptionValue::Bool(s.space_before_type_parameter_list),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_type_parameter_list = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACES_WITHIN_ANGLE_BRACKETS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Spaces inside the angle brackets of type arguments and type parameters.",
        get: |s| OptionValue::Bool(s.spaces_within_angle_brackets),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.spaces_within_angle_brackets = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AFTER_CLOSING_ANGLE_BRACKET_IN_TYPE_ARGUMENT",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space after a closing angle bracket in an explicit type-argument list.",
        get: |s| OptionValue::Bool(s.space_after_closing_angle_bracket_in_type_argument),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_after_closing_angle_bracket_in_type_argument = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_OPENING_ANGLE_BRACKET_IN_TYPE_PARAMETER",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space between a class / interface / record name and its type-parameter list.",
        get: |s| OptionValue::Bool(s.space_before_opening_angle_bracket_in_type_parameter),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_opening_angle_bracket_in_type_parameter = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_AROUND_TYPE_BOUNDS_IN_TYPE_PARAMETERS",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Spaces around the `&`-joined bounds of a type parameter.",
        get: |s| OptionValue::Bool(s.space_around_type_bounds_in_type_parameters),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_around_type_bounds_in_type_parameters = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_COLON_IN_FOREACH",
        section: Section::JavaCodeStyle,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space before the colon in an enhanced-for header.",
        get: |s| OptionValue::Bool(s.space_before_colon_in_foreach),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_colon_in_foreach = b;
            }
        },
    },
    // --- spacing within parens / brackets / braces ---
    OptionDef {
        xml_name: "SPACE_WITHIN_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside any parentheses `( expr )`.",
        get: |s| OptionValue::Bool(s.space_within_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_METHOD_CALL_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside method-call parentheses `f( args )`.",
        get: |s| OptionValue::Bool(s.space_within_method_call_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_method_call_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_EMPTY_METHOD_CALL_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside empty method-call parentheses `f( )` vs `f()`.",
        get: |s| OptionValue::Bool(s.space_within_empty_method_call_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_empty_method_call_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_METHOD_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside method-declaration parentheses `void f( params )`.",
        get: |s| OptionValue::Bool(s.space_within_method_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_method_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_EMPTY_METHOD_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside empty method-declaration parentheses `void f( )` vs `void f()`.",
        get: |s| OptionValue::Bool(s.space_within_empty_method_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_empty_method_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_IF_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside if-condition parentheses `if( cond )`.",
        get: |s| OptionValue::Bool(s.space_within_if_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_if_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_WHILE_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside while / do-while parentheses `while( cond )`.",
        get: |s| OptionValue::Bool(s.space_within_while_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_while_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_FOR_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside for-header parentheses `for( … )`.",
        get: |s| OptionValue::Bool(s.space_within_for_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_for_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_TRY_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside try-with-resources parentheses `try( resource )`.",
        get: |s| OptionValue::Bool(s.space_within_try_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_try_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_CATCH_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside catch parentheses `catch( exc )`.",
        get: |s| OptionValue::Bool(s.space_within_catch_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_catch_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_SWITCH_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside switch parentheses `switch( expr )`.",
        get: |s| OptionValue::Bool(s.space_within_switch_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_switch_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_SYNCHRONIZED_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside synchronized parentheses `synchronized( expr )`.",
        get: |s| OptionValue::Bool(s.space_within_synchronized_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_synchronized_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_CAST_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside cast parentheses `( Type ) expr`.",
        get: |s| OptionValue::Bool(s.space_within_cast_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_cast_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_BRACKETS",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside brackets `[ expr ]` in array indexing.",
        get: |s| OptionValue::Bool(s.space_within_brackets),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_brackets = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_BRACES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside code-block braces `{ … }`.",
        get: |s| OptionValue::Bool(s.space_within_braces),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_braces = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_ARRAY_INITIALIZER_BRACES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside array-initializer braces `{ 1, 3, 5 }`.",
        get: |s| OptionValue::Bool(s.space_within_array_initializer_braces),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_array_initializer_braces = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_EMPTY_ARRAY_INITIALIZER_BRACES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside empty array-initializer braces `{ }` vs `{}`.",
        get: |s| OptionValue::Bool(s.space_within_empty_array_initializer_braces),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_empty_array_initializer_braces = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_WITHIN_ANNOTATION_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space inside annotation parentheses `@Anno( args )`.",
        get: |s| OptionValue::Bool(s.space_within_annotation_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_within_annotation_parentheses = b;
            }
        },
    },
    // --- spacing before parentheses / braces / keywords ---
    OptionDef {
        xml_name: "SPACE_BEFORE_METHOD_CALL_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space before method-call parentheses `f (x)` vs `f(x)`.",
        get: |s| OptionValue::Bool(s.space_before_method_call_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_method_call_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_METHOD_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space before method-declaration parentheses `void f (int p)` vs `void f(int p)`.",
        get: |s| OptionValue::Bool(s.space_before_method_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_method_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_IF_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `if` and its condition `if (...)`. ",
        get: |s| OptionValue::Bool(s.space_before_if_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_if_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_WHILE_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `while` and its condition `while (...)`. ",
        get: |s| OptionValue::Bool(s.space_before_while_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_while_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_FOR_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `for` and its header `for (...)`. ",
        get: |s| OptionValue::Bool(s.space_before_for_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_for_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_TRY_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `try` and its resource list `try (...)`. ",
        get: |s| OptionValue::Bool(s.space_before_try_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_try_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_CATCH_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `catch` and its parameter `catch (...)`. ",
        get: |s| OptionValue::Bool(s.space_before_catch_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_catch_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_SWITCH_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `switch` and its selector `switch (...)`. ",
        get: |s| OptionValue::Bool(s.space_before_switch_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_switch_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_SYNCHRONIZED_PARENTHESES",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `synchronized` and its lock `synchronized (...)`. ",
        get: |s| OptionValue::Bool(s.space_before_synchronized_parentheses),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_synchronized_parentheses = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_ANOTATION_PARAMETER_LIST",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space between an annotation name and its parameter list `@Anno (...)`. (XML name spelled as in IntelliJ source.)",
        get: |s| OptionValue::Bool(s.space_before_anotation_parameter_list),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_anotation_parameter_list = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_CLASS_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space before the opening brace of a class / interface / enum / record / anonymous-class body.",
        get: |s| OptionValue::Bool(s.space_before_class_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_class_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_METHOD_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space before the opening brace of a method / constructor body.",
        get: |s| OptionValue::Bool(s.space_before_method_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_method_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_IF_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space before the opening brace of an `if` body.",
        get: |s| OptionValue::Bool(s.space_before_if_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_if_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_ELSE_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `else` and its body's opening brace.",
        get: |s| OptionValue::Bool(s.space_before_else_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_else_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_WHILE_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space before the opening brace of a `while` body.",
        get: |s| OptionValue::Bool(s.space_before_while_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_while_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_FOR_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space before the opening brace of a `for` / enhanced-`for` body.",
        get: |s| OptionValue::Bool(s.space_before_for_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_for_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_DO_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `do` and its body's opening brace.",
        get: |s| OptionValue::Bool(s.space_before_do_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_do_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_SWITCH_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space before the opening brace of a `switch` body.",
        get: |s| OptionValue::Bool(s.space_before_switch_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_switch_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_TRY_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space before the opening brace of a `try` body.",
        get: |s| OptionValue::Bool(s.space_before_try_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_try_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_CATCH_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space before the opening brace of a `catch` body.",
        get: |s| OptionValue::Bool(s.space_before_catch_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_catch_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_FINALLY_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `finally` and its body's opening brace.",
        get: |s| OptionValue::Bool(s.space_before_finally_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_finally_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_SYNCHRONIZED_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space before the opening brace of a `synchronized` body.",
        get: |s| OptionValue::Bool(s.space_before_synchronized_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_synchronized_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_ARRAY_INITIALIZER_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space between the dimensions of `new T[]` and its initializer `new int[] {`.",
        get: |s| OptionValue::Bool(s.space_before_array_initializer_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_array_initializer_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_ANNOTATION_ARRAY_INITIALIZER_LBRACE",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(false),
        group: "Spaces",
        description: "Space between an annotation's `(` and a bare array-initializer argument `@SuppressWarnings( {…)`.",
        get: |s| OptionValue::Bool(s.space_before_annotation_array_initializer_lbrace),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_annotation_array_initializer_lbrace = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_ELSE_KEYWORD",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `}` and the `else` keyword of an if-chain.",
        get: |s| OptionValue::Bool(s.space_before_else_keyword),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_else_keyword = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_WHILE_KEYWORD",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `}` and the trailing `while` of a do-statement.",
        get: |s| OptionValue::Bool(s.space_before_while_keyword),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_while_keyword = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_CATCH_KEYWORD",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `}` and the `catch` keyword of a try-statement.",
        get: |s| OptionValue::Bool(s.space_before_catch_keyword),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_catch_keyword = b;
            }
        },
    },
    OptionDef {
        xml_name: "SPACE_BEFORE_FINALLY_KEYWORD",
        section: Section::CodeStyleJava,
        default: OptionValue::Bool(true),
        group: "Spaces",
        description: "Space between `}` and the `finally` keyword of a try-statement.",
        get: |s| OptionValue::Bool(s.space_before_finally_keyword),
        set: |s, v| {
            if let OptionValue::Bool(b) = v {
                s.space_before_finally_keyword = b;
            }
        },
    },
];

// ---------------------------------------------------------------------------
// Serde-deserializable XML mirror types
// ---------------------------------------------------------------------------

/// A single `<option name="X" value="Y" />` element.
///
/// Both attributes are optional so nested-valued options — an `<option>` whose
/// setting lives in a `<value>` child tree (the import tables and package
/// lists, e.g. `IMPORT_LAYOUT_TABLE`, `PACKAGES_TO_USE_IMPORT_ON_DEMAND`) — no
/// longer abort the whole parse (R7): they deserialize with a missing `@value`
/// and are skipped by [`OptionMap`]'s attribute decoders, while the layout
/// table's `<value>` children are read with the event API in
/// [`parse_codestyle`] and the package list's with [`OptionMap::get_packages`].
#[derive(Debug, Deserialize)]
struct XmlOption {
    #[serde(rename = "@name", default)]
    name: Option<String>,
    #[serde(rename = "@value", default)]
    value: Option<String>,
    /// The nested `<value>` child tree of a list-valued option (see
    /// [`XmlValue`]); `None` for scalar options.
    #[serde(rename = "value", default)]
    nested: Option<XmlValue>,
}

/// The nested `<value>` child of a list-valued option
/// (`PACKAGES_TO_USE_IMPORT_ON_DEMAND`): `<value><list><option
/// value="pkg.*"/>…</list></value>`. Entries carry the IntelliJ `.*` suffix,
/// which [`OptionMap::get_packages`] strips.
#[derive(Debug, Deserialize, Default)]
struct XmlValue {
    #[serde(rename = "list", default)]
    list: Option<XmlList>,
}

/// The `<option value="pkg.*"/>` entries of a nested package list.
#[derive(Debug, Deserialize, Default)]
struct XmlListOption {
    #[serde(rename = "@value", default)]
    value: Option<String>,
}

/// The entries of a list-valued option's `<list>` element.
#[derive(Debug, Deserialize, Default)]
struct XmlList {
    #[serde(rename = "option", default)]
    options: Vec<XmlListOption>,
}

/// `<indentOptions> <option .../> </indentOptions>`
#[derive(Debug, Deserialize, Default)]
struct XmlIndentOptions {
    #[serde(rename = "option", default)]
    options: Vec<XmlOption>,
}

/// `<JavaCodeStyleSettings> <option .../> ... </JavaCodeStyleSettings>`
#[derive(Debug, Deserialize, Default)]
struct XmlJavaCodeStyleSettings {
    #[serde(rename = "option", default)]
    options: Vec<XmlOption>,
}

/// One `<codeStyleSettings language="...">` block (there can be several).
#[derive(Debug, Deserialize)]
struct XmlCodeStyleSettingsBlock {
    #[serde(rename = "@language")]
    language: String,
    #[serde(rename = "option", default)]
    options: Vec<XmlOption>,
    #[serde(rename = "indentOptions", default)]
    indent_options: Option<XmlIndentOptions>,
}

/// The root `<code_scheme>` element.
#[derive(Debug, Deserialize)]
struct XmlCodeScheme {
    /// Top-level `<option>` children (e.g. SOFT_MARGINS).
    #[serde(rename = "option", default)]
    options: Vec<XmlOption>,

    /// The `<JavaCodeStyleSettings>` block (optional).
    #[serde(rename = "JavaCodeStyleSettings", default)]
    java_code_style: Option<XmlJavaCodeStyleSettings>,

    /// All `<codeStyleSettings language="...">` blocks.
    #[serde(rename = "codeStyleSettings", default)]
    code_style_settings: Vec<XmlCodeStyleSettingsBlock>,
}

// ---------------------------------------------------------------------------
// Option-map helper
// ---------------------------------------------------------------------------

struct OptionMap<'a>(&'a [XmlOption]);

impl<'a> OptionMap<'a> {
    fn get(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|o| o.name.as_deref() == Some(name))
            .and_then(|o| o.value.as_deref())
    }

    fn get_u32(&self, name: &str, default: u32) -> u32 {
        self.get(name)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    fn get_int(&self, name: &str, default: i32) -> i32 {
        self.get(name)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    fn get_bool(&self, name: &str, default: bool) -> bool {
        match self.get(name) {
            Some("true") => true,
            Some("false") => false,
            _ => default,
        }
    }

    /// The raw attribute value of a scalar string option (e.g.
    /// `BUILDER_METHODS`' comma-separated method names), or `default` when
    /// the option is absent.
    fn get_string(&self, name: &str, default: &str) -> String {
        self.get(name).unwrap_or(default).to_string()
    }

    fn get_wrap(&self, name: &str, default: WrapStyle) -> WrapStyle {
        self.get(name)
            .and_then(|v| v.parse::<u32>().ok())
            .map(WrapStyle::from_int)
            .unwrap_or(default)
    }

    fn get_brace(&self, name: &str, default: BraceStyle) -> BraceStyle {
        self.get(name)
            .and_then(|v| v.parse::<u32>().ok())
            .map(BraceStyle::from_int)
            .unwrap_or(default)
    }

    fn get_force(&self, name: &str, default: ForceStyle) -> ForceStyle {
        self.get(name)
            .and_then(|v| v.parse::<u32>().ok())
            .map(ForceStyle::from_int)
            .unwrap_or(default)
    }

    fn get_line_sep(&self, name: &str, default: LineSeparator) -> LineSeparator {
        self.get(name)
            .and_then(LineSeparator::from_str)
            .unwrap_or(default)
    }

    /// The nested `<value><list>` package entries of a list-valued option
    /// (e.g. `PACKAGES_TO_USE_IMPORT_ON_DEMAND`), each stripped of the
    /// IntelliJ `.*` suffix, or `None` when the option is absent or has no
    /// list. An explicitly empty list round-trips as `Some(vec![])` so a
    /// scheme can clear the always-merge packages.
    fn get_packages(&self, name: &str) -> Option<Vec<String>> {
        let opt = self.0.iter().find(|o| o.name.as_deref() == Some(name))?;
        let list = opt.nested.as_ref()?.list.as_ref()?;
        Some(
            list.options
                .iter()
                .filter_map(|o| o.value.as_deref())
                .map(|v| v.strip_suffix(".*").unwrap_or(v).to_string())
                .collect(),
        )
    }
}

// ---------------------------------------------------------------------------
// Import-layout nested-<value> XML helpers
// ---------------------------------------------------------------------------

/// Escape a string for use inside an XML attribute value.
fn xml_attr_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

/// Serialize the nested `<option name="IMPORT_LAYOUT_TABLE"><value>…</value>
/// </option>` fragment for `entries`. The fragment's inner lines carry the
/// relative indent; the section writers prefix every line, so it nests
/// correctly under its block.
fn import_layout_xml(entries: &[ImportLayoutEntry]) -> String {
    let mut out = String::from("<option name=\"IMPORT_LAYOUT_TABLE\">\n  <value>\n");
    for e in entries {
        match e {
            ImportLayoutEntry::EmptyLine => out.push_str("    <emptyLine />\n"),
            ImportLayoutEntry::Package {
                name,
                with_subpackages,
                is_static,
                is_module,
            } => {
                out.push_str("    <package name=\"");
                out.push_str(&xml_attr_escape(name));
                out.push_str(&format!(
                    "\" withSubpackages=\"{}\" static=\"{}\"",
                    with_subpackages, is_static
                ));
                if *is_module {
                    out.push_str(" module=\"true\"");
                }
                out.push_str(" />\n");
            }
        }
    }
    out.push_str("  </value>\n</option>");
    out
}

/// Serialize the nested `<option name="PACKAGES_TO_USE_IMPORT_ON_DEMAND"><value>
/// <list><option value="pkg.*"/>…</list></value></option>` fragment for
/// `packages` (bare package prefixes; the `.*` suffix is appended here, the
/// XML-boundary form). The fragment's inner lines carry the relative indent;
/// the section writers prefix every line, so it nests correctly under its
/// block.
fn packages_xml(packages: &[String]) -> String {
    let mut out =
        String::from("<option name=\"PACKAGES_TO_USE_IMPORT_ON_DEMAND\">\n  <value>\n    <list>\n");
    for p in packages {
        out.push_str("      <option value=\"");
        out.push_str(&xml_attr_escape(p));
        out.push_str(".*\" />\n");
    }
    out.push_str("    </list>\n  </value>\n</option>");
    out
}

/// Read the `IMPORT_LAYOUT_TABLE` entries from a scheme's
/// `<JavaCodeStyleSettings>` block, preserving document order, or `None` when
/// the option is absent. The serde mirror cannot keep the interleaved
/// `<package>` / `<emptyLine>` children across two tag-typed `Vec`s, so this
/// order-preserving scan uses quick-xml's event API; unimplemented nested
/// options stay safely ignored (R7).
fn read_import_layout_entries(xml: &str) -> Option<Vec<ImportLayoutEntry>> {
    use quick_xml::events::Event;

    let mut reader = quick_xml::Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_java_settings = false;
    let mut entries: Option<Vec<ImportLayoutEntry>> = None;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let tag = e.name();
                if tag.as_ref() == b"JavaCodeStyleSettings" {
                    in_java_settings = true;
                } else if in_java_settings && tag.as_ref() == b"option" && entries.is_none() {
                    let is_layout = e.attributes().flatten().any(|a| {
                        a.key.as_ref() == b"name"
                            && a.normalized_value(quick_xml::XmlVersion::Implicit1_0)
                                .map(|v| v == "IMPORT_LAYOUT_TABLE")
                                .unwrap_or(false)
                    });
                    if is_layout {
                        entries = Some(Vec::new());
                    }
                } else if let Some(entries) = entries.as_mut() {
                    if in_java_settings && tag.as_ref() == b"package" {
                        let mut name = String::new();
                        let mut with_subpackages = true;
                        let mut is_static = false;
                        let mut is_module = false;
                        for a in e.attributes().flatten() {
                            let val = a
                                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                                .unwrap_or_default();
                            match a.key.as_ref() {
                                b"name" => name = val.into_owned(),
                                b"withSubpackages" => with_subpackages = val == "true",
                                b"static" => is_static = val == "true",
                                b"module" => is_module = val == "true",
                                _ => {}
                            }
                        }
                        entries.push(ImportLayoutEntry::Package {
                            name,
                            with_subpackages,
                            is_static,
                            is_module,
                        });
                    } else if in_java_settings && tag.as_ref() == b"emptyLine" {
                        entries.push(ImportLayoutEntry::EmptyLine);
                    }
                }
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"JavaCodeStyleSettings" {
                    in_java_settings = false;
                } else if e.name().as_ref() == b"option" && entries.is_some() {
                    // Closing the layout option: return what was read. Only one
                    // occurrence is expected; a second is ignored.
                    return entries;
                }
            }
            Ok(Event::Eof) => break,
            // A malformed document fails the serde parse above first.
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    entries
}

// ---------------------------------------------------------------------------
// Public parsing function
// ---------------------------------------------------------------------------

/// Parse an IntelliJ codestyle XML string into a [`JavaStyle`].
///
/// Missing blocks (`<JavaCodeStyleSettings>`, `<codeStyleSettings language="JAVA">`)
/// are treated as absent — their settings fall back to IntelliJ defaults.
/// Option decoding is driven by the [`OPTIONS`] registry, so parsing and
/// serialization share one definition of every option.
pub fn parse_codestyle(xml: &str) -> Result<JavaStyle, Box<dyn std::error::Error>> {
    let scheme: XmlCodeScheme = quick_xml::de::from_str(xml)?;

    let mut style = JavaStyle::default();

    // Resolve the option containers the registry can read from.
    let top = OptionMap(&scheme.options);
    let java_settings = scheme
        .java_code_style
        .as_ref()
        .map(|block| OptionMap(&block.options));
    let java_block = scheme
        .code_style_settings
        .iter()
        .find(|b| b.language == "JAVA");
    let java = java_block.map(|b| OptionMap(&b.options));
    let indent = java_block
        .and_then(|b| b.indent_options.as_ref())
        .map(|i| OptionMap(&i.options));

    for def in OPTIONS {
        // The import-layout table reads its nested `<value>` subtree with the
        // event API (order-preserving `<package>` / `<emptyLine>` entries the
        // serde mirror cannot represent); absent → the built-in default the
        // style was initialised with stays.
        if matches!(&def.default, OptionValue::ImportLayout(_)) {
            if let Some(entries) = read_import_layout_entries(xml) {
                style.import_layout = entries;
            }
            continue;
        }
        let map = match def.section {
            Section::Root => &top,
            Section::JavaCodeStyle => match &java_settings {
                Some(m) => m,
                None => continue,
            },
            Section::CodeStyleJava => match &java {
                Some(m) => m,
                None => continue,
            },
            Section::IndentOptions => match &indent {
                Some(m) => m,
                None => continue,
            },
        };
        // The package list has no `value` attribute (its entries live in the
        // nested `<value><list>` child), so it is read before the attribute
        // presence guard below; absent → the built-in default stays.
        if matches!(&def.default, OptionValue::Packages(_)) {
            if let Some(packages) = map.get_packages(def.xml_name) {
                (def.set)(&mut style, OptionValue::Packages(packages));
            }
            continue;
        }
        // Options absent from the scheme keep the `JavaStyle` defaults the
        // style was initialised with (identical to the registry defaults).
        // Skipping them — rather than re-applying the default — lets an
        // earlier option that shares a field keep its parsed value:
        // `RIGHT_MARGIN` is registered before `SOFT_MARGINS`, so
        // `SOFT_MARGINS` wins only when the scheme sets both.
        if map.get(def.xml_name).is_none() {
            continue;
        }
        let value = match &def.default {
            OptionValue::Bool(default) => OptionValue::Bool(map.get_bool(def.xml_name, *default)),
            OptionValue::UInt(default) => OptionValue::UInt(map.get_u32(def.xml_name, *default)),
            OptionValue::Int(default) => OptionValue::Int(map.get_int(def.xml_name, *default)),
            OptionValue::Wrap(default) => OptionValue::Wrap(map.get_wrap(def.xml_name, *default)),
            OptionValue::Brace(default) => {
                OptionValue::Brace(map.get_brace(def.xml_name, *default))
            }
            OptionValue::Force(default) => {
                OptionValue::Force(map.get_force(def.xml_name, *default))
            }
            OptionValue::LineSep(default) => {
                OptionValue::LineSep(map.get_line_sep(def.xml_name, *default))
            }
            OptionValue::String(default) => {
                OptionValue::String(map.get_string(def.xml_name, default))
            }
            OptionValue::ImportLayout(_) => unreachable!(),
            OptionValue::Packages(_) => unreachable!(),
        };
        (def.set)(&mut style, value);
    }

    Ok(style)
}

// ---------------------------------------------------------------------------
// Public serialization function
// ---------------------------------------------------------------------------

/// Append `text` to `out`, prefixing every line with `prefix` and ending with
/// one newline. Multi-line fragments (the nested import-table option) are
/// therefore indented on each line under the section that owns them; the
/// single-line scalar options behave exactly as before.
fn push_option(out: &mut String, prefix: &str, text: &str) {
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(prefix);
        out.push_str(line);
    }
    out.push('\n');
}

/// Serialize a [`JavaStyle`] into a minimal `<code_scheme>` document.
///
/// Only options whose value differs from the IntelliJ default are written
/// (IntelliJ's own export convention); absent options fall back to defaults
/// in both consumers, so the file stays minimal while remaining semantically
/// identical. `parse_codestyle(serialize_codestyle(style)) == style`.
pub fn serialize_codestyle(style: &JavaStyle) -> String {
    // Reference defaults computed once per call. For the scalar options these
    // equal the registry literals; for the list-typed import layout the real
    // built-in table lives in `JavaStyle::default` (`OptionDef::default` is an
    // empty type tag), so comparing each value against the corresponding
    // default-style field keeps the minimal-scheme rule uniform across both
    // kinds.
    let defaults = JavaStyle::default();

    let mut root = Vec::new();
    let mut java_settings = Vec::new();
    let mut java = Vec::new();
    let mut indent = Vec::new();

    for def in OPTIONS {
        let value = (def.get)(style);
        if value == (def.get)(&defaults) {
            continue;
        }
        let option = match &value {
            OptionValue::ImportLayout(entries) => import_layout_xml(entries),
            OptionValue::Packages(packages) => packages_xml(packages),
            _ => {
                let xml_value = match &value {
                    OptionValue::Bool(b) => b.to_string(),
                    OptionValue::UInt(n) => n.to_string(),
                    OptionValue::Int(n) => n.to_string(),
                    OptionValue::Wrap(w) => w.to_int().to_string(),
                    OptionValue::Brace(b) => b.to_int().to_string(),
                    OptionValue::Force(f) => f.to_int().to_string(),
                    OptionValue::LineSep(s) => {
                        // The default (`System`) was already skipped above; the
                        // other separators are serialised in their XML-escaped
                        // forms.
                        s.to_xml().unwrap_or("").to_string()
                    }
                    OptionValue::String(s) => xml_attr_escape(s),
                    OptionValue::ImportLayout(_) => unreachable!(),
                    OptionValue::Packages(_) => unreachable!(),
                };
                format!(
                    r#"<option name="{}" value="{}" />"#,
                    def.xml_name, xml_value
                )
            }
        };
        match def.section {
            Section::Root => root.push(option),
            Section::JavaCodeStyle => java_settings.push(option),
            Section::CodeStyleJava => java.push(option),
            Section::IndentOptions => indent.push(option),
        }
    }

    let mut out = String::from("<code_scheme name=\"Project\" version=\"173\">\n");
    for option in &root {
        push_option(&mut out, "  ", option);
    }
    if !java_settings.is_empty() {
        out.push_str("  <JavaCodeStyleSettings>\n");
        for option in &java_settings {
            push_option(&mut out, "    ", option);
        }
        out.push_str("  </JavaCodeStyleSettings>\n");
    }
    if !java.is_empty() || !indent.is_empty() {
        out.push_str("  <codeStyleSettings language=\"JAVA\">\n");
        for option in &java {
            push_option(&mut out, "    ", option);
        }
        if !indent.is_empty() {
            out.push_str("    <indentOptions>\n");
            for option in &indent {
                push_option(&mut out, "      ", option);
            }
            out.push_str("    </indentOptions>\n");
        }
        out.push_str("  </codeStyleSettings>\n");
    }
    out.push_str("</code_scheme>\n");
    out
}
