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
    pub annotation_parameter_wrap: WrapStyle,
    pub assignment_wrap: WrapStyle,

    // --- binary expression wrapping ---
    pub binary_operation_wrap: WrapStyle,

    // --- one-liners ---
    pub keep_simple_blocks_in_one_line: bool,
    pub keep_simple_methods_in_one_line: bool,
    pub keep_simple_lambdas_in_one_line: bool,
    pub keep_control_statement_in_one_line: bool,

    // --- record-specific (JavaCodeStyleSettings) ---
    pub record_components_wrap: WrapStyle,
    pub align_multiline_records: bool,
    pub new_line_after_lparen_in_record_header: bool,

    // --- imports ---
    pub class_count_to_use_import_on_demand: u32,

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

impl Default for JavaStyle {
    fn default() -> Self {
        JavaStyle {
            indent_size: 4,
            continuation_indent_size: 8,
            tab_size: 4,
            use_tab_character: false,
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
            annotation_parameter_wrap: WrapStyle::DoNotWrap,
            assignment_wrap: WrapStyle::DoNotWrap,
            binary_operation_wrap: WrapStyle::DoNotWrap,
            keep_simple_blocks_in_one_line: false,
            keep_simple_methods_in_one_line: false,
            keep_simple_lambdas_in_one_line: false,
            keep_control_statement_in_one_line: true,
            record_components_wrap: WrapStyle::DoNotWrap,
            align_multiline_records: true,
            new_line_after_lparen_in_record_header: false,
            class_count_to_use_import_on_demand: 5,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionValue {
    Bool(bool),
    UInt(u32),
    Wrap(WrapStyle),
    Brace(BraceStyle),
    Force(ForceStyle),
    LineSep(LineSeparator),
}

/// Declarative description of one supported code style option — the single
/// source of truth shared by [`parse_codestyle`], [`serialize_codestyle`] and
/// the GUI. Each entry's `default` equals the corresponding
/// [`JavaStyle::default`] value so the serialize/parse round-trip is exact.
pub struct OptionDef {
    /// The XML `name` attribute, e.g. `"CLASS_BRACE_STYLE"`.
    pub xml_name: &'static str,
    /// The scheme section the option lives in.
    pub section: Section,
    /// The IntelliJ default value.
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
#[derive(Debug, Deserialize)]
struct XmlOption {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@value")]
    value: String,
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
            .find(|o| o.name == name)
            .map(|o| o.value.as_str())
    }

    fn get_u32(&self, name: &str, default: u32) -> u32 {
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
        // Options absent from the scheme keep the `JavaStyle` defaults the
        // style was initialised with (identical to the registry defaults).
        // Skipping them — rather than re-applying the default — lets an
        // earlier option that shares a field keep its parsed value:
        // `RIGHT_MARGIN` is registered before `SOFT_MARGINS`, so
        // `SOFT_MARGINS` wins only when the scheme sets both.
        if map.get(def.xml_name).is_none() {
            continue;
        }
        let value = match def.default {
            OptionValue::Bool(default) => OptionValue::Bool(map.get_bool(def.xml_name, default)),
            OptionValue::UInt(default) => OptionValue::UInt(map.get_u32(def.xml_name, default)),
            OptionValue::Wrap(default) => OptionValue::Wrap(map.get_wrap(def.xml_name, default)),
            OptionValue::Brace(default) => OptionValue::Brace(map.get_brace(def.xml_name, default)),
            OptionValue::Force(default) => OptionValue::Force(map.get_force(def.xml_name, default)),
            OptionValue::LineSep(default) => {
                OptionValue::LineSep(map.get_line_sep(def.xml_name, default))
            }
        };
        (def.set)(&mut style, value);
    }

    Ok(style)
}

// ---------------------------------------------------------------------------
// Public serialization function
// ---------------------------------------------------------------------------

/// Serialize a [`JavaStyle`] into a minimal `<code_scheme>` document.
///
/// Only options whose value differs from the IntelliJ default are written
/// (IntelliJ's own export convention); absent options fall back to defaults
/// in both consumers, so the file stays minimal while remaining semantically
/// identical. `parse_codestyle(serialize_codestyle(style)) == style`.
pub fn serialize_codestyle(style: &JavaStyle) -> String {
    let mut root = Vec::new();
    let mut java_settings = Vec::new();
    let mut java = Vec::new();
    let mut indent = Vec::new();

    for def in OPTIONS {
        let value = (def.get)(style);
        if value == def.default {
            continue;
        }
        let xml_value = match value {
            OptionValue::Bool(b) => b.to_string(),
            OptionValue::UInt(n) => n.to_string(),
            OptionValue::Wrap(w) => w.to_int().to_string(),
            OptionValue::Brace(b) => b.to_int().to_string(),
            OptionValue::Force(f) => f.to_int().to_string(),
            OptionValue::LineSep(s) => {
                // The default (`System`) was already skipped above; the other
                // separators are serialised in their XML-escaped forms.
                s.to_xml().unwrap_or("").to_string()
            }
        };
        let option = format!(
            r#"<option name="{}" value="{}" />"#,
            def.xml_name, xml_value
        );
        match def.section {
            Section::Root => root.push(option),
            Section::JavaCodeStyle => java_settings.push(option),
            Section::CodeStyleJava => java.push(option),
            Section::IndentOptions => indent.push(option),
        }
    }

    let mut out = String::from("<code_scheme name=\"Project\" version=\"173\">\n");
    for option in &root {
        out.push_str("  ");
        out.push_str(option);
        out.push('\n');
    }
    if !java_settings.is_empty() {
        out.push_str("  <JavaCodeStyleSettings>\n");
        for option in &java_settings {
            out.push_str("    ");
            out.push_str(option);
            out.push('\n');
        }
        out.push_str("  </JavaCodeStyleSettings>\n");
    }
    if !java.is_empty() || !indent.is_empty() {
        out.push_str("  <codeStyleSettings language=\"JAVA\">\n");
        for option in &java {
            out.push_str("    ");
            out.push_str(option);
            out.push('\n');
        }
        if !indent.is_empty() {
            out.push_str("    <indentOptions>\n");
            for option in &indent {
                out.push_str("      ");
                out.push_str(option);
                out.push('\n');
            }
            out.push_str("    </indentOptions>\n");
        }
        out.push_str("  </codeStyleSettings>\n");
    }
    out.push_str("</code_scheme>\n");
    out
}
