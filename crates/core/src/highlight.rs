//! tree-sitter-driven syntax-highlighting spans for Java source.
//!
//! The formatter ([`crate::formatter`]) already parses Java with
//! tree-sitter-java; this module reuses the same grammar to expose a small,
//! presentation-agnostic set of highlight spans. It carries no opinion about
//! colours — callers map each [`HighlightKind`] to their own.

use tree_sitter::{Language, Node, Parser};

/// A syntactic category a highlighted range can belong to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HighlightKind {
    /// Keywords and literals (`public`, `class`, `if`, `new`, `true`, `null`, …).
    Keyword,
    /// String, text-block and character literals.
    String,
    /// Numeric literals.
    Number,
    /// Line and block comments.
    Comment,
    /// Annotation names.
    Annotation,
    /// Type references and declarations.
    Type,
    /// Method and constructor names.
    Method,
}

/// A half-open byte range `[start, end)` in the source carrying a
/// [`HighlightKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightSpan {
    /// Start byte offset, inclusive.
    pub start: usize,
    /// End byte offset, exclusive.
    pub end: usize,
    /// The category the range belongs to.
    pub kind: HighlightKind,
}

/// Parse `source` with tree-sitter-java and return its highlight spans,
/// ordered by start offset and non-overlapping.
///
/// Incomplete or unparsable input still yields spans for whatever parsed
/// cleanly; everything else is left untagged so callers keep their default
/// colour.
pub fn highlight_java(source: &str) -> Vec<HighlightSpan> {
    let mut parser = Parser::new();
    let language: Language = tree_sitter_java::LANGUAGE.into();
    if parser.set_language(&language).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source.as_bytes(), None) else {
        return Vec::new();
    };

    let mut spans = Vec::new();
    visit(tree.root_node(), &mut spans);
    spans.sort_by_key(|span| span.start);
    spans
}

/// Walk one node, tagging the ranges it (or its children) own.
fn visit(node: Node<'_>, spans: &mut Vec<HighlightSpan>) {
    let kind = node.kind();
    match kind {
        "line_comment" | "block_comment" => {
            push(node, HighlightKind::Comment, spans);
            return;
        }
        // `string_literal` also covers text blocks (the grammar aliases its
        // single- and multi-line productions to the same node kind).
        "string_literal" | "character_literal" => {
            push(node, HighlightKind::String, spans);
            return;
        }
        "decimal_integer_literal"
        | "hex_integer_literal"
        | "octal_integer_literal"
        | "binary_integer_literal"
        | "decimal_floating_point_literal"
        | "hex_floating_point_literal" => {
            push(node, HighlightKind::Number, spans);
            return;
        }
        "type_identifier"
        | "scoped_type_identifier"
        | "integral_type"
        | "floating_point_type"
        | "boolean_type"
        | "void_type" => {
            push(node, HighlightKind::Type, spans);
            return;
        }
        // Composite nodes tag only their name and then keep recursing, so the
        // surrounding modifiers, parameter/argument types and expressions are
        // still classified by the arms above.
        "method_declaration" | "constructor_declaration" | "method_invocation" => {
            tag_name(node, HighlightKind::Method, spans);
        }
        "annotation" | "marker_annotation" => {
            tag_name(node, HighlightKind::Annotation, spans);
        }
        "class_declaration"
        | "interface_declaration"
        | "enum_declaration"
        | "record_declaration"
        | "annotation_type_declaration" => {
            tag_name(node, HighlightKind::Type, spans);
        }
        _ => {}
    }

    if node.child_count() == 0 {
        if is_keyword(kind) {
            push(node, HighlightKind::Keyword, spans);
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, spans);
    }
}

/// Tag the node's `name` field, if it has one.
fn tag_name(node: Node<'_>, kind: HighlightKind, spans: &mut Vec<HighlightSpan>) {
    if let Some(name) = node.child_by_field_name("name") {
        push(name, kind, spans);
    }
}

/// Record a node's byte range under `kind`, skipping empty ranges.
fn push(node: Node<'_>, kind: HighlightKind, spans: &mut Vec<HighlightSpan>) {
    let range = node.byte_range();
    if range.start < range.end {
        spans.push(HighlightSpan {
            start: range.start,
            end: range.end,
            kind,
        });
    }
}

