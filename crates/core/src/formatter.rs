//! Java source-code formatter driven by IntelliJ codestyle settings.
//!
//! Uses tree-sitter-java to parse source into a CST, then pretty-prints it
//! following the rules encoded in [`crate::config::JavaStyle`].

use std::collections::{HashMap, HashSet};

use tree_sitter::{Language, Node, Parser};

use crate::config::{BraceStyle, ForceStyle, JavaStyle, WrapStyle};

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────────────

/// A parse-level problem found in the input source.
///
/// Reported in addition to (never instead of) the best-effort formatted
/// output: a syntax error or a missing token does not stop formatting, it is
/// surfaced so callers know part of the output may not have been formatted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    /// Human-readable description of what failed to parse.
    pub message: String,
    /// 1-based line where the problem begins.
    pub line: usize,
    /// 1-based column (in characters) where the problem begins.
    pub column: usize,
}

impl std::fmt::Display for ParseDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}:{}", self.message, self.line, self.column)
    }
}

/// Format a Java source string using the provided style settings.
/// Returns the formatted source, ending with exactly one newline.
pub fn format_java(source: &str, style: &JavaStyle) -> String {
    format_java_diagnosed(source, style).0
}

/// Like [`format_java`], but also reports parse-level problems found in the
/// input as [`ParseDiagnostic`]s.
///
/// The formatted output is still produced (best-effort) when diagnostics are
/// non-empty; the diagnostics only describe where the input is not valid
/// Java, so callers can warn. The never-corrupt contract is unchanged:
/// anything the formatter does not model is preserved verbatim.
pub fn format_java_diagnosed(source: &str, style: &JavaStyle) -> (String, Vec<ParseDiagnostic>) {
    let mut parser = Parser::new();
    let language: Language = tree_sitter_java::LANGUAGE.into();
    parser
        .set_language(&language)
        .expect("Failed to load Java grammar");

    let src = source.as_bytes();
    let tree = parser
        .parse(src, None)
        .expect("Failed to parse Java source");

    let diagnostics = collect_parse_diagnostics(tree.root_node(), src);

    let fmt = Fmt { src, style };
    let mut out = fmt.program(tree.root_node());

    // `WRAP_LONG_LINES` post-pass: hard-wrap lines past the right margin at
    // the rightmost safe whitespace. Runs on the LF-normal text, before the
    // separator substitution below.
    if style.wrap_long_lines {
        out = wrap_long_lines(&out, style);
    }

    // Finalisation: collapse any `\r\n` that arrived via verbatim echoes of a
    // CRLF source, trim to exactly one trailing line end, then substitute the
    // configured separator at every line end — including the final newline —
    // when it is not LF. LF output takes the historical code path unchanged,
    // so default (System → LF on the test hosts) output stays byte-identical.
    (
        finalise_line_endings(&out, style.line_separator.resolve()),
        diagnostics,
    )
}

/// Upper bound on the number of diagnostics reported per input. A file full
/// of errors must not flood the caller; the first problems are the useful
/// ones.
const MAX_PARSE_DIAGNOSTICS: usize = 10;

/// Collect the top-most parse problems in `root`: error and missing nodes,
/// skipping anything nested inside an already-reported error region (an
/// `ERROR` node spans its whole malformed subtree and would otherwise flood
/// the report with descendants).
fn collect_parse_diagnostics(root: Node, src: &[u8]) -> Vec<ParseDiagnostic> {
    if !root.has_error() {
        return Vec::new();
    }

    let mut out = Vec::new();
    walk_errors(root, &mut out, src);
    out
}

fn walk_errors(node: Node, out: &mut Vec<ParseDiagnostic>, src: &[u8]) {
    if out.len() >= MAX_PARSE_DIAGNOSTICS {
        return;
    }

    if node.is_error() || node.is_missing() {
        if let Some(diag) = diagnostic_for(node, src) {
            // A missing token is a leaf; an ERROR node's descendants are all
            // part of the same problem, so do not descend into it.
            if out.last() != Some(&diag) {
                out.push(diag);
            }
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_errors(child, out, src);
        if out.len() >= MAX_PARSE_DIAGNOSTICS {
            return;
        }
    }
}

fn diagnostic_for(node: Node, src: &[u8]) -> Option<ParseDiagnostic> {
    let (line, column) = line_col(src, node.start_byte());
    let kind = node.kind();
    let message = if node.is_missing() {
        format!("missing '{}'", kind)
    } else {
        format!("parse error in '{}'", kind)
    };
    Some(ParseDiagnostic {
        message,
        line,
        column,
    })
}

/// 1-based line and column for a byte offset into `src`. The column counts
/// characters (not bytes) since the last newline.
fn line_col(src: &[u8], byte: usize) -> (usize, usize) {
    let before = std::str::from_utf8(&src[..byte.min(src.len())]).unwrap_or("");
    let mut line = 1usize;
    let mut last_newline = 0usize;
    for (i, ch) in before.char_indices() {
        if ch == '\n' {
            line += 1;
            last_newline = i + 1;
        }
    }
    let column = before[last_newline..].chars().count() + 1;
    (line, column)
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

struct Fmt<'s> {
    src: &'s [u8],
    style: &'s JavaStyle,
}

// A chain link: one `.method(args)` piece
struct Link<'s> {
    type_args: Option<Node<'s>>,
    name: Node<'s>,
    args: Node<'s>,
}

/// Which kind of class-like body is being laid out; selects the governing
/// blank-line minimums (interface members use the `*_IN_INTERFACE` variants,
/// anonymous bodies the anonymous-class-header minimum).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BodyKind {
    /// A named class / enum / record body.
    Class,
    /// An interface body.
    Interface,
    /// An anonymous class body (`new X() { … }`).
    Anonymous,
}

impl<'s> Fmt<'s> {
    // ── text helpers ──────────────────────────────────────────────────────────

    fn txt(&self, n: Node) -> &str {
        n.utf8_text(self.src).unwrap_or("")
    }

    /// Indentation string for `level` indent units.
    fn ind(&self, level: usize) -> String {
        self.indent_str(level * self.style.indent_size as usize)
    }

    /// Continuation-indent string: `level` indent units + continuation offset.
    fn cont(&self, level: usize) -> String {
        self.indent_str(
            level * self.style.indent_size as usize + self.style.continuation_indent_size as usize,
        )
    }

    /// Build an indentation string of `width` columns. When
    /// `USE_TAB_CHARACTER` is set, each full `tab_size` column becomes a tab
    /// character and the remainder spaces — a tab-stop model matching
    /// IntelliJ (so `indent_size == tab_size` yields exactly one tab per
    /// level). Otherwise plain spaces are emitted, byte-identical to the
    /// historical output.
    fn indent_str(&self, width: usize) -> String {
        if !self.style.use_tab_character {
            return " ".repeat(width);
        }
        let tab = self.style.tab_size as usize;
        format!("{}{}", "\t".repeat(width / tab), " ".repeat(width % tab))
    }

    /// Column reached by appending `s` at column `c`: a newline resets the
    /// column to 0, a tab advances to the next multiple of `tab_size`, and
    /// every other character advances by one. Equal to `c + s.len()` when `s`
    /// contains no tabs and no newlines, so margin decisions are unchanged for
    /// space-indented output.
    fn col_after(&self, c: usize, s: &str) -> usize {
        let tab = self.style.tab_size as usize;
        let mut c = c;
        for ch in s.chars() {
            match ch {
                '\n' => c = 0,
                '\t' => c += tab - (c % tab),
                _ => c += 1,
            }
        }
        c
    }

    fn fits(&self, c: usize, s: &str) -> bool {
        self.col_after(c, s) <= self.style.right_margin as usize
    }

    /// `KEEP_LINE_BREAKS` retention: true when the option is on and `node`'s
    /// source carries a line break at its own join level — a break between the
    /// tokens that the flat layout would join, not one buried inside a nested
    /// block, parenthesised sub-expression, literal or comment. When retained,
    /// the construct's canonical wrapped layout is rendered even when the flat
    /// form fits; when the option is off — or the source is joined — the
    /// flatten-if-fits path is kept (reflow).
    fn keep_wrapped(&self, node: Node<'s>) -> bool {
        self.style.keep_line_breaks && self.has_join_break(node)
    }

    /// Like [`Self::keep_wrapped`] for the `arguments` child of an invocation
    /// / creation expression (the argument-list interior, not the whole call).
    fn args_keep_wrapped(&self, node: Node<'s>) -> bool {
        self.fld(node, "arguments")
            .map_or(false, |a| self.keep_wrapped(a))
    }

    /// True when the source text of `node`'s inner region — inside its own
    /// outermost bracket pair when the node is exactly `(…)` / `{…}` / `[…]`,
    /// otherwise the whole node — contains a line break at bracket depth 0
    /// outside strings, chars, comments and text blocks.
    fn has_join_break(&self, node: Node<'s>) -> bool {
        let text = self.txt(node);
        if !text.contains('\n') && !text.contains('\r') {
            return false;
        }
        let b = text.as_bytes();
        let (mut lo, mut hi) = (0usize, b.len());
        if b.len() >= 2 {
            let (f, l) = (b[0], b[b.len() - 1]);
            if (f == b'(' && l == b')') || (f == b'{' && l == b'}') || (f == b'[' && l == b']') {
                lo = 1;
                hi -= 1;
            }
        }

        // Mask literal / comment regions (including their line breaks) so
        // only plain-code characters participate in the depth scan below.
        let mut code = vec![false; b.len()];
        // 0 code, 1 string, 2 char, 3 line comment, 4 block comment, 5 text block
        let mut state: u8 = 0;
        let mut i = lo;
        while i < hi {
            let ch = b[i];
            match state {
                0 => match ch {
                    b'"' if b[i..].starts_with(b"\"\"\"") => {
                        state = 5;
                        i += 2;
                    }
                    b'"' => state = 1,
                    b'\'' => state = 2,
                    b'/' if b[i..].starts_with(b"//") => state = 3,
                    b'/' if b[i..].starts_with(b"/*") => state = 4,
                    _ => code[i] = true,
                },
                1 | 2 => {
                    if ch == b'\\' {
                        i += 1;
                    } else if (state == 1 && ch == b'"') || (state == 2 && ch == b'\'') {
                        state = 0;
                    }
                }
                3 => {
                    if ch == b'\n' || ch == b'\r' {
                        state = 0;
                    }
                }
                4 => {
                    if ch == b'*' && i + 1 < hi && b[i + 1] == b'/' {
                        state = 0;
                        i += 1;
                    }
                }
                _ => {
                    if ch == b'\\' {
                        i += 1;
                    } else if ch == b'"' && b[i..].starts_with(b"\"\"\"") {
                        state = 0;
                        i += 2;
                    }
                }
            }
            i += 1;
        }

        let mut depth = 0usize;
        let mut i = lo;
        while i < hi {
            if code[i] {
                match b[i] {
                    b'(' | b'[' | b'{' => depth += 1,
                    b')' | b']' | b'}' => depth = depth.saturating_sub(1),
                    b'\n' | b'\r' if depth == 0 => return true,
                    _ => {}
                }
            }
            i += 1;
        }
        false
    }

    fn named(&self, n: Node<'s>) -> Vec<Node<'s>> {
        let mut cur = n.walk();
        n.named_children(&mut cur).collect()
    }

    fn all_ch(&self, n: Node<'s>) -> Vec<Node<'s>> {
        let mut cur = n.walk();
        n.children(&mut cur).collect()
    }

    fn fld(&self, n: Node<'s>, f: &str) -> Option<Node<'s>> {
        n.child_by_field_name(f)
    }

    /// The separator emitted for a spacing toggle: one space when on, nothing
    /// when off.
    fn sep(on: bool) -> &'static str {
        if on {
            " "
        } else {
            ""
        }
    }

    /// The gap emitted for a before-parenthesis / before-brace /
    /// before-keyword toggle: one space when on, nothing when off.
    fn sp(&self, on: bool) -> &'static str {
        Self::sep(on)
    }

    /// Wrap `inner` in `open`/`close`, padding one space on each side when
    /// `pad` is on. An empty `inner` stays bare (`()` — constructs with an
    /// empty variant use [`Self::within_opt`]); a side whose neighbour is a
    /// newline stays bare so wrapped layouts never gain trailing whitespace.
    fn within(open: char, close: char, pad: bool, inner: &str) -> String {
        if inner.is_empty() {
            return format!("{}{}", open, close);
        }
        let l = if pad && !inner.starts_with('\n') {
            " "
        } else {
            ""
        };
        let r = if pad && !inner.ends_with('\n') {
            " "
        } else {
            ""
        };
        format!("{}{}{}{}{}", open, l, inner, r, close)
    }

    /// Empty-aware variant of [`Self::within`]: the empty pair is padded only
    /// when `pad_empty` is on (`f( )`, `void f( )`, `{ }`).
    fn within_opt(open: char, close: char, pad: bool, pad_empty: bool, inner: &str) -> String {
        if inner.is_empty() {
            return if pad_empty {
                format!("{} {}", open, close)
            } else {
                format!("{}{}", open, close)
            };
        }
        Self::within(open, close, pad, inner)
    }

    /// Render a keyword condition `(expr)` by destructuring its outer
    /// `parenthesized_expression`: the condition's inner expression is
    /// rendered and the paren pair rebuilt with the keyword's own
    /// `SPACE_WITHIN_*` toggle, so plain `SPACE_WITHIN_PARENTHESES` does not
    /// leak into `if` / `while` / `switch` / `synchronized` conditions.
    fn keyword_cond(&self, node: Node<'s>, indent: usize, c: usize, pad: bool) -> String {
        let inner = node
            .named_child(0)
            .map(|n| self.expr(n, indent, c + 1))
            .unwrap_or_default();
        Self::within('(', ')', pad, &inner)
    }

    /// Flat variant of [`Self::keyword_cond`] for the one-line collapse
    /// paths, so a collapsed candidate matches the multi-line padding.
    fn flat_keyword_cond(&self, node: Node<'s>, pad: bool) -> String {
        let inner = node
            .named_child(0)
            .map(|n| self.flat(n))
            .unwrap_or_default();
        Self::within('(', ')', pad, &inner)
    }

    /// Separator emitted around a binary / assignment operator token: one
    /// space when the operator class's `SPACE_AROUND_*` toggle is on, nothing
    /// when off. Unary operators, `instanceof`, ternary `?` / `:`, the
    /// annotation element-value `=`, the switch `->` and the method-reference
    /// `::` are handled at their own sites, not here.
    fn op_sep(&self, op: &str) -> &'static str {
        let on = match op {
            "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>="
            | ">>>=" => self.style.space_around_assignment_operators,
            "&&" | "||" => self.style.space_around_logical_operators,
            "==" | "!=" => self.style.space_around_equality_operators,
            "<" | ">" | "<=" | ">=" => self.style.space_around_relational_operators,
            "&" | "|" | "^" => self.style.space_around_bitwise_operators,
            "+" | "-" => self.style.space_around_additive_operators,
            "*" | "/" | "%" => self.style.space_around_multiplicative_operators,
            "<<" | ">>" | ">>>" => self.style.space_around_shift_operators,
            _ => return " ",
        };
        Self::sep(on)
    }

