//! Java source-code formatter driven by IntelliJ codestyle settings.
//!
//! Uses tree-sitter-java to parse source into a CST, then pretty-prints it
//! following the rules encoded in [`crate::config::JavaStyle`].

use std::collections::{HashMap, HashSet};

use tree_sitter::{Language, Node, Parser};

use crate::config::{BraceStyle, JavaStyle, WrapStyle};

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
    let out = fmt.program(tree.root_node());

    // Normalise to exactly one trailing newline
    let trimmed = out.trim_end_matches('\n');
    (format!("{}\n", trimmed), diagnostics)
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

        let mut out = String::new();

        for c in &header_comments {
            out.push_str(self.txt(*c));
            out.push('\n');
        }

        if let Some(p) = pkg {
            out.push_str(&self.package_decl(p));
        }

        if !imports.is_empty() {
            if !out.is_empty() {
                out.push('\n');
            }
            // Names of top-level types declared in this file; on-demand imports
            // are shadowed by them, so merging must not mask a local type.
            let local_types: Vec<String> = top_types
                .iter()
                .filter_map(|n| self.fld(*n, "name").map(|nm| self.txt(nm).to_string()))
                .collect();
            out.push_str(&self.imports(imports, &local_types));
        }

        for ty in top_types {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&self.type_decl(ty, 0));
            out.push('\n');
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
            .map(|n| self.class_body(n, indent))
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
            .map(|n| self.class_body(n, indent))
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
        // Collect enum constants and member declarations
        let inner = indent + 1;
        let mut out = String::from("{\n");
        let mut in_constants = true;
        let mut first = true;

        for child in self.named(body) {
            match child.kind() {
                "enum_constant" => {
                    if !first {
                        out.push_str(",\n");
                    }
                    out.push_str(&self.ind(inner));
                    out.push_str(self.txt(child));
                    first = false;
                }
                "enum_body_declarations" => {
                    in_constants = false;
                    if !first {
                        out.push_str(";\n");
                    }
                    for member in self.named(child) {
                        out.push('\n');
                        out.push_str(&self.ind(inner));
                        out.push_str(&self.class_member(member, inner));
                        out.push('\n');
                    }
                }
                _ => {}
            }
        }

        if in_constants && !first {
            out.push('\n');
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
            .map(|n| self.class_body(n, indent))
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
        let flat = format!("({})", parts.join(", "));

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

    /// Attach `{ body }` to header following the brace style.
    fn with_brace(&self, header: String, body: String, indent: usize, style: BraceStyle) -> String {
        match style {
            BraceStyle::NextLine | BraceStyle::NextLineShifted | BraceStyle::NextLineShifted2 => {
                format!("{}\n{}{}", header, self.ind(indent), body)
            }
            _ => format!("{} {}", header, body),
        }
    }

    // ── class body ────────────────────────────────────────────────────────────

    fn class_body(&self, node: Node<'s>, indent: usize) -> String {
        let members = self.named(node);
        if members.is_empty() {
            return "{}".to_string();
        }

        let inner = indent + 1;
        let mut out = String::from("{\n");
        let mut first = true;

        for m in members {
            let is_comment = m.is_extra();

            if !first && !is_comment {
                out.push('\n'); // blank line between members
            }

            out.push_str(&self.ind(inner));
            out.push_str(&self.class_member(m, inner));
            out.push('\n');

            first = false;
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
                    .map(|n| self.block(n, indent, c))
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

        // parameters
        if let Some(params) = self.fld(node, "parameters") {
            let pcol = c + self.col_after(0, &out);
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
            out.push_str(&excs.join(", "));
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

        if let Some(params) = self.fld(node, "parameters") {
            let pcol = c + self.col_after(0, &out);
            out.push_str(&self.formal_params(params, indent, pcol, false));
        }

        if let Some(throws) = self.get_throws(node) {
            out.push_str(" throws ");
            let excs: Vec<_> = self
                .named(throws)
                .iter()
                .map(|n| self.flat_type(*n))
                .collect();
            out.push_str(&excs.join(", "));
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
                let width = self.col_after(c, out);
                if width + 1 + one.len() <= self.style.right_margin as usize {
                    out.push(' ');
                    out.push_str(&one);
                    return;
                }
            }
        }
        let body_str = self.block(body, indent, c);
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

        // Single declarator whose initialiser can be wrapped at the operator.
        if decls.len() == 1
            && !out.contains('\n')
            && self.style.assignment_wrap != WrapStyle::DoNotWrap
        {
            if let Some(val) = self.fld(decls[0], "value") {
                let name = self
                    .fld(decls[0], "name")
                    .map(|n| self.txt(n))
                    .unwrap_or("");
                let prefix = format!("{} {}", out, name);
                return format!("{};", self.assign_expr(val, indent, c, &prefix, "="));
            }
        }

        let decl_strs: Vec<String> = decls
            .iter()
            .map(|&d| {
                let name = self.fld(d, "name").map(|n| self.txt(n)).unwrap_or("");
                if let Some(val) = self.fld(d, "value") {
                    let val_col = c + self.col_after(0, &out) + 1 + name.len() + 3;
                    let val_str = self.expr(val, indent, val_col);
                    format!("{} = {}", name, val_str)
                } else {
                    name.to_string()
                }
            })
            .collect();

        out.push(' ');
        out.push_str(&decl_strs.join(", "));
        out.push(';');
        out
    }

    /// Returns the text to place between the declaration header and the block body,
    /// according to the brace style. Caller should `push_str` the result.
    fn brace_before_body(&self, indent: usize, style: BraceStyle, body: &str) -> String {
        match style {
            BraceStyle::NextLine | BraceStyle::NextLineShifted | BraceStyle::NextLineShifted2 => {
                format!("\n{}{}", self.ind(indent), body)
            }
            _ => format!(" {}", body),
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
        let flat_ann = format!("@{}({})", name, flat_inner);

        // Decide whether to expand.
        // ChopDownIfLong (value 5) = expand only when the flat form is too long.
        // When expanded, each argument (and each array element) goes on its own line.
        let needs_expand = match self.style.annotation_parameter_wrap {
            WrapStyle::DoNotWrap => false,
            WrapStyle::WrapAlways => true,
            // WrapIfLong | ChopDownIfLong: only expand when the flat form overflows
            _ => !self.fits(0, &flat_ann),
        } && self.ann_args_need_expand(args_node);

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
            .join(", ")
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
                            "@{}({} = {{\n{}\n{}}})",
                            name,
                            k,
                            elem_strs.join(",\n"),
                            self.ind(indent)
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
                    "@{}({{\n{}\n{}}})",
                    name,
                    elem_strs.join(",\n"),
                    self.ind(indent)
                );
            }
        }

        // Multiple args: one per line
        let arg_strs: Vec<_> = children
            .iter()
            .map(|&c| format!("{}{}", self.ind(inner), self.flat_ann_arg(c)))
            .collect();
        format!(
            "@{}(\n{}\n{})",
            name,
            arg_strs.join(",\n"),
            self.ind(indent)
        )
    }

    // ── formal parameters ─────────────────────────────────────────────────────

    fn formal_params(&self, node: Node<'s>, indent: usize, c: usize, is_call: bool) -> String {
        let params = self.named(node);

        if params.is_empty() {
            return "()".to_string();
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
        let flat = format!("({})", flat_parts.join(", "));

        let should_wrap = match wrap {
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

        match (lparen_nl, rparen_nl) {
            (true, true) => format!("(\n{}\n{})", wrapped, self.ind(indent)),
            (true, false) => format!("(\n{})", wrapped),
            (false, true) => format!("({}\n{})", wrapped, self.ind(indent)),
            (false, false) => format!("(\n{})", wrapped),
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

    fn block(&self, node: Node<'s>, indent: usize, _c: usize) -> String {
        let stmts = self.named(node);
        if stmts.is_empty() {
            return "{}".to_string();
        }

        let inner = indent + 1;
        let mut out = String::from("{\n");

        for (i, s) in stmts.iter().enumerate() {
            // Preserve blank lines from the original source between statements.
            if i > 0 {
                let prev_end = stmts[i - 1].end_byte();
                let cur_start = s.start_byte();
                if self.has_blank_line_between(prev_end, cur_start) {
                    out.push('\n');
                }
            }
            out.push_str(&self.ind(inner));
            let sc = self.col_after(0, &self.ind(inner));
            if s.is_extra() {
                out.push_str(self.txt(*s));
            } else {
                out.push_str(&self.stmt(*s, inner, sc));
            }
            out.push('\n');
        }

        out.push_str(&self.ind(indent));
        out.push('}');
        out
    }

    /// True when the byte range `[prev_end, next_start)` in the source contains
    /// more than one newline, indicating at least one blank line was present.
    fn has_blank_line_between(&self, prev_end: usize, next_start: usize) -> bool {
        if prev_end >= next_start {
            return false;
        }
        let slice = &self.src[prev_end..next_start.min(self.src.len())];
        std::str::from_utf8(slice)
            .map(|s| s.chars().filter(|&c| c == '\n').count() > 1)
            .unwrap_or(false)
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
    /// all simple; `None` if any body is not a simple block.
    fn if_one_line(&self, node: Node<'s>) -> Option<String> {
        let cond = self.fld(node, "condition")?;
        let cond_txt = self.flat(cond);
        if cond_txt.contains('\n') {
            return None;
        }
        let cons = self.fld(node, "consequence")?;
        let cons_txt = self.one_line_body(cons)?;
        // `cond_txt` already includes the parentheses (parenthesized_expression).
        let mut out = format!("if {} {}", cond_txt, cons_txt);
        if let Some(alt) = self.fld(node, "alternative") {
            out.push_str(" else ");
            if alt.kind() == "if_statement" {
                out.push_str(&self.if_one_line(alt)?);
            } else {
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
            "block" => self.block(node, indent, c),
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

        // Single declarator whose initialiser can be wrapped at the operator.
        if decls.len() == 1 && self.style.assignment_wrap != WrapStyle::DoNotWrap {
            if let Some(val) = self.fld(decls[0], "value") {
                let name = self
                    .fld(decls[0], "name")
                    .map(|n| self.txt(n))
                    .unwrap_or("");
                let prefix = format!("{}{}", out, name); // `out` ends with a space
                return format!("{};", self.assign_expr(val, indent, c, &prefix, "="));
            }
        }

        let decl_strs: Vec<String> = decls
            .iter()
            .map(|&d| {
                let name = self.fld(d, "name").map(|n| self.txt(n)).unwrap_or("");
                if let Some(val) = self.fld(d, "value") {
                    let val_col = self.col_after(c, &out) + name.len() + 3;
                    let val_str = self.expr(val, indent, val_col);
                    format!("{} = {}", name, val_str)
                } else {
                    name.to_string()
                }
            })
            .collect();

        out.push_str(&decl_strs.join(", "));
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
        let cond = self
            .fld(node, "condition")
            .map(|n| self.expr(n, indent, c + 4))
            .unwrap_or_default();

        let cons = self
            .fld(node, "consequence")
            .map(|n| self.stmt_as_block_or_inline(n, indent, c))
            .unwrap_or_default();

        let mut out = format!("if {}{}", cond, cons);

        if let Some(alt) = self.fld(node, "alternative") {
            let alt_str = if alt.kind() == "if_statement" {
                format!(" else {}", self.if_stmt(alt, indent, c))
            } else {
                format!(" else{}", self.stmt_as_block_or_inline(alt, indent, c))
            };
            out.push_str(&alt_str);
        }

        out
    }

    fn stmt_as_block_or_inline(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        if node.kind() == "block" {
            format!(" {}", self.block(node, indent, c))
        } else {
            let s = self.stmt(node, indent + 1, self.col_after(0, &self.ind(indent + 1)));
            format!("\n{}{}", self.ind(indent + 1), s)
        }
    }

    fn for_stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let body_node = self.fld(node, "body");

        // Re-create header from source bytes (handles all edge cases of for-init/cond/update)
        let header = if let Some(b) = body_node {
            let raw = std::str::from_utf8(&self.src[node.start_byte()..b.start_byte()])
                .unwrap_or("for (...)");
            normalise_ws(raw)
        } else {
            self.txt(node).to_string()
        };

        // Keep a simple body on one line when enabled and it fits.
        if self.braces_style_inline() && self.style.keep_simple_blocks_in_one_line {
            if let Some(one) = body_node.and_then(|b| self.one_line_body(b)) {
                let candidate = format!("{} {}", header, one);
                if self.fits(c, &candidate) {
                    return candidate;
                }
            }
        }

        let body = body_node
            .map(|n| self.stmt_as_block_or_inline(n, indent, c))
            .unwrap_or_default();

        format!("{}{}", header, body)
    }

    fn enhanced_for(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let ty = self
            .fld(node, "type")
            .map(|n| self.flat_type(n))
            .unwrap_or_default();
        let name = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");

        // Keep a simple body on one line when enabled and it fits.
        if self.braces_style_inline() && self.style.keep_simple_blocks_in_one_line {
            if let (Some(v), Some(b)) = (self.fld(node, "value"), self.fld(node, "body")) {
                if let Some(one) = self.one_line_body(b) {
                    let vtxt = self.flat(v);
                    if !vtxt.contains('\n') {
                        let candidate = format!("for ({} {} : {}) {}", ty, name, vtxt, one);
                        if self.fits(c, &candidate) {
                            return candidate;
                        }
                    }
                }
            }
        }

        let val = self
            .fld(node, "value")
            .map(|n| self.expr(n, indent, c + ty.len() + name.len() + 8))
            .unwrap_or_default();
        let body = self
            .fld(node, "body")
            .map(|n| self.stmt_as_block_or_inline(n, indent, c))
            .unwrap_or_default();
        format!("for ({} {} : {}){}", ty, name, val, body)
    }

    fn while_stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        // Keep a simple body on one line when enabled and it fits.
        if self.braces_style_inline() && self.style.keep_simple_blocks_in_one_line {
            if let (Some(cn), Some(b)) = (self.fld(node, "condition"), self.fld(node, "body")) {
                if let Some(one) = self.one_line_body(b) {
                    let ct = self.flat(cn);
                    if !ct.contains('\n') {
                        let candidate = format!("while {} {}", ct, one);
                        if self.fits(c, &candidate) {
                            return candidate;
                        }
                    }
                }
            }
        }

        let cond = self
            .fld(node, "condition")
            .map(|n| self.expr(n, indent, c + 7))
            .unwrap_or_default();
        let body = self
            .fld(node, "body")
            .map(|n| self.stmt_as_block_or_inline(n, indent, c))
            .unwrap_or_default();
        // `cond` is a parenthesized_expression and already contains its parens.
        format!("while {}{}", cond, body)
    }

    fn do_while(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        // Keep a simple body on one line when enabled and it fits.
        if self.braces_style_inline() && self.style.keep_simple_blocks_in_one_line {
            if let (Some(b), Some(cn)) = (self.fld(node, "body"), self.fld(node, "condition")) {
                if let Some(one) = self.one_line_body(b) {
                    let ct = self.flat(cn);
                    if !ct.contains('\n') {
                        let candidate = format!("do {} while {};", one, ct);
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
                    format!(" {}", self.block(n, indent, c))
                } else {
                    let s = self.stmt(n, indent + 1, self.col_after(0, &self.ind(indent + 1)));
                    format!("\n{}{}\n{}", self.ind(indent + 1), s, self.ind(indent))
                }
            })
            .unwrap_or_default();
        // `cond` is a parenthesized_expression and already contains its parens.
        let cond = self
            .fld(node, "condition")
            .map(|n| self.expr(n, indent, c + 9))
            .unwrap_or_default();
        format!("do{} while {};", body, cond)
    }

    /// Single-line rendering of a `try` statement when the try body and every
    /// catch/finally body is a simple one-statement block; `None` otherwise
    /// (the caller falls through to the multi-line layout).
    fn try_one_line(&self, node: Node<'s>) -> Option<String> {
        let resources = if node.kind() == "try_with_resources_statement" {
            // The resource_specification node already includes its parens.
            self.fld(node, "resources")
                .map(|n| format!(" {}", normalise_ws(self.txt(n))))
                .unwrap_or_default()
        } else {
            String::new()
        };

        let body = self.fld(node, "body")?;
        let body_txt = self.one_line_body(body)?;
        let mut out = format!("try{} {}", resources, body_txt);

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
                    out.push_str(&format!(" catch ({}) {}", param, cbody_txt));
                }
                "finally_clause" => {
                    // The block is a plain child of finally_clause (no field name).
                    let fbody = self.named(ch).into_iter().find(|n| n.kind() == "block")?;
                    let fbody_txt = self.one_line_body(fbody)?;
                    out.push_str(&format!(" finally {}", fbody_txt));
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
                .map(|n| format!(" {}", self.txt(n).trim()))
                .unwrap_or_default()
        } else {
            String::new()
        };

        let body = self
            .fld(node, "body")
            .map(|n| self.block(n, indent, c))
            .unwrap_or_default();

        let mut out = format!("try{} {}", resources, body);

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
                        .map(|n| self.block(n, indent, c))
                        .unwrap_or_default();
                    out.push_str(&format!(" catch ({}) {}", param, cbody));
                }
                "finally_clause" => {
                    // The block is a plain child of finally_clause (no field name).
                    let fbody = self
                        .named(ch)
                        .into_iter()
                        .find(|n| n.kind() == "block")
                        .map(|n| self.block(n, indent, c))
                        .unwrap_or_default();
                    out.push_str(&format!(" finally {}", fbody));
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
                    let lt = self.flat(*lock);
                    if !lt.contains('\n') {
                        let candidate = format!("synchronized {} {}", lt, one);
                        if self.fits(c, &candidate) {
                            return candidate;
                        }
                    }
                }
            }
        }

        // find the parenthesized lock expression and block body
        let children = self.named(node);
        let lock = children
            .iter()
            .find(|n| n.kind() == "parenthesized_expression")
            .map(|n| self.expr(*n, indent, c + 12))
            .unwrap_or_default();
        let body = children
            .iter()
            .find(|n| n.kind() == "block")
            .map(|n| self.block(*n, indent, c))
            .unwrap_or_default();
        // `lock` is a parenthesized_expression and already contains its parens.
        format!("synchronized {} {}", lock, body)
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
        let cond = self
            .fld(node, "condition")
            .map(|n| self.expr(n, indent, c + 7))
            .unwrap_or_default();
        let body = match self.fld(node, "body") {
            Some(b) if b.kind() == "switch_block" => b,
            _ => return self.txt(node).to_string(), // R4
        };

        if self.named(body).is_empty() {
            return format!("switch {} {{}}", cond);
        }

        let inner = indent + 1;
        let mut out = format!("switch {} {{\n", cond);

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
                out.push_str(&self.block(b, indent, 0));
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
        let cond = self.fld(node, "condition").map(|n| self.flat(n))?;
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
        Some(format!("switch {} {{ {} }}", cond, parts.join(" ")))
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
                format!("{}[{}]", arr, idx)
            }
            "assignment_expression" => self.assignment(node, indent, c),
            "binary_expression" => self.binary(node, indent, c),
            "unary_expression" => {
                let op = self
                    .fld(node, "operator")
                    .map(|n| self.txt(n))
                    .unwrap_or("");
                let operand = self
                    .fld(node, "operand")
                    .map(|n| self.expr(n, indent, c + op.len()))
                    .unwrap_or_default();
                format!("{}{}", op, operand)
            }
            "update_expression" => self.txt(node).to_string(),
            "ternary_expression" => self.ternary(node, indent, c),
            "cast_expression" => {
                let ty = self
                    .fld(node, "type")
                    .map(|n| self.flat_type(n))
                    .unwrap_or_default();
                let val = self
                    .fld(node, "value")
                    .map(|n| self.expr(n, indent, c + ty.len() + 2))
                    .unwrap_or_default();
                format!("({}){}", ty, val)
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
            "method_reference" => self.txt(node).to_string(),
            "parenthesized_expression" => {
                let inner = node
                    .named_child(0)
                    .map(|n| self.expr(n, indent, c + 1))
                    .unwrap_or_default();
                format!("({})", inner)
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

        if self.fits(c, &flat) {
            return flat;
        }

        // Detect chain
        if self.style.method_call_chain_wrap != WrapStyle::DoNotWrap {
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
            .map(|n| format!("{}.", self.flat(n)))
            .unwrap_or_default();
        let ta = self
            .fld(node, "type_arguments")
            .map(|n| self.flat_type_args(n))
            .unwrap_or_default();
        let name = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");
        let args = self
            .fld(node, "arguments")
            .map(|n| self.flat_args(n))
            .unwrap_or_else(|| "()".to_string());
        format!("{}{}{}{}", obj, ta, name, args)
    }

    fn flat_args(&self, node: Node<'s>) -> String {
        let inner = self
            .named(node)
            .iter()
            .map(|&a| self.flat(a))
            .collect::<Vec<_>>()
            .join(", ");
        format!("({})", inner)
    }

    fn inv_wrapped(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let obj = self
            .fld(node, "object")
            .map(|n| format!("{}.", self.expr(n, indent, c)))
            .unwrap_or_default();
        let ta = self
            .fld(node, "type_arguments")
            .map(|n| self.flat_type_args(n))
            .unwrap_or_default();
        let name = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");
        let prefix = format!("{}{}{}", obj, ta, name);
        let args_col = self.col_after(c, &prefix);

        let args_str = self
            .fld(node, "arguments")
            .map(|n| self.args_wrapped(n, indent, args_col))
            .unwrap_or_else(|| "()".to_string());

        format!("{}{}", prefix, args_str)
    }

    fn args_wrapped(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let args = self.named(node);
        if args.is_empty() {
            return "()".to_string();
        }

        let flat = self.flat_args(node);
        if self.fits(c, &flat) {
            return flat;
        }

        // Single argument that is a long chain → wrap the chain inline
        if args.len() == 1 && self.style.method_call_chain_wrap != WrapStyle::DoNotWrap {
            if args[0].kind() == "method_invocation" {
                let (base, links) = self.collect_chain(args[0]);
                if links.len() >= 2 {
                    let chain_str = self.fmt_chain(&base, &links, indent, c + 1);
                    return format!("({})", chain_str);
                }
            }
        }

        let wrap = self.style.call_parameters_wrap;
        if wrap == WrapStyle::DoNotWrap {
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
        match (lp, rp) {
            (true, true) => format!("(\n{}\n{})", arg_strs.join(",\n"), self.ind(indent)),
            (true, false) => format!("(\n{})", arg_strs.join(",\n")),
            (false, true) => format!("({}\n{})", arg_strs.join(",\n"), self.ind(indent)),
            (false, false) => format!("(\n{})", arg_strs.join(",\n")),
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

        for (i, link) in links.iter().enumerate() {
            let ta = link
                .type_args
                .map(|n| self.flat_type_args(n))
                .unwrap_or_default();
            let nm = self.txt(link.name);
            let flat_a = self.flat_args(link.args);

            if i == 0 {
                if base.is_empty() {
                    out = format!("{}{}{}", ta, nm, flat_a);
                } else {
                    out = format!("{}.{}{}{}", base, ta, nm, flat_a);
                }
            } else {
                out.push('\n');
                out.push_str(&cont);
                out.push('.');
                out.push_str(&ta);
                out.push_str(nm);
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
        let has_body = self.fld(node, "class_body").is_some();

        if let Some(args_node) = self.fld(node, "arguments") {
            let flat_a = self.flat_args(args_node);
            let flat = format!("{}{}", prefix, flat_a);

            if !has_body && self.fits(c, &flat) {
                return flat;
            }

            let args_str = self.args_wrapped(args_node, indent, c + prefix.len());
            let body_str = self
                .fld(node, "class_body")
                .map(|n| format!(" {}", self.class_body(n, indent)))
                .unwrap_or_default();

            format!("{}{}{}", prefix, args_str, body_str)
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
            Some(r) if self.style.assignment_wrap != WrapStyle::DoNotWrap => {
                self.assign_expr(r, indent, c, &left, op)
            }
            _ => {
                let right = rhs
                    .map(|n| {
                        let rc = c + left.len() + 4; // " op "
                        self.expr(n, indent, rc)
                    })
                    .unwrap_or_default();
                format!("{} {} {}", left, op, right)
            }
        }
    }

    /// Renders `prefix op rhs` honouring `ASSIGNMENT_WRAP`.
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
    ) -> String {
        let rhs_col = self.col_after(c, prefix) + op.len() + 2; // " op " separator
        let same = self.expr(rhs, indent, rhs_col);

        // The RHS wrapped internally; leave the operator on the header line.
        if same.contains('\n') {
            return format!("{} {} {}", prefix, op, same);
        }

        let flat = format!("{} {} {}", prefix, op, same);
        if self.style.assignment_wrap != WrapStyle::WrapAlways && self.fits(c, &flat) {
            return flat;
        }

        let cont = self.cont(indent);
        let cont_col = self.col_after(0, &cont);
        let nl = self.expr(rhs, indent, cont_col);
        format!("{} {}\n{}{}", prefix, op, cont, nl)
    }

    fn binary(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let wrap = self.style.binary_operation_wrap;

        let flat = format!(
            "{} {} {}",
            self.fld(node, "left")
                .map(|n| self.flat(n))
                .unwrap_or_default(),
            self.fld(node, "operator")
                .map(|n| self.txt(n))
                .unwrap_or("+"),
            self.fld(node, "right")
                .map(|n| self.flat(n))
                .unwrap_or_default()
        );

        // DoNotWrap (and the default style) keep today's single-line output.
        if wrap == WrapStyle::DoNotWrap || (wrap != WrapStyle::WrapAlways && self.fits(c, &flat)) {
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
            out.push(' ');
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
        let flat = format!(
            "{} ? {} : {}",
            self.fld(node, "condition")
                .map(|n| self.flat(n))
                .unwrap_or_default(),
            self.fld(node, "consequence")
                .map(|n| self.flat(n))
                .unwrap_or_default(),
            self.fld(node, "alternative")
                .map(|n| self.flat(n))
                .unwrap_or_default()
        );
        if self.fits(c, &flat) {
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
        format!("{}\n{}? {}\n{}: {}", cond, cont, cons, cont, alt)
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
                    format!("({})", ps.join(", "))
                }
                _ => self.txt(n).to_string(),
            })
            .unwrap_or_default();

        let body_node = self
            .fld(node, "body")
            .unwrap_or_else(|| node.named_child(1).unwrap());
        let body = if body_node.kind() == "block" {
            // check keep_simple
            let flat = self.flat_block(body_node);
            if self.style.keep_simple_lambdas_in_one_line && self.fits(c + params.len() + 4, &flat)
            {
                flat
            } else {
                self.block(body_node, indent, c)
            }
        } else {
            self.expr(body_node, indent, c + params.len() + 4)
        };

        format!("{} -> {}", params, body)
    }

    fn flat_formal_params(&self, node: Node<'s>) -> String {
        let params = self.named(node);
        let inner = params
            .iter()
            .map(|&p| self.flat_param(p))
            .collect::<Vec<_>>()
            .join(", ");
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
            .map(|n| format!(" {}", self.array_init(n, indent, c)))
            .unwrap_or_default();
        format!("new {}{}{}", ty, dims.join(""), init)
    }

    fn array_init(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let flat = self.flat_arr_init(node);
        if self.fits(c, &flat) {
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
        format!("{{\n{}\n{}}}", elem_strs.join(",\n"), self.ind(indent))
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
    /// inside the angle brackets and one space after each comma.
    fn flat_type_args(&self, node: Node<'s>) -> String {
        let inner: Vec<_> = self
            .named(node)
            .iter()
            .map(|n| self.flat_type(*n))
            .collect();
        format!("<{}>", inner.join(", "))
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
                return inner.join(", ");
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
                format!("{}[{}]", arr, idx)
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
                format!("{} {} {}", left, op, right)
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
                format!("{} {} {}", left, op, right)
            }
            "unary_expression" => {
                let op = self
                    .fld(node, "operator")
                    .map(|n| self.txt(n))
                    .unwrap_or("");
                let operand = self
                    .fld(node, "operand")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                format!("{}{}", op, operand)
            }
            "update_expression" => self.txt(node).to_string(),
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
                format!("{} ? {} : {}", c, t, f)
            }
            "cast_expression" => {
                let ty = self
                    .fld(node, "type")
                    .map(|n| self.flat_type(n))
                    .unwrap_or_default();
                let val = self
                    .fld(node, "value")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                format!("({}){}", ty, val)
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
                format!("({})", inner)
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
            "method_reference" => self.txt(node).to_string(),
            _ => self.txt(node).to_string(),
        }
    }

    fn flat_annotation(&self, node: Node<'s>) -> String {
        let name = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");
        let args = self
            .fld(node, "arguments")
            .map(|n| format!("({})", self.flat_ann_args(n)))
            .unwrap_or_default();
        format!("@{}{}", name, args)
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
            .unwrap_or_else(|| "()".to_string());
        let body = self
            .fld(node, "class_body")
            .map(|_| " { ... }".to_string())
            .unwrap_or_default();
        format!("new {}{}{}{}", ta, ty, args, body)
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
                    format!("({})", ps.join(", "))
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
        format!("{} -> {}", params, body)
    }

    fn flat_block(&self, node: Node<'s>) -> String {
        let stmts = self.named(node);
        if stmts.is_empty() {
            return "{}".to_string();
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
            return "{}".to_string();
        }
        let inner = elems
            .iter()
            .map(|&e| self.flat(e))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{{{}}}", inner)
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
            .map(|n| format!(" {}", self.flat_arr_init(n)))
            .unwrap_or_default();
        format!("new {}{}{}", ty, dims.join(""), init)
    }
}

// ── standalone utilities ──────────────────────────────────────────────────────

/// Collapse runs of whitespace (including newlines) to a single space.
fn normalise_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
