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

    // --- brace styles ---
    pub class_brace_style: BraceStyle,
    pub method_brace_style: BraceStyle,
    pub other_brace_style: BraceStyle,

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

    // --- record-specific (JavaCodeStyleSettings) ---
    pub record_components_wrap: WrapStyle,
    pub align_multiline_records: bool,
    pub new_line_after_lparen_in_record_header: bool,

    // --- imports ---
    pub class_count_to_use_import_on_demand: u32,
}

impl Default for JavaStyle {
    fn default() -> Self {
        JavaStyle {
            indent_size: 4,
            continuation_indent_size: 8,
            tab_size: 4,
            use_tab_character: false,
            right_margin: 120,
            class_brace_style: BraceStyle::EndOfLine,
            method_brace_style: BraceStyle::EndOfLine,
            other_brace_style: BraceStyle::EndOfLine,
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
            record_components_wrap: WrapStyle::DoNotWrap,
            align_multiline_records: true,
            new_line_after_lparen_in_record_header: false,
            class_count_to_use_import_on_demand: 5,
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
        let value = match def.default {
            OptionValue::Bool(default) => OptionValue::Bool(map.get_bool(def.xml_name, default)),
            OptionValue::UInt(default) => OptionValue::UInt(map.get_u32(def.xml_name, default)),
            OptionValue::Wrap(default) => OptionValue::Wrap(map.get_wrap(def.xml_name, default)),
            OptionValue::Brace(default) => OptionValue::Brace(map.get_brace(def.xml_name, default)),
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