    /// Separator between list items joined on one line: a space before the
    /// comma when `SPACE_BEFORE_COMMA` is on, and a space after when the
    /// caller's after toggle is on (`SPACE_AFTER_COMMA` at the
    /// call/declaration/annotation/array/record/lambda/throws/implements sites,
    /// `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS` at `flat_type_args`).
    fn comma_sep(&self, after: bool) -> &'static str {
        match (self.style.space_before_comma, after) {
            (true, true) => " , ",
            (true, false) => " ,",
            (false, true) => ", ",
            (false, false) => ",",
        }
    }

    /// Separator around a ternary `?`: space before per `SPACE_BEFORE_QUEST`,
    /// space after per `SPACE_AFTER_QUEST`.
    fn quest_sep(&self) -> &'static str {
        match (self.style.space_before_quest, self.style.space_after_quest) {
            (true, true) => " ? ",
            (true, false) => " ?",
            (false, true) => "? ",
            (false, false) => "?",
        }
    }

    /// Separator around a ternary `:`: space before per `SPACE_BEFORE_COLON`,
    /// space after per `SPACE_AFTER_COLON`.
    fn colon_sep(&self) -> &'static str {
        match (self.style.space_before_colon, self.style.space_after_colon) {
            (true, true) => " : ",
            (true, false) => " :",
            (false, true) => ": ",
            (false, false) => ":",
        }
    }

    /// Separator around the enhanced-`for` colon: space before per
    /// `SPACE_BEFORE_COLON_IN_FOREACH`, space after per `SPACE_AFTER_COLON`.
    fn foreach_colon_sep(&self) -> &'static str {
        match (
            self.style.space_before_colon_in_foreach,
            self.style.space_after_colon,
        ) {
            (true, true) => " : ",
            (true, false) => " :",
            (false, true) => ": ",
            (false, false) => ":",
        }
    }

    /// Render an `update_expression` (`i++`, `++i`) from its children so
    /// `SPACE_AROUND_UNARY_OPERATOR` applies. The grammar gives it no fields:
    /// one named operand child plus an anonymous `++` / `--` token, prefix vs
    /// postfix decided by token position. Nodes with extra children (e.g.
    /// comments) are echoed verbatim (R4).
    fn update_expr(&self, node: Node<'s>, indent: usize, c: usize, flat: bool) -> String {
        let ch = self.all_ch(node);
        let sep = Self::sep(self.style.space_around_unary_operator);
        match ch.as_slice() {
            // postfix: `operand ++`
            [operand, op] if operand.is_named() && !op.is_named() => {
                let o = if flat {
                    self.flat(*operand)
                } else {
                    self.expr(*operand, indent, c)
                };
                format!("{}{}{}", o, sep, self.txt(*op))
            }
            // prefix: `++ operand`
            [op, operand] if !op.is_named() && operand.is_named() => {
                let oc = if flat {
                    0
                } else {
                    c + self.txt(*op).len() + sep.len()
                };
                let o = if flat {
                    self.flat(*operand)
                } else {
                    self.expr(*operand, indent, oc)
                };
                format!("{}{}{}", self.txt(*op), sep, o)
            }
            _ => self.txt(node).to_string(),
        }
    }

    /// Render a `method_reference` (`A::new`, `A::<T>new`, `obj::m`) from its
    /// children so `SPACE_AROUND_METHOD_REF_DBL_COLON` applies. The node has
    /// no fields; children are `qualifier :: [type_arguments] name`. Nodes
    /// with comment children or unexpected tokens are echoed verbatim (R4).
    fn method_ref(&self, node: Node<'s>) -> String {
        let ch = self.all_ch(node);
        for n in &ch {
            if n.is_named() {
                if matches!(n.kind(), "line_comment" | "block_comment") {
                    return self.txt(node).to_string();
                }
            } else if !matches!(self.txt(*n), "::" | "new") {
                return self.txt(node).to_string();
            }
        }
        let sep = Self::sep(self.style.space_around_method_ref_dbl_colon);
        let mut out = String::new();
        for n in ch {
            if n.is_named() {
                if n.kind() == "type_arguments" {
                    out.push_str(&self.flat_type_args(n));
                } else {
                    out.push_str(&self.flat(n));
                }
            } else if self.txt(n) == "::" {
                out.push_str(sep);
                out.push_str("::");
                out.push_str(sep);
            } else {
                out.push_str(self.txt(n));
            }
        }
        out
    }

    /// Find the `modifiers` child node by kind (tree-sitter-java 0.23 does not
    /// give it a field name).
    fn get_mods(&self, n: Node<'s>) -> Option<Node<'s>> {
        self.named(n).into_iter().find(|c| c.kind() == "modifiers")
    }

    /// Find the `throws` clause node by kind. The grammar does not give it a
    /// field name, so it must be located among the children.
    fn get_throws(&self, n: Node<'s>) -> Option<Node<'s>> {
        self.all_ch(n).into_iter().find(|c| c.kind() == "throws")
    }

    // ── blank-line spacing (KEEP_BLANK_LINES_* caps + BLANK_LINES_* minimums) ──

    /// Number of blank lines in the source byte range `[prev_end, next_start)`:
    /// whitespace-only lines strictly between the two constructs. Comment text
    /// is content, so a comment line is never counted as blank.
    fn blank_lines_between(&self, prev_end: usize, next_start: usize) -> usize {
        if prev_end >= next_start {
            return 0;
        }
        let slice = &self.src[prev_end..next_start.min(self.src.len())];
        let text = std::str::from_utf8(slice).unwrap_or("");
        let segments: Vec<&str> = text.split('\n').collect();
        // The first segment is the tail of the previous line and the last is
        // the head (indentation) of the next line; only the full lines between
        // them can be blank.
        let mut blanks = 0;
        for seg in segments
            .iter()
            .take(segments.len().saturating_sub(1))
            .skip(1)
        {
            if seg.trim().is_empty() {
                blanks += 1;
            }
        }
        blanks
    }

    /// Push `n` blank lines onto `out` (each `'\n'` after a terminated line
    /// produces one blank line).
    fn push_blanks(&self, out: &mut String, n: usize) {
        for _ in 0..n {
            out.push('\n');
        }
    }

    /// Insert the configured blank lines before a construct that starts at
    /// `cur_start`, based on the source gap after the content ending at
    /// `prev_end` (none when `prev_end` is `None`, i.e. at the file start).
    fn insert_gap(
        &self,
        out: &mut String,
        prev_end: Option<usize>,
        cur_start: usize,
        keep_cap: u32,
        required_min: u32,
    ) {
        if let Some(pe) = prev_end {
            let existing = self.blank_lines_between(pe, cur_start);
            let blanks = self.spacing(existing, keep_cap, required_min);
            self.push_blanks(out, blanks);
        }
    }

    /// True when `node` carries an annotation among its modifiers.
    fn has_annotation(&self, node: Node<'s>) -> bool {
        self.get_mods(node).map_or(false, |mods| {
            self.all_ch(mods)
                .into_iter()
                .any(|c| matches!(c.kind(), "annotation" | "marker_annotation"))
        })
    }

    /// Whether a member carries any extra comment children.
    fn is_comment_node(&self, node: Node<'s>) -> bool {
        node.is_extra() || matches!(node.kind(), "line_comment" | "block_comment")
    }

    /// Governing `BLANK_LINES_*` "around" minimum for one class-body member:
    /// fields (annotated fields use `BLANK_LINES_AROUND_FIELD_WITH_ANNOTATIONS`),
    /// methods/constructors, nested types and initializer blocks; interfaces
    /// use the `*_IN_INTERFACE` variants for fields and methods.
    fn member_around_min(&self, m: Node<'s>, kind: BodyKind) -> u32 {
        let s = self.style;
        let in_interface = kind == BodyKind::Interface;
        match m.kind() {
            "field_declaration" | "constant_declaration" => {
                if in_interface {
                    s.blank_lines_around_field_in_interface
                } else if self.has_annotation(m) {
                    s.blank_lines_around_field_with_annotations
                } else {
                    s.blank_lines_around_field
                }
            }
            "method_declaration"
            | "constructor_declaration"
            | "compact_constructor_declaration" => {
                if in_interface {
                    s.blank_lines_around_method_in_interface
                } else {
                    s.blank_lines_around_method
                }
            }
            "class_declaration"
            | "interface_declaration"
            | "enum_declaration"
            | "record_declaration" => s.blank_lines_around_class,
            // A bare `block` child of a class body is an instance initializer
            // (the grammar models it directly, without an `instance_initializer`
            // wrapper).
            "static_initializer" | "instance_initializer" | "block" => {
                s.blank_lines_around_initializer
            }
            _ => 0,
        }
    }

    /// Minimum blank lines after a body's header line (before its first
    /// member): anonymous bodies use `BLANK_LINES_AFTER_ANONYMOUS_CLASS_HEADER`,
    /// everything else `BLANK_LINES_AFTER_CLASS_HEADER`.
    fn body_header_min(&self, kind: BodyKind) -> u32 {
        if kind == BodyKind::Anonymous {
            self.style.blank_lines_after_anonymous_class_header
        } else {
            self.style.blank_lines_after_class_header
        }
    }

    /// The shared blank-line rule used at every vertical gap:
    /// `emitted = max(min(existing, keep_cap), required_min)`.
    fn spacing(&self, existing: usize, keep_cap: u32, required_min: u32) -> usize {
        (existing.min(keep_cap as usize)).max(required_min as usize)
    }

    /// Blank lines to emit before the class-body member `m`. `prev` is the
    /// previous content member (when the header has already been passed) and
    /// `anchor` the source byte of the body's opening `{` (the gap for the
    /// first member is measured from there). `keep` is
    /// `KEEP_BLANK_LINES_IN_DECLARATIONS`.
    fn member_gap(
        &self,
        prev: Option<Node<'s>>,
        m: Node<'s>,
        anchor: usize,
        kind: BodyKind,
    ) -> usize {
        let (start, min) = match prev {
            None => (anchor, self.body_header_min(kind)),
            Some(p) => (
                p.end_byte(),
                self.member_around_min(p, kind)
                    .max(self.member_around_min(m, kind)),
            ),
        };
        let existing = self.blank_lines_between(start, m.start_byte());
        self.spacing(existing, self.style.keep_blank_lines_in_declarations, min)
    }

    // ── program ───────────────────────────────────────────────────────────────

    fn program(&self, node: Node<'s>) -> String {
        let mut pkg: Option<Node<'s>> = None;
        let mut imports: Vec<Node<'s>> = Vec::new();
        let mut top_types: Vec<Node<'s>> = Vec::new();
        let mut header_comments: Vec<Node<'s>> = Vec::new();

        for child in self.named(node) {
            match child.kind() {
                "package_declaration" => pkg = Some(child),
                "import_declaration" => imports.push(child),
                "line_comment" | "block_comment"
                    if pkg.is_none() && imports.is_empty() && top_types.is_empty() =>
                {
                    header_comments.push(child);
                }
                _ => top_types.push(child),
            }
        }

        let s = self.style;
        let mut out = String::new();

        for c in &header_comments {
            out.push_str(self.txt(*c));
            out.push('\n');
        }

        // Byte offset of the end of the content emitted so far (None when the
        // file does not yet contain anything, so no leading gap is inserted).
        let mut prev_end: Option<usize> = header_comments.last().map(|c| c.end_byte());
        let has_pkg = pkg.is_some();
        let has_imports = !imports.is_empty();

        if let Some(p) = pkg {
            self.insert_gap(
                &mut out,
                prev_end,
                p.start_byte(),
                s.keep_blank_lines_between_package_declaration_and_header,
                s.blank_lines_before_package,
            );
            out.push_str(&self.package_decl(p));
            prev_end = Some(p.end_byte());
        }

        if has_imports {
            // Names of top-level types declared in this file; on-demand imports
            // are shadowed by them, so merging must not mask a local type.
            let local_types: Vec<String> = top_types
                .iter()
                .filter_map(|n| self.fld(*n, "name").map(|nm| self.txt(nm).to_string()))
                .collect();
            let first = imports[0];
            self.insert_gap(
                &mut out,
                prev_end,
                first.start_byte(),
                s.keep_blank_lines_in_declarations,
                if has_pkg {
                    s.blank_lines_after_package
                        .max(s.blank_lines_before_imports)
                } else {
                    s.blank_lines_before_imports
                },
            );
            let last_import = imports[imports.len() - 1];
            out.push_str(&self.imports(imports, &local_types));
            prev_end = Some(last_import.end_byte());
        }

        for (i, ty) in top_types.iter().enumerate() {
            let (keep_cap, required_min) = if i == 0 {
                // The gap before the first top-level type is the section
                // boundary after imports/package (or the header when neither
                // exists); later top-level types are spaced by
                // `BLANK_LINES_AROUND_CLASS`.
                let min = if has_imports {
                    s.blank_lines_after_imports
                } else if has_pkg {
                    s.blank_lines_after_package
                } else {
                    s.blank_lines_around_class
                };
                (s.keep_blank_lines_in_declarations, min)
            } else {
                (
                    s.keep_blank_lines_in_declarations,
                    s.blank_lines_around_class,
                )
            };
            self.insert_gap(&mut out, prev_end, ty.start_byte(), keep_cap, required_min);
            out.push_str(&self.type_decl(*ty, 0));
            out.push('\n');
            prev_end = Some(ty.end_byte());
        }

        out
    }

    fn package_decl(&self, node: Node<'s>) -> String {
        // Try field "name" first, fall back to scanning named children
        let name = self
            .fld(node, "name")
            .map(|n| self.txt(n))
            .unwrap_or_else(|| {
                self.named(node)
                    .into_iter()
                    .find(|n| matches!(n.kind(), "scoped_identifier" | "identifier"))
                    .map(|n| self.txt(n))
                    .unwrap_or("")
            });
        format!("package {};\n", name)
    }

    // ── imports ───────────────────────────────────────────────────────────────

    fn imports(&self, nodes: Vec<Node<'s>>, local_types: &[String]) -> String {
        let merged = self.merge_on_demand_imports(&nodes, local_types);

        // Preserve original order but ensure a blank line before java/javax
        let is_java = |t: &String| t.contains(" java.") || t.contains(" javax.");

        let third_party: Vec<&String> = merged.iter().filter(|t| !is_java(t)).collect();
        let java: Vec<&String> = merged.iter().filter(|t| is_java(t)).collect();

        let mut out = String::new();
        for n in &third_party {
            out.push_str(n);
            out.push('\n');
        }
        if !third_party.is_empty() && !java.is_empty() {
            out.push('\n');
        }
        for n in &java {
            out.push_str(n);
            out.push('\n');
        }
        out
    }

    /// Collapses single-type imports from the same package into one on-demand
    /// import (`import pkg.*;`) when more than
    /// [`class_count_to_use_import_on_demand`](JavaStyle::class_count_to_use_import_on_demand)
    /// imports of that package are present.
    ///
    /// Merging is deliberately conservative: it is skipped when the file
    /// already uses a wildcard import, when a simple name would become
    /// ambiguous (imported from another package) or when it collides with a
    /// top-level type declared in the same file. Static imports are never
    /// merged.
    fn merge_on_demand_imports(&self, nodes: &[Node<'s>], local_types: &[String]) -> Vec<String> {
        struct Entry {
            is_static: bool,
            pkg: String,
            simple: String,
            is_wildcard: bool,
        }

        let entries: Vec<Entry> = nodes
            .iter()
            .map(|n| {
                let t = self.txt(*n).trim();
                let mut e = Entry {
                    is_static: false,
                    pkg: String::new(),
                    simple: String::new(),
                    is_wildcard: false,
                };
                if let Some(rest) = t.strip_prefix("import ") {
                    let (is_static, path) = match rest.strip_prefix("static ") {
                        Some(p) => (true, p),
                        None => (false, rest),
                    };
                    let path = path.trim().trim_end_matches(';').trim();
                    e.is_static = is_static;
                    if let Some((pkg, simple)) = path.rsplit_once('.') {
                        e.pkg = pkg.to_string();
                        e.simple = simple.to_string();
                        e.is_wildcard = simple == "*";
                    } else if !is_static {
                        // A single segment (default package); leave untouched.
                        e.simple = path.to_string();
                    }
                }
                e
            })
            .collect();

        // Bail out entirely when the file already contains any wildcard import:
        // removing the redundant single imports could change name resolution.
        if entries.iter().any(|e| e.is_wildcard) {
            return nodes
                .iter()
                .map(|n| self.txt(*n).trim().to_string())
                .collect();
        }

        // simple name -> set of packages that import it (non-static only).
        let mut name_pkgs: HashMap<&str, HashSet<&str>> = HashMap::new();
        let mut groups: HashMap<&str, Vec<usize>> = HashMap::new();
        for (i, e) in entries.iter().enumerate() {
            if e.is_static || e.is_wildcard || e.pkg.is_empty() {
                continue;
            }
            name_pkgs
                .entry(e.simple.as_str())
                .or_default()
                .insert(e.pkg.as_str());
            groups.entry(e.pkg.as_str()).or_default().push(i);
        }

        let threshold = self.style.class_count_to_use_import_on_demand as usize;
        let local: HashSet<&str> = local_types.iter().map(|s| s.as_str()).collect();

        // Decide which packages are replaced by a single on-demand import.
        let mut collapse: HashSet<&str> = HashSet::new();
        for (&pkg, idxs) in &groups {
            if idxs.len() <= threshold {
                continue;
            }
            let safe = idxs.iter().all(|&i| {
                let e = &entries[i];
                !local.contains(e.simple.as_str())
                    && name_pkgs
                        .get(e.simple.as_str())
                        .map_or(false, |pkgs| pkgs.len() == 1 && pkgs.contains(pkg))
            });
            if safe {
                collapse.insert(pkg);
            }
        }

        let mut out: Vec<String> = Vec::with_capacity(nodes.len());
        for (i, n) in nodes.iter().enumerate() {
            let e = &entries[i];
            let replaced = !e.is_static && !e.is_wildcard && collapse.contains(e.pkg.as_str());
            if replaced {
                // Emit the on-demand import once, at the first import's position.
                if groups[&e.pkg.as_str()][0] == i {
                    out.push(format!("import {}.*;", e.pkg));
                }
            } else {
                out.push(self.txt(*n).trim().to_string());
            }
        }
        out
    }

    // ── type declarations ─────────────────────────────────────────────────────

    fn type_decl(&self, node: Node<'s>, indent: usize) -> String {
        match node.kind() {
            "class_declaration" => self.class_decl(node, indent),
            "interface_declaration" => self.iface_decl(node, indent),
            "enum_declaration" => self.enum_decl(node, indent),
            "record_declaration" => self.record_decl(node, indent),
            _ => self.txt(node).to_string(),
        }
    }

    fn class_decl(&self, node: Node<'s>, indent: usize) -> String {
        let mut header = String::new();

        if let Some(mods) = self.get_mods(node) {
            header.push_str(&self.modifiers(mods, indent));
            if !header.ends_with('\n') && !header.is_empty() {
                header.push(' ');
            }
        }

        header.push_str("class ");
        header.push_str(self.fld(node, "name").map(|n| self.txt(n)).unwrap_or(""));

        if let Some(tp) = self.fld(node, "type_parameters") {
            if self.style.space_before_type_parameter_list {
                header.push(' ');
            }
            header.push_str(&self.flat_type_params(tp));
        }
        if let Some(sc) = self.fld(node, "superclass") {
            // The `superclass` node starts with the `extends` keyword; print a
            // canonical keyword followed by just the supertype.
            header.push_str(" extends ");
            if let Some(ty) = self.named(sc).first() {
                header.push_str(&self.flat_type(*ty));
            }
        }
        if let Some(ifaces) = self.fld(node, "interfaces") {
            header.push_str(" implements ");
            header.push_str(&self.flat_type_list(ifaces));
        }

        let body = self
            .fld(node, "body")
            .map(|n| self.class_body(n, indent, BodyKind::Class))
            .unwrap_or_default();

        self.with_brace(header, body, indent, self.style.class_brace_style)
    }

    fn iface_decl(&self, node: Node<'s>, indent: usize) -> String {
        let mut header = String::new();

        if let Some(mods) = self.get_mods(node) {
            header.push_str(&self.modifiers(mods, indent));
            if !header.ends_with('\n') && !header.is_empty() {
                header.push(' ');
            }
        }

        header.push_str("interface ");
        header.push_str(self.fld(node, "name").map(|n| self.txt(n)).unwrap_or(""));

        if let Some(tp) = self.fld(node, "type_parameters") {
            if self.style.space_before_type_parameter_list {
                header.push(' ');
            }
            header.push_str(&self.flat_type_params(tp));
        }
        if let Some(ext) = self
            .all_ch(node)
            .into_iter()
            .find(|c| c.kind() == "extends_interfaces")
        {
            header.push_str(" extends ");
            header.push_str(&self.flat_type_list(ext));
        }

        let body = self
            .fld(node, "body")
            .map(|n| self.class_body(n, indent, BodyKind::Interface))
            .unwrap_or_default();

        self.with_brace(header, body, indent, self.style.class_brace_style)
    }

    fn enum_decl(&self, node: Node<'s>, indent: usize) -> String {
        let mut header = String::new();

        if let Some(mods) = self.get_mods(node) {
            header.push_str(&self.modifiers(mods, indent));
            if !header.ends_with('\n') && !header.is_empty() {
                header.push(' ');
            }
        }

        header.push_str("enum ");
        header.push_str(self.fld(node, "name").map(|n| self.txt(n)).unwrap_or(""));

        if let Some(ifaces) = self.fld(node, "interfaces") {
            header.push_str(" implements ");
            header.push_str(&self.flat_type_list(ifaces));
        }

        // Enum body: keep original text for enum constants; format methods
        if let Some(body) = self.fld(node, "body") {
            let body_str = self.enum_body(node, body, indent);
            self.with_brace(header, body_str, indent, self.style.class_brace_style)
        } else {
            header
        }
    }

    fn enum_body(&self, _enum_node: Node<'s>, body: Node<'s>, indent: usize) -> String {
        // Collect enum constants and member declarations. Constants keep their
        // original text and comma layout; the member declarations after the
        // `;` are routed through the same member-spacing path as class bodies.
        let inner = indent + 1;
        let mut out = String::from("{\n");
        let mut in_constants = true;
        let mut first = true;
        let mut last_const: Option<Node<'s>> = None;
        let mut last_content: Option<Node<'s>> = None;

        for child in self.named(body) {
            match child.kind() {
                "enum_constant" => {
                    if !first {
                        out.push_str(",\n");
                    }
                    out.push_str(&self.ind(inner));
                    out.push_str(self.txt(child));
                    first = false;
                    last_const = Some(child);
                    last_content = Some(child);
                }
                "enum_body_declarations" => {
                    in_constants = false;
                    if !first {
                        out.push_str(";\n");
                    }
                    let mut prev = last_const;
                    for member in self.named(child) {
                        if self.is_comment_node(member) {
                            out.push_str(&self.ind(inner));
                            out.push_str(self.txt(member));
                            out.push('\n');
                            continue;
                        }
                        let gap = self.member_gap(prev, member, body.start_byte(), BodyKind::Class);
                        self.push_blanks(&mut out, gap);
                        out.push_str(&self.ind(inner));
                        out.push_str(&self.class_member(member, inner));
                        out.push('\n');
                        prev = Some(member);
                        last_content = Some(member);
                    }
                }
                _ => {}
            }
        }

        // Terminate the last constant line when the body ends with a constant
        // list and no `;` / declaration section follows.
        if in_constants && !first {
            out.push('\n');
        }

        // Closing gap: blank lines before the closing brace.
        if let Some(lc) = last_content {
            let existing =
                self.blank_lines_between(lc.end_byte(), body.end_byte().saturating_sub(1));
            let blanks = self.spacing(
                existing,
                self.style.keep_blank_lines_before_rbrace,
                self.style.blank_lines_before_class_end,
            );
            self.push_blanks(&mut out, blanks);
        }

        out.push_str(&self.ind(indent));
        out.push('}');
        out
    }

    fn record_decl(&self, node: Node<'s>, indent: usize) -> String {
        let mut header = String::new();

        if let Some(mods) = self.get_mods(node) {
            header.push_str(&self.modifiers(mods, indent));
            if !header.ends_with('\n') && !header.is_empty() {
                header.push(' ');
            }
        }

        header.push_str("record ");
        header.push_str(self.fld(node, "name").map(|n| self.txt(n)).unwrap_or(""));

        if let Some(tp) = self.fld(node, "type_parameters") {
            if self.style.space_before_type_parameter_list {
                header.push(' ');
            }
            header.push_str(&self.flat_type_params(tp));
        }

        let c = self.col_after(0, &self.ind(indent));
        if let Some(params) = self.fld(node, "parameters") {
            header.push_str(&self.record_components(params, indent, c, &header));
        }

        if let Some(ifaces) = self.fld(node, "interfaces") {
            header.push_str(" implements ");
            header.push_str(&self.flat_type_list(ifaces));
        }

        let body = self
            .fld(node, "body")
            .map(|n| self.class_body(n, indent, BodyKind::Class))
            .unwrap_or_default();

        self.with_brace(header, body, indent, self.style.class_brace_style)
    }

    /// Formats a record header's component list (`(…)`).
    ///
    /// Honors `RECORD_COMPONENTS_WRAP`, `NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER`
    /// and `ALIGN_MULTILINE_RECORDS`. When a wrap is needed, components are
    /// placed one per line; with `new_line_after_lparen_in_record_header` the
    /// opening paren stays on the header line and components start on the next
    /// line, otherwise the first component stays inline after the paren.
    fn record_components(&self, node: Node<'s>, indent: usize, c: usize, header: &str) -> String {
        let comps = self.named(node);
        if comps.is_empty() {
            return "()".to_string();
        }

        let parts: Vec<String> = comps.iter().map(|&p| self.flat_param(p)).collect();
        let flat = format!(
            "({})",
            parts.join(self.comma_sep(self.style.space_after_comma))
        );

        // Column of the opening paren within the physical line (tab-aware:
        // the header may carry indentation from annotation lines).
        let open_col = self.col_after(c, header);

        let should_wrap = match self.style.record_components_wrap {
            WrapStyle::DoNotWrap => false,
            WrapStyle::WrapAlways => true,
            _ => !self.fits(open_col, &flat),
        };
        if !should_wrap {
            return flat;
        }

        // Alignment column: right after '(' when aligning, otherwise a single
        // continuation indent level below the record header.
        let inner_indent = if self.style.align_multiline_records {
            " ".repeat(open_col + 1)
        } else {
            self.cont(indent)
        };

        if self.style.new_line_after_lparen_in_record_header {
            let lines: Vec<String> = parts
                .iter()
                .map(|p| format!("{}{}", inner_indent, p))
                .collect();
            format!("(\n{}\n{})", lines.join(",\n"), self.ind(indent))
        } else {
            // First component stays on the header line, the rest wrap.
            let rest: Vec<String> = parts
                .iter()
                .skip(1)
                .map(|p| format!("{}{}", inner_indent, p))
                .collect();
            if rest.is_empty() {
                flat
            } else {
                format!("({},\n{})", parts[0], rest.join(",\n"))
            }
        }
    }

    /// Attach `{ body }` to header following the brace style. The space
    /// between an end-of-line brace and the header follows
    /// `SPACE_BEFORE_CLASS_LBRACE`; a next-line brace sits at line start,
    /// where the toggle is moot.
    fn with_brace(&self, header: String, body: String, indent: usize, style: BraceStyle) -> String {
        match style {
            BraceStyle::NextLine | BraceStyle::NextLineShifted | BraceStyle::NextLineShifted2 => {
                format!("{}\n{}{}", header, self.ind(indent), body)
            }
            _ => format!(
                "{}{}{}",
                header,
                self.sp(self.style.space_before_class_lbrace),
                body
            ),
        }
    }

    // ── class body ────────────────────────────────────────────────────────────

    fn class_body(&self, node: Node<'s>, indent: usize, kind: BodyKind) -> String {
        let members = self.named(node);
        if members.is_empty() {
            return Self::within_opt(
                '{',
                '}',
                self.style.space_within_braces,
                self.style.space_within_braces,
                "",
            );
        }

        let inner = indent + 1;
        let anchor = node.start_byte(); // the opening `{`
        let mut out = String::from("{\n");
        let mut prev: Option<Node<'s>> = None;
        let mut last: Option<Node<'s>> = None;

        for m in members {
            if self.is_comment_node(m) {
                // Comments are content but take no part in the spacing
                // options: they are emitted in place, without their own gap.
                out.push_str(&self.ind(inner));
                out.push_str(self.txt(m));
                out.push('\n');
                last = Some(m);
                continue;
            }

            let gap = self.member_gap(prev, m, anchor, kind);
            self.push_blanks(&mut out, gap);
            out.push_str(&self.ind(inner));
            out.push_str(&self.class_member(m, inner));
            out.push('\n');
            prev = Some(m);
            last = Some(m);
        }

        // Closing gap: blank lines before the closing brace. Measured from
        // the last emitted member (comments included) so re-formatting the
        // output reproduces the same count.
        if let Some(l) = last {
            let existing =
                self.blank_lines_between(l.end_byte(), node.end_byte().saturating_sub(1));
            let blanks = self.spacing(
                existing,
                self.style.keep_blank_lines_before_rbrace,
                self.style.blank_lines_before_class_end,
            );
            self.push_blanks(&mut out, blanks);
        }

        out.push_str(&self.ind(indent));
        out.push('}');
        out
    }

    fn class_member(&self, node: Node<'s>, indent: usize) -> String {
        // Column at which this member's line starts; tab-aware so margin
        // decisions match the visual column of tab-indented output.
        let c = self.col_after(0, &self.ind(indent));
        match node.kind() {
            "method_declaration" => self.method_decl(node, indent, c),
            "constructor_declaration" => self.constructor_decl(node, indent, c),
            "compact_constructor_declaration" => self.compact_constructor_decl(node, indent, c),
            "field_declaration" => self.field_decl(node, indent, c),
            "class_declaration" => self.class_decl(node, indent),
            "interface_declaration" => self.iface_decl(node, indent),
            "enum_declaration" => self.enum_decl(node, indent),
            "record_declaration" => self.record_decl(node, indent),
            "static_initializer" | "instance_initializer" => {
                let blk = self
                    .named(node)
                    .into_iter()
                    .find(|n| n.kind() == "block")
                    .map(|n| self.block(n, indent, c, 0))
                    .unwrap_or_default();
                if node.kind() == "static_initializer" {
                    format!("static {}", blk)
                } else {
                    blk
                }
            }
            "line_comment" | "block_comment" => self.txt(node).to_string(),
            _ => self.txt(node).to_string(),
        }
    }

    // ── method / constructor / field ──────────────────────────────────────────

    fn method_decl(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let mut out = String::new();

        if let Some(mods) = self.get_mods(node) {
            out.push_str(&self.modifiers(mods, indent));
            if !out.ends_with(' ') && !out.ends_with('\n') {
                out.push(' ');
            }
        }

        // type_parameters (generic method)
        if let Some(tp) = self.fld(node, "type_parameters") {
            out.push_str(&self.flat_type_params(tp));
            out.push(' ');
        }

        // return type
        if let Some(ty) = self.fld(node, "type") {
            out.push_str(&self.flat_type(ty));
            out.push(' ');
        }

        // name
        out.push_str(self.fld(node, "name").map(|n| self.txt(n)).unwrap_or(""));

        // parameters — the name→`(` gap follows SPACE_BEFORE_METHOD_PARENTHESES
        if let Some(params) = self.fld(node, "parameters") {
            let gap = self.sp(self.style.space_before_method_parentheses);
            let pcol = c + self.col_after(0, &out) + gap.len();
            out.push_str(gap);
            out.push_str(&self.formal_params(params, indent, pcol, false));
        }

        // throws
        if let Some(throws) = self.get_throws(node) {
            out.push_str(" throws ");
            let excs: Vec<_> = self
                .named(throws)
                .iter()
                .map(|n| self.flat_type(*n))
                .collect();
            out.push_str(&excs.join(self.comma_sep(self.style.space_after_comma)));
        }

        // body or semicolon
        match self.fld(node, "body") {
            Some(body) => self.method_body(body, indent, &mut out, c),
            None => out.push(';'),
        }

        out
    }

    fn constructor_decl(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let mut out = String::new();

        if let Some(mods) = self.get_mods(node) {
            out.push_str(&self.modifiers(mods, indent));
            if !out.ends_with(' ') && !out.ends_with('\n') {
                out.push(' ');
            }
        }

        // type_parameters (generic constructor)
        if let Some(tp) = self.fld(node, "type_parameters") {
            out.push_str(&self.flat_type_params(tp));
            out.push(' ');
        }

        out.push_str(self.fld(node, "name").map(|n| self.txt(n)).unwrap_or(""));

        // parameters — the name→`(` gap follows SPACE_BEFORE_METHOD_PARENTHESES
        if let Some(params) = self.fld(node, "parameters") {
            let gap = self.sp(self.style.space_before_method_parentheses);
            let pcol = c + self.col_after(0, &out) + gap.len();
            out.push_str(gap);
            out.push_str(&self.formal_params(params, indent, pcol, false));
        }

        if let Some(throws) = self.get_throws(node) {
            out.push_str(" throws ");
            let excs: Vec<_> = self
                .named(throws)
                .iter()
                .map(|n| self.flat_type(*n))
                .collect();
            out.push_str(&excs.join(self.comma_sep(self.style.space_after_comma)));
        }

        if let Some(body) = self.fld(node, "body") {
            self.method_body(body, indent, &mut out, c);
        }

        out
    }

    /// Compact constructor of a record (`Foo { ... }`): no parameter list.
    fn compact_constructor_decl(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let mut out = String::new();

        if let Some(mods) = self.get_mods(node) {
            out.push_str(&self.modifiers(mods, indent));
            if !out.ends_with(' ') && !out.ends_with('\n') {
                out.push(' ');
            }
        }

        out.push_str(self.fld(node, "name").map(|n| self.txt(n)).unwrap_or(""));

        if let Some(body) = self.fld(node, "body") {
            self.method_body(body, indent, &mut out, c);
        }

        out
    }

    /// Appends a method/constructor body block to `out`.
    ///
    /// When `KEEP_SIMPLE_METHODS_IN_ONE_LINE` is enabled (and the brace style
    /// keeps the brace on the same line), a body that is a single simple
    /// statement is rendered as `{ stmt }` if the resulting line fits within
    /// the right margin.
    fn method_body(&self, body: Node<'s>, indent: usize, out: &mut String, c: usize) {
        if self.style.keep_simple_methods_in_one_line
            && matches!(
                self.style.method_brace_style,
                BraceStyle::EndOfLine | BraceStyle::NextLineIfWrapped
            )
        {
            if let Some(one) = self.one_line_body(body) {
                // Column at which the current (last) line of `out` starts.
                let gap = self.sp(self.style.space_before_method_lbrace);
                let width = self.col_after(c, out);
                if width + gap.len() + one.len() <= self.style.right_margin as usize {
                    out.push_str(gap);
                    out.push_str(&one);
                    return;
                }
            }
        }
        let body_str = self.block(body, indent, c, self.style.blank_lines_before_method_body);
        out.push_str(&self.brace_before_body(indent, self.style.method_brace_style, &body_str));
    }

    fn field_decl(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let mut out = String::new();

        if let Some(mods) = self.get_mods(node) {
            out.push_str(&self.modifiers(mods, indent));
            if !out.ends_with(' ') && !out.ends_with('\n') {
                out.push(' ');
            }
        }

        let ty = self
            .fld(node, "type")
            .map(|n| self.flat_type(n))
            .unwrap_or_default();
        out.push_str(&ty);

        let decls: Vec<Node<'s>> = self
            .named(node)
            .into_iter()
            .filter(|n| n.kind() == "variable_declarator")
            .collect();

        // Single declarator whose initialiser can be wrapped at the operator
        // (always under `KEEP_LINE_BREAKS` when the declaration spans rows).
        if decls.len() == 1
            && !out.contains('\n')
            && (self.style.assignment_wrap != WrapStyle::DoNotWrap || self.keep_wrapped(node))
        {
            if let Some(val) = self.fld(decls[0], "value") {
                let name = self
                    .fld(decls[0], "name")
                    .map(|n| self.txt(n))
                    .unwrap_or("");
                let prefix = format!("{} {}", out, name);
                return format!(
                    "{};",
                    self.assign_expr(val, indent, c, &prefix, "=", self.keep_wrapped(node))
                );
            }
        }

        let decl_strs: Vec<String> = decls
            .iter()
            .map(|&d| {
                let name = self.fld(d, "name").map(|n| self.txt(n)).unwrap_or("");
                if let Some(val) = self.fld(d, "value") {
                    let sep = self.op_sep("=");
                    let val_col =
                        c + self.col_after(0, &out) + 1 + name.len() + sep.len() + 1 + sep.len();
                    let val_str = self.expr(val, indent, val_col);
                    format!("{}{}={}{}", name, sep, sep, val_str)
                } else {
                    name.to_string()
                }
            })
            .collect();

        out.push(' ');
        out.push_str(&decl_strs.join(self.comma_sep(self.style.space_after_comma)));
        out.push(';');
        out
    }

    /// Returns the text to place between the declaration header and the block
    /// body, according to the brace style. Caller should `push_str` the
    /// result. The space before an end-of-line brace follows
    /// `SPACE_BEFORE_METHOD_LBRACE`.
    fn brace_before_body(&self, indent: usize, style: BraceStyle, body: &str) -> String {
        match style {
            BraceStyle::NextLine | BraceStyle::NextLineShifted | BraceStyle::NextLineShifted2 => {
                format!("\n{}{}", self.ind(indent), body)
            }
            _ => format!("{}{}", self.sp(self.style.space_before_method_lbrace), body),
        }
    }

    // ── modifiers ─────────────────────────────────────────────────────────────

    /// Format `modifiers` node. Returns annotations each on their own line
    /// (already including the trailing newline+indent), followed by keyword
    /// modifiers joined by spaces. The caller appends a space before the next
    /// token when the result doesn't already end with a newline.
    fn modifiers(&self, node: Node<'s>, indent: usize) -> String {
        // Use all children: keyword modifiers (public, static, …) are UNNAMED nodes.
        let children = self.all_ch(node);
        let mut ann_lines: Vec<String> = Vec::new();
        let mut keywords: Vec<String> = Vec::new();

        for ch in children {
            match ch.kind() {
                "annotation" | "marker_annotation" => {
                    ann_lines.push(self.annotation(ch, indent));
                }
                _ => {
                    let t = self.txt(ch).trim().to_string();
                    if !t.is_empty() {
                        keywords.push(t);
                    }
                }
            }
        }

        let mut out = String::new();
        for ann in &ann_lines {
            out.push_str(ann);
            out.push('\n');
            out.push_str(&self.ind(indent));
        }
        out.push_str(&keywords.join(" "));
        out
    }

    // ── annotations ──────────────────────────────────────────────────────────

    fn annotation(&self, node: Node<'s>, indent: usize) -> String {
        let name = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");

        if node.kind() == "marker_annotation" {
            return format!("@{}", name);
        }

        let args_node = match self.fld(node, "arguments") {
            Some(n) => n,
            None => return format!("@{}", name),
        };

        // Try flat
        let flat_inner = self.flat_ann_args(args_node);
        let flat_ann = format!(
            "@{}{}{}",
            name,
            self.sp(self.style.space_before_anotation_parameter_list),
            self.ann_parens(&flat_inner),
        );

        // Decide whether to expand.
        // ChopDownIfLong (value 5) = expand only when the flat form is too long.
        // When expanded, each argument (and each array element) goes on its own line.
        // `KEEP_LINE_BREAKS` overrides the wrap-code decision when the
        // annotation's argument list spans source rows.
        let needs_expand = (self.keep_wrapped(args_node)
            || match self.style.annotation_parameter_wrap {
                WrapStyle::DoNotWrap => false,
                WrapStyle::WrapAlways => true,
                // WrapIfLong | ChopDownIfLong: only expand when the flat form overflows
                _ => !self.fits(0, &flat_ann),
            })
            && self.ann_args_need_expand(args_node);

        if needs_expand {
            self.annotation_expanded(name, args_node, indent)
        } else {
            flat_ann
        }
    }

    /// True when the annotation argument list contains an array initializer
    /// (which should trigger expansion under ChopDownIfLong).
    fn ann_args_need_expand(&self, args: Node<'s>) -> bool {
        for ch in self.named(args) {
            match ch.kind() {
                "element_value_array_initializer" => return true,
                "element_value_pair" => {
                    if let Some(v) = self.fld(ch, "value") {
                        if v.kind() == "element_value_array_initializer" {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn flat_ann_args(&self, node: Node<'s>) -> String {
        self.named(node)
            .iter()
            .map(|&c| self.flat_ann_arg(c))
            .collect::<Vec<_>>()
            .join(self.comma_sep(self.style.space_after_comma))
    }

    fn flat_ann_arg(&self, node: Node<'s>) -> String {
        match node.kind() {
            "element_value_pair" => {
                let k = self.fld(node, "key").map(|n| self.txt(n)).unwrap_or("");
                let v = self
                    .fld(node, "value")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                format!("{} = {}", k, v)
            }
            "element_value_array_initializer" => self.flat_arr_init(node),
            _ => self.flat(node),
        }
    }

    /// Wrap an annotation's `( arguments )` pair around `inner`: one space
    /// just inside each paren when `SPACE_WITHIN_ANNOTATION_PARENTHESES` is
    /// on, and when `inner` is a bare array initializer (it starts with `{`)
    /// the gap between `(` and `{` follows
    /// `SPACE_BEFORE_ANNOTATION_ARRAY_INITIALIZER_LBRACE` — the
    /// `@SuppressWarnings( {…)` shape. An empty argument list stays `()`.
    fn ann_parens(&self, inner: &str) -> String {
        if inner.is_empty() {
            return "()".to_string();
        }
        let pad = self.style.space_within_annotation_parentheses;
        let lbrace =
            inner.starts_with('{') && self.style.space_before_annotation_array_initializer_lbrace;
        let l = if (pad || lbrace) && !inner.starts_with('\n') {
            " "
        } else {
            ""
        };
        let r = if pad && !inner.ends_with('\n') {
            " "
        } else {
            ""
        };
        format!("({}{}{})", l, inner, r)
    }

    fn annotation_expanded(&self, name: &str, args: Node<'s>, indent: usize) -> String {
        let inner = indent + 1;
        let children = self.named(args);

        // Single element_value_pair with array → expand the array value
        if children.len() == 1 {
            let ch = children[0];
            if ch.kind() == "element_value_pair" {
                let k = self.fld(ch, "key").map(|n| self.txt(n)).unwrap_or("");
                if let Some(v) = self.fld(ch, "value") {
                    if v.kind() == "element_value_array_initializer" {
                        let elems = self.named(v);
                        let elem_strs: Vec<_> = elems
                            .iter()
                            .map(|&e| format!("{}{}", self.ind(inner), self.flat(e)))
                            .collect();
                        return format!(
                            "@{}{}{}",
                            name,
                            self.sp(self.style.space_before_anotation_parameter_list),
                            self.ann_parens(&format!(
                                "{} = {{\n{}\n{}}}",
                                k,
                                elem_strs.join(",\n"),
                                self.ind(indent)
                            )),
                        );
                    }
                }
            }
            // Single array value (no key)
            if ch.kind() == "element_value_array_initializer" {
                let elems = self.named(ch);
                let elem_strs: Vec<_> = elems
                    .iter()
                    .map(|&e| format!("{}{}", self.ind(inner), self.flat(e)))
                    .collect();
                return format!(
                    "@{}{}{}",
                    name,
                    self.sp(self.style.space_before_anotation_parameter_list),
                    self.ann_parens(&format!(
                        "{{\n{}\n{}}}",
                        elem_strs.join(",\n"),
                        self.ind(indent)
                    )),
                );
            }
        }

        // Multiple args: one per line
        let arg_strs: Vec<_> = children
            .iter()
            .map(|&c| format!("{}{}", self.ind(inner), self.flat_ann_arg(c)))
            .collect();
        format!(
            "@{}{}{}",
            name,
            self.sp(self.style.space_before_anotation_parameter_list),
            self.ann_parens(&format!("\n{}\n{}", arg_strs.join(",\n"), self.ind(indent))),
        )
    }

    // ── formal parameters ─────────────────────────────────────────────────────

    fn formal_params(&self, node: Node<'s>, indent: usize, c: usize, is_call: bool) -> String {
        let params = self.named(node);

        if params.is_empty() {
            return Self::within_opt(
                '(',
                ')',
                self.style.space_within_method_parentheses,
                self.style.space_within_empty_method_parentheses,
                "",
            );
        }

        let wrap = if is_call {
            self.style.call_parameters_wrap
        } else {
            self.style.method_parameters_wrap
        };
        let lparen_nl = if is_call {
            self.style.call_parameters_lparen_on_next_line
        } else {
            self.style.method_parameters_lparen_on_next_line
        };
        let rparen_nl = if is_call {
            self.style.call_parameters_rparen_on_next_line
        } else {
            self.style.method_parameters_rparen_on_next_line
        };

        let flat_parts: Vec<String> = params.iter().map(|&p| self.flat_param(p)).collect();
        let flat = Self::within_opt(
            '(',
            ')',
            self.style.space_within_method_parentheses,
            self.style.space_within_empty_method_parentheses,
            &flat_parts.join(self.comma_sep(self.style.space_after_comma)),
        );

        let should_wrap = self.keep_wrapped(node)
            || match wrap {
                WrapStyle::DoNotWrap => false,
                WrapStyle::WrapAlways => true,
                _ => !self.fits(c, &flat),
            };

        if !should_wrap {
            return flat;
        }

        let inner = indent + 1;
        let ind = self.ind(inner);
        let wrapped = flat_parts
            .iter()
            .map(|p| format!("{}{}", ind, p))
            .collect::<Vec<_>>()
            .join(",\n");

        let pad = self.style.space_within_method_parentheses;
        match (lparen_nl, rparen_nl) {
            (true, true) => Self::within(
                '(',
                ')',
                pad,
                &format!("\n{}\n{}", wrapped, self.ind(indent)),
            ),
            (true, false) => Self::within('(', ')', pad, &format!("\n{}", wrapped)),
            (false, true) => {
                Self::within('(', ')', pad, &format!("{}\n{}", wrapped, self.ind(indent)))
            }
            (false, false) => Self::within('(', ')', pad, &format!("\n{}", wrapped)),
        }
    }

    fn flat_param(&self, node: Node<'s>) -> String {
        match node.kind() {
            "formal_parameter" => {
                let mods = self
                    .get_mods(node)
                    .map(|m| {
                        let s = self.flat_mods(m);
                        if s.is_empty() {
                            String::new()
                        } else {
                            format!("{} ", s)
                        }
                    })
                    .unwrap_or_default();
                let ty = self
                    .fld(node, "type")
                    .map(|n| self.flat_type(n))
                    .unwrap_or_default();
                let nm = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");
                format!("{}{} {}", mods, ty, nm)
            }
            "spread_parameter" => {
                // variadic: `Type... name` (annotations may precede the type)
                let ch = self.named(node);
                if ch.len() == 2 && ch[1].kind() == "variable_declarator" {
                    let name = self.fld(ch[1], "name").map(|n| self.txt(n)).unwrap_or("");
                    format!("{}... {}", self.flat_type(ch[0]), name)
                } else {
                    self.txt(node).trim().to_string()
                }
            }
            "receiver_parameter" => self.txt(node).to_string(),
            _ => self.txt(node).to_string(),
        }
    }

    fn flat_mods(&self, node: Node<'s>) -> String {
        // All children: keyword modifiers are unnamed nodes.
        self.all_ch(node)
            .into_iter()
            .filter_map(|c| {
                let t = self.txt(c).trim().to_string();
                if t.is_empty() {
                    None
                } else {
                    Some(t)
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    // ── block ─────────────────────────────────────────────────────────────────

    /// Renders a statement block `{ … }`. `body_lead_min` is the minimum
    /// blank lines to insert at the start of the body (`BLANK_LINES_BEFORE_METHOD_BODY`
    /// for method/constructor bodies, 0 otherwise); existing source runs after
    /// the `{`, between statements and before the `}` are preserved up to the
    /// `KEEP_BLANK_LINES_IN_CODE` / `KEEP_BLANK_LINES_BEFORE_RBRACE` caps.
    fn block(&self, node: Node<'s>, indent: usize, _c: usize, body_lead_min: u32) -> String {
        let stmts = self.named(node);
        if stmts.is_empty() {
            return Self::within_opt(
                '{',
                '}',
                self.style.space_within_braces,
                self.style.space_within_braces,
                "",
            );
        }

        let inner = indent + 1;
        let keep = self.style.keep_blank_lines_in_code;
        let mut out = String::from("{\n");

        for (i, s) in stmts.iter().enumerate() {
            let blanks = if i == 0 {
                // Leading gap after the opening brace.
                let existing = self.blank_lines_between(node.start_byte(), s.start_byte());
                self.spacing(existing, keep, body_lead_min)
            } else {
                // Gap between the previous statement/comment and this one.
                let prev_end = stmts[i - 1].end_byte();
                let cur_start = s.start_byte();
                let existing = self.blank_lines_between(prev_end, cur_start);
                self.spacing(existing, keep, 0)
            };
            self.push_blanks(&mut out, blanks);
            out.push_str(&self.ind(inner));
            let sc = self.col_after(0, &self.ind(inner));
            if s.is_extra() {
                out.push_str(self.txt(*s));
            } else {
                out.push_str(&self.stmt(*s, inner, sc));
            }
            out.push('\n');
        }

        // Closing gap before the right brace.
        let last = stmts[stmts.len() - 1];
        let existing = self.blank_lines_between(last.end_byte(), node.end_byte().saturating_sub(1));
        let blanks = self.spacing(existing, self.style.keep_blank_lines_before_rbrace, 0);
        self.push_blanks(&mut out, blanks);

        out.push_str(&self.ind(indent));
        out.push('}');
        out
    }

    /// Whether the configured “other” (statement block) brace style keeps the
    /// opening brace on the same line, i.e. a simple block can be kept inline.
    fn braces_style_inline(&self) -> bool {
        matches!(
            self.style.other_brace_style,
            BraceStyle::EndOfLine | BraceStyle::NextLineIfWrapped
        )
    }

    /// Renders `node` (a `block`) as a single line `{ stmt }` when it contains
    /// exactly one simple statement; returns `None` otherwise. Empty blocks are
    /// already rendered inline by [`Self::block`] and return `None` here.
    fn one_line_body(&self, node: Node<'s>) -> Option<String> {
        if node.kind() != "block" {
            return None;
        }
        let stmts = self.named(node);
        if stmts.len() != 1 {
            return None;
        }
        let s = stmts[0];
        if s.is_extra()
            || !matches!(
                s.kind(),
                "expression_statement"
                    | "local_variable_declaration"
                    | "return_statement"
                    | "throw_statement"
                    | "assert_statement"
                    | "break_statement"
                    | "continue_statement"
                    | "empty_statement"
            )
        {
            return None;
        }
        let txt = self.stmt(s, 0, 0);
        if txt.contains('\n') {
            return None;
        }
        Some(format!("{{ {} }}", txt))
    }

    /// Single-line rendering of an `if`/`else if`/`else` chain whose blocks are
    /// all simple; `None` if any body is not a simple block. The `if`→`(` gap
    /// follows `SPACE_BEFORE_IF_PARENTHESES`, the body gaps the corresponding
    /// `SPACE_BEFORE_*_LBRACE` toggles, and the `}`→`else` gap
    /// `SPACE_BEFORE_ELSE_KEYWORD`.
    fn if_one_line(&self, node: Node<'s>) -> Option<String> {
        let cond = self.fld(node, "condition")?;
        let cond_txt = self.flat_keyword_cond(cond, self.style.space_within_if_parentheses);
        if cond_txt.contains('\n') {
            return None;
        }
        let cons = self.fld(node, "consequence")?;
        let cons_txt = self.one_line_body(cons)?;
        // `cond_txt` already includes the parentheses (parenthesized_expression).
        let mut out = format!(
            "if{}{}{}{}",
            self.sp(self.style.space_before_if_parentheses),
            cond_txt,
            self.sp(self.style.space_before_if_lbrace),
            cons_txt
        );
        if let Some(alt) = self.fld(node, "alternative") {
            let else_gap = self.sp(self.style.space_before_else_keyword);
            if alt.kind() == "if_statement" {
                out.push_str(&format!("{}else {}", else_gap, self.if_one_line(alt)?));
            } else {
                out.push_str(&format!(
                    "{}else{}",
                    else_gap,
                    self.sp(self.style.space_before_else_lbrace)
                ));
                out.push_str(&self.one_line_body(alt)?);
            }
        }
        Some(out)
    }

    // ── statements ────────────────────────────────────────────────────────────

    fn stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        match node.kind() {
            "expression_statement" => {
                let e = node
                    .named_child(0)
                    .map(|n| self.expr(n, indent, c))
                    .unwrap_or_default();
                format!("{};", e)
            }
            "local_variable_declaration" => self.local_var(node, indent, c),
            "return_statement" => {
                if let Some(e) = node.named_child(0) {
                    format!("return {};", self.expr(e, indent, c + 7))
                } else {
                    "return;".to_string()
                }
            }
            "throw_statement" => {
                let e = node
                    .named_child(0)
                    .map(|n| self.expr(n, indent, c + 6))
                    .unwrap_or_default();
                format!("throw {};", e)
            }
            "if_statement" => self.if_stmt(node, indent, c),
            "for_statement" => self.for_stmt(node, indent, c),
            "enhanced_for_statement" => self.enhanced_for(node, indent, c),
            "while_statement" => self.while_stmt(node, indent, c),
            "do_statement" => self.do_while(node, indent, c),
            "try_statement" | "try_with_resources_statement" => self.try_stmt(node, indent, c),
            "synchronized_statement" => self.sync_stmt(node, indent, c),
            // In tree-sitter-java 0.23 the switch statement and the switch
            // expression are the same node kind (`switch_expression`); the
            // statement position always uses the multi-line layout.
            "switch_expression" => self.switch_stmt(node, indent, c),
            "assert_statement" => self.assert_stmt(node, indent, c),
            "break_statement" => {
                let label = node
                    .named_child(0)
                    .map(|n| format!(" {}", self.txt(n)))
                    .unwrap_or_default();
                format!("break{};", label)
            }
            "continue_statement" => {
                let label = node
                    .named_child(0)
                    .map(|n| format!(" {}", self.txt(n)))
                    .unwrap_or_default();
                format!("continue{};", label)
            }
            "labeled_statement" => {
                let label = self.fld(node, "label").map(|n| self.txt(n)).unwrap_or("");
                let body = self
                    .fld(node, "statement")
                    .or_else(|| node.named_child(1))
                    .map(|n| self.stmt(n, indent, c))
                    .unwrap_or_default();
                format!("{}:\n{}{}", label, self.ind(indent), body)
            }
            "block" => self.block(node, indent, c, 0),
            "empty_statement" => ";".to_string(),
            "line_comment" | "block_comment" => self.txt(node).to_string(),
            _ => self.txt(node).to_string(),
        }
    }

    fn local_var(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let mut out = String::new();

        if let Some(mods) = self.get_mods(node) {
            let ms = self.flat_mods(mods);
            if !ms.is_empty() {
                out.push_str(&ms);
                out.push(' ');
            }
        }

        let ty = self
            .fld(node, "type")
            .map(|n| self.flat_type(n))
            .unwrap_or_default();
        out.push_str(&ty);
        out.push(' ');

        let decls: Vec<Node<'s>> = self
            .named(node)
            .into_iter()
            .filter(|n| n.kind() == "variable_declarator")
            .collect();

        // Single declarator whose initialiser can be wrapped at the operator
        // (always under `KEEP_LINE_BREAKS` when the declaration spans rows).
        if decls.len() == 1
            && (self.style.assignment_wrap != WrapStyle::DoNotWrap || self.keep_wrapped(node))
        {
            if let Some(val) = self.fld(decls[0], "value") {
                let name = self
                    .fld(decls[0], "name")
                    .map(|n| self.txt(n))
                    .unwrap_or("");
                let prefix = format!("{}{}", out, name); // `out` ends with a space
                return format!(
                    "{};",
                    self.assign_expr(val, indent, c, &prefix, "=", self.keep_wrapped(node))
                );
            }
        }

        let decl_strs: Vec<String> = decls
            .iter()
            .map(|&d| {
                let name = self.fld(d, "name").map(|n| self.txt(n)).unwrap_or("");
                if let Some(val) = self.fld(d, "value") {
                    let sep = self.op_sep("=");
                    let val_col = self.col_after(c, &out) + name.len() + sep.len() + 1 + sep.len();
                    let val_str = self.expr(val, indent, val_col);
                    format!("{}{}={}{}", name, sep, sep, val_str)
                } else {
                    name.to_string()
                }
            })
            .collect();

        out.push_str(&decl_strs.join(self.comma_sep(self.style.space_after_comma)));
        out.push(';');
        out
    }

    fn if_stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        // Keep simple bodies on one line when enabled and the whole statement fits.
        if self.braces_style_inline() && self.style.keep_simple_blocks_in_one_line {
            if let Some(one) = self.if_one_line(node) {
                if self.fits(c, &one) {
                    return one;
                }
            }
        }

        // The `condition` child is a `parenthesized_expression`, i.e. it already
        // includes its own parentheses.
        let p_gap = self.sp(self.style.space_before_if_parentheses);
        let cond = self
            .fld(node, "condition")
            .map(|n| {
                self.keyword_cond(
                    n,
                    indent,
                    c + 3 + p_gap.len(),
                    self.style.space_within_if_parentheses,
                )
            })
            .unwrap_or_default();

        let cons = self
            .fld(node, "consequence")
            .map(|n| {
                self.stmt_as_block_or_inline(
                    n,
                    indent,
                    c,
                    self.style.if_brace_force,
                    self.style.space_before_if_lbrace,
                )
            })
            .unwrap_or_default();

        let mut out = format!("if{}{}{}", p_gap, cond, cons);

        if let Some(alt) = self.fld(node, "alternative") {
            let alt_str = if alt.kind() == "if_statement" {
                format!(
                    "{}else {}",
                    self.sp(self.style.space_before_else_keyword),
                    self.if_stmt(alt, indent, c)
                )
            } else {
                format!(
                    "{}else{}",
                    self.sp(self.style.space_before_else_keyword),
                    self.stmt_as_block_or_inline(
                        alt,
                        indent,
                        c,
                        self.style.if_brace_force,
                        self.style.space_before_else_lbrace,
                    )
                )
            };
            out.push_str(&alt_str);
        }

        out
    }

    /// Renders a statement body: a block is joined after the header with the
    /// `lbrace` gap (`SPACE_BEFORE_*_LBRACE` of the governing construct); a
    /// brace-less body keeps its own line(s), wrapped in `{ … }` only when
    /// `force` demands it (the forced brace uses the same `lbrace` gap).
    fn stmt_as_block_or_inline(
        &self,
        node: Node<'s>,
        indent: usize,
        c: usize,
        force: ForceStyle,
        lbrace: bool,
    ) -> String {
        if node.kind() == "block" {
            format!("{}{}", self.sp(lbrace), self.block(node, indent, c, 0))
        } else {
            let s = self.stmt(node, indent + 1, self.col_after(0, &self.ind(indent + 1)));
            match force {
                ForceStyle::DoNotForce => format!("\n{}{}", self.ind(indent + 1), s),
                // Exactly the bytes `block()` emits for a single-statement
                // block, so a forced body and a braced source converge.
                ForceStyle::ForceAlways => format!(
                    "{}{{\n{}{}\n{}}}",
                    self.sp(lbrace),
                    self.ind(indent + 1),
                    s,
                    self.ind(indent)
                ),
                ForceStyle::ForceIfMultiline => {
                    if s.contains('\n') {
                        format!(
                            "{}{{\n{}{}\n{}}}",
                            self.sp(lbrace),
                            self.ind(indent + 1),
                            s,
                            self.ind(indent)
                        )
                    } else {
                        format!("\n{}{}", self.ind(indent + 1), s)
                    }
                }
            }
        }
    }

    fn for_stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let body_node = self.fld(node, "body");

        // Re-create header from source bytes (handles all edge cases of for-init/cond/update)
        let header = if let Some(b) = body_node {
            let raw = std::str::from_utf8(&self.src[node.start_byte()..b.start_byte()])
                .unwrap_or("for (...)");
            let norm = normalise_ws(raw);
            let h = normalise_for_semis(
                &norm,
                self.style.space_before_semicolon,
                self.style.space_after_semicolon,
            )
            .unwrap_or_else(|| self.rebuild_for_header(node));
            let padded = pad_outer_parens(&h, self.style.space_within_for_parentheses);
            // The rebuilt header keeps the source's `for`↔`(` gap; pin it to
            // SPACE_BEFORE_FOR_PARENTHESES.
            self.pin_keyword_gap(&padded, "for", self.style.space_before_for_parentheses)
        } else {
            self.txt(node).to_string()
        };

        // Keep a simple body on one line when enabled and it fits.
        if self.braces_style_inline() && self.style.keep_simple_blocks_in_one_line {
            if let Some(one) = body_node.and_then(|b| self.one_line_body(b)) {
                let candidate = format!(
                    "{}{}{}",
                    header,
                    self.sp(self.style.space_before_for_lbrace),
                    one
                );
                if self.fits(c, &candidate) {
                    return candidate;
                }
            }
        }

        let body = body_node
            .map(|n| {
                self.stmt_as_block_or_inline(
                    n,
                    indent,
                    c,
                    self.style.for_brace_force,
                    self.style.space_before_for_lbrace,
                )
            })
            .unwrap_or_default();

        format!("{}{}", header, body)
    }

    /// Pin the gap between a clause keyword and its opening paren inside a
    /// textual header rebuilt from source bytes (`for (…)`): the source's gap
    /// is replaced by `SPACE_BEFORE_*_PARENTHESES`. Headers that do not start
    /// with `kw (` are returned unchanged (R4).
    fn pin_keyword_gap(&self, header: &str, kw: &str, on: bool) -> String {
        if let Some(open) = header.find('(') {
            if header[..open].trim_end() == kw {
                return format!("{}{}{}", kw, Self::sep(on), &header[open..]);
            }
        }
        header.to_string()
    }

    /// Rebuild a `for` header from the statement's init / condition / update
    /// children, spacing each `;` per `SPACE_BEFORE_SEMICOLON` /
    /// `SPACE_AFTER_SEMICOLON` but never inserting a space next to an empty
    /// slot (`for (;;)` stays compact) or before `)`. Used when the raw
    /// header has an awkward empty-slot shape; the child texts keep the
    /// source's content verbatim (R4), including multi-expression init/update
    /// lists and string literals containing `;`.
    fn rebuild_for_header(&self, node: Node<'s>) -> String {
        let before = Self::sep(self.style.space_before_semicolon);
        let after = Self::sep(self.style.space_after_semicolon);

        let init = self.for_part_text(node, "init");
        let cond = self
            .fld(node, "condition")
            .map(|n| normalise_ws(self.txt(n)))
            .unwrap_or_default();
        let upd = self.for_part_text(node, "update");

        let mut out = String::from("for (");
        if self.style.space_within_for_parentheses {
            out.push(' ');
        }
        if !init.is_empty() {
            out.push_str(&init);
            out.push_str(before);
        }
        out.push(';');
        if !cond.is_empty() {
            out.push_str(after);
            out.push_str(&cond);
            out.push_str(before);
        }
        out.push(';');
        if !upd.is_empty() {
            out.push_str(after);
            out.push_str(&upd);
        }
        if self.style.space_within_for_parentheses {
            out.push(' ');
        }
        out.push(')');
        out
    }

    /// The text of a `for` init / update slot: all field children (an init
    /// may be a `local_variable_declaration` whose text includes its trailing
    /// `;`, or a comma-separated expression list; an update is a
    /// comma-separated expression list) joined with `, `.
    fn for_part_text(&self, node: Node<'s>, field: &str) -> String {
        let mut cursor = node.walk();
        let parts: Vec<String> = node
            .children_by_field_name(field, &mut cursor)
            .map(|n| normalise_ws(self.txt(n)))
            .collect();
        let mut joined = parts.join(", ");
        if joined.ends_with(';') {
            joined.pop(); // a single local_variable_declaration includes its `;`
        }
        joined.trim().to_string()
    }

    fn enhanced_for(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let ty = self
            .fld(node, "type")
            .map(|n| self.flat_type(n))
            .unwrap_or_default();
        let name = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");
        let colon = self.foreach_colon_sep();
        let p_gap = self.sp(self.style.space_before_for_parentheses);
        let l_gap = self.sp(self.style.space_before_for_lbrace);

        // Keep a simple body on one line when enabled and it fits.
        if self.braces_style_inline() && self.style.keep_simple_blocks_in_one_line {
            if let (Some(v), Some(b)) = (self.fld(node, "value"), self.fld(node, "body")) {
                if let Some(one) = self.one_line_body(b) {
                    let vtxt = self.flat(v);
                    if !vtxt.contains('\n') {
                        let inner = format!("{} {}{}{}", ty, name, colon, vtxt);
                        let candidate = format!(
                            "for{}{}{}{}",
                            p_gap,
                            Self::within('(', ')', self.style.space_within_for_parentheses, &inner,),
                            l_gap,
                            one
                        );
                        if self.fits(c, &candidate) {
                            return candidate;
                        }
                    }
                }
            }
        }

        let val = self
            .fld(node, "value")
            .map(|n| self.expr(n, indent, c + ty.len() + name.len() + 7 + p_gap.len()))
            .unwrap_or_default();
        let body = self
            .fld(node, "body")
            .map(|n| {
                self.stmt_as_block_or_inline(
                    n,
                    indent,
                    c,
                    self.style.for_brace_force,
                    self.style.space_before_for_lbrace,
                )
            })
            .unwrap_or_default();
        let inner = format!("{} {}{}{}", ty, name, colon, val);
        format!(
            "for{}{}{}",
            p_gap,
            Self::within('(', ')', self.style.space_within_for_parentheses, &inner),
            body
        )
    }

    fn while_stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        // Keep a simple body on one line when enabled and it fits.
        if self.braces_style_inline() && self.style.keep_simple_blocks_in_one_line {
            if let (Some(cn), Some(b)) = (self.fld(node, "condition"), self.fld(node, "body")) {
                if let Some(one) = self.one_line_body(b) {
                    let ct = self.flat_keyword_cond(cn, self.style.space_within_while_parentheses);
                    if !ct.contains('\n') {
                        let candidate = format!(
                            "while{}{}{}{}",
                            self.sp(self.style.space_before_while_parentheses),
                            ct,
                            self.sp(self.style.space_before_while_lbrace),
                            one
                        );
                        if self.fits(c, &candidate) {
                            return candidate;
                        }
                    }
                }
            }
        }

        let p_gap = self.sp(self.style.space_before_while_parentheses);
        let cond = self
            .fld(node, "condition")
            .map(|n| {
                self.keyword_cond(
                    n,
                    indent,
                    c + 6 + p_gap.len(),
                    self.style.space_within_while_parentheses,
                )
            })
            .unwrap_or_default();
        let body = self
            .fld(node, "body")
            .map(|n| {
                self.stmt_as_block_or_inline(
                    n,
                    indent,
                    c,
                    self.style.while_brace_force,
                    self.style.space_before_while_lbrace,
                )
            })
            .unwrap_or_default();
        // `cond` is a parenthesized_expression and already contains its parens.
        format!("while{}{}{}", p_gap, cond, body)
    }

    fn do_while(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        // Keep a simple body on one line when enabled and it fits.
        if self.braces_style_inline() && self.style.keep_simple_blocks_in_one_line {
            if let (Some(b), Some(cn)) = (self.fld(node, "body"), self.fld(node, "condition")) {
                if let Some(one) = self.one_line_body(b) {
                    let ct = self.flat_keyword_cond(cn, self.style.space_within_while_parentheses);
                    if !ct.contains('\n') {
                        let candidate = format!(
                            "do{}{}{}while{}{};",
                            self.sp(self.style.space_before_do_lbrace),
                            one,
                            self.sp(self.style.space_before_while_keyword),
                            self.sp(self.style.space_before_while_parentheses),
                            ct
                        );
                        if self.fits(c, &candidate) {
                            return candidate;
                        }
                    }
                }
            }
        }

        let body = self
            .fld(node, "body")
            .map(|n| {
                if n.kind() == "block" {
                    format!(
                        "{}{}",
                        self.sp(self.style.space_before_do_lbrace),
                        self.block(n, indent, c, 0)
                    )
                } else {
                    let s = self.stmt(n, indent + 1, self.col_after(0, &self.ind(indent + 1)));
                    let braced = match self.style.dowhile_brace_force {
                        ForceStyle::DoNotForce => false,
                        ForceStyle::ForceAlways => true,
                        ForceStyle::ForceIfMultiline => s.contains('\n'),
                    };
                    if braced {
                        // Same bytes as `block()` for a single-statement block.
                        format!(
                            "{}{{\n{}{}\n{}}}",
                            self.sp(self.style.space_before_do_lbrace),
                            self.ind(indent + 1),
                            s,
                            self.ind(indent)
                        )
                    } else {
                        format!("\n{}{}\n{}", self.ind(indent + 1), s, self.ind(indent))
                    }
                }
            })
            .unwrap_or_default();
        // `cond` is a parenthesized_expression and already contains its parens.
        let w_gap = self.sp(self.style.space_before_while_keyword);
        let p_gap = self.sp(self.style.space_before_while_parentheses);
        let cond = self
            .fld(node, "condition")
            .map(|n| {
                self.keyword_cond(
                    n,
                    indent,
                    c + 8 + p_gap.len(),
                    self.style.space_within_while_parentheses,
                )
            })
            .unwrap_or_default();
        format!("do{}{}while{}{};", body, w_gap, p_gap, cond)
    }

    /// Single-line rendering of a `try` statement when the try body and every
    /// catch/finally body is a simple one-statement block; `None` otherwise
    /// (the caller falls through to the multi-line layout).
    fn try_one_line(&self, node: Node<'s>) -> Option<String> {
        let resources = if node.kind() == "try_with_resources_statement" {
            // The resource_specification node already includes its parens.
            self.fld(node, "resources")
                .map(|n| {
                    let t = normalise_ws(self.txt(n));
                    format!(
                        "{}{}",
                        self.sp(self.style.space_before_try_parentheses),
                        pad_outer_parens(&t, self.style.space_within_try_parentheses)
                    )
                })
                .unwrap_or_default()
        } else {
            String::new()
        };

        let body = self.fld(node, "body")?;
        let body_txt = self.one_line_body(body)?;
        let mut out = format!(
            "try{}{}{}",
            resources,
            self.sp(self.style.space_before_try_lbrace),
            body_txt
        );

        for ch in self.named(node) {
            match ch.kind() {
                "catch_clause" => {
                    let param = self
                        .named(ch)
                        .into_iter()
                        .find(|n| n.kind() == "catch_formal_parameter")
                        .map(|n| normalise_ws(self.txt(n)))
                        .unwrap_or_default();
                    let cbody = self.fld(ch, "body")?;
                    let cbody_txt = self.one_line_body(cbody)?;
                    let catch_head =
                        Self::within('(', ')', self.style.space_within_catch_parentheses, &param);
                    out.push_str(self.sp(self.style.space_before_catch_keyword));
                    out.push_str("catch");
                    out.push_str(self.sp(self.style.space_before_catch_parentheses));
                    out.push_str(&catch_head);
                    out.push_str(self.sp(self.style.space_before_catch_lbrace));
                    out.push_str(&cbody_txt);
                }
                "finally_clause" => {
                    // The block is a plain child of finally_clause (no field name).
                    let fbody = self.named(ch).into_iter().find(|n| n.kind() == "block")?;
                    let fbody_txt = self.one_line_body(fbody)?;
                    out.push_str(self.sp(self.style.space_before_finally_keyword));
                    out.push_str("finally");
                    out.push_str(self.sp(self.style.space_before_finally_lbrace));
                    out.push_str(&fbody_txt);
                }
                _ => {}
            }
        }

        Some(out)
    }

    fn try_stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        // Keep simple try/catch/finally bodies on one line when enabled and
        // the whole statement fits.
        if self.braces_style_inline() && self.style.keep_simple_blocks_in_one_line {
            if let Some(one) = self.try_one_line(node) {
                if self.fits(c, &one) {
                    return one;
                }
            }
        }

        let resources = if node.kind() == "try_with_resources_statement" {
            // The resource_specification node already includes its parens.
            self.fld(node, "resources")
                .map(|n| {
                    let t = self.txt(n).trim();
                    format!(
                        "{}{}",
                        self.sp(self.style.space_before_try_parentheses),
                        pad_outer_parens(t, self.style.space_within_try_parentheses)
                    )
                })
                .unwrap_or_default()
        } else {
            String::new()
        };

        let body = self
            .fld(node, "body")
            .map(|n| self.block(n, indent, c, 0))
            .unwrap_or_default();

        let mut out = format!(
            "try{}{}{}",
            resources,
            self.sp(self.style.space_before_try_lbrace),
            body
        );

        for ch in self.named(node) {
            match ch.kind() {
                "catch_clause" => {
                    let param = self
                        .named(ch)
                        .into_iter()
                        .find(|n| n.kind() == "catch_formal_parameter")
                        .map(|n| self.txt(n).to_string())
                        .unwrap_or_default();
                    let cbody = self
                        .fld(ch, "body")
                        .map(|n| self.block(n, indent, c, 0))
                        .unwrap_or_default();
                    let catch_head =
                        Self::within('(', ')', self.style.space_within_catch_parentheses, &param);
                    out.push_str(self.sp(self.style.space_before_catch_keyword));
                    out.push_str("catch");
                    out.push_str(self.sp(self.style.space_before_catch_parentheses));
                    out.push_str(&catch_head);
                    out.push_str(self.sp(self.style.space_before_catch_lbrace));
                    out.push_str(&cbody);
                }
                "finally_clause" => {
                    // The block is a plain child of finally_clause (no field name).
                    let fbody = self
                        .named(ch)
                        .into_iter()
                        .find(|n| n.kind() == "block")
                        .map(|n| self.block(n, indent, c, 0))
                        .unwrap_or_default();
                    out.push_str(self.sp(self.style.space_before_finally_keyword));
                    out.push_str("finally");
                    out.push_str(self.sp(self.style.space_before_finally_lbrace));
                    out.push_str(&fbody);
                }
                _ => {}
            }
        }

        out
    }

    fn sync_stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        // Keep a simple body on one line when enabled and it fits.
        if self.braces_style_inline() && self.style.keep_simple_blocks_in_one_line {
            let children = self.named(node);
            if let (Some(lock), Some(body)) = (
                children
                    .iter()
                    .find(|n| n.kind() == "parenthesized_expression"),
                children.iter().find(|n| n.kind() == "block"),
            ) {
                if let Some(one) = self.one_line_body(*body) {
                    let lt = self
                        .flat_keyword_cond(*lock, self.style.space_within_synchronized_parentheses);
                    if !lt.contains('\n') {
                        let candidate = format!(
                            "synchronized{}{}{}{}",
                            self.sp(self.style.space_before_synchronized_parentheses),
                            lt,
                            self.sp(self.style.space_before_synchronized_lbrace),
                            one
                        );
                        if self.fits(c, &candidate) {
                            return candidate;
                        }
                    }
                }
            }
        }

        // find the parenthesized lock expression and block body
        let children = self.named(node);
        let p_gap = self.sp(self.style.space_before_synchronized_parentheses);
        let lock = children
            .iter()
            .find(|n| n.kind() == "parenthesized_expression")
            .map(|n| {
                self.keyword_cond(
                    *n,
                    indent,
                    c + 11 + p_gap.len(),
                    self.style.space_within_synchronized_parentheses,
                )
            })
            .unwrap_or_default();
        let body = children
            .iter()
            .find(|n| n.kind() == "block")
            .map(|n| self.block(*n, indent, c, 0))
            .unwrap_or_default();
        // `lock` is a parenthesized_expression and already contains its parens.
        format!(
            "synchronized{}{}{}{}",
            p_gap,
            lock,
            self.sp(self.style.space_before_synchronized_lbrace),
            body
        )
    }

    fn assert_stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let children = self.named(node);
        match children.len() {
            0 => "assert;".to_string(),
            1 => format!("assert {};", self.expr(children[0], indent, c + 7)),
            _ => format!(
                "assert {} : {};",
                self.expr(children[0], indent, c + 7),
                self.expr(children[1], indent, c)
            ),
        }
    }

    /// Multi-line layout for a `switch_expression` node — tree-sitter-java
    /// 0.23 represents both the switch statement and the switch expression
    /// with this single kind. Renders `switch (cond) {` on the header line,
    /// `case`/`default` labels indented one level and their statements a
    /// further level, and the closing `}` at the statement indent, matching
    /// IntelliJ's default switch layout. Any unmodelled shape falls back to
    /// the verbatim source echo (R4).
    fn switch_stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let p_gap = self.sp(self.style.space_before_switch_parentheses);
        let l_gap = self.sp(self.style.space_before_switch_lbrace);
        let cond = self
            .fld(node, "condition")
            .map(|n| {
                self.keyword_cond(
                    n,
                    indent,
                    c + 6 + p_gap.len(),
                    self.style.space_within_switch_parentheses,
                )
            })
            .unwrap_or_default();
        let body = match self.fld(node, "body") {
            Some(b) if b.kind() == "switch_block" => b,
            _ => return self.txt(node).to_string(), // R4
        };

        if self.named(body).is_empty() {
            return format!(
                "switch{}{}{}{}",
                p_gap,
                cond,
                l_gap,
                Self::within_opt(
                    '{',
                    '}',
                    self.style.space_within_braces,
                    self.style.space_within_braces,
                    "",
                )
            );
        }

        let inner = indent + 1;
        let mut out = format!("switch{}{}{}{{\n", p_gap, cond, l_gap);

        for ch in self.named(body) {
            match ch.kind() {
                "switch_block_statement_group" => self.switch_group(ch, inner, &mut out),
                "switch_rule" => self.switch_rule(ch, inner, &mut out),
                // Comments and any other stray nodes keep their text,
                // indented to the label level (R4).
                _ => {
                    out.push_str(&self.ind(inner));
                    out.push_str(self.txt(ch));
                    out.push('\n');
                }
            }
        }

        out.push_str(&self.ind(indent));
        out.push('}');
        out
    }

    /// Lay out one colon-form case group (`switch_block_statement_group`):
    /// each `switch_label` on its own line followed by `:`, then the group's
    /// statements one indent level deeper.
    fn switch_group(&self, node: Node<'s>, indent: usize, out: &mut String) {
        for ch in self.named(node) {
            if ch.kind() == "switch_label" {
                out.push_str(&self.ind(indent));
                out.push_str(self.txt(ch));
                out.push_str(":\n");
            } else {
                let sc = self.col_after(0, &self.ind(indent + 1));
                out.push_str(&self.ind(indent + 1));
                out.push_str(&self.stmt(ch, indent + 1, sc));
                out.push('\n');
            }
        }
    }

    /// Lay out one arrow-form rule (`switch_rule`): `case X -> body` with an
    /// inline expression/throw body or a block body. If the body would wrap
    /// (render with a newline), the whole rule is echoed verbatim (R4) rather
    /// than producing a misaligned continuation.
    fn switch_rule(&self, node: Node<'s>, indent: usize, out: &mut String) {
        let mut label = String::new();
        let mut body: Option<Node<'s>> = None;
        for ch in self.named(node) {
            if ch.kind() == "switch_label" {
                label = self.txt(ch).to_string();
            } else {
                body = Some(ch);
            }
        }

        let head = format!("{}{} -> ", self.ind(indent), label);
        match body {
            Some(b) if b.kind() == "block" => {
                out.push_str(&head);
                out.push_str(&self.block(b, indent, 0, 0));
                out.push('\n');
            }
            Some(b) => {
                let s = self.stmt(b, indent, 0);
                if s.contains('\n') {
                    out.push_str(&self.ind(indent));
                    out.push_str(self.txt(node));
                    out.push('\n');
                } else {
                    out.push_str(&head);
                    out.push_str(&s);
                    out.push('\n');
                }
            }
            None => {
                out.push_str(&self.ind(indent));
                out.push_str(self.txt(node));
                out.push('\n');
            }
        }
    }

    /// Render a `switch_expression` used in expression position (assignment
    /// RHS, return value, argument): a single line when the whole switch fits
    /// the current column, otherwise the multi-line [`Self::switch_stmt`]
    /// layout.
    fn switch_expr(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        if let Some(one) = self.switch_one_line(node) {
            if self.fits(c, &one) {
                return one;
            }
        }
        self.switch_stmt(node, indent, c)
    }

    /// One-line rendering of a whole switch; `None` when any part (condition,
    /// label or body) would need a newline.
    fn switch_one_line(&self, node: Node<'s>) -> Option<String> {
        let cond = self
            .fld(node, "condition")
            .map(|n| self.flat_keyword_cond(n, self.style.space_within_switch_parentheses))?;
        if cond.contains('\n') {
            return None;
        }
        let body = self.fld(node, "body")?;
        if body.kind() != "switch_block" {
            return None;
        }

        let mut parts: Vec<String> = Vec::new();
        for ch in self.named(body) {
            match ch.kind() {
                "switch_block_statement_group" => {
                    let mut group = String::new();
                    for gc in self.named(ch) {
                        if gc.kind() == "switch_label" {
                            if !group.is_empty() {
                                group.push(' ');
                            }
                            group.push_str(self.txt(gc));
                            group.push(':');
                        } else {
                            let s = self.stmt(gc, 0, 0);
                            if s.contains('\n') {
                                return None;
                            }
                            group.push(' ');
                            group.push_str(&s);
                        }
                    }
                    parts.push(group);
                }
                "switch_rule" => {
                    let mut label = String::new();
                    let mut body_txt: Option<String> = None;
                    for rc in self.named(ch) {
                        if rc.kind() == "switch_label" {
                            label = self.txt(rc).to_string();
                        } else if rc.kind() == "block" {
                            let b = self.flat_block(rc);
                            if b.contains('\n') {
                                return None;
                            }
                            body_txt = Some(b);
                        } else {
                            let s = self.stmt(rc, 0, 0);
                            if s.contains('\n') {
                                return None;
                            }
                            body_txt = Some(s);
                        }
                    }
                    parts.push(format!("{} -> {}", label, body_txt?));
                }
                _ => return None,
            }
        }

        if parts.is_empty() {
            return None;
        }
        Some(format!(
            "switch{}{}{}{{ {} }}",
            self.sp(self.style.space_before_switch_parentheses),
            cond,
            self.sp(self.style.space_before_switch_lbrace),
            parts.join(" ")
        ))
    }

    // ── expressions ───────────────────────────────────────────────────────────

    fn expr(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        if node.is_extra() {
            return self.txt(node).to_string();
        }
        match node.kind() {
            "method_invocation" => self.method_inv(node, indent, c),
            "object_creation_expression" => self.new_expr(node, indent, c),
            "field_access" => self.field_access(node, indent, c),
            "array_access" => {
                let arr = self
                    .fld(node, "array")
                    .map(|n| self.expr(n, indent, c))
                    .unwrap_or_default();
                let idx = self
                    .fld(node, "index")
                    .map(|n| self.expr(n, indent, c))
                    .unwrap_or_default();
                format!(
                    "{}{}",
                    arr,
                    Self::within('[', ']', self.style.space_within_brackets, &idx)
                )
            }
            "assignment_expression" => self.assignment(node, indent, c),
            "binary_expression" => self.binary(node, indent, c),
            "unary_expression" => {
                let op = self
                    .fld(node, "operator")
                    .map(|n| self.txt(n))
                    .unwrap_or("");
                let sep = Self::sep(self.style.space_around_unary_operator);
                let operand = self
                    .fld(node, "operand")
                    .map(|n| self.expr(n, indent, c + op.len() + sep.len()))
                    .unwrap_or_default();
                format!("{}{}{}", op, sep, operand)
            }
            "update_expression" => self.update_expr(node, indent, c, false),
            "ternary_expression" => self.ternary(node, indent, c),
            "cast_expression" => {
                let ty = self
                    .fld(node, "type")
                    .map(|n| self.flat_type(n))
                    .unwrap_or_default();
                let pad_ty = Self::within('(', ')', self.style.space_within_cast_parentheses, &ty);
                let sep = Self::sep(self.style.space_after_type_cast);
                let val = self
                    .fld(node, "value")
                    .map(|n| self.expr(n, indent, c + pad_ty.len() + sep.len()))
                    .unwrap_or_default();
                format!("{}{}{}", pad_ty, sep, val)
            }
            "instanceof_expression" => {
                let left = self
                    .fld(node, "left")
                    .map(|n| self.expr(n, indent, c))
                    .unwrap_or_default();
                let right = self
                    .fld(node, "right")
                    .map(|n| self.flat_type(n))
                    .unwrap_or_default();
                format!("{} instanceof {}", left, right)
            }
            "lambda_expression" => self.lambda(node, indent, c),
            "method_reference" => self.method_ref(node),
            "parenthesized_expression" => {
                let inner = node
                    .named_child(0)
                    .map(|n| self.expr(n, indent, c + 1))
                    .unwrap_or_default();
                Self::within('(', ')', self.style.space_within_parentheses, &inner)
            }
            "array_creation_expression" => self.array_creation(node, indent, c),
            "array_initializer" => self.array_init(node, indent, c),
            "switch_expression" => self.switch_expr(node, indent, c),
            _ => self.txt(node).to_string(),
        }
    }

    // ── method invocation + chain ─────────────────────────────────────────────

    fn method_inv(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let flat = self.flat_inv(node);
        let keep = self.keep_wrapped(node) || self.args_keep_wrapped(node);

        if !keep && self.fits(c, &flat) {
            return flat;
        }

        // Detect chain
        if keep || self.style.method_call_chain_wrap != WrapStyle::DoNotWrap {
            let (base, links) = self.collect_chain(node);
            if links.len() >= 2 {
                return self.fmt_chain(&base, &links, indent, c);
            }
        }

        // Wrap argument list
        self.inv_wrapped(node, indent, c)
    }

    fn flat_inv(&self, node: Node<'s>) -> String {
        let obj = self
            .fld(node, "object")
            .map(|n| format!("{}{}", self.flat(n), "."))
            .unwrap_or_default();
        let ta = self
            .fld(node, "type_arguments")
            .map(|n| self.flat_type_args(n))
            .unwrap_or_default();
        let name = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");
        let args = self
            .fld(node, "arguments")
            .map(|n| self.flat_args(n))
            .unwrap_or_else(|| {
                Self::within_opt(
                    '(',
                    ')',
                    self.style.space_within_method_call_parentheses,
                    self.style.space_within_empty_method_call_parentheses,
                    "",
                )
            });
        format!(
            "{}{}{}{}{}",
            obj,
            ta,
            name,
            self.sp(self.style.space_before_method_call_parentheses),
            args
        )
    }

    fn flat_args(&self, node: Node<'s>) -> String {
        let inner = self
            .named(node)
            .iter()
            .map(|&a| self.flat(a))
            .collect::<Vec<_>>()
            .join(self.comma_sep(self.style.space_after_comma));
        Self::within_opt(
            '(',
            ')',
            self.style.space_within_method_call_parentheses,
            self.style.space_within_empty_method_call_parentheses,
            &inner,
        )
    }

    fn inv_wrapped(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let obj = self
            .fld(node, "object")
            .map(|n| format!("{}{}", self.expr(n, indent, c), "."))
            .unwrap_or_default();
        let ta = self
            .fld(node, "type_arguments")
            .map(|n| self.flat_type_args(n))
            .unwrap_or_default();
        let name = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");
        let gap = self.sp(self.style.space_before_method_call_parentheses);
        let prefix = format!("{}{}{}{}", obj, ta, name, gap);
        let args_col = self.col_after(c, &prefix);

        let args_str = self
            .fld(node, "arguments")
            .map(|n| self.args_wrapped(n, indent, args_col))
            .unwrap_or_else(|| {
                Self::within_opt(
                    '(',
                    ')',
                    self.style.space_within_method_call_parentheses,
                    self.style.space_within_empty_method_call_parentheses,
                    "",
                )
            });

        format!("{}{}", prefix, args_str)
    }

    fn args_wrapped(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let args = self.named(node);
        if args.is_empty() {
            return Self::within_opt(
                '(',
                ')',
                self.style.space_within_method_call_parentheses,
                self.style.space_within_empty_method_call_parentheses,
                "",
            );
        }

        let keep = self.keep_wrapped(node);
        let flat = self.flat_args(node);
        if !keep && self.fits(c, &flat) {
            return flat;
        }

        // Single argument that is a long chain → wrap the chain inline
        if !keep && args.len() == 1 && self.style.method_call_chain_wrap != WrapStyle::DoNotWrap {
            if args[0].kind() == "method_invocation" {
                let (base, links) = self.collect_chain(args[0]);
                if links.len() >= 2 {
                    let chain_str = self.fmt_chain(&base, &links, indent, c + 1);
                    return format!("({})", chain_str);
                }
            }
        }

        let wrap = self.style.call_parameters_wrap;
        if !keep && wrap == WrapStyle::DoNotWrap {
            return flat;
        }

        let inner = indent + 1;
        let ind = self.ind(inner);
        let arg_strs: Vec<String> = args
            .iter()
            .map(|&a| {
                let ac = self.col_after(0, &ind);
                format!("{}{}", ind, self.expr(a, inner, ac))
            })
            .collect();

        let (lp, rp) = (
            self.style.call_parameters_lparen_on_next_line,
            self.style.call_parameters_rparen_on_next_line,
        );
        let pad = self.style.space_within_method_call_parentheses;
        match (lp, rp) {
            (true, true) => Self::within(
                '(',
                ')',
                pad,
                &format!("\n{}\n{}", arg_strs.join(",\n"), self.ind(indent)),
            ),
            (true, false) => Self::within('(', ')', pad, &format!("\n{}", arg_strs.join(",\n"))),
            (false, true) => Self::within(
                '(',
                ')',
                pad,
                &format!("{}\n{}", arg_strs.join(",\n"), self.ind(indent)),
            ),
            (false, false) => Self::within('(', ')', pad, &format!("\n{}", arg_strs.join(",\n"))),
        }
    }

    // Collect a method-invocation chain bottom-up.
    // Returns (base_flat_string, links_from_first_to_last).
    fn collect_chain(&self, node: Node<'s>) -> (String, Vec<Link<'s>>) {
        let mut links = Vec::new();
        let mut cur = node;

        loop {
            let name = match self.fld(cur, "name") {
                Some(n) => n,
                None => {
                    links.reverse();
                    return (self.flat(cur), links);
                }
            };
            let args = match self.fld(cur, "arguments") {
                Some(n) => n,
                None => {
                    links.reverse();
                    return (self.flat(cur), links);
                }
            };
            let type_args = self.fld(cur, "type_arguments");
            links.push(Link {
                type_args,
                name,
                args,
            });

            match self.fld(cur, "object") {
                Some(obj) if obj.kind() == "method_invocation" => cur = obj,
                Some(obj) => {
                    links.reverse();
                    return (self.flat(obj), links);
                }
                None => {
                    links.reverse();
                    return (String::new(), links);
                }
            }
        }
    }

    fn fmt_chain(&self, base: &str, links: &[Link<'s>], indent: usize, _c: usize) -> String {
        let cont = self.cont(indent);
        let mut out = String::new();
        let gap = self.sp(self.style.space_before_method_call_parentheses);

        for (i, link) in links.iter().enumerate() {
            let ta = link
                .type_args
                .map(|n| self.flat_type_args(n))
                .unwrap_or_default();
            let nm = self.txt(link.name);
            let flat_a = self.flat_args(link.args);

            if i == 0 {
                if base.is_empty() {
                    out = format!("{}{}{}{}", ta, nm, gap, flat_a);
                } else {
                    out = format!("{}.{}{}{}{}", base, ta, nm, gap, flat_a);
                }
            } else {
                out.push('\n');
                out.push_str(&cont);
                out.push('.');
                out.push_str(&ta);
                out.push_str(nm);
                out.push_str(gap);
                out.push_str(&flat_a);
            }
        }

        out
    }

    // ── new / field_access / assignment / binary … ────────────────────────────

    fn new_expr(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let ta = self
            .fld(node, "type_arguments")
            .map(|n| self.flat_type_args(n))
            .unwrap_or_default();
        let ty = self
            .fld(node, "type")
            .map(|n| self.flat_type(n))
            .unwrap_or_default();
        let prefix = format!("new {}{}", ta, ty);
        // The gap between the type and the constructor-call parentheses follows
        // SPACE_BEFORE_METHOD_CALL_PARENTHESES (constructor calls share the
        // method-call toggle).
        let call_gap = self.sp(self.style.space_before_method_call_parentheses);
        // An anonymous class body is a plain `class_body` child of the
        // `object_creation_expression` (the grammar gives it no field name).
        let body_node = self
            .all_ch(node)
            .into_iter()
            .find(|c| c.kind() == "class_body");
        let has_body = body_node.is_some();

        if let Some(args_node) = self.fld(node, "arguments") {
            let flat_a = self.flat_args(args_node);
            let flat = format!("{}{}{}", prefix, call_gap, flat_a);

            if !has_body && !self.args_keep_wrapped(node) && self.fits(c, &flat) {
                return flat;
            }

            let args_str = self.args_wrapped(args_node, indent, c + prefix.len() + call_gap.len());
            let body_str = body_node
                .map(|n| {
                    format!(
                        "{}{}",
                        self.sp(self.style.space_before_class_lbrace),
                        self.class_body(n, indent, BodyKind::Anonymous)
                    )
                })
                .unwrap_or_default();

            format!("{}{}{}{}", prefix, call_gap, args_str, body_str)
        } else if let Some(b) = body_node {
            format!(
                "{}{}{}",
                prefix,
                self.sp(self.style.space_before_class_lbrace),
                self.class_body(b, indent, BodyKind::Anonymous)
            )
        } else {
            prefix
        }
    }

    fn field_access(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let obj = self
            .fld(node, "object")
            .map(|n| self.expr(n, indent, c))
            .unwrap_or_default();
        let field = self.fld(node, "field").map(|n| self.txt(n)).unwrap_or("");
        format!("{}.{}", obj, field)
    }

    fn assignment(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let left = self
            .fld(node, "left")
            .map(|n| self.flat(n))
            .unwrap_or_default();
        let rhs = self.fld(node, "right");
        let op = self
            .all_ch(node)
            .into_iter()
            .find(|n| {
                !n.is_named() && {
                    let t = self.txt(*n);
                    matches!(
                        t,
                        "=" | "+="
                            | "-="
                            | "*="
                            | "/="
                            | "&="
                            | "|="
                            | "^="
                            | "%="
                            | "<<="
                            | ">>="
                            | ">>>="
                    )
                }
            })
            .map(|n| self.txt(n))
            .unwrap_or("=");

        match rhs {
            Some(r)
                if self.style.assignment_wrap != WrapStyle::DoNotWrap
                    || self.keep_wrapped(node) =>
            {
                self.assign_expr(r, indent, c, &left, op, self.keep_wrapped(node))
            }
            _ => {
                let sep = self.op_sep(op);
                let right = rhs
                    .map(|n| {
                        let rc = c + left.len() + sep.len() + op.len() + sep.len();
                        self.expr(n, indent, rc)
                    })
                    .unwrap_or_default();
                format!("{}{}{}{}{}", left, sep, op, sep, right)
            }
        }
    }

    /// Renders `prefix op rhs` honouring `ASSIGNMENT_WRAP` and, when `keep` is
    /// set, `KEEP_LINE_BREAKS`.
    ///
    /// The RHS is first given the chance to wrap internally (e.g. as a method
    /// chain); if that keeps the whole line within the margin the operator is
    /// left in place. Otherwise the RHS is moved to the next line at the
    /// continuation indent. `c` is the column where `prefix` begins.
    fn assign_expr(
        &self,
        rhs: Node<'s>,
        indent: usize,
        c: usize,
        prefix: &str,
        op: &str,
        keep: bool,
    ) -> String {
        let sep = self.op_sep(op);
        let rhs_col = self.col_after(c, prefix) + sep.len() + op.len() + sep.len();
        let same = self.expr(rhs, indent, rhs_col);

        // The RHS wrapped internally; leave the operator on the header line.
        if same.contains('\n') {
            return format!("{}{}{}{}{}", prefix, sep, op, sep, same);
        }

        if keep {
            // `KEEP_LINE_BREAKS`: the initialiser's source spans rows, so the
            // RHS moves to the continuation line after the operator even
            // though the flat form fits.
            let cont = self.cont(indent);
            let cont_col = self.col_after(0, &cont);
            let nl = self.expr(rhs, indent, cont_col);
            return format!("{}{}{}\n{}{}", prefix, sep, op, cont, nl);
        }

        let flat = format!("{}{}{}{}{}", prefix, sep, op, sep, same);
        if self.style.assignment_wrap != WrapStyle::WrapAlways && self.fits(c, &flat) {
            return flat;
        }

        let cont = self.cont(indent);
        let cont_col = self.col_after(0, &cont);
        let nl = self.expr(rhs, indent, cont_col);
        format!("{}{}{}\n{}{}", prefix, sep, op, cont, nl)
    }

    fn binary(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let wrap = self.style.binary_operation_wrap;

        let left = self
            .fld(node, "left")
            .map(|n| self.flat(n))
            .unwrap_or_default();
        let op = self
            .fld(node, "operator")
            .map(|n| self.txt(n))
            .unwrap_or("+");
        let right = self
            .fld(node, "right")
            .map(|n| self.flat(n))
            .unwrap_or_default();
        let sep = self.op_sep(op);
        let flat = format!("{}{}{}{}{}", left, sep, op, sep, right);

        // DoNotWrap (and the default style) keep today's single-line output;
        // `KEEP_LINE_BREAKS` overrides that when the expression's source
        // spans rows.
        if !self.keep_wrapped(node)
            && (wrap == WrapStyle::DoNotWrap
                || (wrap != WrapStyle::WrapAlways && self.fits(c, &flat)))
        {
            return flat;
        }

        // Walk the left-associative spine (`a + b + c` parses as `(a + b) + c`)
        // so the expression is broken at its top-level operators, preserving
        // the exact token order — only whitespace changes, so R5 holds by
        // construction. `ops[i]` sits between `operands[i]` and `operands[i+1]`.
        let mut operands: Vec<Node<'s>> = Vec::new();
        let mut ops: Vec<String> = Vec::new();
        self.binary_spine(node, &mut operands, &mut ops);
        if operands.len() < 2 {
            return flat;
        }

        let cont = self.cont(indent);
        let mut out = self.binary_operand(operands[0], indent, c, wrap);
        for i in 1..operands.len() {
            out.push('\n');
            out.push_str(&cont);
            out.push_str(&ops[i - 1]);
            out.push_str(self.op_sep(&ops[i - 1]));
            out.push_str(&self.binary_operand(operands[i], indent, self.col_after(0, &cont), wrap));
        }
        out
    }

    /// Collect the operands and operators of a left-associative binary chain
    /// in source order: `operands.len() == ops.len() + 1`.
    fn binary_spine(&self, node: Node<'s>, operands: &mut Vec<Node<'s>>, ops: &mut Vec<String>) {
        let left = self.fld(node, "left");
        let right = self.fld(node, "right");
        let op = self
            .fld(node, "operator")
            .map(|n| self.txt(n))
            .unwrap_or("+")
            .to_string();
        match left {
            Some(l) if l.kind() == "binary_expression" => self.binary_spine(l, operands, ops),
            Some(l) => operands.push(l),
            None => {}
        }
        if let Some(r) = right {
            operands.push(r);
        }
        ops.push(op);
    }

    /// Render one operand of a broken binary spine. `ChopDownIfLong` recurses
    /// into an operand that is itself a binary expression so a long nested
    /// chain can wrap further; the other styles keep operands flat.
    fn binary_operand(&self, n: Node<'s>, indent: usize, c: usize, wrap: WrapStyle) -> String {
        if wrap == WrapStyle::ChopDownIfLong && n.kind() == "binary_expression" {
            self.binary(n, indent, c)
        } else {
            self.flat(n).to_string()
        }
    }

    fn ternary(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let q = self.quest_sep();
        let cl = self.colon_sep();
        let flat = format!(
            "{}{}{}{}{}",
            self.fld(node, "condition")
                .map(|n| self.flat(n))
                .unwrap_or_default(),
            q,
            self.fld(node, "consequence")
                .map(|n| self.flat(n))
                .unwrap_or_default(),
            cl,
            self.fld(node, "alternative")
                .map(|n| self.flat(n))
                .unwrap_or_default()
        );
        if !self.keep_wrapped(node) && self.fits(c, &flat) {
            return flat;
        }
        let cont = self.cont(indent);
        let cont_col = self.col_after(0, &cont);
        let cond = self
            .fld(node, "condition")
            .map(|n| self.expr(n, indent, c))
            .unwrap_or_default();
        let cons = self
            .fld(node, "consequence")
            .map(|n| self.expr(n, indent, cont_col))
            .unwrap_or_default();
        let alt = self
            .fld(node, "alternative")
            .map(|n| self.expr(n, indent, cont_col))
            .unwrap_or_default();
        format!(
            "{}\n{}{}{}{}\n{}{}{}{}",
            cond,
            cont,
            "?",
            Self::sep(self.style.space_after_quest),
            cons,
            cont,
            ":",
            Self::sep(self.style.space_after_colon),
            alt
        )
    }

    fn lambda(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let params = self
            .fld(node, "parameters")
            .map(|n| match n.kind() {
                "formal_parameters" => {
                    let flat = self.flat_formal_params(n);
                    flat
                }
                "inferred_parameters" => {
                    let ps: Vec<_> = self
                        .named(n)
                        .iter()
                        .map(|n| self.txt(*n).to_string())
                        .collect();
                    format!(
                        "({})",
                        ps.join(self.comma_sep(self.style.space_after_comma))
                    )
                }
                _ => self.txt(n).to_string(),
            })
            .unwrap_or_default();

        let body_node = self
            .fld(node, "body")
            .unwrap_or_else(|| node.named_child(1).unwrap());
        let sep = Self::sep(self.style.space_around_lambda_arrow);
        let arrow_col = sep.len() + 2 + sep.len();
        let body = if body_node.kind() == "block" {
            // check keep_simple
            let flat = self.flat_block(body_node);
            if self.style.keep_simple_lambdas_in_one_line
                && self.fits(c + params.len() + arrow_col, &flat)
            {
                flat
            } else {
                self.block(body_node, indent, c, 0)
            }
        } else {
            self.expr(body_node, indent, c + params.len() + arrow_col)
        };

        format!("{}{}->{}{}", params, sep, sep, body)
    }

    fn flat_formal_params(&self, node: Node<'s>) -> String {
        let params = self.named(node);
        let inner = params
            .iter()
            .map(|&p| self.flat_param(p))
            .collect::<Vec<_>>()
            .join(self.comma_sep(self.style.space_after_comma));
        format!("({})", inner)
    }

    fn array_creation(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let ty = self
            .fld(node, "type")
            .map(|n| self.flat_type(n))
            .unwrap_or_default();
        let dims: Vec<_> = self
            .named(node)
            .into_iter()
            .filter(|n| matches!(n.kind(), "dimensions_expr" | "dimensions"))
            .map(|n| {
                if n.kind() == "dimensions_expr" {
                    format!("[{}]", self.expr(n, indent, c))
                } else {
                    self.txt(n).to_string()
                }
            })
            .collect();
        let init = self
            .fld(node, "value")
            .map(|n| {
                format!(
                    "{}{}",
                    self.sp(self.style.space_before_array_initializer_lbrace),
                    self.array_init(n, indent, c)
                )
            })
            .unwrap_or_default();
        format!("new {}{}{}", ty, dims.join(""), init)
    }

    fn array_init(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let flat = self.flat_arr_init(node);
        if !self.keep_wrapped(node) && self.fits(c, &flat) {
            return flat;
        }
        let inner = indent + 1;
        let elem_strs: Vec<_> = self
            .named(node)
            .iter()
            .map(|&e| {
                format!(
                    "{}{}",
                    self.ind(inner),
                    self.expr(e, inner, self.col_after(0, &self.ind(inner)))
                )
            })
            .collect();
        let inner_str = format!("\n{}\n{}", elem_strs.join(",\n"), self.ind(indent));
        Self::within(
            '{',
            '}',
            self.style.space_within_array_initializer_braces,
            &inner_str,
        )
    }

    // ── type renderers ────────────────────────────────────────────────────────

    /// Render a type node canonically: no space inside angle brackets, no space
    /// before a comma, one space after a comma, and no stray spaces around
    /// nested brackets. Handles `type_identifier`, `scoped_type_identifier`,
    /// `generic_type`, arrays, primitives, wildcards and annotated types; any
    /// shape that does not match the expected structure is echoed verbatim (R4).
    fn flat_type(&self, node: Node<'s>) -> String {
        let t = self.txt(node);
        match node.kind() {
            // Simple names and primitives: single tokens, nothing to normalise.
            "type_identifier"
            | "void_type"
            | "integral_type"
            | "floating_point_type"
            | "boolean_type" => t.to_string(),
            // `A<B, C>` / `A.B<C, D>`: base type + canonical argument list.
            "generic_type" => {
                let ch = self.named(node);
                if ch.len() == 2 {
                    let base = self.flat_type(ch[0]);
                    let args = self.flat_type_args(ch[1]);
                    format!("{}{}", base, args)
                } else {
                    t.to_string()
                }
            }
            // `a.b.C`: scope dotted with name; the scope may itself be scoped
            // or generic, and the name may carry annotations (then fall back).
            "scoped_type_identifier" => {
                let ch = self.named(node);
                if ch.len() == 2 {
                    format!("{}.{}", self.flat_type(ch[0]), self.flat_type(ch[1]))
                } else {
                    t.to_string()
                }
            }
            // `T[]` / `List<A>[][]`: element + canonical dimension pairs.
            "array_type" => {
                if let (Some(e), Some(d)) =
                    (self.fld(node, "element"), self.fld(node, "dimensions"))
                {
                    format!("{}{}", self.flat_type(e), self.flat_dimensions(d))
                } else {
                    t.to_string()
                }
            }
            // `?` / `? extends T` / `? super T`.
            "wildcard" => match self.named(node).as_slice() {
                [] => "?".to_string(),
                [x] if x.kind() != "super" => format!("? extends {}", self.flat_type(*x)),
                [s, x] if s.kind() == "super" => format!("? super {}", self.flat_type(*x)),
                _ => t.to_string(),
            },
            // `@A T`: annotations echoed verbatim, joined with a single space.
            "annotated_type" => {
                let ch = self.named(node);
                if let Some((ty, anns)) = ch.split_last() {
                    let a = anns
                        .iter()
                        .map(|n| self.txt(*n).trim())
                        .collect::<Vec<_>>()
                        .join(" ");
                    if !a.is_empty() {
                        return format!("{} {}", a, self.flat_type(*ty));
                    }
                }
                t.to_string()
            }
            // Anything else (e.g. ERROR subtrees): echo verbatim (R4).
            _ => t.to_string(),
        }
    }

    /// Render a `type_arguments` node canonically: `<A, B>` with no space
    /// inside the angle brackets and a comma separated per
    /// `SPACE_AFTER_COMMA_IN_TYPE_ARGUMENTS` (default: one space after).
    fn flat_type_args(&self, node: Node<'s>) -> String {
        let inner: Vec<_> = self
            .named(node)
            .iter()
            .map(|n| self.flat_type(*n))
            .collect();
        format!(
            "<{}>",
            inner.join(self.comma_sep(self.style.space_after_comma_in_type_arguments))
        )
    }

    /// Rebuild a `dimensions` node (e.g. `[]`, `[][]`, `@A []`) canonically:
    /// bracket pairs with no inner space, annotations echoed verbatim.
    fn flat_dimensions(&self, node: Node<'s>) -> String {
        let mut out = String::new();
        for c in self.all_ch(node) {
            if c.is_named() {
                if !out.is_empty() && !out.ends_with(' ') {
                    out.push(' ');
                }
                out.push_str(self.txt(c).trim());
                out.push(' ');
            } else {
                out.push_str(self.txt(c));
            }
        }
        out
    }

    /// Render a declaration `type_parameters` node canonically: `<T, U>` with
    /// no space inside the angle brackets and one space after each comma.
    fn flat_type_params(&self, node: Node<'s>) -> String {
        let inner: Vec<_> = self
            .named(node)
            .iter()
            .map(|n| self.flat_type_param(*n))
            .collect();
        format!("<{}>", inner.join(", "))
    }

    /// Render one `type_parameter`, e.g. `T extends Number & Serializable`.
    fn flat_type_param(&self, node: Node<'s>) -> String {
        let ch = self.named(node);
        match ch.as_slice() {
            [name] => self.flat_type(*name),
            [name, bound] if bound.kind() == "type_bound" => format!(
                "{} extends {}",
                self.flat_type(*name),
                self.flat_type_bound(*bound)
            ),
            _ => self.txt(node).trim().to_string(),
        }
    }

    /// Render a `type_bound` (the `extends A & B` part of a type parameter).
    fn flat_type_bound(&self, node: Node<'s>) -> String {
        let inner: Vec<_> = self
            .named(node)
            .iter()
            .map(|n| self.flat_type(*n))
            .collect();
        inner.join(" & ")
    }

    /// Render the type list of an `extends_interfaces` / `super_interfaces`
    /// clause canonically: `A, B` with one space after each comma. Falls back
    /// to the clause text without its keyword when no `type_list` is found (R4).
    fn flat_type_list(&self, node: Node<'s>) -> String {
        if let Some(tl) = self
            .named(node)
            .into_iter()
            .find(|n| n.kind() == "type_list")
        {
            let inner: Vec<_> = self.named(tl).iter().map(|n| self.flat_type(*n)).collect();
            if !inner.is_empty() {
                return inner.join(self.comma_sep(self.style.space_after_comma));
            }
        }
        let t = self.txt(node).trim();
        t.strip_prefix("implements")
            .or_else(|| t.strip_prefix("extends"))
            .unwrap_or(t)
            .trim()
            .to_string()
    }

    // ── flat (one-line) versions ──────────────────────────────────────────────

    fn flat(&self, node: Node<'s>) -> String {
        match node.kind() {
            "method_invocation" => self.flat_inv(node),
            "object_creation_expression" => self.flat_new(node),
            "field_access" => self.flat_field_access(node),
            "array_access" => {
                let arr = self
                    .fld(node, "array")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                let idx = self
                    .fld(node, "index")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                format!(
                    "{}{}",
                    arr,
                    Self::within('[', ']', self.style.space_within_brackets, &idx)
                )
            }
            "assignment_expression" => {
                let left = self
                    .fld(node, "left")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                let right = self
                    .fld(node, "right")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                let op = self
                    .all_ch(node)
                    .into_iter()
                    .find(|n| {
                        !n.is_named()
                            && matches!(
                                self.txt(*n),
                                "=" | "+="
                                    | "-="
                                    | "*="
                                    | "/="
                                    | "&="
                                    | "|="
                                    | "^="
                                    | "%="
                                    | "<<="
                                    | ">>="
                                    | ">>>="
                            )
                    })
                    .map(|n| self.txt(n))
                    .unwrap_or("=");
                let sep = self.op_sep(op);
                format!("{}{}{}{}{}", left, sep, op, sep, right)
            }
            "binary_expression" => {
                let left = self
                    .fld(node, "left")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                let op = self
                    .fld(node, "operator")
                    .map(|n| self.txt(n))
                    .unwrap_or("+");
                let right = self
                    .fld(node, "right")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                let sep = self.op_sep(op);
                format!("{}{}{}{}{}", left, sep, op, sep, right)
            }
            "unary_expression" => {
                let op = self
                    .fld(node, "operator")
                    .map(|n| self.txt(n))
                    .unwrap_or("");
                let sep = Self::sep(self.style.space_around_unary_operator);
                let operand = self
                    .fld(node, "operand")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                format!("{}{}{}", op, sep, operand)
            }
            "update_expression" => self.update_expr(node, 0, 0, true),
            "ternary_expression" => {
                let c = self
                    .fld(node, "condition")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                let t = self
                    .fld(node, "consequence")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                let f = self
                    .fld(node, "alternative")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                format!("{}{}{}{}{}", c, self.quest_sep(), t, self.colon_sep(), f)
            }
            "cast_expression" => {
                let ty = self
                    .fld(node, "type")
                    .map(|n| self.flat_type(n))
                    .unwrap_or_default();
                let pad_ty = Self::within('(', ')', self.style.space_within_cast_parentheses, &ty);
                let sep = Self::sep(self.style.space_after_type_cast);
                let val = self
                    .fld(node, "value")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                format!("{}{}{}", pad_ty, sep, val)
            }
            "instanceof_expression" => {
                let left = self
                    .fld(node, "left")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                let right = self
                    .fld(node, "right")
                    .map(|n| self.flat_type(n))
                    .unwrap_or_default();
                format!("{} instanceof {}", left, right)
            }
            "parenthesized_expression" => {
                let inner = node
                    .named_child(0)
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                Self::within('(', ')', self.style.space_within_parentheses, &inner)
            }
            "lambda_expression" => self.flat_lambda(node),
            // Flat contexts cannot contain newlines: use the one-line switch
            // rendering when possible, else echo the source verbatim (R4).
            "switch_expression" => self
                .switch_one_line(node)
                .unwrap_or_else(|| self.txt(node).to_string()),
            "array_initializer" | "element_value_array_initializer" => self.flat_arr_init(node),
            "array_creation_expression" => self.flat_arr_creation(node),
            "annotation" => self.flat_annotation(node),
            "marker_annotation" => {
                let name = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");
                format!("@{}", name)
            }
            "element_value_pair" => {
                let k = self.fld(node, "key").map(|n| self.txt(n)).unwrap_or("");
                let v = self
                    .fld(node, "value")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                format!("{} = {}", k, v)
            }
            "method_reference" => self.method_ref(node),
            _ => self.txt(node).to_string(),
        }
    }

    fn flat_annotation(&self, node: Node<'s>) -> String {
        let name = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");
        match self.fld(node, "arguments") {
            Some(n) => format!(
                "@{}{}{}",
                name,
                self.sp(self.style.space_before_anotation_parameter_list),
                self.ann_parens(&self.flat_ann_args(n)),
            ),
            None => format!("@{}", name),
        }
    }

    fn flat_new(&self, node: Node<'s>) -> String {
        let ta = self
            .fld(node, "type_arguments")
            .map(|n| self.flat_type_args(n))
            .unwrap_or_default();
        let ty = self
            .fld(node, "type")
            .map(|n| self.flat_type(n))
            .unwrap_or_default();
        let args = self
            .fld(node, "arguments")
            .map(|n| self.flat_args(n))
            .unwrap_or_else(|| {
                Self::within_opt(
                    '(',
                    ')',
                    self.style.space_within_method_call_parentheses,
                    self.style.space_within_empty_method_call_parentheses,
                    "",
                )
            });
        // The anonymous body cannot render flat; this placeholder keeps the
        // margin estimate honest. The joins follow the same toggles as
        // [`Self::new_expr`]: a gap before the constructor parens and one
        // before the anonymous body's `{` when a body is present.
        let body = self
            .fld(node, "class_body")
            .map(|_| format!("{}{{ ... }}", self.sp(self.style.space_before_class_lbrace)))
            .unwrap_or_default();
        format!(
            "new {}{}{}{}{}",
            ta,
            ty,
            self.sp(self.style.space_before_method_call_parentheses),
            args,
            body
        )
    }

    fn flat_field_access(&self, node: Node<'s>) -> String {
        let obj = self
            .fld(node, "object")
            .map(|n| self.flat(n))
            .unwrap_or_default();
        let field = self.fld(node, "field").map(|n| self.txt(n)).unwrap_or("");
        format!("{}.{}", obj, field)
    }

    fn flat_lambda(&self, node: Node<'s>) -> String {
        let params = self
            .fld(node, "parameters")
            .map(|n| match n.kind() {
                "formal_parameters" => self.flat_formal_params(n),
                "inferred_parameters" => {
                    let ps: Vec<_> = self
                        .named(n)
                        .iter()
                        .map(|n| self.txt(*n).to_string())
                        .collect();
                    format!(
                        "({})",
                        ps.join(self.comma_sep(self.style.space_after_comma))
                    )
                }
                _ => self.txt(n).to_string(),
            })
            .unwrap_or_default();
        let body = self
            .fld(node, "body")
            .or_else(|| node.named_child(1))
            .map(|n| {
                if n.kind() == "block" {
                    self.flat_block(n)
                } else {
                    self.flat(n)
                }
            })
            .unwrap_or_default();
        let sep = Self::sep(self.style.space_around_lambda_arrow);
        format!("{}{}->{}{}", params, sep, sep, body)
    }

    fn flat_block(&self, node: Node<'s>) -> String {
        let stmts = self.named(node);
        if stmts.is_empty() {
            return Self::within_opt(
                '{',
                '}',
                self.style.space_within_braces,
                self.style.space_within_braces,
                "",
            );
        }
        let inner = stmts
            .iter()
            .map(|&s| self.flat(s))
            .collect::<Vec<_>>()
            .join("; ");
        format!("{{ {} }}", inner)
    }

    fn flat_arr_init(&self, node: Node<'s>) -> String {
        let elems = self.named(node);
        if elems.is_empty() {
            return Self::within_opt(
                '{',
                '}',
                self.style.space_within_array_initializer_braces,
                self.style.space_within_empty_array_initializer_braces,
                "",
            );
        }
        let inner = elems
            .iter()
            .map(|&e| self.flat(e))
            .collect::<Vec<_>>()
            .join(self.comma_sep(self.style.space_after_comma));
        Self::within_opt(
            '{',
            '}',
            self.style.space_within_array_initializer_braces,
            self.style.space_within_empty_array_initializer_braces,
            &inner,
        )
    }

    fn flat_arr_creation(&self, node: Node<'s>) -> String {
        let ty = self
            .fld(node, "type")
            .map(|n| self.flat_type(n))
            .unwrap_or_default();
        let dims: Vec<_> = self
            .named(node)
            .into_iter()
            .filter(|n| matches!(n.kind(), "dimensions_expr" | "dimensions"))
            .map(|n| {
                if n.kind() == "dimensions_expr" {
                    format!("[{}]", self.flat(n))
                } else {
                    self.txt(n).to_string()
                }
            })
            .collect();
        let init = self
            .fld(node, "value")
            .map(|n| {
                format!(
                    "{}{}",
                    self.sp(self.style.space_before_array_initializer_lbrace),
                    self.flat_arr_init(n)
                )
            })
            .unwrap_or_default();
        format!("new {}{}{}", ty, dims.join(""), init)
    }
}

// ── standalone utilities ──────────────────────────────────────────────────────

/// Pad the outermost paren pair of a textual header (`for (…)`, `try (…)`)
/// with an idempotent insertion: one space just inside each paren, added only
/// when the neighbour is not already a space, so a padded header reformats to
/// itself.
fn pad_outer_parens(s: &str, pad: bool) -> String {
    if !pad {
        return s.to_string();
    }
    match (s.find('('), s.rfind(')')) {
        (Some(o), Some(c)) if o < c => {
            let inner = &s[o + 1..c];
            let l = if inner.starts_with(' ') { "" } else { " " };
            let r = if inner.ends_with(' ') { "" } else { " " };
            format!("{}{}{}{}{}", &s[..o + 1], l, inner, r, &s[c..])
        }
        _ => s.to_string(),
    }
}

/// Collapse runs of whitespace (including newlines) to a single space.
fn normalise_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normalise the whitespace around each `;` of a normalised `for` header per
/// `SPACE_BEFORE_SEMICOLON` / `SPACE_AFTER_SEMICOLON`: a space before the `;`
/// when `before` is on, a space after when `after` is on. Returns `None` when
/// the header hits an awkward empty-slot edge (an empty init / condition /
/// update slot, e.g. `for (;;)`) — the caller then rebuilds the header from
/// the statement's init/condition/update children instead.
fn normalise_for_semis(header: &str, before: bool, after: bool) -> Option<String> {
    let mut parts = header.split(';');
    let a = parts.next()?.trim();
    let b = parts.next()?.trim();
    let c = parts.next()?.trim();
    if parts.next().is_some() {
        return None; // more than two `;` — unexpected shape
    }
    // Awkward empty-slot edges: `(;` (empty init), `;;` (empty condition) or
    // `;)` (empty update).
    if a.ends_with('(') || b.is_empty() || c.starts_with(')') {
        return None;
    }
    let bf = if before { " " } else { "" };
    let af = if after { " " } else { "" };
    Some(format!("{}{};{}{}{};{}{}", a, bf, af, b, bf, af, c))
}

// ─────────────────────────────────────────────────────────────────────────────
// Output finalisation: line separator + WRAP_LONG_LINES
// ─────────────────────────────────────────────────────────────────────────────

/// Final output pass for the configured [`LineSeparator`](crate::config::LineSeparator).
///
/// The engine emits LF internally, so a CRLF / CR document must be converted
/// only at the very end — and only the *engine's* line ends, not newlines that
/// arrived verbatim inside echoed text (block comments) from a CRLF source,
/// which would otherwise double the `\r`. Collapse those first, trim to
/// exactly one trailing line end, then substitute `\n` → `sep` when the
/// resolved separator is not LF. LF output takes the historical code path
/// (`trim_end_matches('\n')` + one `\n`), so default output stays
/// byte-identical and re-formatting a CRLF document yields the same
/// separators (idempotent).
fn finalise_line_endings(out: &str, sep: &'static str) -> String {
    let collapsed = out.replace("\r\n", "\n");
    let trimmed = collapsed.trim_end_matches('\n');
    if sep == "\n" {
        format!("{}\n", trimmed)
    } else {
        format!("{}{}", trimmed.replace('\n', sep), sep)
    }
}

/// Logical column width of `s` on the formatter's tab-stop model: a tab
/// advances to the next multiple of `tab_size`, every other character by one.
fn columns(s: &str, tab_size: usize) -> usize {
    let mut col = 0usize;
    for ch in s.chars() {
        match ch {
            '\t' => col += tab_size - (col % tab_size),
            _ => col += 1,
        }
    }
    col
}

/// Build an indentation string of `width` columns exactly like
/// [`Fmt::indent_str`]: tabs per `TAB_SIZE` when `USE_TAB_CHARACTER` is set,
/// plain spaces otherwise.
fn indent_columns(width: usize, style: &JavaStyle) -> String {
    if !style.use_tab_character {
        return " ".repeat(width);
    }
    let tab = style.tab_size as usize;
    format!("{}{}", "\t".repeat(width / tab), " ".repeat(width % tab))
}

/// Width in columns of the leading whitespace run of `line`.
fn leading_indent_width(line: &str, tab_size: usize) -> usize {
    let end = line
        .char_indices()
        .find(|(_, c)| *c != ' ' && *c != '\t')
        .map(|(i, _)| i)
        .unwrap_or(line.len());
    columns(&line[..end], tab_size)
}

/// Lexical region of a line during the hard-wrap scan. Only block comments
/// and text blocks can span lines in Java, so only those two states carry
/// across lines; strings, chars and `//` comments are line-local.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanState {
    Code,
    Str,
    Char,
    LineComment,
    BlockComment,
    TextBlock,
}

/// Scan `line` from `state` (the state carried over from the previous line,
/// relevant only for block comments / text blocks), advancing the lexical
/// state to the end of the line and collecting the byte index of every space
/// that is a safe hard-wrap candidate: it sits in plain code (outside
/// strings, chars, comments and text blocks), at or before the right margin,
/// with code both before and after. Candidates are only offered when the
/// scan starts in [`ScanState::Code`] — a line that starts inside a comment
/// or text block is never wrapped.
fn scan_line(line: &str, state: ScanState, margin: usize, tab: usize) -> (Vec<usize>, ScanState) {
    let bytes = line.as_bytes();
    let mut state = state;
    let mut i = 0usize;
    let mut col = 0usize;
    let mut candidates: Vec<usize> = Vec::new();
    let mut in_code_text = false;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            ScanState::Code => match b {
                b' ' => {
                    if col <= margin && in_code_text {
                        candidates.push(i);
                    }
                    col += 1;
                    i += 1;
                }
                b'\t' => {
                    col += tab - (col % tab);
                    i += 1;
                }
                b'"' if bytes[i..].starts_with(b"\"\"\"") => {
                    state = ScanState::TextBlock;
                    i += 3;
                }
                b'"' => {
                    state = ScanState::Str;
                    i += 1;
                }
                b'\'' => {
                    state = ScanState::Char;
                    i += 1;
                }
                b'/' if bytes[i..].starts_with(b"//") => {
                    state = ScanState::LineComment;
                    i += 2;
                }
                b'/' if bytes[i..].starts_with(b"/*") => {
                    state = ScanState::BlockComment;
                    i += 2;
                }
                _ => {
                    in_code_text = true;
                    col += 1;
                    i += utf8_len(b);
                }
            },
            ScanState::Str | ScanState::Char => {
                match b {
                    b'\\' => {
                        // Skip the escaped character (escape targets are ASCII).
                        i = (i + 2).min(bytes.len());
                    }
                    b'"' if state == ScanState::Str => {
                        state = ScanState::Code;
                        i += 1;
                    }
                    b'\'' if state == ScanState::Char => {
                        state = ScanState::Code;
                        i += 1;
                    }
                    _ => {
                        col += 1;
                        i += utf8_len(b);
                    }
                }
            }
            ScanState::LineComment => {
                // A line comment runs to the end of its line.
                break;
            }
            ScanState::BlockComment => {
                if b == b'*' && bytes.get(i + 1) == Some(&b'/') {
                    state = ScanState::Code;
                    i += 2;
                } else {
                    col += 1;
                    i += utf8_len(b);
                }
            }
            ScanState::TextBlock => {
                if b == b'\\' {
                    i = (i + 2).min(bytes.len());
                } else if b == b'"' && bytes[i..].starts_with(b"\"\"\"") {
                    state = ScanState::Code;
                    i += 3;
                } else {
                    i += utf8_len(b);
                }
            }
        }
        if col > margin {
            // Columns only grow; no later space can still be at or before the
            // margin.
            break;
        }
    }

    // Keep only candidates with real code on both sides (a space inside the
    // leading indent or before end-of-line whitespace is not a wrap point).
    candidates.retain(|&i| !line[..i].trim().is_empty() && !line[i + 1..].trim().is_empty());
    (candidates, state)
}

/// Byte length of the UTF-8 character whose first byte is `b`.
fn utf8_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else {
        4
    }
}

/// Hard-wrap a single line (already including its leading indentation) for
/// `WRAP_LONG_LINES`: when the line's logical width exceeds the right margin,
/// break it at the rightmost safe space at or before the margin and continue
/// at the line's own leading indent plus `continuation_indent_size`, repeating
/// on each continuation until it fits or has no safe boundary (an over-long
/// string literal or single token is left intact).
fn hard_wrap_line(line: &str, style: &JavaStyle) -> String {
    let tab = style.tab_size as usize;
    let margin = style.right_margin as usize;
    if columns(line, tab) <= margin {
        return line.to_string();
    }
    let (candidates, _) = scan_line(line, ScanState::Code, margin, tab);
    match candidates.last() {
        None => line.to_string(),
        Some(&i) => {
            let head = line[..i].trim_end();
            let lead = leading_indent_width(line, tab);
            let cont = indent_columns(lead + style.continuation_indent_size as usize, style);
            let rest = line[i + 1..].trim_start_matches([' ', '\t']);
            let tail = hard_wrap_line(&format!("{}{}", cont, rest), style);
            format!("{}\n{}", head, tail)
        }
    }
}

/// The `WRAP_LONG_LINES` post-`program` line pass. Walks the LF-normal
/// output line by line, tracking block-comment / text-block state across
/// lines (a line that starts inside one is never wrapped — `WRAP_COMMENTS`
/// governs comments and string content must stay verbatim), and hard-wraps
/// every other line whose width exceeds the right margin. The break points
/// are a pure function of the flat text, so re-formatting reproduces them.
fn wrap_long_lines(text: &str, style: &JavaStyle) -> String {
    let tab = style.tab_size as usize;
    let margin = style.right_margin as usize;
    let text = text.replace("\r\n", "\n");
    let mut state = ScanState::Code;
    let mut out: Vec<String> = Vec::new();
    for line in text.split('\n') {
        let starts_in_span = matches!(state, ScanState::BlockComment | ScanState::TextBlock);
        let (_, end_state) = scan_line(line, state, margin, tab);
        state = end_state;
        if starts_in_span {
            out.push(line.to_string());
        } else {
            out.push(hard_wrap_line(line, style));
        }
    }
    out.join("\n")
}