/// Whether a leaf token is a Java keyword or keyword literal.
///
/// The set is the grammar's own anonymous word tokens (tree-sitter-java
/// 0.23.5), so it stays in step with what the parser can emit; primitive-type
/// tokens are listed for completeness even though they surface as the named
/// `*_type` nodes handled above.
fn is_keyword(kind: &str) -> bool {
    matches!(
        kind,
        "abstract"
            | "assert"
            | "boolean"
            | "break"
            | "byte"
            | "case"
            | "catch"
            | "char"
            | "class"
            | "continue"
            | "default"
            | "do"
            | "double"
            | "else"
            | "enum"
            | "exports"
            | "extends"
            | "false"
            | "final"
            | "finally"
            | "float"
            | "for"
            | "if"
            | "implements"
            | "import"
            | "instanceof"
            | "int"
            | "interface"
            | "long"
            | "module"
            | "native"
            | "new"
            | "non-sealed"
            | "null_literal"
            | "open"
            | "opens"
            | "package"
            | "permits"
            | "private"
            | "protected"
            | "provides"
            | "public"
            | "record"
            | "requires"
            | "return"
            | "sealed"
            | "short"
            | "static"
            | "strictfp"
            | "super"
            | "switch"
            | "synchronized"
            | "this"
            | "throw"
            | "throws"
            | "to"
            | "transient"
            | "transitive"
            | "true"
            | "try"
            | "uses"
            | "void"
            | "volatile"
            | "when"
            | "while"
            | "with"
            | "yield"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `(source text, category)` pairs of every span in `src`.
    fn tagged(src: &str) -> Vec<(&str, HighlightKind)> {
        highlight_java(src)
            .into_iter()
            .map(|span| (&src[span.start..span.end], span.kind))
            .collect()
    }

    const SAMPLE: &str = r#"package demo;

/** A demo. */
@Deprecated
public class Demo {
    private int count = 42;
    private String name = "hi";
    private char letter = 'x';

    public String greet(java.util.List<String> names) {
        // say hello
        /* block */
        return names.get(0);
    }
}
"#;

    #[test]
    fn maps_each_category() {
        let spans = tagged(SAMPLE);
        for expected in [
            ("public", HighlightKind::Keyword),
            ("class", HighlightKind::Keyword),
            ("private", HighlightKind::Keyword),
            ("return", HighlightKind::Keyword),
            ("int", HighlightKind::Type),
            ("String", HighlightKind::Type),
            ("java.util.List", HighlightKind::Type),
            ("Demo", HighlightKind::Type),
            ("42", HighlightKind::Number),
            ("\"hi\"", HighlightKind::String),
            ("'x'", HighlightKind::String),
            ("// say hello", HighlightKind::Comment),
            ("/* block */", HighlightKind::Comment),
            ("Deprecated", HighlightKind::Annotation),
            ("greet", HighlightKind::Method),
            ("get", HighlightKind::Method),
        ] {
            assert!(
                spans.contains(&expected),
                "expected {expected:?} in {spans:?}"
            );
        }
    }

    #[test]
    fn covers_text_blocks_and_keyword_literals() {
        let src = "class T {\n  String s = \"\"\"\n    hi\n    \"\"\";\n  boolean b = true;\n  Object o = null;\n  boolean c = false;\n}\n";
        let spans = tagged(src);
        assert!(spans
            .iter()
            .any(|(text, kind)| text.starts_with("\"\"\"") && *kind == HighlightKind::String));
        assert!(spans.contains(&("true", HighlightKind::Keyword)));
        assert!(spans.contains(&("false", HighlightKind::Keyword)));
        assert!(spans.contains(&("null", HighlightKind::Keyword)));
    }

    #[test]
    fn spans_are_sorted_and_disjoint() {
        let spans = highlight_java(SAMPLE);
        for pair in spans.windows(2) {
            assert!(
                pair[0].end <= pair[1].start,
                "spans overlap or are unsorted: {:?} then {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn partial_input_does_not_panic() {
        // Unterminated constructs still yield whatever parsed cleanly.
        let _ = highlight_java("public class Broken { String s = \"unterminated");
        let _ = highlight_java("");
    }
}
