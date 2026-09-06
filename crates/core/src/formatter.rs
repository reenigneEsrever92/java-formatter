//! Java source-code formatter driven by IntelliJ codestyle settings.
//!
//! Uses tree-sitter-java to parse source into a CST, then pretty-prints it
//! following the rules encoded in [`crate::config::JavaStyle`].

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};

use tree_sitter::{Language, Node, Parser};

use crate::config::{BraceStyle, ForceStyle, ImportLayoutEntry, JavaStyle, WrapStyle};

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

    // `import module …;` is not a tree-sitter-java production (it would parse
    // as an `ERROR` node), so the import-region module lines are collected and
    // blanked out of the parsed source in place (equal-length spaces keep byte
    // offsets and diagnostics unaffected); the kept lines are re-emitted at the
    // layout table's module slot by `imports()`.
    let (parse_src, module_imports) = prepare_module_imports(source, style);

    let src = parse_src.as_bytes();
    let tree = parser
        .parse(src, None)
        .expect("Failed to parse Java source");

    let diagnostics = collect_parse_diagnostics(tree.root_node(), src);

    // The formatter reads the ORIGINAL source for node text and byte-gap
    // spacing, so the blanked module lines neither leak into the output nor
    // count as preserved blank lines; only tree-sitter parses the masked text.
    let fmt = Fmt {
        src: source.as_bytes(),
        style,
        module_imports,
    };
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
// Module imports (`import module …;`)
// ─────────────────────────────────────────────────────────────────────────────

/// An `import module …;` line collected from the file's import region (the
/// leading blank / comment / package / import lines before the first type).
/// tree-sitter-java has no production for it (it would parse as an `ERROR`
/// node), so such lines are blanked out of the parsed source in place and
/// re-emitted at the layout table's module slot.
struct ModuleImport {
    /// The trimmed full source line, e.g. `import module java.base;`.
    line: String,
    /// Byte range of the line in the original source (including its ending).
    start: usize,
    end: usize,
}

/// The module name of a trimmed `import module <name>;` line, or `None` when
/// the line does not match that shape (a trailing `//` or `/*` comment after
/// the `;` is tolerated).
fn module_import_name(t: &str) -> Option<String> {
    let rest = t.strip_prefix("import")?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix("module")?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let path = rest.trim_start();
    let semi = path.find(';')?;
    let name = path[..semi].trim();
    let tail = path[semi + 1..].trim_start();
    if !tail.is_empty() && !tail.starts_with("//") && !tail.starts_with("/*") {
        return None;
    }
    let valid = !name.is_empty()
        && name.split('.').all(|seg| {
            let mut chars = seg.chars();
            match chars.next() {
                Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
                _ => return false,
            }
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
        });
    if !valid {
        return None;
    }
    Some(name.to_string())
}

/// Scan `source`'s import region for `import module …;` lines and return the
/// source with each such line blanked in place (equal-length spaces, newline
/// preserved, so byte offsets and diagnostics are unaffected) together with
/// the lines to keep, per the module-import options:
///
/// * `PRESERVE_MODULE_IMPORTS` off → none are kept;
/// * `DELETE_UNUSED_MODULE_IMPORTS` on → of the clearly-unused cases only a
///   repeated identical `import module` line is provable without symbol
///   resolution, so duplicates beyond the first are dropped and every other
///   line is kept (conservative).
fn prepare_module_imports<'a>(
    source: &'a str,
    style: &JavaStyle,
) -> (Cow<'a, str>, Vec<ModuleImport>) {
    // One pass over the import region: collect each `import module …;` line,
    // the byte range of its content to blank, and the lines to keep. When no
    // module line exists the source is returned borrowed, so the common
    // module-free path parses the original text with no copy.
    let mut kept: Vec<ModuleImport> = Vec::new();
    let mut mask_ranges: Vec<(usize, usize)> = Vec::new(); // (start, content bytes)
    let mut seen: HashSet<String> = HashSet::new();
    let mut in_block = false;
    let mut region_open = true;
    let mut offset = 0usize;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']).trim();
        let mut is_module = false;
        if region_open {
            if in_block {
                if trimmed.contains("*/") {
                    in_block = false;
                }
            } else if trimmed.starts_with("/*") {
                if !trimmed.contains("*/") {
                    in_block = true;
                }
            } else if !trimmed.is_empty() && !trimmed.starts_with("//") {
                if let Some(name) = module_import_name(trimmed) {
                    is_module = true;
                    if style.preserve_module_imports {
                        let duplicate = style.delete_unused_module_imports && !seen.insert(name);
                        if !duplicate {
                            kept.push(ModuleImport {
                                line: trimmed.to_string(),
                                start: offset,
                                end: offset + line.len(),
                            });
                        }
                    }
                } else if !trimmed.starts_with("import ") && !trimmed.starts_with("package ") {
                    // First non-import / non-package content: the import
                    // region (and the scan) ends here.
                    region_open = false;
                }
            }
        }
        if is_module {
            // The content bytes to blank: everything except the line ending.
            let mut content_len = line.len();
            if content_len > 0 && line.as_bytes()[content_len - 1] == b'\n' {
                content_len -= 1;
            }
            if content_len > 0 && line.as_bytes()[content_len - 1] == b'\r' {
                content_len -= 1;
            }
            mask_ranges.push((offset, content_len));
        }
        offset += line.len();
    }
    if mask_ranges.is_empty() {
        return (Cow::Borrowed(source), kept);
    }
    // Blank the module lines in place (equal-length spaces keep byte offsets
    // and diagnostics unaffected).
    let mut masked = source.to_string();
    for (start, len) in mask_ranges {
        masked.replace_range(start..start + len, &" ".repeat(len));
    }
    (Cow::Owned(masked), kept)
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

struct Fmt<'s> {
    src: &'s [u8],
    style: &'s JavaStyle,
    /// The preserved `import module …;` lines (see [`prepare_module_imports`]).
    module_imports: Vec<ModuleImport>,
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

/// One emitted statement / member line, buffered so the columnar
/// `ALIGN_CONSECUTIVE_*` / `ALIGN_GROUP_*` / `ALIGN_SUBSEQUENT_*` options can
/// pad runs of consecutive lines before they are written.
struct BodyLine {
    /// Blank lines to emit before this line.
    blanks: usize,
    /// Whether the emitter prefixes `ind(inner)` to `text` — false for
    /// comments, which render their own column placement.
    indented: bool,
    /// The line content (statement/member text or a full comment).
    text: String,
    /// Columnar-alignment candidate info: the run kind, the column (within
    /// `text`) of the element to align, and the offset (within `text`) where
    /// padding is inserted. `None` for comments and other non-candidates,
    /// which break runs.
    align: Option<(u8, usize, usize)>,
}

/// Pad the lines of each run — a maximal stretch of consecutive candidate
/// lines of the same kind with no blank line and no non-candidate between — so
/// the aligned element (declaration/method name, assignment `=`) shares one
/// column. Space-based, like the continuation alignment (README tab note); a
/// line whose element already sits at the widest column is left alone.
fn pad_column_runs(lines: &mut [BodyLine]) {
    let mut i = 0;
    while i < lines.len() {
        let start_kind = match lines[i].align {
            Some((k, _, _)) => k,
            None => {
                i += 1;
                continue;
            }
        };
        let mut j = i + 1;
        while j < lines.len()
            && lines[j].blanks == 0
            && lines[j].align.is_some_and(|(k, _, _)| k == start_kind)
        {
            j += 1;
        }
        let max = lines[i..j]
            .iter()
            .filter_map(|l| l.align)
            .map(|(_, col, _)| col)
            .max()
            .unwrap_or(0);
        for l in &mut lines[i..j] {
            if let Some((_, col, ins)) = l.align {
                if col < max && ins <= l.text.len() {
                    l.text.insert_str(ins, &" ".repeat(max - col));
                }
            }
        }
        i = j;
    }
}

// ── javadoc ─────────────────────────────────────────────────────────────────

/// Which javadoc tag a tag block carries; drives the shape validation and the
/// rendering (name required, `@exception` normalisation, alignment group).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JavadocTagKind {
    Param,
    Throws,
    Exception,
    Return,
    /// Any other `@tag` (`@see`, `@since`, `@author`, …) — free text.
    Other,
}

/// One javadoc tag block: the tag head plus its description lines (the first
/// description line and any continuation lines, `*`-stripped).
struct JavadocTag {
    kind: JavadocTagKind,
    /// The `@param` name / `@throws`-`@exception` type / unknown tag name
    /// (`""` for `@return`).
    name: String,
    desc: Vec<String>,
}

/// Parsed javadoc content: the description lines (an empty string marks an
/// empty line) followed by the ordered tag blocks.
struct JavadocDoc {
    description: Vec<String>,
    tags: Vec<JavadocTag>,
}

/// The layout-relevant classification of one import line's text: whether it is
/// a static import, whether it is an on-demand (`.*`) import, and its package
/// part (`pkg.Owner` for a static member import, `pkg` for a type import,
/// empty for a default-package single-segment import).
fn classify_import_line(text: &str) -> (bool, bool, String) {
    let mut is_static = false;
    let mut path = text;
    if let Some(rest) = path.strip_prefix("import ") {
        path = rest;
    }
    if let Some(rest) = path.strip_prefix("static ") {
        is_static = true;
        path = rest;
    }
    let path = path.trim().trim_end_matches(';').trim();
    let (is_on_demand, pkg) = match path.rsplit_once('.') {
        Some((pkg, simple)) => (simple == "*", pkg.to_string()),
        None => (false, String::new()),
    };
    (is_static, is_on_demand, pkg)
}

/// True when import package `pkg` belongs to a table entry named `name`:
/// equal packages always match, subpackages only when the entry is
/// `withSubpackages` (a package-boundary prefix match, so `java` does not
/// capture `javafx`).
fn entry_matches_pkg(name: &str, with_subpackages: bool, pkg: &str) -> bool {
    match pkg.strip_prefix(name) {
        Some("") => true,
        Some(rest) => with_subpackages && rest.starts_with('.'),
        None => false,
    }
}

/// The import-layout entry whose group owns an import with package `pkg` and
/// the given static-ness, or `None` when no entry matches (callers fall back
/// to an implicit trailing group so imports are never dropped). Among named
/// prefix matches the longest name wins; the empty-name entries are the
/// catch-alls (the first eligible one in table order). With
/// `LAYOUT_STATIC_IMPORTS_SEPARATELY` on, an entry's `static` attribute must
/// match the line; off, the attribute is ignored and static imports join the
/// ordinary package sections.
fn layout_entry_for(
    entries: &[ImportLayoutEntry],
    is_static: bool,
    pkg: &str,
    separate: bool,
) -> Option<usize> {
    let mut named: Option<(usize, usize)> = None; // (name length, index) — longest wins
    let mut catch_all: Option<usize> = None; // first eligible empty-name entry
    for (i, e) in entries.iter().enumerate() {
        let (name, with_sub, static_attr, is_module) = match e {
            ImportLayoutEntry::EmptyLine => continue,
            ImportLayoutEntry::Package {
                name,
                with_subpackages,
                is_static,
                is_module,
            } => (name.as_str(), *with_subpackages, *is_static, *is_module),
        };
        if is_module {
            continue;
        }
        if separate && static_attr != is_static {
            continue;
        }
        if name.is_empty() {
            if catch_all.is_none() {
                catch_all = Some(i);
            }
            continue;
        }
        if entry_matches_pkg(name, with_sub, pkg) && named.is_none_or(|(len, _)| name.len() > len) {
            named = Some((name.len(), i));
        }
    }
    named.map(|(_, i)| i).or(catch_all)
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
        let cont_size = self.style.continuation_indent_size as usize;
        // `USE_RELATIVE_INDENTS` (gated on `USE_TAB_CHARACTER`): the
        // continuation width is measured from the construct's own indent
        // level — the continuation offset over one indent unit (`cont_size -
        // indent_size`) is added to the level's own indentation instead of
        // to the full level columns, so deeper nesting still deepens the
        // continuation but it sits one unit closer to the statement. The
        // default (`false`, or any space-indented scheme) keeps today's
        // `level * indent + continuation` width byte-identical.
        let width = if self.style.use_relative_indents && self.style.use_tab_character {
            let indent = self.style.indent_size as usize;
            level * indent + cont_size.saturating_sub(indent)
        } else {
            level * self.style.indent_size as usize + cont_size
        };
        self.indent_str(width)
    }

    /// Alignment prefix: `width` spaces, so the following token starts at
    /// 0-based column `width` — the column of the construct's first element
    /// the continuation lines are aligned under. Space-based like the record
    /// header alignment (see the README tab note), because a tab-prefixed
    /// line could not land exactly on the alignment column.
    fn align_prefix(&self, width: usize) -> String {
        " ".repeat(width)
    }

    /// Build an indentation string of `width` columns. When
    /// `USE_TAB_CHARACTER` is set, each full `tab_size` column becomes a tab
    /// character and the remainder spaces — a tab-stop model matching
    /// IntelliJ (so `indent_size == tab_size` yields exactly one tab per
    /// level). `SMART_TABS` restricts tab characters to indentation that
    /// lands exactly on a tab stop: an indent whose width is not a whole
    /// number of tabs is emitted as pure spaces (alignment and continuation
    /// indents that cannot land on a tab stay space-based). Otherwise plain
    /// spaces are emitted, byte-identical to the historical output.
    fn indent_str(&self, width: usize) -> String {
        if !self.style.use_tab_character {
            return " ".repeat(width);
        }
        let tab = self.style.tab_size as usize;
        if self.style.smart_tabs && !width.is_multiple_of(tab) {
            return " ".repeat(width);
        }
        format!("{}{}", "\t".repeat(width / tab), " ".repeat(width % tab))
    }

    /// Per-construct continuation width: an explicit width (`>= 0`) renders
    /// `level` indent units plus `width` continuation columns (the scheme's
    /// per-construct override of the continuation indent for that construct
    /// kind); `-1` (the built-in default, "inherit") returns `fallback`
    /// byte-for-byte, keeping today's layout (AC2).
    fn construct_ind(&self, level: usize, width: i32, fallback: &str) -> String {
        if width >= 0 {
            self.indent_str(level * self.style.indent_size as usize + width as usize)
        } else {
            fallback.to_string()
        }
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
    /// is content, so a comment line is never counted as blank — including
    /// whitespace-only lines inside a block comment, which a naive
    /// whitespace-only line test would miscount (and re-count on every
    /// reformat, breaking idempotency for the javadoc layouts that emit
    /// blank lines without a `*`).
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
        let mut in_block = false;
        for seg in segments
            .iter()
            .take(segments.len().saturating_sub(1))
            .skip(1)
        {
            let in_block_at_start = in_block;
            let chars: Vec<char> = seg.chars().collect();
            let mut blank = true;
            let mut i = 0;
            while i < chars.len() {
                if in_block {
                    if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                        in_block = false;
                        blank = false;
                        i += 2;
                        continue;
                    }
                    i += 1;
                } else if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    in_block = true;
                    blank = false;
                    i += 2;
                } else if chars[i] == '/' && chars.get(i + 1) == Some(&'/') {
                    blank = false;
                    break;
                } else {
                    if !chars[i].is_whitespace() {
                        blank = false;
                    }
                    i += 1;
                }
            }
            if blank && !in_block_at_start {
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

    /// Push `n` blank lines inside a block body at indent level `level`. With
    /// `KEEP_INDENTS_ON_EMPTY_LINES` each preserved blank line carries the
    /// level's indent before the newline (IntelliJ keeps the indent on empty
    /// lines "as if they contained some code"); otherwise plain blank lines,
    /// byte-identical to the historical output. The indented blanks still
    /// count as blank lines on re-format (whitespace-only), so the emitted
    /// output stays idempotent.
    fn push_indented_blanks(&self, out: &mut String, n: usize, level: usize) {
        if self.style.keep_indents_on_empty_lines && n > 0 {
            let pad = self.ind(level);
            for _ in 0..n {
                out.push_str(&pad);
                out.push('\n');
            }
        } else {
            self.push_blanks(out, n);
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

    // ── comments ─────────────────────────────────────────────────────────────

    /// Render a `line_comment` / `block_comment` node as its complete output
    /// line(s). Every standalone-comment emit site routes through here so the
    /// comment layout options apply uniformly:
    ///
    /// 1. **Column.** A comment whose source text starts in column 1 stays at
    ///    column 1 when `KEEP_FIRST_COLUMN_COMMENT` is set; otherwise a line
    ///    comment goes to column 1 when `LINE_COMMENT_AT_FIRST_COLUMN` is set
    ///    and a block comment when `BLOCK_COMMENT_AT_FIRST_COLUMN` is set;
    ///    otherwise it is emitted at the contextual `indent`.
    /// 2. **Space after `//`.** With `LINE_COMMENT_ADD_SPACE_ON_REFORMAT` one
    ///    space follows the `//` of an ordinary line comment when absent;
    ///    `LINE_COMMENT_ADD_SPACE_IN_SUPPRESSION` does the same for
    ///    `//noinspection` suppression comments only (a space there would
    ///    break the suppression, so the two flags are independent).
    /// 3. **Wrap.** With `WRAP_COMMENTS` an over-margin single-line comment is
    ///    broken at word boundaries; continuation lines repeat the comment's
    ///    column prefix (`//` for line comments, aligned ` * ` text for block
    ///    comments).
    ///
    /// Comment text is preserved — only indentation, the optional space and
    /// line breaks change (R5); multi-line block comments keep their source
    /// interior verbatim (R4). Stray non-comment extras keep the historical
    /// indented echo so call sites can route any extra through here.
    fn comment(&self, node: Node<'s>, indent: usize) -> String {
        // Javadoc detection takes precedence over the generic block-comment
        // handling: a standalone `/** … */` under `ENABLE_JAVADOC_FORMATTING`
        // is laid out by the javadoc engine (at the passed indent, every line
        // self-prefixed), everything else keeps the column / space / wrap
        // behaviour below.
        if let Some(rendered) = self.javadoc(node, indent) {
            return rendered;
        }
        let text = self.txt(node);
        if !matches!(node.kind(), "line_comment" | "block_comment") {
            return format!("{}{}", self.ind(indent), text);
        }
        let is_line = node.kind() == "line_comment";

        // 1. Column placement: a first-column source comment is kept there by
        //    KEEP_FIRST_COLUMN_COMMENT, and the per-kind *_AT_FIRST_COLUMN
        //    toggle pins the comment to column 1; otherwise the contextual
        //    indent is used.
        let src_col0 = node.start_position().column == 0;
        let first_column = (src_col0 && self.style.keep_first_column_comment)
            || (is_line && self.style.line_comment_at_first_column)
            || (!is_line && self.style.block_comment_at_first_column);
        let ind = self.ind(indent);
        let pad = if first_column { "" } else { ind.as_str() };

        // 2. Optional space after `//`: ON_REFORMAT for ordinary line
        //    comments, IN_SUPPRESSION for `//noinspection` comments only.
        let mut body = text.to_string();
        if is_line {
            let suppression = body.starts_with("//noinspection");
            let add_space = if suppression {
                self.style.line_comment_add_space_in_suppression
            } else {
                self.style.line_comment_add_space_on_reformat
            };
            let rest = &body[2..];
            if add_space && !rest.is_empty() && !rest.starts_with(' ') {
                body = format!("// {}", rest);
            }
        }

        // 3. WRAP_COMMENTS: break an over-margin single-line comment at word
        //    boundaries. Multi-line comments (block comments spanning rows)
        //    keep their source layout verbatim (R4).
        if self.style.wrap_comments && !body.contains('\n') && !body.contains('\r') {
            let pad_col = self.col_after(0, pad);
            if !self.fits(pad_col, &body) {
                return if is_line {
                    self.wrap_line_comment(pad, &body)
                } else {
                    self.wrap_block_comment(pad, &body)
                };
            }
        }

        format!("{}{}", pad, body)
    }

    /// `WRAP_COMMENTS` for a single-line `//…` comment: words are moved onto
    /// continuation lines that repeat the comment's own marker form (with or
    /// without the space after `//`), each line within the right margin.
    fn wrap_line_comment(&self, pad: &str, body: &str) -> String {
        let words: Vec<&str> = body[2..].split_whitespace().collect();
        if words.is_empty() {
            return format!("{}{}", pad, body);
        }
        let marker = if body[2..].starts_with(' ') {
            "// "
        } else {
            "//"
        };
        let margin = self.style.right_margin as usize;
        let start_col = self.col_after(0, pad) + marker.chars().count();
        let mut lines: Vec<String> = Vec::new();
        let mut cur = String::new();
        let mut cur_col = start_col;
        for w in &words {
            let wc = w.chars().count();
            if !cur.is_empty() {
                if cur_col + 1 + wc > margin {
                    lines.push(cur.clone());
                    cur.clear();
                    cur_col = start_col;
                } else {
                    cur.push(' ');
                    cur_col += 1;
                }
            }
            cur.push_str(w);
            cur_col += wc;
        }
        lines.push(cur);
        lines
            .iter()
            .map(|l| format!("{}{}{}", pad, marker, l))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// `WRAP_COMMENTS` for a single-line `/* … */` comment: the content words
    /// are laid out with the first line opening `/* ` and continuation lines
    /// aligning under the `*` (` * `), closing with ` */` on the last line.
    fn wrap_block_comment(&self, pad: &str, body: &str) -> String {
        let words: Vec<&str> = body[2..body.len() - 2].split_whitespace().collect();
        if words.is_empty() {
            return format!("{}{}", pad, body);
        }
        let margin = self.style.right_margin as usize;
        let first_col = self.col_after(0, pad) + 3; // "/* "
        let cont_col = self.col_after(0, pad) + 3; // " * "
        let mut lines: Vec<String> = Vec::new();
        let mut cur = String::new();
        let mut cur_col = first_col;
        let last_idx = words.len() - 1;
        for (i, w) in words.iter().enumerate() {
            let last = i == last_idx;
            let wc = w.chars().count();
            // The closing ` */` rides on the line holding the last word.
            let closing = if last { 3 } else { 0 };
            if !cur.is_empty() {
                if cur_col + 1 + wc + closing > margin {
                    lines.push(cur.clone());
                    cur.clear();
                    cur_col = cont_col;
                } else {
                    cur.push(' ');
                    cur_col += 1;
                }
            }
            cur.push_str(w);
            cur_col += wc;
        }
        lines.push(cur);
        let n = lines.len();
        lines
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let prefix = if i == 0 { "/* " } else { " * " };
                let close = if i + 1 == n { " */" } else { "" };
                format!("{}{}{}{}", pad, prefix, l, close)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    // ── javadoc ────────────────────────────────────────────────────────────

    /// Javadoc layout engine (`ENABLE_JAVADOC_FORMATTING` + the `JD_*`
    /// options). Returns the fully rendered comment — every line prefixed
    /// with `ind(indent)` — when `node` is a standalone `/** … */` block
    /// comment whose text parses cleanly and the gate is on, and `None`
    /// otherwise (the caller keeps the verbatim echo, R4). Callers that
    /// normally add their own indent prefix route through [`Fmt::comment`]
    /// with `indented: false`, so the per-line indent is never doubled.
    ///
    /// The rewrite is whitespace/layout only (R5): prose and tag text are
    /// preserved and never reordered (only the option-driven empty/unknown
    /// drops apply), and the layout is a pure function of the parsed content,
    /// so formatting the output again reproduces it (R6).
    fn javadoc(&self, node: Node<'s>, indent: usize) -> Option<String> {
        if !self.style.enable_javadoc_formatting || node.kind() != "block_comment" {
            return None;
        }
        let text = self.txt(node);
        if !text.starts_with("/**") || text == "/**/" || !self.comment_alone_on_line(node) {
            return None;
        }
        let ind = self.ind(indent);

        // One-line javadoc: kept verbatim under `JD_DO_NOT_WRAP_ONE_LINE_COMMENTS`,
        // expanded to the multi-line form otherwise.
        if !text.contains('\n') {
            if self.style.jd_do_not_wrap_one_line_comments {
                return Some(format!("{}{}", ind, text));
            }
            let inner = text[3..text.len() - 2].trim();
            let doc = self.parse_javadoc_body(&[inner.to_string()])?;
            return Some(self.render_javadoc(&doc, &ind));
        }

        // Multi-line: the `*/` terminator must be alone on the final line.
        let tail = &text[text.rfind('\n').map_or(0, |i| i + 1)..];
        if tail.trim() != "*/" {
            return None;
        }
        let body = &text[3..text.len() - 2];
        let mut content: Vec<String> = Vec::new();
        for (i, raw) in body.split('\n').enumerate() {
            let raw = raw.strip_suffix('\r').unwrap_or(raw);
            if i == 0 {
                // The remainder of the `/**` line is a description line when
                // non-blank (no `*` prefix precedes it).
                let rest = raw.trim();
                if !rest.is_empty() {
                    content.push(rest.to_string());
                }
                continue;
            }
            let stripped = raw.trim_start();
            if stripped.is_empty() {
                content.push(String::new());
            } else if let Some(rest) = stripped.strip_prefix('*') {
                content.push(
                    rest.strip_prefix(' ')
                        .unwrap_or(rest)
                        .trim_end()
                        .to_string(),
                );
            } else if !self.style.jd_leading_asterisks_are_enabled {
                // `JD_LEADING_ASTERISKS_ARE_ENABLED` off: the per-line `*` is
                // optional (the rendered form carries none, R6).
                content.push(stripped.trim_end().to_string());
            } else {
                // A non-blank line without the `*` prefix: not cleanly
                // parseable — verbatim echo (R4).
                return None;
            }
        }
        let doc = self.parse_javadoc_body(&content)?;
        Some(self.render_javadoc(&doc, &ind))
    }

    /// Whether the comment node is the only content on its source line:
    /// whitespace from the line start to the comment and whitespace from the
    /// comment to the line end. Guards the javadoc pass against comments
    /// embedded in a code line, which a multi-line render would corrupt.
    fn comment_alone_on_line(&self, node: Node<'s>) -> bool {
        let src = std::str::from_utf8(self.src).unwrap_or("");
        let start = node.start_byte();
        let end = node.end_byte();
        let before = &src[..start];
        let before_ok = match before.rfind('\n') {
            Some(i) => before[i + 1..].chars().all(char::is_whitespace),
            None => before.chars().all(char::is_whitespace),
        };
        before_ok
            && src[end..]
                .split('\n')
                .next()
                .unwrap_or("")
                .chars()
                .all(char::is_whitespace)
    }

    /// Phases 1+2: split the `*`-stripped content lines (an empty string
    /// marks an empty line) into the description block and the ordered tag
    /// blocks. Returns `None` when a tag line is malformed (`@param` /
    /// `@throws` / `@exception` without a name, a bare `@`), which keeps the
    /// comment verbatim (R4).
    fn parse_javadoc_body(&self, content: &[String]) -> Option<JavadocDoc> {
        let mut description: Vec<String> = Vec::new();
        let mut tags: Vec<JavadocTag> = Vec::new();
        let mut in_tags = false;
        for line in content {
            let trimmed = line.trim_start();
            if trimmed.starts_with('@') {
                let (kind, name, rest) = self.parse_javadoc_tag(trimmed)?;
                in_tags = true;
                let mut tag = JavadocTag {
                    kind,
                    name,
                    desc: Vec::new(),
                };
                if !rest.is_empty() {
                    tag.desc.push(rest);
                }
                tags.push(tag);
            } else if in_tags {
                if trimmed.is_empty() {
                    // Blank lines inside the tag region are layout-only.
                    continue;
                }
                match tags.last_mut() {
                    Some(t) => t.desc.push(line.clone()),
                    None => description.push(line.clone()),
                }
            } else {
                description.push(line.clone());
            }
        }
        // A `<p>` standing alone is IntelliJ's paragraph break: treat it as
        // an empty line so the options shape it and the rendered form
        // re-parses to the same structure (R6).
        for d in &mut description {
            if d.trim() == "<p>" {
                *d = String::new();
            }
        }
        // Leading and trailing empty lines are dropped; interior ones are the
        // `JD_KEEP_EMPTY_LINES` / `JD_P_AT_EMPTY_LINES` concern.
        while description.first().is_some_and(|l| l.is_empty()) {
            description.remove(0);
        }
        while description.last().is_some_and(|l| l.is_empty()) {
            description.pop();
        }
        Some(JavadocDoc { description, tags })
    }

    /// Shape-check one `@tag …` line: `@param` / `@throws` / `@exception`
    /// require a name, `@return` is bare, any other `@tag` is free text.
    /// Returns the kind, the name and the remaining description text.
    fn parse_javadoc_tag(&self, line: &str) -> Option<(JavadocTagKind, String, String)> {
        let tag_end = line[1..]
            .find(char::is_whitespace)
            .map_or(line.len(), |i| i + 1);
        let tag = &line[1..tag_end];
        let rest = line[tag_end..].trim_start();
        let (name, d) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
        match tag {
            "param" if !name.is_empty() => Some((
                JavadocTagKind::Param,
                name.to_string(),
                d.trim_start().to_string(),
            )),
            "throws" if !name.is_empty() => Some((
                JavadocTagKind::Throws,
                name.to_string(),
                d.trim_start().to_string(),
            )),
            "exception" if !name.is_empty() => Some((
                JavadocTagKind::Exception,
                name.to_string(),
                d.trim_start().to_string(),
            )),
            "return" => Some((JavadocTagKind::Return, String::new(), rest.to_string())),
            _ if !tag.is_empty() => {
                Some((JavadocTagKind::Other, tag.to_string(), rest.to_string()))
            }
            _ => None,
        }
    }

    /// Phase 3: lay the parsed structure out per the `JD_*` options, every
    /// line prefixed with the caller's indent. A pure function of the parsed
    /// content, so the output re-parses to the same layout (R6).
    fn render_javadoc(&self, doc: &JavadocDoc, ind: &str) -> String {
        let s = self.style;
        let mut lines: Vec<String> = Vec::new();

        // Description: line breaks kept per `JD_PRESERVE_LINE_FEEDS` or merged
        // per paragraph; empty lines kept per `JD_KEEP_EMPTY_LINES` and
        // rendered as `<p>` per `JD_P_AT_EMPTY_LINES`.
        let blank = || {
            if s.jd_p_at_empty_lines {
                "<p>".to_string()
            } else {
                String::new()
            }
        };
        let mut desc: Vec<String> = Vec::new();
        if s.jd_preserve_line_feeds {
            for l in &doc.description {
                if l.is_empty() {
                    if s.jd_keep_empty_lines {
                        desc.push(blank());
                    }
                } else {
                    desc.push(l.clone());
                }
            }
        } else {
            let mut para = String::new();
            for l in &doc.description {
                if l.is_empty() {
                    // With `JD_KEEP_EMPTY_LINES` on, the empty line breaks the
                    // paragraph and renders as `<p>` / blank; with it off the
                    // empty line vanishes and the surrounding text re-flows
                    // together (so the output re-parses identically, R6).
                    if s.jd_keep_empty_lines {
                        if !para.is_empty() {
                            desc.push(std::mem::take(&mut para));
                        }
                        desc.push(blank());
                    }
                } else if para.is_empty() {
                    para.push_str(l.trim_start());
                } else {
                    para.push(' ');
                    para.push_str(l.trim_start());
                }
            }
            if !para.is_empty() {
                desc.push(para);
            }
        }
        lines.extend(desc);
        if !lines.is_empty() && !doc.tags.is_empty() && s.jd_add_blank_after_description {
            lines.push(String::new());
        }

        // Tags in source order (R5); empty / unknown tags dropped per the
        // `JD_KEEP_*` options. Alignment columns are computed over the kept
        // tags of one group so the layout stays content-only (R6).
        let kept: Vec<&JavadocTag> = doc
            .tags
            .iter()
            .filter(|t| match t.kind {
                JavadocTagKind::Other => s.jd_keep_invalid_tags,
                JavadocTagKind::Param => s.jd_keep_empty_parameter || !t.desc.is_empty(),
                JavadocTagKind::Return => s.jd_keep_empty_return || !t.desc.is_empty(),
                JavadocTagKind::Throws | JavadocTagKind::Exception => {
                    s.jd_keep_empty_exception || !t.desc.is_empty()
                }
            })
            .collect();
        let header = |t: &JavadocTag| -> String {
            match t.kind {
                JavadocTagKind::Param => format!("@param {}", t.name),
                JavadocTagKind::Return => "@return".to_string(),
                JavadocTagKind::Throws => format!("@throws {}", t.name),
                JavadocTagKind::Exception if s.jd_use_throws_not_exception => {
                    format!("@throws {}", t.name)
                }
                JavadocTagKind::Exception => format!("@exception {}", t.name),
                JavadocTagKind::Other => format!("@{}", t.name),
            }
        };
        let param_col = if s.jd_align_param_comments {
            kept.iter()
                .filter(|t| t.kind == JavadocTagKind::Param)
                .map(|t| header(t).len())
                .max()
                .map(|m| m + 1)
        } else {
            None
        };
        let exc_col = if s.jd_align_exception_comments {
            kept.iter()
                .filter(|t| matches!(t.kind, JavadocTagKind::Throws | JavadocTagKind::Exception))
                .map(|t| header(t).len())
                .max()
                .map(|m| m + 1)
        } else {
            None
        };

        for (i, t) in kept.iter().enumerate() {
            let h = header(t);
            let col = match t.kind {
                JavadocTagKind::Param => param_col,
                JavadocTagKind::Throws | JavadocTagKind::Exception => exc_col,
                _ => None,
            };
            if t.desc.is_empty() {
                lines.push(h);
            } else if t.kind == JavadocTagKind::Param && s.jd_param_description_on_new_line {
                // `JD_PARAM_DESCRIPTION_ON_NEW_LINE`: the description starts
                // on its own line, indented to the description column.
                lines.push(h.clone());
                let c = col.unwrap_or(h.len() + 1);
                for (j, d) in t.desc.iter().enumerate() {
                    if j == 0 || s.jd_indent_on_continuation {
                        lines.push(format!("{}{}", " ".repeat(c), d.trim_start()));
                    } else {
                        lines.push(d.clone());
                    }
                }
            } else {
                let first = match col {
                    Some(c) => format!(
                        "{}{} {}",
                        h,
                        " ".repeat(c.saturating_sub(h.len() + 1)),
                        t.desc[0].trim_start()
                    ),
                    None => format!("{} {}", h, t.desc[0].trim_start()),
                };
                lines.push(first);
                let c = col.unwrap_or(h.len() + 1);
                for (_, d) in t.desc.iter().enumerate().skip(1) {
                    if s.jd_indent_on_continuation {
                        // `JD_INDENT_ON_CONTINUATION`: continuation lines sit
                        // at the tag's description column.
                        lines.push(format!("{}{}", " ".repeat(c), d.trim_start()));
                    } else {
                        lines.push(d.clone());
                    }
                }
            }
            // `JD_ADD_BLANK_AFTER_PARM_COMMENTS` / `JD_ADD_BLANK_AFTER_RETURN`:
            // a blank line after the block's last tag, before the next one.
            let blank_after = match t.kind {
                JavadocTagKind::Param => s.jd_add_blank_after_parm_comments,
                JavadocTagKind::Return => s.jd_add_blank_after_return,
                _ => false,
            };
            if blank_after && kept.len() > i + 1 && kept[i + 1].kind != t.kind {
                lines.push(String::new());
            }
        }

        // Per-line prefix: the leading `*` per
        // `JD_LEADING_ASTERISKS_ARE_ENABLED`, the `/**` / `*/` delimiters
        // always; content rides on ` * ` lines, never on the `/**` line.
        let mut out = format!("{}/**", ind);
        for l in &lines {
            let body = l.trim_end();
            if s.jd_leading_asterisks_are_enabled {
                out.push('\n');
                out.push_str(ind);
                if body.is_empty() {
                    out.push_str(" *");
                } else {
                    out.push_str(" * ");
                    out.push_str(body);
                }
            } else {
                out.push('\n');
                if !body.is_empty() {
                    out.push_str(ind);
                    out.push(' ');
                    out.push_str(body);
                }
            }
        }
        out.push('\n');
        out.push_str(ind);
        out.push_str(" */");
        out
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
            out.push_str(&self.comment(*c, 0));
            out.push('\n');
        }

        // Byte offset of the end of the content emitted so far (None when the
        // file does not yet contain anything, so no leading gap is inserted).
        let mut prev_end: Option<usize> = header_comments.last().map(|c| c.end_byte());
        let has_pkg = pkg.is_some();
        // A module-import file with no regular imports still has an import
        // section (the preserved module lines are emitted from it).
        let has_imports = !imports.is_empty() || !self.module_imports.is_empty();

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
            // The section's first byte in the source: the topmost import
            // content — the first real import node, or a preserved module line
            // when one sits above the imports (module lines above the section
            // otherwise leak their surrounding blank lines into the gap).
            let section_start = imports
                .first()
                .map(|n| n.start_byte())
                .into_iter()
                .chain(self.module_imports.first().map(|m| m.start))
                .min();
            if let Some(start) = section_start {
                self.insert_gap(
                    &mut out,
                    prev_end,
                    start,
                    s.keep_blank_lines_in_declarations,
                    if has_pkg {
                        s.blank_lines_after_package
                            .max(s.blank_lines_before_imports)
                    } else {
                        s.blank_lines_before_imports
                    },
                );
            }
            // The file's own package name (from `package …;`), used by
            // `LAYOUT_ON_DEMAND_IMPORT_FROM_SAME_PACKAGE_FIRST`.
            let own_package = pkg.map(|p| self.package_name(p));
            // The end of the emitted import content in the source: the last
            // real import, or the preserved module region when none. Computed
            // before `imports()` consumes the node list.
            let section_end = if imports.is_empty() {
                self.module_imports.last().map(|m| m.end)
            } else {
                Some(imports[imports.len() - 1].end_byte())
            };
            out.push_str(&self.imports(imports, &local_types, own_package.as_deref()));
            prev_end = section_end;
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
            // A javadoc block between the header and a top-level type is laid
            // out by the javadoc engine when the gate is on; every other node
            // (and any comment with the gate off) keeps the verbatim echo.
            if let Some(j) = self.javadoc(*ty, 0) {
                out.push_str(&j);
            } else {
                out.push_str(&self.type_decl(*ty, 0));
            }
            out.push('\n');
            prev_end = Some(ty.end_byte());
        }

        out
    }

    fn package_decl(&self, node: Node<'s>) -> String {
        format!("package {};\n", self.package_name(node))
    }

    /// The package name of a `package …;` declaration (e.g. `com.example`).
    fn package_name(&self, node: Node<'s>) -> String {
        // Try field "name" first, fall back to scanning named children.
        self.fld(node, "name")
            .map(|n| self.txt(n).to_string())
            .unwrap_or_else(|| {
                self.named(node)
                    .into_iter()
                    .find(|n| matches!(n.kind(), "scoped_identifier" | "identifier"))
                    .map(|n| self.txt(n).to_string())
                    .unwrap_or_default()
            })
    }

    // ── imports ───────────────────────────────────────────────────────────────

    /// Format the import section. The file's `import_declaration` nodes are
    /// first merged into on-demand imports (see [`Self::merge_on_demand_imports`]),
    /// then every emitted line is ordered and grouped per
    /// [`JavaStyle::import_layout`] (see [`Self::layout_imports`]).
    fn imports(
        &self,
        nodes: Vec<Node<'s>>,
        local_types: &[String],
        own_package: Option<&str>,
    ) -> String {
        let merged = self.merge_on_demand_imports(&nodes, local_types);
        self.layout_imports(&nodes, &merged, own_package)
    }

    /// Lay the merged import lines out per the import-layout table
    /// ([`JavaStyle::import_layout`], java.md "Import-table format").
    ///
    /// Each emitted line is classified (static / package / on-demand) from its
    /// text and matched to the table entry that owns its group: among named
    /// prefix matches the longest name wins (`withSubpackages` extends the
    /// match to subpackages), the empty-name entries are the catch-alls, and
    /// module lines own the reserved module slot (a table without it puts them
    /// at the head of the section). Groups emit in table order with one blank
    /// line per `<emptyLine/>` entry strictly between their table positions and
    /// no trailing blank; a group keeps its internal source order, optionally
    /// moving the file's own-package on-demand import to the front
    /// (`LAYOUT_ON_DEMAND_IMPORT_FROM_SAME_PACKAGE_FIRST`); source blank lines
    /// inside a group are preserved only under
    /// `KEEP_BLANK_LINES_BETWEEN_IMPORTS`.
    fn layout_imports(
        &self,
        nodes: &[Node<'s>],
        merged: &[(usize, String)],
        own_package: Option<&str>,
    ) -> String {
        let s = self.style;
        let entries = &s.import_layout;
        let separate = s.layout_static_imports_separately;

        // merged position -> the table-entry index owning its group (the
        // implicit trailing group `entries.len()` when nothing matched).
        let mut groups: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
        for (pos, (_, text)) in merged.iter().enumerate() {
            let (is_static, _, pkg) = classify_import_line(text);
            let key = layout_entry_for(entries, is_static, &pkg, separate)
                .map(|i| i as i64)
                .unwrap_or(entries.len() as i64);
            groups.entry(key).or_default().push(pos);
        }

        // The module slot: the table's reserved module entry when present,
        // otherwise a virtual slot before every table entry.
        let module_key: i64 = entries
            .iter()
            .position(|e| {
                matches!(
                    e,
                    ImportLayoutEntry::Package {
                        is_module: true,
                        ..
                    }
                )
            })
            .map(|i| i as i64)
            .unwrap_or(-1);
        let has_module = !self.module_imports.is_empty();
        if has_module {
            groups.entry(module_key).or_default();
        }

        // Source blank gap before each merged line, from the byte range between
        // its import node and the previous merged line's node. Indexed by
        // merged position; `emit_group` uses it only for lines adjacent in both
        // the merged list and the emitted order.
        let mut blank_before = vec![0usize; merged.len()];
        for i in 1..merged.len() {
            let (prev_idx, _) = merged[i - 1];
            let (idx, _) = merged[i];
            if prev_idx < idx {
                blank_before[i] =
                    self.blank_lines_between(nodes[prev_idx].end_byte(), nodes[idx].start_byte());
            }
        }

        let mut out = String::new();
        let mut prev_key: Option<i64> = None;
        for (&key, positions) in &groups {
            let is_module_slot = has_module && key == module_key;
            if positions.is_empty() && !is_module_slot {
                continue;
            }
            if let Some(pk) = prev_key {
                // Blanks between the two emitted groups: one per `<emptyLine/>`
                // entry strictly between their table positions.
                let gap = entries
                    .iter()
                    .enumerate()
                    .filter(|&(i, e)| {
                        let i = i as i64;
                        i > pk && i < key && matches!(e, ImportLayoutEntry::EmptyLine)
                    })
                    .count();
                self.push_blanks(&mut out, gap);
            }
            if is_module_slot {
                for m in &self.module_imports {
                    out.push_str(&m.line);
                    out.push('\n');
                }
            }
            self.emit_group(
                &mut out,
                merged,
                &blank_before,
                positions,
                own_package,
                s.keep_blank_lines_between_imports,
                s.layout_on_demand_import_from_same_package_first,
            );
            prev_key = Some(key);
        }
        out
    }

    /// Append one import group's lines in `positions` (merged order). When
    /// `same_pkg_first` is set and the file has an own package, the group's
    /// own-package on-demand lines move to the front, preserving relative
    /// order. With `keep_blanks`, a source blank gap is emitted before a line
    /// when the previous emitted line is its direct merged-list predecessor
    /// (the gap travelled with the later line).
    #[allow(clippy::too_many_arguments)]
    fn emit_group(
        &self,
        out: &mut String,
        merged: &[(usize, String)],
        blank_before: &[usize],
        positions: &[usize],
        own_package: Option<&str>,
        keep_blanks: bool,
        same_pkg_first: bool,
    ) {
        let own = own_package.unwrap_or("");
        let is_own_on_demand = |p: usize| -> bool {
            let (_, on_demand, pkg) = classify_import_line(&merged[p].1);
            on_demand && pkg == own
        };
        let mut order: Vec<usize> = Vec::with_capacity(positions.len());
        if same_pkg_first && !own.is_empty() {
            let front: Vec<usize> = positions
                .iter()
                .copied()
                .filter(|&p| is_own_on_demand(p))
                .collect();
            order.extend(front.iter().copied());
            order.extend(positions.iter().copied().filter(|p| !front.contains(p)));
        } else {
            order.extend(positions.iter().copied());
        }
        let mut prev_emitted: Option<usize> = None;
        for p in order {
            let gap = if keep_blanks && prev_emitted.is_some_and(|pp| p == pp + 1) {
                blank_before[p]
            } else {
                0
            };
            if gap > 0 {
                self.push_blanks(out, gap);
            }
            out.push_str(&merged[p].1);
            out.push('\n');
            prev_emitted = Some(p);
        }
    }

    /// Collapses single-type imports of one package into one on-demand import
    /// (`import pkg.*;`) and static member imports of one owner into
    /// `import static pkg.Owner.*;` when the merge rules apply.
    ///
    /// A non-static package group collapses when the group size exceeds
    /// [`class_count_to_use_import_on_demand`](JavaStyle::class_count_to_use_import_on_demand),
    /// or when the package is listed in
    /// [`packages_to_use_import_on_demand`](JavaStyle::packages_to_use_import_on_demand)
    /// (any count, including a single import), or when
    /// [`use_single_class_imports`](JavaStyle::use_single_class_imports) is off
    /// (any non-empty group). A static member group collapses when its size
    /// exceeds [`names_count_to_use_import_on_demand`](JavaStyle::names_count_to_use_import_on_demand).
    ///
    /// Merging is deliberately conservative: it is skipped when the file
    /// already uses a wildcard import, when a simple name would become
    /// ambiguous (the same name imported from another package / owner) or when
    /// it collides with a top-level type declared in the same file. Each
    /// collapsed group is emitted as one wildcard line at its first import's
    /// position.
    /// Returns each emitted line paired with the index of the import node it
    /// came from (a collapsed wildcard keeps its first import's index) so the
    /// layout pass can recover the source blank gaps.
    fn merge_on_demand_imports(
        &self,
        nodes: &[Node<'s>],
        local_types: &[String],
    ) -> Vec<(usize, String)> {
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
                .enumerate()
                .map(|(i, n)| (i, self.txt(*n).trim().to_string()))
                .collect();
        }

        // simple name -> set of packages / owners that import it. For a static
        // member import `pkg.Owner.m`, `pkg` holds the owner (`pkg.Owner`);
        // each import kind is tracked separately (member and type names live
        // in different namespaces, so a name clash only matters within one
        // kind).
        let mut name_pkgs: HashMap<&str, HashSet<&str>> = HashMap::new();
        let mut groups: HashMap<&str, Vec<usize>> = HashMap::new();
        let mut name_owners: HashMap<&str, HashSet<&str>> = HashMap::new();
        let mut static_groups: HashMap<&str, Vec<usize>> = HashMap::new();
        for (i, e) in entries.iter().enumerate() {
            if e.is_wildcard || e.pkg.is_empty() {
                continue;
            }
            if e.is_static {
                name_owners
                    .entry(e.simple.as_str())
                    .or_default()
                    .insert(e.pkg.as_str());
                static_groups.entry(e.pkg.as_str()).or_default().push(i);
            } else {
                name_pkgs
                    .entry(e.simple.as_str())
                    .or_default()
                    .insert(e.pkg.as_str());
                groups.entry(e.pkg.as_str()).or_default().push(i);
            }
        }

        let class_count = self.style.class_count_to_use_import_on_demand as usize;
        let names_count = self.style.names_count_to_use_import_on_demand as usize;
        let local: HashSet<&str> = local_types.iter().map(|s| s.as_str()).collect();

        // A group collapses only when every member's simple name is imported
        // from exactly this package / owner (dropping a single import could
        // otherwise hand name precedence to a remaining same-name single
        // import from elsewhere) and does not collide with a local top-level
        // type.
        let safe = |e: &Entry, names: &HashMap<&str, HashSet<&str>>| {
            !local.contains(e.simple.as_str())
                && names
                    .get(e.simple.as_str())
                    .is_some_and(|owners| owners.len() == 1 && owners.contains(e.pkg.as_str()))
        };

        // Decide which packages are replaced by a single on-demand import.
        let mut collapse: HashSet<&str> = HashSet::new();
        for (&pkg, idxs) in &groups {
            let listed = self
                .style
                .packages_to_use_import_on_demand
                .iter()
                .any(|p| p == pkg);
            if !(idxs.len() > class_count || listed || !self.style.use_single_class_imports) {
                continue;
            }
            if idxs.iter().all(|&i| safe(&entries[i], &name_pkgs)) {
                collapse.insert(pkg);
            }
        }

        // Static member groups collapse per owner above the names count.
        let mut static_collapse: HashSet<&str> = HashSet::new();
        for (&owner, idxs) in &static_groups {
            if idxs.len() <= names_count {
                continue;
            }
            if idxs.iter().all(|&i| safe(&entries[i], &name_owners)) {
                static_collapse.insert(owner);
            }
        }

        let mut out: Vec<(usize, String)> = Vec::with_capacity(nodes.len());
        for (i, n) in nodes.iter().enumerate() {
            let e = &entries[i];
            let replaced = if e.is_static {
                static_collapse.contains(e.pkg.as_str())
            } else {
                collapse.contains(e.pkg.as_str())
            };
            if replaced {
                // Emit the on-demand import once, at the first import's position.
                let first = if e.is_static {
                    static_groups[&e.pkg.as_str()][0]
                } else {
                    groups[&e.pkg.as_str()][0]
                };
                if first == i {
                    if e.is_static {
                        out.push((i, format!("import static {}.*;", e.pkg)));
                    } else {
                        out.push((i, format!("import {}.*;", e.pkg)));
                    }
                }
            } else {
                out.push((i, self.txt(*n).trim().to_string()));
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
        let c = self.col_after(0, &self.ind(indent));
        let mods = self.get_mods(node);
        let per_line = mods
            .map(|m| self.mods_per_line(m, indent))
            .unwrap_or_default();
        let inline = mods.and_then(|m| self.mods_inline(m, indent));

        // The post-modifier header tail (class keyword, name, type
        // parameters, extends / implements) is built per modifier form; the
        // inline form's first line is measured for codes 1/5.
        let tail_with = |prefix: &str, has_mods: bool| -> String {
            let mut header = String::new();
            header.push_str(prefix);
            if has_mods {
                self.mods_tail(&mut header, indent);
            }
            header.push_str("class ");
            header.push_str(self.fld(node, "name").map(|n| self.txt(n)).unwrap_or(""));

            if let Some(tp) = self.fld(node, "type_parameters") {
                // The name→`<` gap follows either the shipped
                // `SPACE_BEFORE_TYPE_PARAMETER_LIST` or
                // `SPACE_BEFORE_OPENING_ANGLE_BRACKET_IN_TYPE_PARAMETER`;
                // both default off, either on inserts the single space.
                if self.style.space_before_type_parameter_list
                    || self
                        .style
                        .space_before_opening_angle_bracket_in_type_parameter
                {
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
                self.append_type_clause(&mut header, "implements", ifaces, indent, c);
            }
            header
        };

        let use_inline = match mods {
            Some(m) => self.mods_inline_decision(
                m,
                indent,
                self.style.class_annotation_wrap,
                &self.inline_tail_first(&tail_with, &inline, c, indent, true),
                c,
            ),
            None => false,
        };

        let header = if use_inline {
            tail_with(inline.as_deref().unwrap_or(""), true)
        } else {
            tail_with(&per_line, mods.is_some())
        };

        // `KEEP_SIMPLE_CLASSES_IN_ONE_LINE`: a body whose members are all
        // simple collapses to one line (see `simple_class_one_line`);
        // otherwise the multi-line `class_body` layout is used.
        let body = match self.fld(node, "body") {
            Some(body_node) => {
                if let Some(one) = self.simple_class_one_line(&header, body_node, indent, c) {
                    return one;
                }
                self.class_body(body_node, indent, BodyKind::Class)
            }
            None => String::new(),
        };

        self.with_brace(header, body, indent, self.style.class_brace_style)
    }

    fn iface_decl(&self, node: Node<'s>, indent: usize) -> String {
        let c = self.col_after(0, &self.ind(indent));
        let mods = self.get_mods(node);
        let per_line = mods
            .map(|m| self.mods_per_line(m, indent))
            .unwrap_or_default();
        let inline = mods.and_then(|m| self.mods_inline(m, indent));

        let tail_with = |prefix: &str, has_mods: bool| -> String {
            let mut header = String::new();
            header.push_str(prefix);
            if has_mods {
                self.mods_tail(&mut header, indent);
            }
            header.push_str("interface ");
            header.push_str(self.fld(node, "name").map(|n| self.txt(n)).unwrap_or(""));

            if let Some(tp) = self.fld(node, "type_parameters") {
                if self.style.space_before_type_parameter_list
                    || self
                        .style
                        .space_before_opening_angle_bracket_in_type_parameter
                {
                    header.push(' ');
                }
                header.push_str(&self.flat_type_params(tp));
            }
            if let Some(ext) = self
                .all_ch(node)
                .into_iter()
                .find(|c| c.kind() == "extends_interfaces")
            {
                self.append_type_clause(&mut header, "extends", ext, indent, c);
            }
            header
        };

        let use_inline = match mods {
            Some(m) => self.mods_inline_decision(
                m,
                indent,
                self.style.class_annotation_wrap,
                &self.inline_tail_first(&tail_with, &inline, c, indent, true),
                c,
            ),
            None => false,
        };

        let header = if use_inline {
            tail_with(inline.as_deref().unwrap_or(""), true)
        } else {
            tail_with(&per_line, mods.is_some())
        };

        // `KEEP_SIMPLE_CLASSES_IN_ONE_LINE`: same collapse as `class_decl`.
        let body = match self.fld(node, "body") {
            Some(body_node) => {
                if let Some(one) = self.simple_class_one_line(&header, body_node, indent, c) {
                    return one;
                }
                self.class_body(body_node, indent, BodyKind::Interface)
            }
            None => String::new(),
        };

        self.with_brace(header, body, indent, self.style.class_brace_style)
    }

    fn enum_decl(&self, node: Node<'s>, indent: usize) -> String {
        let c = self.col_after(0, &self.ind(indent));
        let mods = self.get_mods(node);
        let per_line = mods
            .map(|m| self.mods_per_line(m, indent))
            .unwrap_or_default();
        let inline = mods.and_then(|m| self.mods_inline(m, indent));

        let tail_with = |prefix: &str, has_mods: bool| -> String {
            let mut header = String::new();
            header.push_str(prefix);
            if has_mods {
                self.mods_tail(&mut header, indent);
            }
            header.push_str("enum ");
            header.push_str(self.fld(node, "name").map(|n| self.txt(n)).unwrap_or(""));

            if let Some(ifaces) = self.fld(node, "interfaces") {
                self.append_type_clause(&mut header, "implements", ifaces, indent, c);
            }
            header
        };

        let use_inline = match mods {
            Some(m) => self.mods_inline_decision(
                m,
                indent,
                self.style.class_annotation_wrap,
                &self.inline_tail_first(&tail_with, &inline, c, indent, true),
                c,
            ),
            None => false,
        };

        let header = if use_inline {
            tail_with(inline.as_deref().unwrap_or(""), true)
        } else {
            tail_with(&per_line, mods.is_some())
        };

        // Enum body: keep original text for enum constants; format methods
        if let Some(body) = self.fld(node, "body") {
            // `ENUM_CONSTANTS_WRAP` / `SPACE_INSIDE_ONE_LINE_ENUM_BRACES`:
            // a constant-only body (no `;` declarations section) whose
            // constants each render on one line collapses to the flat
            // `{A, B}` body — always under `DoNotWrap` (the absent-option
            // default, matching the codebase's do-not-wrap convention),
            // one constant per line under `WrapAlways`, and flat iff the
            // composed declaration fits the margin under `WrapIfLong` /
            // `ChopDownIfLong` (5 == 1 here — the constants are echoed
            // verbatim, so there is no in-constant chopping to do). The
            // spacing option pads the flat body only (`{ A, B }`).
            let flat = self.enum_one_line_body(body, indent);
            let use_flat = match (&flat, self.style.enum_constants_wrap) {
                (Some(_), WrapStyle::DoNotWrap) => true,
                (Some(_), WrapStyle::WrapAlways) => false,
                (Some(fb), _) => {
                    let gap = self.sp(self.style.space_before_class_lbrace);
                    self.fits(c, &format!("{}{}{}", header, gap, fb))
                }
                (None, _) => false,
            };
            if use_flat {
                self.with_brace(header, flat.unwrap(), indent, self.style.class_brace_style)
            } else {
                let body_str = self.enum_body(node, body, indent);
                self.with_brace(header, body_str, indent, self.style.class_brace_style)
            }
        } else {
            header
        }
    }

    /// The flat one-line `{A, B}` body of an enum whose body holds only
    /// constants — no `enum_body_declarations` (`;` and any members after it
    /// keep the expanded `enum_body` layout) — and where every constant
    /// renders on a single line. Each constant goes through the shared
    /// `enum_constant` renderer (unannotated constants echo their source
    /// text, R4/R5), so a constant that spans source lines or whose
    /// `ENUM_FIELD_ANNOTATION_WRAP` placement puts its annotations on their
    /// own lines keeps the expanded layout. `None` leaves the caller's
    /// expanded layout in place. The constants are joined with `", "`;
    /// `SPACE_INSIDE_ONE_LINE_ENUM_BRACES` pads the braces (`{ A, B }`).
    fn enum_one_line_body(&self, body: Node<'s>, indent: usize) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();
        for child in self.named(body) {
            match child.kind() {
                "enum_constant" => parts.push(self.enum_constant(child, indent)),
                "enum_body_declarations" => return None,
                _ => {}
            }
        }
        if parts.is_empty() || parts.iter().any(|p| p.contains('\n')) {
            return None;
        }
        let inner = parts.join(", ");
        if self.style.space_inside_one_line_enum_braces {
            Some(format!("{{ {} }}", inner))
        } else {
            Some(format!("{{{}}}", inner))
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
                    out.push_str(&self.enum_constant(child, inner));
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
                            out.push_str(&self.comment(member, inner));
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

    /// Render one enum constant at `indent` (the enum body's inner level).
    /// Constants without annotations keep their original source echo (R4/R5);
    /// an annotated constant renders its annotation prefix per
    /// `ENUM_FIELD_ANNOTATION_WRAP` (default `DoNotWrap` → inline `@A A`) and
    /// echoes the rest of the constant — name, `(arguments)` and any constant
    /// class `body` — verbatim from the source bytes.
    fn enum_constant(&self, node: Node<'s>, indent: usize) -> String {
        let mods = match self.get_mods(node) {
            Some(m) => m,
            None => return self.txt(node).to_string(),
        };
        let (anns, _) = self.mods_parts(mods);
        if anns.is_empty() {
            return self.txt(node).to_string();
        }

        // The constant's remainder after the annotation prefix, verbatim.
        let rest = {
            let text = self.txt(node);
            let off = mods.end_byte() - node.start_byte();
            text.get(off..).unwrap_or("").trim_start()
        };

        let c = self.col_after(0, &self.ind(indent));
        let inline_str: Vec<String> = anns.iter().map(|&a| self.flat_annotation(a)).collect();
        let inline = inline_str.join(" ");
        let tail_first = rest.split('\n').next().unwrap_or("");
        let expressible = anns
            .iter()
            .all(|&a| !self.annotation(a, indent).contains('\n'));
        let use_inline = expressible
            && match self.style.enum_field_annotation_wrap {
                WrapStyle::DoNotWrap => true,
                WrapStyle::WrapAlways => false,
                _ => self.fits(c, &format!("{} {}", inline, tail_first)),
            };
        if use_inline {
            format!("{} {}", inline, rest)
        } else {
            let lines: Vec<String> = anns.iter().map(|&a| self.annotation(a, indent)).collect();
            let gap = format!("\n{}", self.ind(indent));
            format!("{}{}{}", lines.join(&gap), gap, rest)
        }
    }

    fn record_decl(&self, node: Node<'s>, indent: usize) -> String {
        let c = self.col_after(0, &self.ind(indent));
        let mods = self.get_mods(node);
        let per_line = mods
            .map(|m| self.mods_per_line(m, indent))
            .unwrap_or_default();
        let inline = mods.and_then(|m| self.mods_inline(m, indent));

        let tail_with = |prefix: &str, has_mods: bool| -> String {
            let mut header = String::new();
            header.push_str(prefix);
            if has_mods {
                self.mods_tail(&mut header, indent);
            }
            header.push_str("record ");
            header.push_str(self.fld(node, "name").map(|n| self.txt(n)).unwrap_or(""));

            if let Some(tp) = self.fld(node, "type_parameters") {
                if self.style.space_before_type_parameter_list
                    || self
                        .style
                        .space_before_opening_angle_bracket_in_type_parameter
                {
                    header.push(' ');
                }
                header.push_str(&self.flat_type_params(tp));
            }

            if let Some(params) = self.fld(node, "parameters") {
                header.push_str(&self.record_components(params, indent, c, &header));
            }

            if let Some(ifaces) = self.fld(node, "interfaces") {
                self.append_type_clause(&mut header, "implements", ifaces, indent, c);
            }
            header
        };

        let use_inline = match mods {
            Some(m) => self.mods_inline_decision(
                m,
                indent,
                self.style.class_annotation_wrap,
                &self.inline_tail_first(&tail_with, &inline, c, indent, true),
                c,
            ),
            None => false,
        };

        let header = if use_inline {
            tail_with(inline.as_deref().unwrap_or(""), true)
        } else {
            tail_with(&per_line, mods.is_some())
        };

        // `KEEP_SIMPLE_CLASSES_IN_ONE_LINE`: same collapse as `class_decl`
        // (the record body is formatted as a class body).
        let body = match self.fld(node, "body") {
            Some(body_node) => {
                if let Some(one) = self.simple_class_one_line(&header, body_node, indent, c) {
                    return one;
                }
                self.class_body(body_node, indent, BodyKind::Class)
            }
            None => String::new(),
        };

        self.with_brace(header, body, indent, self.style.class_brace_style)
    }

    /// Formats a record header's component list (`(…)`).
    ///
    /// Honors the record-header options `RECORD_COMPONENTS_WRAP` (wrapped
    /// components one per line), `NEW_LINE_AFTER_LPAREN_IN_RECORD_HEADER` (a
    /// wrapped header's components start on the line below the `(`),
    /// `RPAREN_ON_NEW_LINE_IN_RECORD_HEADER` (a wrapped header's `)` on its
    /// own line at the record indent), `ALIGN_MULTILINE_RECORDS` (component
    /// lines pad under the first component), `SPACE_WITHIN_RECORD_HEADER` (one
    /// space just inside a `(` / `)` that shares its line with a component),
    /// `ANNOTATION_NEW_LINE_IN_RECORD_COMPONENT` (an own-line component's
    /// annotations each on their own line above the declaration core) and
    /// `BLANK_LINES_BETWEEN_RECORD_COMPONENTS` (bare blank lines between the
    /// components of a wrapped header). A header that fits the margin — or a
    /// lparen-attached single-component header — keeps its flat single-line
    /// rendering; with `new_line_after_lparen_in_record_header` the opening
    /// paren stays on the header line and components start on the next line,
    /// otherwise the first component stays inline after the paren.
    fn record_components(&self, node: Node<'s>, indent: usize, c: usize, header: &str) -> String {
        let comps = self.named(node);
        if comps.is_empty() {
            return "()".to_string();
        }

        // `SPACE_WITHIN_RECORD_HEADER`: one space just inside each paren that
        // shares its line with a component (the flat form, the lparen-off
        // first line and the glued `)`); a paren alone on its own line gets
        // no pad.
        let pad = Self::sep(self.style.space_within_record_header);

        let parts: Vec<String> = comps.iter().map(|&p| self.flat_param(p)).collect();
        let flat = format!(
            "({}{}{})",
            pad,
            parts.join(self.comma_sep(self.style.space_after_comma)),
            pad
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

        // `BLANK_LINES_BETWEEN_RECORD_COMPONENTS`: between consecutive
        // wrapped components `n` bare blank lines — the inter-block `,\n`
        // becomes `,` followed by `n + 1` newlines. Inert on a header that
        // is not wrapped (the flat return above).
        let sep = format!(
            ",{}",
            "\n".repeat(self.style.blank_lines_between_record_components as usize + 1)
        );
        // Annotation-rendering level for an own-line component's annotations.
        let level = indent + 1;

        if self.style.new_line_after_lparen_in_record_header {
            // `(` stays alone on its line, every component starts its own
            // line below it and `)` closes alone at the record indent — the
            // closing shape the rparen option produces, whichever value the
            // option carries.
            let pref = if self.style.align_multiline_records {
                self.align_prefix(open_col + 1)
            } else {
                self.cont(indent)
            };
            let lines: Vec<String> = comps
                .iter()
                .map(|&p| {
                    format!(
                        "{}{}",
                        pref,
                        self.record_component_block(p, &pref, level, true)
                    )
                })
                .collect();
            format!("(\n{}\n{})", lines.join(&sep), self.ind(indent))
        } else {
            // The first component stays on the header line after `(` (its
            // inline column right of the paren plus the pad), the rest start
            // their own lines. A lone component cannot wrap and keeps the
            // flat form.
            if comps.len() == 1 {
                return flat;
            }
            let pref = if self.style.align_multiline_records {
                // The aligned column sits under the first inline component,
                // so it shifts by the pad when the pad is on.
                self.align_prefix(open_col + 1 + pad.len())
            } else {
                self.cont(indent)
            };
            let mut out = format!("({}{}", pad, self.flat_param(comps[0]));
            for &p in &comps[1..] {
                out.push_str(&sep);
                out.push_str(&pref);
                out.push_str(&self.record_component_block(p, &pref, level, true));
            }
            // `RPAREN_ON_NEW_LINE_IN_RECORD_HEADER`: `)` closes on its own
            // line at the record indent; otherwise it glues to the last
            // component's line (padded when the pad is on).
            if self.style.rparen_on_new_line_in_record_header {
                out.push('\n');
                out.push_str(&self.ind(indent));
                out.push(')');
            } else {
                out.push_str(pad);
                out.push(')');
            }
            out
        }
    }

    /// Render one record component of a wrapped header as whole lines at its
    /// own column: the caller prefixes the first line and `prefix` repeats on
    /// every line after it. Under `ANNOTATION_NEW_LINE_IN_RECORD_COMPONENT` an
    /// own-line `formal_parameter` with annotations renders one line per
    /// annotation (the existing annotation rendering, tokens verbatim)
    /// followed by the declaration core — keyword modifiers + type + name,
    /// `flat_param`'s assembly with the annotation text removed — at `level`;
    /// every other component (no annotations, a non-`formal_parameter` shape,
    /// or a component sharing the `(` line) keeps its inline `flat_param`.
    fn record_component_block(
        &self,
        node: Node<'s>,
        prefix: &str,
        level: usize,
        own_line: bool,
    ) -> String {
        if own_line
            && self.style.annotation_new_line_in_record_component
            && node.kind() == "formal_parameter"
        {
            if let Some(mods) = self.get_mods(node) {
                let (anns, keywords) = self.mods_parts(mods);
                if !anns.is_empty() {
                    let mut out = String::new();
                    for (i, a) in anns.iter().enumerate() {
                        if i > 0 {
                            out.push_str(&format!("\n{}", prefix));
                        }
                        out.push_str(&self.annotation(*a, level));
                    }
                    let kw = keywords.join(" ");
                    let ty = self
                        .fld(node, "type")
                        .map(|n| self.flat_type(n))
                        .unwrap_or_default();
                    let nm = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");
                    let core = if kw.is_empty() {
                        format!("{} {}", ty, nm)
                    } else {
                        format!("{} {} {}", kw, ty, nm)
                    };
                    out.push_str(&format!("\n{}{}", prefix, core));
                    return out;
                }
            }
        }
        self.flat_param(node)
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

    /// True when `node` (a class / interface / record body) belongs to a
    /// type declaration reached at the top level — the declaration sits
    /// directly under the `program` node, so `body → declaration → program`.
    /// Anonymous class bodies and nested type declarations (whose declaration
    /// sits under a class body) report `false`.
    fn is_top_level_class_body(&self, node: Node<'s>) -> bool {
        node.parent()
            .and_then(|decl| decl.parent())
            .map(|gp| gp.kind() == "program")
            .unwrap_or(false)
    }

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

        let inner = if self.style.do_not_indent_top_level_class_members
            && indent == 0
            && self.is_top_level_class_body(node)
        {
            // `DO_NOT_INDENT_TOP_LEVEL_CLASS_MEMBERS`: members of a class
            // reached at the top level (via `program` → `type_decl`) sit at
            // the class declaration indent instead of one level deeper.
            // Nested classes — even when the un-indented top-level member
            // layout puts them at level 0 — and anonymous classes keep the
            // normal `indent + 1`.
            0
        } else {
            indent + 1
        };
        let anchor = node.start_byte(); // the opening `{`
        let mut prev: Option<Node<'s>> = None;
        let mut last: Option<Node<'s>> = None;

        let mut lines: Vec<BodyLine> = Vec::with_capacity(members.len());
        for m in members {
            if self.is_comment_node(m) {
                // Comments are content but take no part in the spacing
                // options: they are emitted in place, without their own gap.
                // `comment` renders the full line(s) (column placement, the
                // optional space after `//`, wrapping), so the member indent
                // prefix is not added here. Comments break columnar runs.
                lines.push(BodyLine {
                    blanks: 0,
                    indented: false,
                    text: self.comment(m, inner),
                    align: None,
                });
                last = Some(m);
                continue;
            }

            let gap = self.member_gap(prev, m, anchor, kind);
            prev = Some(m);
            last = Some(m);
            let text = self.class_member(m, inner);
            let align = self.member_align_elem(m, &text);
            lines.push(BodyLine {
                blanks: gap,
                indented: true,
                text,
                align,
            });
        }

        // Columnar alignment over output-adjacent members
        // (`ALIGN_GROUP_FIELD_DECLARATIONS`, `ALIGN_SUBSEQUENT_SIMPLE_METHODS`).
        pad_column_runs(&mut lines);

        let mut out = String::from("{\n");
        for l in lines {
            self.push_blanks(&mut out, l.blanks);
            if l.indented {
                out.push_str(&self.ind(inner));
            }
            out.push_str(&l.text);
            out.push('\n');
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

    /// Columnar-alignment element of a class-body member for
    /// `ALIGN_GROUP_FIELD_DECLARATIONS` / `ALIGN_SUBSEQUENT_SIMPLE_METHODS`:
    /// the column of the declaration / method name when the member is a
    /// single-line single-declarator field or a single-line method, else
    /// `None`. Padding is inserted right before the name, so the names of a
    /// run share one column. Multi-line members (wrapped initialisers,
    /// annotations on their own lines, block bodies) and multi-declarator
    /// fields are not aligned and break runs.
    fn member_align_elem(&self, m: Node<'s>, text: &str) -> Option<(u8, usize, usize)> {
        if text.contains('\n') {
            return None;
        }
        let decl_col = |s: &Self, node: Node<'s>, text: &str| -> Option<(u8, usize, usize)> {
            let col = s.decl_name_col(node, text)?;
            Some((0, col, col))
        };
        match m.kind() {
            "field_declaration"
                if self.style.align_group_field_declarations && self.single_declarator(m) =>
            {
                decl_col(self, m, text)
            }
            "method_declaration" if self.style.align_subsequent_simple_methods => {
                let col = self.method_name_col(m, text)?;
                Some((2, col, col))
            }
            _ => None,
        }
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
            "line_comment" | "block_comment" => self.comment(node, indent),
            _ => self.txt(node).to_string(),
        }
    }

    /// `KEEP_SIMPLE_CLASSES_IN_ONE_LINE` collapse for class / interface /
    /// record bodies: when the option is on, the class brace style keeps the
    /// `{` on the header line (`EndOfLine` / `NextLineIfWrapped`), the
    /// rendered `header` is single-line, and every member renders without a
    /// newline (comments / extras reject — R4 — as do members whose own
    /// layout is multi-line: a block body, a wrapped field…), the whole
    /// one-line declaration `class A { … }` is returned when it fits the
    /// margin. Members are rendered at the declaration's own `indent` and
    /// collapse recursively (a simple method member needs
    /// `KEEP_SIMPLE_METHODS_IN_ONE_LINE` to render one line itself, a nested
    /// simple class `KEEP_SIMPLE_CLASSES_IN_ONE_LINE`). Enums and anonymous
    /// classes are out of scope (separate change requests). `None` leaves
    /// the caller's multi-line layout in place.
    fn simple_class_one_line(
        &self,
        header: &str,
        body: Node<'s>,
        indent: usize,
        c: usize,
    ) -> Option<String> {
        if !self.style.keep_simple_classes_in_one_line
            || !matches!(
                self.style.class_brace_style,
                BraceStyle::EndOfLine | BraceStyle::NextLineIfWrapped
            )
            || header.contains('\n')
        {
            return None;
        }
        let members = self.named(body);
        if members.is_empty() {
            return None;
        }
        let mut rendered: Vec<String> = Vec::with_capacity(members.len());
        for m in members {
            if self.is_comment_node(m) {
                return None;
            }
            let text = self.class_member(m, indent);
            if text.contains('\n') {
                return None;
            }
            rendered.push(text);
        }
        let inner = rendered.join(" ");
        let block = self.present_block(&inner, indent);
        let candidate = format!(
            "{}{}{}",
            header,
            self.body_gap(&block, self.style.space_before_class_lbrace),
            block
        );
        if self.fits_lines(c, &candidate) {
            Some(candidate)
        } else {
            None
        }
    }

    // ── method / constructor / field ──────────────────────────────────────────

    fn method_decl(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let mods = self.get_mods(node);
        let per_line = mods
            .map(|m| self.mods_per_line(m, indent))
            .unwrap_or_default();
        let inline = mods.and_then(|m| self.mods_inline(m, indent));

        // The full declaration is built per modifier form (its column
        // arithmetic — parameter paren column, throws clause column, one-line
        // body candidate — depends on the modifier prefix); the inline form's
        // first line is measured for codes 1/5.
        let tail_with = |prefix: &str, has_mods: bool| -> String {
            let mut out = String::new();
            out.push_str(prefix);
            if has_mods {
                self.mods_tail(&mut out, indent);
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
                let excs: Vec<String> = self
                    .named(throws)
                    .iter()
                    .map(|n| self.flat_type(*n))
                    .collect();
                let cur = self.col_after(c, &out);
                out.push_str(&self.clause_list(
                    "throws",
                    &excs,
                    self.style.throws_keyword_wrap,
                    self.style.throws_list_wrap,
                    indent,
                    cur,
                    self.style.align_multiline_throws_list,
                    self.style.align_throws_keyword,
                ));
            }

            // body or semicolon
            match self.fld(node, "body") {
                Some(body) => self.method_body(body, indent, &mut out, c),
                None => out.push(';'),
            }

            out
        };

        let use_inline = match mods {
            Some(m) => self.mods_inline_decision(
                m,
                indent,
                self.style.method_annotation_wrap,
                &self.inline_tail_first(&tail_with, &inline, c, indent, true),
                c,
            ),
            None => false,
        };

        if use_inline {
            tail_with(inline.as_deref().unwrap_or(""), true)
        } else {
            tail_with(&per_line, mods.is_some())
        }
    }

    fn constructor_decl(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let mods = self.get_mods(node);
        let per_line = mods
            .map(|m| self.mods_per_line(m, indent))
            .unwrap_or_default();
        let inline = mods.and_then(|m| self.mods_inline(m, indent));

        let tail_with = |prefix: &str, has_mods: bool| -> String {
            let mut out = String::new();
            out.push_str(prefix);
            if has_mods {
                self.mods_tail(&mut out, indent);
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
                let excs: Vec<String> = self
                    .named(throws)
                    .iter()
                    .map(|n| self.flat_type(*n))
                    .collect();
                let cur = self.col_after(c, &out);
                out.push_str(&self.clause_list(
                    "throws",
                    &excs,
                    self.style.throws_keyword_wrap,
                    self.style.throws_list_wrap,
                    indent,
                    cur,
                    self.style.align_multiline_throws_list,
                    self.style.align_throws_keyword,
                ));
            }

            if let Some(body) = self.fld(node, "body") {
                self.method_body(body, indent, &mut out, c);
            }

            out
        };

        let use_inline = match mods {
            Some(m) => self.mods_inline_decision(
                m,
                indent,
                self.style.method_annotation_wrap,
                &self.inline_tail_first(&tail_with, &inline, c, indent, true),
                c,
            ),
            None => false,
        };

        if use_inline {
            tail_with(inline.as_deref().unwrap_or(""), true)
        } else {
            tail_with(&per_line, mods.is_some())
        }
    }

    /// Compact constructor of a record (`Foo { ... }`): no parameter list.
    fn compact_constructor_decl(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let mods = self.get_mods(node);
        let per_line = mods
            .map(|m| self.mods_per_line(m, indent))
            .unwrap_or_default();
        let inline = mods.and_then(|m| self.mods_inline(m, indent));

        let tail_with = |prefix: &str, has_mods: bool| -> String {
            let mut out = String::new();
            out.push_str(prefix);
            if has_mods {
                self.mods_tail(&mut out, indent);
            }
            out.push_str(self.fld(node, "name").map(|n| self.txt(n)).unwrap_or(""));
            if let Some(body) = self.fld(node, "body") {
                self.method_body(body, indent, &mut out, c);
            }
            out
        };

        let use_inline = match mods {
            Some(m) => self.mods_inline_decision(
                m,
                indent,
                self.style.method_annotation_wrap,
                &self.inline_tail_first(&tail_with, &inline, c, indent, true),
                c,
            ),
            None => false,
        };

        if use_inline {
            tail_with(inline.as_deref().unwrap_or(""), true)
        } else {
            tail_with(&per_line, mods.is_some())
        }
    }

    /// Appends a method/constructor body block to `out`.
    ///
    /// When `KEEP_SIMPLE_METHODS_IN_ONE_LINE` is enabled (and the brace style
    /// keeps the brace on the same line), a body that is a single simple
    /// statement is rendered as a one-line `{s}` / `{ s }` block (per the two
    /// Java one-line-body toggles) when the resulting declaration fits within
    /// the right margin.
    fn method_body(&self, body: Node<'s>, indent: usize, out: &mut String, c: usize) {
        if self.style.keep_simple_methods_in_one_line
            && matches!(
                self.style.method_brace_style,
                BraceStyle::EndOfLine | BraceStyle::NextLineIfWrapped
            )
        {
            if let Some(one) = self.one_line_body(body, indent) {
                let gap = self.body_gap(&one, self.style.space_before_method_lbrace);
                let candidate = format!("{}{}{}", out, gap, one);
                // Column at which the declaration's (first) line starts.
                if self.fits_lines(c, &candidate) {
                    *out = candidate;
                    return;
                }
            }
        }
        let body_str = self.block(body, indent, c, self.style.blank_lines_before_method_body);
        out.push_str(&self.brace_before_body(indent, self.style.method_brace_style, &body_str));
    }

    fn field_decl(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let mods = self.get_mods(node);
        let per_line = mods
            .map(|m| self.mods_per_line(m, indent))
            .unwrap_or_default();
        let inline = mods.and_then(|m| self.mods_inline(m, indent));

        let tail_with = |prefix: &str, has_mods: bool| -> String {
            let mut out = String::new();
            out.push_str(prefix);
            if has_mods {
                self.mods_tail(&mut out, indent);
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
                        self.assign_expr(
                            val,
                            indent,
                            c,
                            &prefix,
                            "=",
                            self.keep_wrapped(node),
                            None
                        )
                    );
                }
            }

            let decl_strs: Vec<String> = decls
                .iter()
                .map(|&d| {
                    let name = self.fld(d, "name").map(|n| self.txt(n)).unwrap_or("");
                    if let Some(val) = self.fld(d, "value") {
                        let sep = self.op_sep("=");
                        let val_col = c
                            + self.col_after(0, &out)
                            + 1
                            + name.len()
                            + sep.len()
                            + 1
                            + sep.len();
                        let val_str = self.expr(val, indent, val_col);
                        Self::join_sep(&format!("{}{}=", name, sep), sep, &val_str)
                    } else {
                        name.to_string()
                    }
                })
                .collect();

            // `KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE` guard: a multi-declarator
            // field declaration is one statement — its declarator list is joined
            // on the line and never split per declarator. (The engine has no
            // per-declarator break layout, so the option is honoured by
            // construction; the read keeps the inline-join guarantee explicit
            // and load-bearing for a future declarator-level wrap.)
            let _guard = self.style.keep_multiple_expressions_in_one_line;

            out.push(' ');
            out.push_str(&decl_strs.join(self.comma_sep(self.style.space_after_comma)));
            out.push(';');
            out
        };

        let use_inline = match mods {
            Some(m) => self.mods_inline_decision(
                m,
                indent,
                self.style.field_annotation_wrap,
                &self.inline_tail_first(&tail_with, &inline, c, indent, true),
                c,
            ),
            None => false,
        };

        if use_inline {
            tail_with(inline.as_deref().unwrap_or(""), true)
        } else {
            tail_with(&per_line, mods.is_some())
        }
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

    /// Split a `modifiers` node into its annotations (in source order) and its
    /// keyword modifiers. All children participate: keyword modifiers
    /// (public, static, …) are UNNAMED nodes.
    fn mods_parts(&self, node: Node<'s>) -> (Vec<Node<'s>>, Vec<String>) {
        let mut anns: Vec<Node<'s>> = Vec::new();
        let mut keywords: Vec<String> = Vec::new();
        for ch in self.all_ch(node) {
            match ch.kind() {
                "annotation" | "marker_annotation" => anns.push(ch),
                _ => {
                    let t = self.txt(ch).trim().to_string();
                    if !t.is_empty() {
                        keywords.push(t);
                    }
                }
            }
        }
        (anns, keywords)
    }

    /// True when a `modifiers` node carries exactly one annotation — the case
    /// the `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION[*_IN_PARAMETER]` exemptions
    /// keep inline regardless of the placement wrap code.
    fn mods_single_annotation(&self, node: Node<'s>) -> bool {
        self.mods_parts(node).0.len() == 1
    }

    /// The inline modifier list — annotations (in their canonical flat form)
    /// and keyword modifiers joined with single spaces, `@A @B public`.
    /// `None` when an annotation's own argument list renders multi-line, which
    /// cannot be joined inline (R5): the declaration then falls back to the
    /// one-annotation-per-line layout.
    fn mods_inline(&self, node: Node<'s>, indent: usize) -> Option<String> {
        let (anns, keywords) = self.mods_parts(node);
        if anns.is_empty() && keywords.is_empty() {
            return Some(String::new());
        }
        let mut parts: Vec<String> = Vec::new();
        for a in &anns {
            if self.annotation(*a, indent).contains('\n') {
                return None;
            }
            parts.push(self.flat_annotation(*a));
        }
        parts.extend(keywords);
        Some(parts.join(" "))
    }

    /// The one-annotation-per-line form of a `modifiers` node: each
    /// annotation on its own line (already including the trailing
    /// newline+indent), followed by the keyword modifiers joined by spaces —
    /// exactly the historical `modifiers()` shape (the wrap-always layout).
    fn mods_per_line(&self, node: Node<'s>, indent: usize) -> String {
        let (anns, keywords) = self.mods_parts(node);
        let mut out = String::new();
        for ann in &anns {
            out.push_str(&self.annotation(*ann, indent));
            out.push('\n');
            out.push_str(&self.ind(indent));
        }
        out.push_str(&keywords.join(" "));
        out
    }

    /// The single-annotation exemption at the member / type / local-variable
    /// placement sites (`DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION`): force the
    /// inline form for a lone annotation regardless of the wrap code, but only
    /// when the inline form is expressible (R5).
    fn single_ann_exempts(&self, mods: Node<'s>, inline_expressible: bool) -> bool {
        self.style.do_not_wrap_after_single_annotation
            && inline_expressible
            && self.mods_single_annotation(mods)
    }

    /// Decide whether a member / type declaration's `modifiers` node renders
    /// inline (single-space-joined) or one annotation per line.
    ///
    /// `wrap` is the governing `*_ANNOTATION_WRAP` option; `tail_first` is the
    /// first line of the caller's post-modifier header tail, used to measure
    /// codes `1` / `5` (which keep the inline form only when the composed
    /// first line fits the margin — the two codes behave identically at this
    /// granularity); `c` is the column of the declaration's first line.
    fn mods_inline_decision(
        &self,
        mods: Node<'s>,
        indent: usize,
        wrap: WrapStyle,
        tail_first: &str,
        c: usize,
    ) -> bool {
        let inline = self.mods_inline(mods, indent);
        let expressible = inline.is_some();
        let mut use_inline = match wrap {
            WrapStyle::DoNotWrap => true,
            WrapStyle::WrapAlways => false,
            _ => match &inline {
                Some(s) => self.fits(c, &format!("{} {}", s, tail_first)),
                None => false,
            },
        };
        use_inline &= expressible;
        if self.single_ann_exempts(mods, expressible) {
            use_inline = true;
        }
        use_inline
    }

    /// The first line of the post-modifier header tail, measured under the
    /// inline hypothesis: the caller builds its header with the inline
    /// modifier prefix via `build`, and the tail's first line is what follows
    /// the `inline + " "` join. When no inline form is expressible this is
    /// unused (codes 1/5 fall back to one-per-line).
    fn inline_tail_first(
        &self,
        build: &dyn Fn(&str, bool) -> String,
        inline: &Option<String>,
        _c: usize,
        _indent: usize,
        _has_mods: bool,
    ) -> String {
        let Some(s) = inline else {
            return String::new();
        };
        let header = build(s, true);
        let first_line = header.split('\n').next().unwrap_or("");
        first_line
            .strip_prefix(&format!("{} ", s))
            .unwrap_or("")
            .to_string()
    }

    /// Append the gap between a declaration's modifier list and the next
    /// token: a single space normally, or — under `MODIFIER_LIST_WRAP` — a
    /// line break so the rest of the declaration starts at `ind(indent)`.
    /// When the modifiers already end with a newline or the annotation-list
    /// indent (an annotations-only list), the break is already in place and
    /// nothing is appended.
    fn mods_tail(&self, s: &mut String, indent: usize) {
        if s.ends_with('\n') || s.ends_with(' ') {
            return;
        }
        if self.style.modifier_list_wrap {
            s.push('\n');
            s.push_str(&self.ind(indent));
        } else {
            s.push(' ');
        }
    }

    /// Join `prefix` + `sep` + `value`, dropping `sep` when `value` starts
    /// with a newline (a construct that begins on its own line, e.g. an
    /// array initializer with `ARRAY_INITIALIZER_LBRACE_ON_NEXT_LINE`) so
    /// the join never leaves a trailing space behind (R5).
    fn join_sep(prefix: &str, sep: &str, value: &str) -> String {
        if value.starts_with('\n') {
            format!("{}{}", prefix, value)
        } else {
            format!("{}{}{}", prefix, sep, value)
        }
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
                self.ann_eq(k, &v)
            }
            "element_value_array_initializer" => self.flat_arr_init(node),
            _ => self.flat(node),
        }
    }

    /// Join an annotation `element_value_pair`'s key and value per
    /// `SPACE_AROUND_ANNOTATION_EQ` (`key = value` when on, `key=value` off).
    fn ann_eq(&self, k: &str, v: &str) -> String {
        if self.style.space_around_annotation_eq {
            format!("{} = {}", k, v)
        } else {
            format!("{}={}", k, v)
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
                                "{}{{\n{}\n{}}}",
                                self.ann_eq(k, ""),
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

        // Multiple args: laid out per the four body-layout toggles —
        // NEW_LINE_AFTER_LPAREN_IN_ANNOTATION (first argument stays on the
        // `(` line when off), RPAREN_ON_NEW_LINE_IN_ANNOTATION (`)` attaches
        // to the last argument line when off), and
        // ALIGN_MULTILINE_ANNOTATION_PARAMETERS (continuation indent when
        // off, padding under the first argument — one column after `(` —
        // when on, the record-header model).
        let arg_strs: Vec<String> = children.iter().map(|&c| self.flat_ann_arg(c)).collect();

        // Column of the `(` within its physical line: the annotation's line
        // starts at `ind(indent)` and the paren follows `@name` plus the
        // optional pre-paren gap.
        let open_col = self.col_after(0, &self.ind(indent))
            + self.col_after(
                0,
                &format!(
                    "@{}{}",
                    name,
                    self.sp(self.style.space_before_anotation_parameter_list)
                ),
            );
        let pad_len = usize::from(self.style.space_within_annotation_parentheses);
        let element_prefix = if self.style.align_multiline_annotation_parameters {
            self.align_prefix(open_col + 1 + pad_len)
        } else {
            self.ind(indent + 1)
        };
        let body = if self.style.new_line_after_lparen_in_annotation {
            let lines: Vec<String> = arg_strs
                .iter()
                .map(|p| format!("{}{}", element_prefix, p))
                .collect();
            format!("\n{}", lines.join(",\n"))
        } else {
            let mut s = arg_strs[0].clone();
            for p in &arg_strs[1..] {
                s.push_str(",\n");
                s.push_str(&element_prefix);
                s.push_str(p);
            }
            s
        };
        let tail = if self.style.rparen_on_new_line_in_annotation {
            format!("\n{}", self.ind(indent))
        } else {
            String::new()
        };
        format!(
            "@{}{}{}",
            name,
            self.sp(self.style.space_before_anotation_parameter_list),
            self.ann_parens(&format!("{}{}", body, tail)),
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

        // `PARAMETER_ANNOTATION_WRAP`: when a declared formal parameter carries
        // annotations and the option demands the own-line placement, the list
        // takes its wrapped one-parameter-per-line layout too (the own-line
        // placement is not expressible in the flat form). Codes 1/5 wrap the
        // list when the flat form overflows the margin.
        let ann_demand = if is_call {
            false
        } else if params.iter().any(|p| {
            p.kind() == "formal_parameter"
                && self
                    .get_mods(*p)
                    .is_some_and(|m| !self.mods_parts(m).0.is_empty())
        }) {
            match self.style.parameter_annotation_wrap {
                WrapStyle::DoNotWrap => false,
                WrapStyle::WrapAlways => true,
                _ => !self.fits(c, &flat),
            }
        } else {
            false
        };

        let should_wrap = self.keep_wrapped(node)
            || ann_demand
            || match wrap {
                WrapStyle::DoNotWrap => false,
                WrapStyle::WrapAlways => true,
                _ => !self.fits(c, &flat),
            };

        if !should_wrap {
            return flat;
        }

        let inner = indent + 1;
        // `DECLARATION_PARAMETER_INDENT` / `CALL_PARAMETER_INDENT`: an
        // explicit width overrides the continuation indent for this construct
        // kind only (the other kinds keep their widths); `-1` (default)
        // inherits today's `ind(inner)` byte-for-byte.
        let ind = if is_call {
            self.construct_ind(indent, self.style.call_parameter_indent, &self.ind(inner))
        } else {
            self.construct_ind(
                indent,
                self.style.declaration_parameter_indent,
                &self.ind(inner),
            )
        };
        // `ALIGN_MULTILINE_PARAMETERS` (declarations) /
        // `ALIGN_MULTILINE_PARAMETERS_IN_CALLS` (calls): when the first
        // parameter stays on the header line after `(` (the
        // lparen-stays / rparen-alone arm) it is glued directly after `(` and
        // the remaining lines pad with spaces to the column after `(` — the
        // same two canonical layouts `record_components` distinguishes. Where
        // every parameter begins its own line the elements already share the
        // first parameter's own column, so alignment leaves them unchanged.
        let align_on = if is_call {
            self.style.align_multiline_parameters_in_calls
        } else {
            self.style.align_multiline_parameters
        };
        let first_inline = !lparen_nl && rparen_nl;
        let wrapped = if first_inline && align_on {
            let pref = self.align_prefix(c + 1);
            let parts: Vec<String> = params
                .iter()
                .map(|&p| self.wrapped_param(p, &pref, inner))
                .collect();
            let mut s = String::new();
            let mut it = parts.iter();
            if let Some(first) = it.next() {
                s.push_str(first);
            }
            for p in it {
                s.push_str(",\n");
                s.push_str(&pref);
                s.push_str(p);
            }
            s
        } else {
            params
                .iter()
                .map(|&p| format!("{}{}", ind, self.wrapped_param(p, &ind, inner)))
                .collect::<Vec<_>>()
                .join(",\n")
        };

        // `ALIGN_MULTILINE_METHOD_BRACKETS`: a closing paren on its own line
        // aligns under the opening paren's column instead of the declaration
        // indent.
        let tail = if rparen_nl {
            if self.style.align_multiline_method_brackets {
                format!("\n{}", self.align_prefix(c))
            } else {
                format!("\n{}", self.ind(indent))
            }
        } else {
            String::new()
        };
        // Elements begin their own lines after `(` when the lparen moves or
        // the rparen stays attached (same layout for both lparen settings, as
        // the option files pin); only the lparen-stays / rparen-alone arm
        // keeps the first element on the header line.
        let lead_nl = lparen_nl || !rparen_nl;
        let inner = if lead_nl {
            format!("\n{}{}", wrapped, tail)
        } else {
            format!("{}{}", wrapped, tail)
        };
        let pad = self.style.space_within_method_parentheses;
        Self::within('(', ')', pad, &inner)
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

    /// Render one formal parameter in the per-line (wrapped) parameter list,
    /// honouring `PARAMETER_ANNOTATION_WRAP` + the
    /// `DO_NOT_WRAP_AFTER_SINGLE_ANNOTATION_IN_PARAMETER` exemption: an
    /// annotated parameter breaks so the annotations sit on their own lines
    /// (at the element prefix) and the type / name continues on the next line
    /// at the same prefix; a lone annotation under the exemption (or a wrap
    /// code that keeps the param inline) renders exactly like `flat_param`.
    /// `prefix` is the line prefix of the element in the wrapped list (`ind`
    /// for the own-line layout, the alignment prefix for the first-inline
    /// arm); `level` is the indent level to feed the annotation renderer.
    fn wrapped_param(&self, node: Node<'s>, prefix: &str, level: usize) -> String {
        if node.kind() != "formal_parameter" {
            return self.flat_param(node);
        }
        let mods = match self.get_mods(node) {
            Some(m) => m,
            None => return self.flat_param(node),
        };
        let (anns, _) = self.mods_parts(mods);
        if anns.is_empty() {
            return self.flat_param(node);
        }
        let inline = self.flat_param(node);
        let single_exempt =
            anns.len() == 1 && self.style.do_not_wrap_after_single_annotation_in_parameter;
        let pcol = self.col_after(0, prefix);
        let should_break = match self.style.parameter_annotation_wrap {
            WrapStyle::DoNotWrap => false,
            WrapStyle::WrapAlways => true,
            _ => !self.fits(pcol, &inline),
        };
        if single_exempt || !should_break {
            return inline;
        }
        let ann_lines: Vec<String> = anns.iter().map(|&a| self.annotation(a, level)).collect();
        let ty = self
            .fld(node, "type")
            .map(|n| self.flat_type(n))
            .unwrap_or_default();
        let nm = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");
        format!(
            "{}\n{}{} {}",
            ann_lines.join(&format!("\n{}", prefix)),
            prefix,
            ty,
            nm
        )
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

    /// True when `node` (a declaration) carries exactly one `variable_declarator`.
    fn single_declarator(&self, node: Node<'s>) -> bool {
        self.named(node)
            .into_iter()
            .filter(|n| n.kind() == "variable_declarator")
            .count()
            == 1
    }

    /// The column (within a rendered single-line declaration) where the
    /// declarator name starts — the element `ALIGN_CONSECUTIVE_VARIABLE_`
    /// `DECLARATIONS` / `ALIGN_GROUP_FIELD_DECLARATIONS` align. The name
    /// follows the canonical `[mods ]type ` prefix; when the rendered text
    /// does not start with that prefix (an R4 echo or annotation line) `None`
    /// keeps the member out of a run.
    fn decl_name_col(&self, node: Node<'s>, text: &str) -> Option<usize> {
        if text.contains('\n') || text.starts_with('@') {
            return None;
        }
        let mods = self
            .get_mods(node)
            .map(|m| self.flat_mods(m))
            .unwrap_or_default();
        let ty = self
            .fld(node, "type")
            .map(|n| self.flat_type(n))
            .unwrap_or_default();
        let prefix = if mods.is_empty() {
            format!("{} ", ty)
        } else {
            format!("{} {} ", mods, ty)
        };
        if !text.starts_with(&prefix) {
            return None;
        }
        Some(prefix.len())
    }

    /// The column (within a rendered single-line method declaration) where
    /// the method name starts — the element
    /// `ALIGN_SUBSEQUENT_SIMPLE_METHODS` aligns. The name follows the
    /// canonical `[mods ]type ` prefix; `None` keeps the member out of a run
    /// (a method with no name-prefix match, annotations, or a wrapped body).
    fn method_name_col(&self, node: Node<'s>, text: &str) -> Option<usize> {
        if text.contains('\n') || text.starts_with('@') {
            return None;
        }
        let mut prefix = String::new();
        if let Some(mods) = self.get_mods(node) {
            let ms = self.flat_mods(mods);
            if !ms.is_empty() {
                prefix.push_str(&ms);
                prefix.push(' ');
            }
        }
        if let Some(tp) = self.fld(node, "type_parameters") {
            prefix.push_str(&self.flat_type_params(tp));
            prefix.push(' ');
        }
        let ty = self.fld(node, "type")?;
        prefix.push_str(&self.flat_type(ty));
        prefix.push(' ');
        if !text.starts_with(&prefix) {
            return None;
        }
        Some(prefix.len())
    }

    /// Columnar-alignment element of a block statement for
    /// `ALIGN_CONSECUTIVE_VARIABLE_DECLARATIONS` (kind 0: the declarator
    /// name) / `ALIGN_CONSECUTIVE_ASSIGNMENTS` (kind 1: the `=`), or `None`
    /// for non-candidates, which break runs.
    fn stmt_align_elem(&self, s: Node<'s>, text: &str) -> Option<(u8, usize, usize)> {
        match s.kind() {
            "local_variable_declaration"
                if self.style.align_consecutive_variable_declarations
                    && self.single_declarator(s) =>
            {
                let col = self.decl_name_col(s, text)?;
                Some((0, col, col))
            }
            "expression_statement" if self.style.align_consecutive_assignments => {
                let e = s.named_child(0)?;
                if e.kind() != "assignment_expression" || text.contains('\n') {
                    return None;
                }
                let left = self
                    .fld(e, "left")
                    .map(|n| self.flat(n))
                    .unwrap_or_default();
                if left.is_empty() || !text.starts_with(&left) {
                    return None;
                }
                let op = self
                    .all_ch(e)
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
                // The operator starts `left.len() + sep.len()` columns in;
                // padding inserted right after the left side shifts it to the
                // run's widest operator column.
                let eq_col = left.len() + sep.len();
                Some((1, eq_col, left.len()))
            }
            _ => None,
        }
    }

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
        let sc = self.col_after(0, &self.ind(inner));
        let mut lines: Vec<BodyLine> = Vec::with_capacity(stmts.len());

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
            if s.is_extra() {
                // `comment` renders the full line(s) — column placement, the
                // optional space after `//`, and WRAP_COMMENTS — so the
                // statement indent prefix is not added for extras. Comments
                // break columnar runs.
                lines.push(BodyLine {
                    blanks,
                    indented: false,
                    text: self.comment(*s, inner),
                    align: None,
                });
            } else {
                let text = self.stmt(*s, inner, sc);
                let align = self.stmt_align_elem(*s, &text);
                lines.push(BodyLine {
                    blanks,
                    indented: true,
                    text,
                    align,
                });
            }
        }

        // Columnar alignment over consecutive statements
        // (`ALIGN_CONSECUTIVE_VARIABLE_DECLARATIONS`,
        // `ALIGN_CONSECUTIVE_ASSIGNMENTS`).
        pad_column_runs(&mut lines);

        let mut out = String::from("{\n");
        for l in lines {
            self.push_indented_blanks(&mut out, l.blanks, inner);
            if l.indented {
                out.push_str(&self.ind(inner));
            }
            out.push_str(&l.text);
            out.push('\n');
        }

        // Closing gap before the right brace.
        let last = stmts[stmts.len() - 1];
        let existing = self.blank_lines_between(last.end_byte(), node.end_byte().saturating_sub(1));
        let blanks = self.spacing(existing, self.style.keep_blank_lines_before_rbrace, 0);
        self.push_indented_blanks(&mut out, blanks, inner);

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

    /// True when every physical line of `s` fits within the right margin:
    /// the first line is measured from column `c` (the statement column),
    /// every later line from column 0 — its indentation is part of `s`, so
    /// [`Self::col_after`] from 0 reproduces the physical line width.
    /// Single-line text behaves exactly like [`Self::fits`].
    fn fits_lines(&self, c: usize, s: &str) -> bool {
        let mut col = c;
        for ch in s.chars() {
            match ch {
                '\n' => {
                    if col > self.style.right_margin as usize {
                        return false;
                    }
                    col = 0;
                }
                '\t' => col += self.style.tab_size as usize - (col % self.style.tab_size as usize),
                _ => col += 1,
            }
        }
        col <= self.style.right_margin as usize
    }

    /// Presents a one-line `{ … }` block from its rendered inner text,
    /// honouring the two Java one-line-body toggles: with
    /// `SPACES_INSIDE_BLOCK_BRACES_WHEN_BODY_IS_PRESENT` on the block is
    /// padded (`{ s }`), with it off (absent/false — the built-in default)
    /// flush (`{s}`); and with `NEW_LINE_WHEN_BODY_IS_PRESENTED` on the
    /// block starts on a fresh line at `indent` — the statement head's own
    /// indent — instead of following the head on its line. Callers append
    /// the result straight after their head text, using [`Self::body_gap`]
    /// for the gap between the head and the block.
    fn present_block(&self, inner: &str, indent: usize) -> String {
        let block = if self.style.spaces_inside_block_braces_when_body_is_present {
            format!("{{ {} }}", inner)
        } else {
            format!("{{{}}}", inner)
        };
        if self.style.new_line_when_body_is_presented {
            format!("\n{}{}", self.ind(indent), block)
        } else {
            block
        }
    }

    /// The gap between a statement head and a collapsed one-line block: the
    /// `SPACE_BEFORE_*_LBRACE` space for an end-of-line block, nothing when
    /// `NEW_LINE_WHEN_BODY_IS_PRESENTED` moved the block onto its own line
    /// (its presentation already carries the newline and indent, so a space
    /// would dangle at the line end — R5).
    fn body_gap(&self, block: &str, lbrace: bool) -> &'static str {
        if block.starts_with('\n') {
            ""
        } else {
            self.sp(lbrace)
        }
    }

    /// Renders `node` (a `block`) as a one-line `{s}` / `{ s }` body when it
    /// contains exactly one simple statement; returns `None` otherwise. The
    /// block presentation follows the two one-line-body toggles
    /// ([`Self::present_block`]); empty blocks are already rendered inline
    /// by [`Self::block`] and return `None` here.
    fn one_line_body(&self, node: Node<'s>, indent: usize) -> Option<String> {
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
        Some(self.present_block(&txt, indent))
    }

    /// Single-line rendering of an `if`/`else if`/`else` chain whose blocks are
    /// all simple; `None` if any body is not a simple block. The `if`→`(` gap
    /// follows `SPACE_BEFORE_IF_PARENTHESES`, the body gaps the corresponding
    /// `SPACE_BEFORE_*_LBRACE` toggles (unless the body presentation already
    /// starts its own line), and the `}`→`else` gap
    /// `SPACE_BEFORE_ELSE_KEYWORD`.
    fn if_one_line(&self, node: Node<'s>, indent: usize) -> Option<String> {
        let cond = self.fld(node, "condition")?;
        let cond_txt = self.flat_keyword_cond(cond, self.style.space_within_if_parentheses);
        if cond_txt.contains('\n') {
            return None;
        }
        let cons = self.fld(node, "consequence")?;
        let cons_txt = self.one_line_body(cons, indent)?;
        // `cond_txt` already includes the parentheses (parenthesized_expression).
        let mut out = format!(
            "if{}{}{}{}",
            self.sp(self.style.space_before_if_parentheses),
            cond_txt,
            self.body_gap(&cons_txt, self.style.space_before_if_lbrace),
            cons_txt
        );
        if let Some(alt) = self.fld(node, "alternative") {
            let else_gap = self.sp(self.style.space_before_else_keyword);
            if alt.kind() == "if_statement" {
                out.push_str(&format!(
                    "{}else {}",
                    else_gap,
                    self.if_one_line(alt, indent)?
                ));
            } else {
                let alt_txt = self.one_line_body(alt, indent)?;
                out.push_str(&format!(
                    "{}else{}",
                    else_gap,
                    self.body_gap(&alt_txt, self.style.space_before_else_lbrace)
                ));
                out.push_str(&alt_txt);
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
                // `WRAP_SEMICOLON_AFTER_CALL_CHAIN`: the `;` of a wrapped
                // chained call moves to its own line at the statement indent.
                if self.style.wrap_semicolon_after_call_chain
                    && node.named_child(0).map(|n| n.kind()) == Some("method_invocation")
                    && e.contains('\n')
                {
                    format!("{}\n{};", e, self.ind(indent))
                } else {
                    format!("{};", e)
                }
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
            "switch_expression" => self.switch_stmt(node, indent, c, false),
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
                // tree-sitter-java 0.23 gives the node no field names:
                // `named_child(0)` is the label identifier, `named_child(1)`
                // the labeled statement.
                let label = node.named_child(0).map(|n| self.txt(n)).unwrap_or_default();
                let body = node
                    .named_child(1)
                    .map(|n| self.stmt(n, indent, c))
                    .unwrap_or_default();
                // `LABEL_INDENT_SIZE` / `LABEL_INDENT_ABSOLUTE`: the label
                // line's position. The label is the (caller-prefixed) first
                // line of the statement text, so relative adds the configured
                // width to the statement indent (`ind(indent)`), while
                // absolute puts the label at the width from the margin — the
                // caller's statement-indent prefix is compensated on the
                // label line, so labels whose width is shallower than the
                // statement indent stay at the smallest achievable column.
                // The default (`0`, non-absolute) puts the label at the
                // statement indent, as before.
                let width = self.style.label_indent_size as usize;
                let stmt_col = indent * self.style.indent_size as usize;
                let label_col = if self.style.label_indent_absolute {
                    width
                } else {
                    stmt_col + width
                };
                let extra = label_col.saturating_sub(stmt_col);
                format!(
                    "{}{}:\n{}{}",
                    self.indent_str(extra),
                    label,
                    self.ind(indent),
                    body
                )
            }
            "block" => self.block(node, indent, c, 0),
            "empty_statement" => ";".to_string(),
            "line_comment" | "block_comment" => self.comment(node, indent),
            _ => self.txt(node).to_string(),
        }
    }

    fn local_var(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let mods = self.get_mods(node);
        let per_line = mods
            .map(|m| self.mods_per_line(m, indent))
            .unwrap_or_default();
        let inline = mods.and_then(|m| self.mods_inline(m, indent));

        // The declaration is built per modifier form; the inline form's first
        // line is measured for codes 1/5. Local variables join the modifiers
        // to the type with a single space (the statement never breaks after
        // the modifier list the way member declarations can).
        let tail_with = |prefix: &str, has_mods: bool| -> String {
            let mut out = String::new();
            out.push_str(prefix);
            if has_mods && !prefix.is_empty() && !prefix.ends_with(' ') && !prefix.ends_with('\n') {
                out.push(' ');
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
                        self.assign_expr(
                            val,
                            indent,
                            c,
                            &prefix,
                            "=",
                            self.keep_wrapped(node),
                            None
                        )
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
                            self.col_after(c, &out) + name.len() + sep.len() + 1 + sep.len();
                        let val_str = self.expr(val, indent, val_col);
                        Self::join_sep(&format!("{}{}=", name, sep), sep, &val_str)
                    } else {
                        name.to_string()
                    }
                })
                .collect();

            out.push_str(&decl_strs.join(self.comma_sep(self.style.space_after_comma)));
            out.push(';');
            out
        };

        let use_inline = match mods {
            Some(m) => self.mods_inline_decision(
                m,
                indent,
                self.style.variable_annotation_wrap,
                &self.inline_tail_first(&tail_with, &inline, c, indent, true),
                c,
            ),
            None => false,
        };

        if use_inline {
            tail_with(inline.as_deref().unwrap_or(""), true)
        } else {
            tail_with(&per_line, mods.is_some())
        }
    }

    /// Whether the `if_one_line` collapse in [`Self::if_stmt`] would
    /// contradict a clause-keyword option: `ELSE_ON_NEW_LINE` (any
    /// alternative starts a new line) or, with `SPECIAL_ELSE_IF_TREATMENT`
    /// off, a fused `else if` alternative that must nest inside an
    /// `else { … }` block instead of staying on the chain's line.
    fn if_alt_breaks_one_line(&self, node: Node<'s>) -> bool {
        match self.fld(node, "alternative") {
            None => false,
            Some(alt) => {
                self.style.else_on_new_line
                    || (alt.kind() == "if_statement" && !self.style.special_else_if_treatment)
            }
        }
    }

    fn if_stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        // Keep simple bodies on one line when enabled and the whole statement fits.
        if self.braces_style_inline()
            && self.style.keep_simple_blocks_in_one_line
            && !self.if_alt_breaks_one_line(node)
        {
            if let Some(one) = self.if_one_line(node, indent) {
                if self.fits_lines(c, &one) {
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
                    node,
                    indent,
                    c,
                    self.style.if_brace_force,
                    self.style.space_before_if_lbrace,
                )
            })
            .unwrap_or_default();

        let mut out = format!("if{}{}{}", p_gap, cond, cons);

        if let Some(alt) = self.fld(node, "alternative") {
            // `ELSE_ON_NEW_LINE` puts the keyword on a fresh line at the
            // statement indent; otherwise the `}`→`else` gap follows
            // `SPACE_BEFORE_ELSE_KEYWORD`.
            let kw_gap = if self.style.else_on_new_line {
                format!("\n{}", self.ind(indent))
            } else {
                self.sp(self.style.space_before_else_keyword).to_string()
            };
            let alt_str = if alt.kind() == "if_statement" {
                if self.style.special_else_if_treatment {
                    format!("{}else {}", kw_gap, self.if_stmt(alt, indent, c))
                } else {
                    // `SPECIAL_ELSE_IF_TREATMENT` off: fuse via an explicit
                    // `else { if … }` block. The braces group a single `if`, so
                    // semantics are unchanged (R5) and the braces survive a
                    // reformat (R6).
                    let inner =
                        self.if_stmt(alt, indent + 1, self.col_after(0, &self.ind(indent + 1)));
                    format!(
                        "{}else{}{{\n{}{}\n{}}}",
                        kw_gap,
                        self.sp(self.style.space_before_else_lbrace),
                        self.ind(indent + 1),
                        inner,
                        self.ind(indent)
                    )
                }
            } else {
                format!(
                    "{}else{}",
                    kw_gap,
                    self.stmt_as_block_or_inline(
                        alt,
                        node,
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
    /// `force` demands it (the forced brace uses the same `lbrace` gap). When
    /// `KEEP_CONTROL_STATEMENT_IN_ONE_LINE` is on and the source already has
    /// the brace-less body on the statement's header line (see
    /// [`Self::body_kept_inline`]), it is joined after the header with a
    /// single space instead.
    fn stmt_as_block_or_inline(
        &self,
        node: Node<'s>,
        owner: Node<'s>,
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
                ForceStyle::DoNotForce => {
                    if self.body_kept_inline(owner, node.start_byte()) {
                        format!(" {}", s)
                    } else {
                        format!("\n{}{}", self.ind(indent + 1), s)
                    }
                }
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
                    } else if self.body_kept_inline(owner, node.start_byte()) {
                        format!(" {}", s)
                    } else {
                        format!("\n{}{}", self.ind(indent + 1), s)
                    }
                }
            }
        }
    }

    /// The byte offset just past the last token of `node` that precedes the
    /// whitespace gap before a body starting at `body_start` — the closing `)`
    /// or clause keyword the body follows.
    fn header_end_before(&self, node: Node<'s>, body_start: usize) -> usize {
        let mut cur = node.walk();
        let mut end = node.start_byte();
        for ch in node.children(&mut cur) {
            if ch.start_byte() >= body_start {
                break;
            }
            end = end.max(ch.end_byte());
        }
        end
    }

    /// `KEEP_CONTROL_STATEMENT_IN_ONE_LINE` body join: true when the option is
    /// set and the source gap between `node`'s last header token and a
    /// brace-less body at `body_start` holds no newline and no comment — the
    /// body sits on the header's line and may be joined with a single space.
    fn body_kept_inline(&self, node: Node<'s>, body_start: usize) -> bool {
        if !self.style.keep_control_statement_in_one_line {
            return false;
        }
        let end = self.header_end_before(node, body_start);
        let gap = &self.src[end..body_start.min(self.src.len())];
        gap.iter().all(|b| *b == b' ' || *b == b'\t')
    }

    fn for_stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let body_node = self.fld(node, "body");
        let wrap = self.style.for_statement_wrap;

        let header = if wrap != WrapStyle::DoNotWrap {
            // `FOR_STATEMENT_WRAP`: re-render the header from its
            // init / condition / update fields and break it at the
            // semicolons when it wraps (see `wrapped_for_header`).
            self.wrapped_for_header(node, indent, c)
        } else if let Some(b) = body_node {
            // Re-create header from source bytes (handles all edge cases of for-init/cond/update)
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

        // Keep a simple body on one line when enabled and it fits; a header
        // that already wrapped keeps the body on its own line.
        if self.braces_style_inline()
            && self.style.keep_simple_blocks_in_one_line
            && !header.contains('\n')
        {
            if let Some(one) = body_node.and_then(|b| self.one_line_body(b, indent)) {
                let candidate = format!(
                    "{}{}{}",
                    header,
                    self.body_gap(&one, self.style.space_before_for_lbrace),
                    one
                );
                if self.fits_lines(c, &candidate) {
                    return candidate;
                }
            }
        }

        let body = body_node
            .map(|n| {
                self.stmt_as_block_or_inline(
                    n,
                    node,
                    indent,
                    c,
                    self.style.for_brace_force,
                    self.style.space_before_for_lbrace,
                )
            })
            .unwrap_or_default();

        format!("{}{}", header, body)
    }

    /// Re-render a classic `for` header from its init / condition / update
    /// children per `FOR_STATEMENT_WRAP`: the flat form (spaced like
    /// [`Self::rebuild_for_header`]) is returned when it fits (or under
    /// do-not-wrap, which is handled by the caller's verbatim path); when it
    /// must wrap, each non-empty slot moves to its own continuation line,
    /// broken after its `;`, and `FOR_STATEMENT_LPAREN/RPAREN_ON_NEXT_LINE`
    /// put the parens on their own lines. Only whitespace changes, never
    /// token order (R5).
    fn wrapped_for_header(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        let before = Self::sep(self.style.space_before_semicolon);
        let after = Self::sep(self.style.space_after_semicolon);
        let pad = self.style.space_within_for_parentheses;
        let gap = self.sp(self.style.space_before_for_parentheses);

        let init = self.for_part_text(node, "init");
        let cond = self
            .fld(node, "condition")
            .map(|n| normalise_ws(self.txt(n)))
            .unwrap_or_default();
        let upd = self.for_part_text(node, "update");

        // Flat form: the same field-based construction as `rebuild_for_header`.
        let mut flat = String::from("for");
        flat.push_str(gap);
        flat.push('(');
        if pad {
            flat.push(' ');
        }
        if !init.is_empty() {
            flat.push_str(&init);
            flat.push_str(before);
        }
        flat.push(';');
        if !cond.is_empty() {
            flat.push_str(after);
            flat.push_str(&cond);
            flat.push_str(before);
        }
        flat.push(';');
        if !upd.is_empty() {
            flat.push_str(after);
            flat.push_str(&upd);
        }
        if pad {
            flat.push(' ');
        }
        flat.push(')');

        let should_wrap = match self.style.for_statement_wrap {
            WrapStyle::DoNotWrap => false,
            WrapStyle::WrapAlways => true,
            _ => !self.fits(c, &flat),
        };
        if !should_wrap {
            return flat;
        }

        let cont = self.cont(indent);
        // `ALIGN_MULTILINE_FOR`: the cond / update continuation lines align
        // under the init slot (the first element after `(`), which sits right
        // after the opening paren on the header (or paren) line. Where the
        // lparen moves to its own line the paren line itself is at `cont`, so
        // the alignment column is `cont` width + 1 — the record-header model
        // for a lparen on its own line.
        let lparen_nl = self.style.for_statement_lparen_on_next_line;
        let part_pref = if self.style.align_multiline_for {
            let paren_col = if lparen_nl {
                self.col_after(0, &cont)
            } else {
                self.col_after(c, "for") + gap.len()
            };
            self.align_prefix(paren_col + 1 + if pad { 1 } else { 0 })
        } else {
            cont.clone()
        };
        let mut out = String::from("for");
        if lparen_nl {
            out.push('\n');
            out.push_str(&cont);
        } else {
            out.push_str(gap);
        }
        out.push('(');
        if pad {
            out.push(' ');
        }
        if !init.is_empty() {
            out.push_str(&init);
            out.push_str(before);
        }
        out.push(';');
        if !cond.is_empty() {
            out.push('\n');
            out.push_str(&part_pref);
            out.push_str(&cond);
            out.push_str(before);
            out.push(';');
        }
        if !upd.is_empty() {
            out.push('\n');
            out.push_str(&part_pref);
            out.push_str(&upd);
        }
        if self.style.for_statement_rparen_on_next_line {
            out.push('\n');
            out.push_str(&self.ind(indent));
        }
        out.push(')');
        out
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
        // `KEEP_MULTIPLE_EXPRESSIONS_IN_ONE_LINE` guard: a classic `for`
        // header's init / update slot list stays joined on its line — the
        // per-slot wrap breaks at the `;` (`FOR_STATEMENT_WRAP`), never
        // inside the comma-separated list. The engine has no per-expression
        // break layout, so the option is honoured by construction; the read
        // keeps that explicit and load-bearing for a future expression-level
        // break.
        let _guard = self.style.keep_multiple_expressions_in_one_line;
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
        let wrap = self.style.for_statement_wrap;

        // Keep a simple body on one line when enabled and it fits.
        if self.braces_style_inline() && self.style.keep_simple_blocks_in_one_line {
            if let (Some(v), Some(b)) = (self.fld(node, "value"), self.fld(node, "body")) {
                if let Some(one) = self.one_line_body(b, indent) {
                    let vtxt = self.flat(v);
                    if !vtxt.contains('\n') {
                        let inner = format!("{} {}{}{}", ty, name, colon, vtxt);
                        let candidate = format!(
                            "for{}{}{}{}",
                            p_gap,
                            Self::within('(', ')', self.style.space_within_for_parentheses, &inner,),
                            self.body_gap(&one, self.style.space_before_for_lbrace),
                            one
                        );
                        if self.fits_lines(c, &candidate) {
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
                    node,
                    indent,
                    c,
                    self.style.for_brace_force,
                    self.style.space_before_for_lbrace,
                )
            })
            .unwrap_or_default();
        let inner = format!("{} {}{}{}", ty, name, colon, val);

        if wrap == WrapStyle::DoNotWrap {
            return format!(
                "for{}{}{}",
                p_gap,
                Self::within('(', ')', self.style.space_within_for_parentheses, &inner),
                body
            );
        }

        // The one-line header for the margin decision: the value renders
        // flat so the header's width is measured honestly.
        let flat_val = self
            .fld(node, "value")
            .map(|n| self.flat(n))
            .unwrap_or_default();
        let flat = format!(
            "for{}{}",
            p_gap,
            Self::within(
                '(',
                ')',
                self.style.space_within_for_parentheses,
                &format!("{} {}{}{}", ty, name, colon, flat_val),
            ),
        );
        if wrap != WrapStyle::WrapAlways && self.fits(c, &flat) {
            return format!(
                "for{}{}{}",
                p_gap,
                Self::within('(', ')', self.style.space_within_for_parentheses, &inner),
                body
            );
        }

        // Wrapped: the header breaks at its `:` and the value moves to a
        // continuation line; `FOR_STATEMENT_LPAREN/RPAREN_ON_NEXT_LINE` put
        // the parens on their own lines.
        let cont = self.cont(indent);
        // `ALIGN_MULTILINE_FOR`: the value line aligns under the type (the
        // first element after `(`), the same column the classic header's
        // init starts at.
        let lparen_nl = self.style.for_statement_lparen_on_next_line;
        let value_pref = if self.style.align_multiline_for {
            let paren_col = if lparen_nl {
                self.col_after(0, &cont)
            } else {
                self.col_after(c, "for") + p_gap.len()
            };
            self.align_prefix(
                paren_col
                    + 1
                    + if self.style.space_within_for_parentheses {
                        1
                    } else {
                        0
                    },
            )
        } else {
            cont.clone()
        };
        let mut w = String::from("for");
        if lparen_nl {
            w.push('\n');
            w.push_str(&cont);
        } else {
            w.push_str(p_gap);
        }
        w.push('(');
        if self.style.space_within_for_parentheses {
            w.push(' ');
        }
        w.push_str(&ty);
        w.push(' ');
        w.push_str(name);
        // The `:` ends the first header line; the before half of the
        // separator stays, the after half is replaced by the newline (R5).
        w.push_str(&Self::sep(self.style.space_before_colon_in_foreach));
        w.push(':');
        w.push('\n');
        w.push_str(&value_pref);
        w.push_str(&val);
        if self.style.space_within_for_parentheses && !val.ends_with('\n') {
            w.push(' ');
        }
        if self.style.for_statement_rparen_on_next_line {
            w.push('\n');
            w.push_str(&self.ind(indent));
        }
        w.push(')');
        format!("{}{}", w, body)
    }

    fn while_stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        // Keep a simple body on one line when enabled and it fits.
        if self.braces_style_inline() && self.style.keep_simple_blocks_in_one_line {
            if let (Some(cn), Some(b)) = (self.fld(node, "condition"), self.fld(node, "body")) {
                if let Some(one) = self.one_line_body(b, indent) {
                    let ct = self.flat_keyword_cond(cn, self.style.space_within_while_parentheses);
                    if !ct.contains('\n') {
                        let candidate = format!(
                            "while{}{}{}{}",
                            self.sp(self.style.space_before_while_parentheses),
                            ct,
                            self.body_gap(&one, self.style.space_before_while_lbrace),
                            one
                        );
                        if self.fits_lines(c, &candidate) {
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
                    node,
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
        // Keep a simple body on one line when enabled and it fits; a trailing
        // `while` on its own line (`WHILE_ON_NEW_LINE`) would contradict that.
        if self.braces_style_inline()
            && self.style.keep_simple_blocks_in_one_line
            && !self.style.while_on_new_line
        {
            if let (Some(b), Some(cn)) = (self.fld(node, "body"), self.fld(node, "condition")) {
                if let Some(one) = self.one_line_body(b, indent) {
                    let ct = self.flat_keyword_cond(cn, self.style.space_within_while_parentheses);
                    if !ct.contains('\n') {
                        let candidate = format!(
                            "do{}{}{}while{}{};",
                            self.body_gap(&one, self.style.space_before_do_lbrace),
                            one,
                            self.sp(self.style.space_before_while_keyword),
                            self.sp(self.style.space_before_while_parentheses),
                            ct
                        );
                        if self.fits_lines(c, &candidate) {
                            return candidate;
                        }
                    }
                }
            }
        }

        // Whether the rendered body text already ends at the statement indent
        // (the own-line brace-less arm below emits `\n<indent>` for the tail).
        let mut at_statement_indent = false;
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
                    } else if self.body_kept_inline(node, n.start_byte()) {
                        // `KEEP_CONTROL_STATEMENT_IN_ONE_LINE`: the source has
                        // the body on the `do` line, so join it there.
                        format!(" {}", s)
                    } else {
                        at_statement_indent = true;
                        format!("\n{}{}\n{}", self.ind(indent + 1), s, self.ind(indent))
                    }
                }
            })
            .unwrap_or_default();
        // `cond` is a parenthesized_expression and already contains its parens.
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
        let w_gap = self.sp(self.style.space_before_while_keyword);
        let head = format!("do{}", body);
        if self.style.while_on_new_line {
            // The trailing `while` starts a fresh line at the statement indent;
            // the own-line brace-less body already ends with that indent.
            let kw = format!("while{}{};", p_gap, cond);
            if at_statement_indent {
                format!("{}{}", head, kw)
            } else {
                format!("{}\n{}{}", head, self.ind(indent), kw)
            }
        } else {
            format!("{}{}while{}{};", head, w_gap, p_gap, cond)
        }
    }

    /// Canonical pieces of a `catch_formal_parameter` node: the keyword
    /// modifiers (`final`), the catch types (each canonical via
    /// [`Self::flat_type`]) and the parameter name (the `name` field, hoisted
    /// from the grammar's hidden `_variable_declarator_id`). `None` when the
    /// node does not have the expected shape (a comment subtree or a missing
    /// piece) — callers then fall back to the verbatim echo (R4).
    fn catch_pieces(&self, node: Node<'s>) -> Option<(String, Vec<String>, String)> {
        if node.kind() != "catch_formal_parameter" || self.has_comment_subtree(node) {
            return None;
        }
        let mods = self
            .get_mods(node)
            .map(|m| self.flat_mods(m))
            .unwrap_or_default();
        let catch_type = self
            .named(node)
            .into_iter()
            .find(|c| c.kind() == "catch_type")?;
        let types: Vec<String> = self
            .named(catch_type)
            .iter()
            .map(|&t| self.flat_type(t))
            .collect();
        if types.is_empty() {
            return None;
        }
        let name = self.fld(node, "name").map(|n| self.txt(n)).unwrap_or("");
        if name.is_empty() {
            return None;
        }
        Some((mods, types, name.to_string()))
    }

    /// Render a catch clause's parameter head — the `(…)` text that follows
    /// the `catch` keyword — honouring `MULTI_CATCH_TYPES_WRAP` (codes `0` /
    /// `1` / `2` / `5`) and `ALIGN_TYPES_IN_MULTI_CATCH` on the
    /// record-components pattern: the flat `(final A | B e)` head is kept
    /// unless the wrap code engages — only ever for a multi-type list — when
    /// the first type stays on the `catch (` line and each following type
    /// starts its own line with the `|` operator leading the continuation
    /// (the binary operator-placement convention), padded to the first type's
    /// column when the align option is on, else to the continuation indent.
    /// `lparen_col` is the column of the opening paren on its physical line;
    /// the flat head's margin check is measured from there, covering the
    /// whole `catch (…)` line. Single-type catches and unmodelled shapes keep
    /// the flat / verbatim echo (R4); `DoNotWrap` (the absent default) never
    /// wraps.
    fn catch_param_head(&self, param: Node<'s>, indent: usize, lparen_col: usize) -> String {
        let pad = self.style.space_within_catch_parentheses;
        let Some((mods, types, name)) = self.catch_pieces(param) else {
            return Self::within('(', ')', pad, self.txt(param));
        };
        let mods_pref = if mods.is_empty() {
            String::new()
        } else {
            format!("{} ", mods)
        };
        let flat_inner = format!("{}{} {}", mods_pref, types.join(" | "), name);
        let flat = Self::within('(', ')', pad, &flat_inner);
        let should_wrap = match self.style.multi_catch_types_wrap {
            WrapStyle::DoNotWrap => false,
            WrapStyle::WrapAlways => types.len() > 1,
            _ => types.len() > 1 && !self.fits(lparen_col, &flat),
        };
        if !should_wrap {
            return flat;
        }
        // The first type's column on the `catch (` line: right of the paren,
        // past the within-pad and any modifier prefix.
        let first_col = lparen_col + 1 + if pad { 1 } else { 0 } + mods_pref.len();
        let pref = if self.style.align_types_in_multi_catch {
            self.align_prefix(first_col)
        } else {
            self.cont(indent)
        };
        let mut inner = format!("{}{}", mods_pref, types[0]);
        for ty in &types[1..] {
            inner.push('\n');
            inner.push_str(&pref);
            inner.push_str("| ");
            inner.push_str(ty);
        }
        inner.push(' ');
        inner.push_str(&name);
        Self::within('(', ')', pad, &inner)
    }

    /// Single-line rendering of a `try` statement when the try body and every
    /// catch/finally body is a simple one-statement block; `None` otherwise
    /// (the caller falls through to the multi-line layout). `c` is the column
    /// where the statement starts.
    fn try_one_line(&self, node: Node<'s>, indent: usize, c: usize) -> Option<String> {
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
        let body_txt = self.one_line_body(body, indent)?;
        let mut out = format!(
            "try{}{}{}",
            resources,
            self.body_gap(&body_txt, self.style.space_before_try_lbrace),
            body_txt
        );

        for ch in self.named(node) {
            match ch.kind() {
                "catch_clause" => {
                    let param = self
                        .named(ch)
                        .into_iter()
                        .find(|n| n.kind() == "catch_formal_parameter");
                    let pad = self.style.space_within_catch_parentheses;
                    // The canonical flat head — or the whitespace-normalised
                    // verbatim echo for an unmodelled shape (R4). When the
                    // flat head would wrap under `MULTI_CATCH_TYPES_WRAP` at
                    // its one-line column, the whole one-line collapse is
                    // abandoned so the multi-line layout can wrap the list
                    // (a one-line `catch (…)` must not contradict the wrap
                    // code).
                    let catch_head = match param.and_then(|p| self.catch_pieces(p)) {
                        Some((mods, types, name)) => {
                            let mods_pref = if mods.is_empty() {
                                String::new()
                            } else {
                                format!("{} ", mods)
                            };
                            let flat_inner = format!("{}{} {}", mods_pref, types.join(" | "), name);
                            let flat = Self::within('(', ')', pad, &flat_inner);
                            // The head's `(` sits after the keyword and its
                            // two gaps on the running one-line text.
                            let lparen_col = self.col_after(c, &out)
                                + self.sp(self.style.space_before_catch_keyword).len()
                                + 5
                                + self.sp(self.style.space_before_catch_parentheses).len();
                            let should_wrap = match self.style.multi_catch_types_wrap {
                                WrapStyle::DoNotWrap => false,
                                WrapStyle::WrapAlways => types.len() > 1,
                                _ => types.len() > 1 && !self.fits(lparen_col, &flat),
                            };
                            if should_wrap {
                                return None;
                            }
                            flat
                        }
                        None => match param {
                            Some(p) => Self::within('(', ')', pad, &normalise_ws(self.txt(p))),
                            None => Self::within('(', ')', pad, ""),
                        },
                    };
                    let cbody = self.fld(ch, "body")?;
                    let cbody_txt = self.one_line_body(cbody, indent)?;
                    out.push_str(self.sp(self.style.space_before_catch_keyword));
                    out.push_str("catch");
                    out.push_str(self.sp(self.style.space_before_catch_parentheses));
                    out.push_str(&catch_head);
                    out.push_str(self.body_gap(&cbody_txt, self.style.space_before_catch_lbrace));
                    out.push_str(&cbody_txt);
                }
                "finally_clause" => {
                    // The block is a plain child of finally_clause (no field name).
                    let fbody = self.named(ch).into_iter().find(|n| n.kind() == "block")?;
                    let fbody_txt = self.one_line_body(fbody, indent)?;
                    out.push_str(self.sp(self.style.space_before_finally_keyword));
                    out.push_str("finally");
                    out.push_str(self.body_gap(&fbody_txt, self.style.space_before_finally_lbrace));
                    out.push_str(&fbody_txt);
                }
                _ => {}
            }
        }

        Some(out)
    }

    fn try_stmt(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        // Keep simple try/catch/finally bodies on one line when enabled and
        // the whole statement fits; clause keywords on their own lines
        // (`CATCH_ON_NEW_LINE` / `FINALLY_ON_NEW_LINE`) would contradict that.
        if self.braces_style_inline()
            && self.style.keep_simple_blocks_in_one_line
            && !self.style.catch_on_new_line
            && !self.style.finally_on_new_line
        {
            if let Some(one) = self.try_one_line(node, indent, c) {
                if self.fits_lines(c, &one) {
                    return one;
                }
            }
        }

        let resources = if node.kind() == "try_with_resources_statement" {
            // The resource_specification node already includes its parens.
            self.fld(node, "resources")
                .map(|n| {
                    if self.style.resource_list_wrap != WrapStyle::DoNotWrap {
                        // Canonical resource-list layout (flat when it fits,
                        // one resource per line otherwise). Unmodelled spec
                        // shapes fall back to the verbatim echo below (R4).
                        self.resource_list(n, indent, c)
                    } else {
                        let t = self.txt(n).trim();
                        format!(
                            "{}{}",
                            self.sp(self.style.space_before_try_parentheses),
                            pad_outer_parens(t, self.style.space_within_try_parentheses)
                        )
                    }
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
                        .find(|n| n.kind() == "catch_formal_parameter");
                    // Column of the clause's `(` on its physical line: the
                    // keyword starts at the statement indent on its own line
                    // (`CATCH_ON_NEW_LINE`) or right after the previous
                    // body's `}` — which sits at `ind(indent)` — plus the
                    // keyword gap.
                    let indent_col = self.col_after(0, &self.ind(indent));
                    let catch_col = if self.style.catch_on_new_line {
                        indent_col
                    } else {
                        indent_col + 1 + self.sp(self.style.space_before_catch_keyword).len()
                    };
                    let lparen_col =
                        catch_col + 5 + self.sp(self.style.space_before_catch_parentheses).len();
                    let pad = self.style.space_within_catch_parentheses;
                    let cbody = self
                        .fld(ch, "body")
                        .map(|n| self.block(n, indent, c, 0))
                        .unwrap_or_default();
                    let catch_head = match param {
                        Some(p) => self.catch_param_head(p, indent, lparen_col),
                        None => Self::within('(', ')', pad, ""),
                    };
                    // The previous body's `}` sits at `ind(indent)`, so the
                    // newline + indent is all the fresh line needs.
                    if self.style.catch_on_new_line {
                        out.push('\n');
                        out.push_str(&self.ind(indent));
                    } else {
                        out.push_str(self.sp(self.style.space_before_catch_keyword));
                    }
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
                    if self.style.finally_on_new_line {
                        out.push('\n');
                        out.push_str(&self.ind(indent));
                    } else {
                        out.push_str(self.sp(self.style.space_before_finally_keyword));
                    }
                    out.push_str("finally");
                    out.push_str(self.sp(self.style.space_before_finally_lbrace));
                    out.push_str(&fbody);
                }
                _ => {}
            }
        }

        out
    }

    // ── try-with-resources resource-list layout (RESOURCE_LIST_WRAP) ────────

    /// Render the resource-list tail of a `try` statement (the gap before the
    /// parens plus the `(…)` list) canonically. CST shape (tree-sitter-java
    /// 0.23): the `resource_specification` field node includes its own
    /// parens — children are `(`, then alternating `resource` (named,
    /// one resource each) and `;` (anonymous) nodes, then `)`. Specs whose
    /// shape is not exactly that — extra children such as comments — fall
    /// back to the verbatim echo (R4).
    ///
    /// When the list must wrap, one resource goes per continuation line at
    /// `ind(indent + 1)` with `;` separators, mirroring `args_wrapped`'s four
    /// `(lparen_on_next_line, rparen_on_next_line)` paren layouts.
    fn resource_list(&self, spec: Node<'s>, indent: usize, c: usize) -> String {
        let gap = self.sp(self.style.space_before_try_parentheses);
        let pad = self.style.space_within_try_parentheses;
        let fallback = || {
            let t = self.txt(spec).trim();
            format!("{}{}", gap, pad_outer_parens(t, pad))
        };

        let resources: Vec<String> = match self.recognised_resource_spec(spec) {
            Some(res) if !res.is_empty() && !self.has_comment_subtree(spec) => {
                res.iter().map(|n| normalise_ws(self.txt(*n))).collect()
            }
            _ => return fallback(),
        };

        // The column of the opening `(`: `try` starts at `c` and the gap
        // precedes the parens.
        let paren_col = self.col_after(c, "try") + gap.len();
        let semi = format!(
            "{}{}{}",
            Self::sep(self.style.space_before_semicolon),
            ";",
            Self::sep(self.style.space_after_semicolon)
        );
        let flat = Self::within('(', ')', pad, &resources.join(&semi));

        let should_wrap = match self.style.resource_list_wrap {
            WrapStyle::DoNotWrap => false,
            WrapStyle::WrapAlways => resources.len() > 1,
            _ => resources.len() > 1 && !self.fits(paren_col, &flat),
        };
        if !should_wrap {
            return format!("{}{}", gap, flat);
        }

        let inner = indent + 1;
        let ind = self.ind(inner);
        let before = Self::sep(self.style.space_before_semicolon);
        let (lp, rp) = (
            self.style.resource_list_lparen_on_next_line,
            self.style.resource_list_rparen_on_next_line,
        );
        let line_of = |r: &str, last: bool| {
            let mut line = String::new();
            if !last {
                line.push_str(before);
                line.push(';');
            }
            format!("{}{}", r, line)
        };
        // `ALIGN_MULTILINE_RESOURCES`: in the arm that keeps the first
        // resource on the header line after `(` it is glued right after `(`
        // and the remaining lines pad to the column after `(`; the arms where
        // every resource begins its own line already share the first
        // resource's column and stay unchanged.
        let mut body = String::new();
        let lead_nl = lp || !rp;
        if !lp && rp && self.style.align_multiline_resources {
            let pref = self.align_prefix(paren_col + 1);
            body.push_str(&line_of(&resources[0], resources.len() == 1));
            for (i, r) in resources.iter().enumerate().skip(1) {
                body.push('\n');
                body.push_str(&pref);
                body.push_str(&line_of(r, i + 1 == resources.len()));
            }
        } else {
            let lines: Vec<String> = resources
                .iter()
                .enumerate()
                .map(|(i, r)| format!("{}{}", ind, line_of(r, i + 1 == resources.len())))
                .collect();
            body = lines.join("\n");
        }
        let tail = if rp {
            format!("\n{}", self.ind(indent))
        } else {
            String::new()
        };
        let wrapped = if lead_nl {
            Self::within('(', ')', pad, &format!("\n{}{}", body, tail))
        } else {
            Self::within('(', ')', pad, &format!("{}{}", body, tail))
        };
        format!("{}{}", gap, wrapped)
    }

    /// Extract the resource nodes of a `resource_specification` when its
    /// children match the canonical `( resource ; resource … )` shape; `None`
    /// for any other shape (comments, stray tokens) so callers can fall back
    /// to the verbatim echo (R4).
    fn recognised_resource_spec(&self, spec: Node<'s>) -> Option<Vec<Node<'s>>> {
        let ch = self.all_ch(spec);
        if ch.len() < 3 || self.txt(ch[0]) != "(" || self.txt(ch[ch.len() - 1]) != ")" {
            return None;
        }
        let mut out = Vec::new();
        for n in &ch[1..ch.len() - 1] {
            if n.is_named() {
                if n.kind() != "resource" {
                    return None;
                }
                out.push(*n);
            } else if self.txt(*n) != ";" {
                return None;
            }
        }
        Some(out)
    }

    /// True when `node`'s subtree contains a comment (line or block). Used to
    /// keep comments out of the canonical resource-list renderer (R4).
    fn has_comment_subtree(&self, node: Node<'s>) -> bool {
        let mut cur = node.walk();
        for ch in node.children(&mut cur) {
            if matches!(ch.kind(), "line_comment" | "block_comment") {
                return true;
            }
            if self.has_comment_subtree(ch) {
                return true;
            }
        }
        false
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
                if let Some(one) = self.one_line_body(*body, indent) {
                    let lt = self
                        .flat_keyword_cond(*lock, self.style.space_within_synchronized_parentheses);
                    if !lt.contains('\n') {
                        let candidate = format!(
                            "synchronized{}{}{}{}",
                            self.sp(self.style.space_before_synchronized_parentheses),
                            lt,
                            self.body_gap(&one, self.style.space_before_synchronized_lbrace),
                            one
                        );
                        if self.fits_lines(c, &candidate) {
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
        let wrap = self.style.assert_statement_wrap;
        match children.len() {
            0 => "assert;".to_string(),
            1 => {
                let e = children[0];
                let flat = format!("assert {};", self.flat(e));
                // DoNotWrap (and the default style) keep today's one-line
                // output; the statement only wraps per `ASSERT_STATEMENT_WRAP`.
                if wrap == WrapStyle::DoNotWrap
                    || (wrap != WrapStyle::WrapAlways && self.fits(c, &flat))
                {
                    return format!("assert {};", self.expr(e, indent, c + 7));
                }
                let cont = self.cont(indent);
                let cont_col = self.col_after(0, &cont);
                format!(
                    "assert\n{}{};",
                    cont,
                    self.assert_side(e, indent, cont_col, wrap)
                )
            }
            _ => {
                let e0 = children[0];
                let e1 = children[1];
                let flat = format!("assert {} : {};", self.flat(e0), self.flat(e1));
                if wrap == WrapStyle::DoNotWrap
                    || (wrap != WrapStyle::WrapAlways && self.fits(c, &flat))
                {
                    return format!(
                        "assert {} : {};",
                        self.expr(e0, indent, c + 7),
                        self.expr(e1, indent, c)
                    );
                }
                let cont = self.cont(indent);
                let cont_col = self.col_after(0, &cont);
                let s0 = self.assert_side(e0, indent, c + 7, wrap);
                let s1 = self.assert_side(e1, indent, cont_col, wrap);
                if self.style.assert_statement_colon_on_next_line {
                    // The `:` starts the continuation line; the before half of
                    // the separator is dropped (R5).
                    format!(
                        "assert {}\n{}{}{}{};",
                        s0,
                        cont,
                        ":",
                        Self::sep(self.style.space_after_colon),
                        s1
                    )
                } else {
                    // Default: the `:` ends the expression's line.
                    format!(
                        "assert {}{}{}\n{}{};",
                        s0,
                        Self::sep(self.style.space_before_colon),
                        ":",
                        cont,
                        s1
                    )
                }
            }
        }
    }

    /// Render one side of a broken assert statement. `ChopDownIfLong`
    /// recurses through `expr` so an overflowing expression side can wrap
    /// internally; the other styles keep the side flat.
    fn assert_side(&self, n: Node<'s>, indent: usize, c: usize, wrap: WrapStyle) -> String {
        if wrap == WrapStyle::ChopDownIfLong {
            self.expr(n, indent, c)
        } else {
            self.flat(n).to_string()
        }
    }

    /// Multi-line layout for a `switch_expression` node — tree-sitter-java
    /// 0.23 represents both the switch statement and the switch expression
    /// with this single kind. Renders `switch (cond) {` on the header line,
    /// `case`/`default` labels indented one level and their statements a
    /// further level, and the closing `}` at the statement indent, matching
    /// IntelliJ's default switch layout. The case-layout options
    /// (`INDENT_CASE_FROM_SWITCH`, `CASE_STATEMENT_ON_NEW_LINE`,
    /// `INDENT_BREAK_FROM_CASE`) govern the layout; `is_value` is true when
    /// the switch is used as a value (see [`Self::switch_expr`]), which lets
    /// [`Self::switch_rule`] honour the `SWITCH_EXPRESSIONS_WRAP` chop-down
    /// behaviour for overflowing nested switch expressions. Any unmodelled
    /// shape falls back to the verbatim source echo (R4).
    fn switch_stmt(&self, node: Node<'s>, indent: usize, c: usize, is_value: bool) -> String {
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

        // `INDENT_CASE_FROM_SWITCH` (default on): labels one level below the
        // `switch`; off, labels sit at the switch indent. Statements are
        // always one further level.
        let label_level = if self.style.indent_case_from_switch {
            indent + 1
        } else {
            indent
        };
        let statement_level = label_level + 1;
        let mut out = format!("switch{}{}{}{{\n", p_gap, cond, l_gap);

        for ch in self.named(body) {
            match ch.kind() {
                "switch_block_statement_group" => {
                    self.switch_group(ch, label_level, statement_level, &mut out)
                }
                "switch_rule" => self.switch_rule(ch, label_level, is_value, &mut out),
                // Comments and any other stray nodes keep their text; `comment`
                // renders the full line(s), indented to the label level unless
                // a column-1 option applies (R4).
                _ => {
                    out.push_str(&self.comment(ch, label_level));
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
    /// statements one indent level deeper. `label_level` is the indent level
    /// of the labels, `statement_level` of their statements. With
    /// `CASE_STATEMENT_ON_NEW_LINE` off the group's first (single-line)
    /// statement is joined onto the last label's line (`case 1: foo();`);
    /// with `INDENT_BREAK_FROM_CASE` off `break` / `continue` / `return`
    /// statements render at the label level.
    fn switch_group(
        &self,
        node: Node<'s>,
        label_level: usize,
        statement_level: usize,
        out: &mut String,
    ) {
        let inline_first = !self.style.case_statement_on_new_line;
        // The current line ends with a `label:` that has not yet been closed.
        let mut open_label = false;
        for ch in self.named(node) {
            if ch.kind() == "switch_label" {
                if open_label {
                    out.push('\n');
                }
                out.push_str(&self.ind(label_level));
                out.push_str(self.txt(ch));
                out.push(':');
                open_label = true;
            } else if self.is_comment_node(ch) {
                if open_label {
                    out.push('\n');
                    open_label = false;
                }
                // Comments inside a case group render through the comment
                // helper (full line, column-1 / space / wrap options); the
                // statement indent prefix is skipped here.
                out.push_str(&self.comment(ch, statement_level));
                out.push('\n');
            } else {
                // `INDENT_BREAK_FROM_CASE` off: the jump statements line up
                // with the case label instead of the statement indent.
                let level = if !self.style.indent_break_from_case
                    && matches!(
                        ch.kind(),
                        "break_statement" | "continue_statement" | "return_statement"
                    ) {
                    label_level
                } else {
                    statement_level
                };
                let sc = self.col_after(0, &self.ind(level));
                // `CASE_STATEMENT_ON_NEW_LINE` off joins the group's first
                // statement onto the label line (single-line bodies only; a
                // body that itself wraps keeps its own line).
                if open_label && inline_first && level == statement_level {
                    let s = self.stmt(ch, level, sc);
                    if !s.contains('\n') {
                        out.push(' ');
                        out.push_str(&s);
                        out.push('\n');
                        open_label = false;
                        continue;
                    }
                }
                if open_label {
                    out.push('\n');
                    open_label = false;
                }
                out.push_str(&self.ind(level));
                out.push_str(&self.stmt(ch, level, sc));
                out.push('\n');
            }
        }
        if open_label {
            out.push('\n');
        }
    }

    /// Lay out one arrow-form rule (`switch_rule`): `case X -> body` with an
    /// inline expression/throw body or a block body. If the body would wrap
    /// (render with a newline), the whole rule is echoed verbatim (R4) rather
    /// than producing a misaligned continuation — except that a switch
    /// expression used as a value (`is_value`) under
    /// `SWITCH_EXPRESSIONS_WRAP` = chop-down (`5`) breaks an overflowing
    /// nested switch-expression body into its (self-aligned) multi-line
    /// layout instead.
    fn switch_rule(&self, node: Node<'s>, indent: usize, is_value: bool, out: &mut String) {
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
                    let chop_nested = is_value
                        && self.style.switch_expressions_wrap == WrapStyle::ChopDownIfLong
                        && b.kind() == "expression_statement"
                        && self.named(b).first().map(|n| n.kind()) == Some("switch_expression");
                    if chop_nested {
                        out.push_str(&head);
                        out.push_str(&s);
                        out.push('\n');
                    } else {
                        out.push_str(&self.ind(indent));
                        out.push_str(self.txt(node));
                        out.push('\n');
                    }
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
    /// RHS, return value, argument) per `SWITCH_EXPRESSIONS_WRAP`: `0`
    /// (DoNotWrap) keeps the single-line form whenever one exists, `1`
    /// (WrapIfLong, the default) and `5` (ChopDownIfLong) use it only when it
    /// fits the current column, and `2` (WrapAlways) always uses the
    /// multi-line [`Self::switch_stmt`] layout. `5` additionally breaks an
    /// overflowing nested switch expression in the body (see
    /// [`Self::switch_rule`]).
    fn switch_expr(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        match self.style.switch_expressions_wrap {
            WrapStyle::DoNotWrap => {
                if let Some(one) = self.switch_one_line(node) {
                    return one;
                }
            }
            WrapStyle::WrapAlways => {}
            _ => {
                if let Some(one) = self.switch_one_line(node) {
                    if self.fits(c, &one) {
                        return one;
                    }
                }
            }
        }
        self.switch_stmt(node, indent, c, true)
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

    /// Render a `string_literal` node, honouring the Text-block options of
    /// java.md. A text block is a `string_literal` whose text spans lines
    /// (tree-sitter-java gives no separate node kind); only those are
    /// touched — ordinary strings and single-line text blocks are echoed
    /// verbatim (R4).
    ///
    /// `STRIP_WHITESPACE_FROM_BLANK_LINES_IN_TEXT_BLOCKS` trims every
    /// whitespace-only line inside the literal to empty; a blank line's
    /// whitespace is never part of the text-block value (the incidental-
    /// whitespace algorithm excludes blank lines), so the deviation is
    /// value-safe and limited to layout. It applies wherever the literal is
    /// rendered.
    ///
    /// `ALIGN_MULTILINE_TEXT_BLOCKS` applies on the expression path only
    /// (`align` true), where the statement's indent level is known: every
    /// non-opening line that carries visible content — the content lines and
    /// the closing-delimiter line — shifts by one uniform delta so the first
    /// content line sits at the canonical continuation column
    /// (`col_after(0, cont(indent))`, the column the formatter's own
    /// continuation lines use). A uniform shift preserves relative
    /// indentation and moves the incidental-whitespace minimum with the
    /// lines, so the stripped string value is unchanged (R5); whitespace-only
    /// lines are left in place, and when a shift left would cut into a
    /// visible line's own leading whitespace (fewer spaces than the delta,
    /// or a tab in the leading run) the renderer falls back to the verbatim
    /// echo rather than alter the value. Both options default `false`, so
    /// absent schemes keep today's byte-for-byte echo; each transform
    /// re-applies to its own output as a no-op (R6).
    fn string_literal(&self, node: Node<'s>, indent: usize, align: bool) -> String {
        let t = self.txt(node);
        let strip = self.style.strip_whitespace_from_blank_lines_in_text_blocks;
        let do_align = align && self.style.align_multiline_text_blocks;
        if !t.contains('\n') || (!strip && !do_align) {
            return t.to_string();
        }
        // A trailing `\r` per line is CRLF line-ending noise that the
        // finalisation pass collapses anyway; drop it so blank-line
        // detection and column arithmetic see plain `\n` lines.
        let mut lines: Vec<String> = t
            .split('\n')
            .map(|l| l.trim_end_matches('\r').to_string())
            .collect();
        if strip {
            for l in &mut lines {
                if l.trim().is_empty() {
                    l.clear();
                }
            }
        }
        if do_align {
            // The canonical continuation column: the block's content lines
            // sit where the formatter's own continuation lines would.
            let target = self.col_after(0, &self.cont(indent));
            // Line 0 holds the opening delimiter (it is glued onto the
            // statement's line and never shifts); every later line that
            // carries visible content shifts with the same delta.
            let visible: Vec<usize> = (1..lines.len())
                .filter(|&i| !lines[i].trim().is_empty())
                .collect();
            let Some(&anchor) = visible.first() else {
                return lines.join("\n");
            };
            let anchor_col = lines[anchor].chars().take_while(|&c| c == ' ').count();
            let delta = target as i64 - anchor_col as i64;
            if delta > 0 {
                let pad = " ".repeat(delta as usize);
                for &i in &visible {
                    lines[i] = format!("{}{}", pad, lines[i]);
                }
            } else if delta < 0 {
                let cut = (-delta) as usize;
                for &i in &visible {
                    let lead = lines[i].chars().take_while(|&c| c == ' ').count();
                    // Cutting past a visible line's own leading whitespace —
                    // or into a tab run — would alter the literal's value:
                    // fall back to the verbatim echo.
                    if lead < cut || lines[i].as_bytes().get(lead) == Some(&b'\t') {
                        return t.to_string();
                    }
                }
                for &i in &visible {
                    lines[i] = lines[i][cut..].to_string();
                }
            }
        }
        lines.join("\n")
    }

    fn expr(&self, node: Node<'s>, indent: usize, c: usize) -> String {
        self.expr_ac(node, indent, c, None)
    }

    /// [`Self::expr`] carrying an inherited alignment column (`acol`), set by
    /// an enclosing wrapped assignment (`ALIGN_MULTILINE_ASSIGNMENT`) or
    /// parenthesized expression (`ALIGN_MULTILINE_PARENTHESIZED_EXPRESSION`):
    /// continuation lines of nested binary / ternary / chained-call
    /// expressions pad to that column when the nested construct's own align
    /// option is off (its own option wins when on).
    fn expr_ac(&self, node: Node<'s>, indent: usize, c: usize, acol: Option<usize>) -> String {
        if node.is_extra() {
            if matches!(node.kind(), "line_comment" | "block_comment") {
                return self.comment(node, indent);
            }
            return self.txt(node).to_string();
        }
        match node.kind() {
            "method_invocation" => self.method_inv_ac(node, indent, c, acol),
            "object_creation_expression" => self.new_expr(node, indent, c),
            "field_access" => self.field_access(node, indent, c),
            "array_access" => {
                let arr = self
                    .fld(node, "array")
                    .map(|n| self.expr_ac(n, indent, c, acol))
                    .unwrap_or_default();
                let idx = self
                    .fld(node, "index")
                    .map(|n| self.expr_ac(n, indent, c, acol))
                    .unwrap_or_default();
                format!(
                    "{}{}",
                    arr,
                    Self::within('[', ']', self.style.space_within_brackets, &idx)
                )
            }
            "assignment_expression" => self.assignment(node, indent, c, acol),
            "binary_expression" => self.binary_ac(node, indent, c, acol),
            "unary_expression" => {
                let op = self
                    .fld(node, "operator")
                    .map(|n| self.txt(n))
                    .unwrap_or("");
                let sep = Self::sep(self.style.space_around_unary_operator);
                let operand = self
                    .fld(node, "operand")
                    .map(|n| self.expr_ac(n, indent, c + op.len() + sep.len(), acol))
                    .unwrap_or_default();
                format!("{}{}{}", op, sep, operand)
            }
            "update_expression" => self.update_expr(node, indent, c, false),
            "ternary_expression" => self.ternary_ac(node, indent, c, acol),
            "cast_expression" => {
                let ty = self
                    .fld(node, "type")
                    .map(|n| self.flat_type(n))
                    .unwrap_or_default();
                let pad_ty = Self::within('(', ')', self.style.space_within_cast_parentheses, &ty);
                let sep = Self::sep(self.style.space_after_type_cast);
                let val = self
                    .fld(node, "value")
                    .map(|n| self.expr_ac(n, indent, c + pad_ty.len() + sep.len(), acol))
                    .unwrap_or_default();
                format!("{}{}{}", pad_ty, sep, val)
            }
            "instanceof_expression" => {
                let left = self
                    .fld(node, "left")
                    .map(|n| self.expr_ac(n, indent, c, acol))
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
                // `ALIGN_MULTILINE_PARENTHESIZED_EXPRESSION`: when the inner
                // expression wraps, its continuation lines align to the
                // column right after `(` instead of the continuation indent
                // (the record-header model). The inherited `acol` flows
                // through unchanged when the option is off.
                let inner_acol = if self.style.align_multiline_parenthesized_expression {
                    Some(c + 1)
                } else {
                    acol
                };
                let inner = node
                    .named_child(0)
                    .map(|n| self.expr_ac(n, indent, c + 1, inner_acol))
                    .unwrap_or_default();
                let pad = self.style.space_within_parentheses;
                if inner.contains('\n') {
                    // `PARENTHESES_EXPRESSION_LPAREN/RPAREN_WRAP`: when the
                    // inner expression wraps, the parens move to their own
                    // lines (the inner sits at the continuation indent, the
                    // `)` at the statement indent); the padding toggle stays
                    // away from the newlines, matching `within`.
                    let lparen = self.style.parentheses_expression_lparen_wrap;
                    let rparen = self.style.parentheses_expression_rparen_wrap;
                    let head = if lparen {
                        format!("(\n{}", self.cont(indent))
                    } else if pad && !inner.starts_with('\n') {
                        "( ".to_string()
                    } else {
                        "(".to_string()
                    };
                    let tail = if rparen {
                        format!("\n{})", self.ind(indent))
                    } else if pad && !inner.ends_with('\n') {
                        " )".to_string()
                    } else {
                        ")".to_string()
                    };
                    format!("{}{}{}", head, inner, tail)
                } else {
                    Self::within('(', ')', pad, &inner)
                }
            }
            "array_creation_expression" => self.array_creation(node, indent, c),
            "array_initializer" => self.array_init(node, indent, c, c),
            "switch_expression" => self.switch_expr(node, indent, c),
            "string_literal" => self.string_literal(node, indent, true),
            _ => self.txt(node).to_string(),
        }
    }

    // ── method invocation + chain ─────────────────────────────────────────────

    /// The wrapped rendering of an invocation, with an inherited alignment
    /// column (see [`Self::binary_ac`]).
    fn method_inv_ac(
        &self,
        node: Node<'s>,
        indent: usize,
        c: usize,
        acol: Option<usize>,
    ) -> String {
        let flat = self.flat_inv(node);
        let keep = self.keep_wrapped(node) || self.args_keep_wrapped(node);

        if !keep && self.fits(c, &flat) {
            return flat;
        }

        // Detect chain
        if keep || self.style.method_call_chain_wrap != WrapStyle::DoNotWrap {
            let (base, links) = self.collect_chain(node);
            if links.len() >= 2 {
                // PREFER_PARAMETERS_WRAP: when the option is on and the tail
                // call's argument list would itself wrap (either it overflows
                // or its source lines are kept), wrap the arguments instead of
                // breaking the surrounding chain. Applied even when `keep` is
                // set so reformatting wrapped-argument output is a no-op.
                if self.style.prefer_parameters_wrap
                    && self.style.call_parameters_wrap != WrapStyle::DoNotWrap
                    && (self.args_keep_wrapped(node) || self.overflowing_args(node, indent, c))
                {
                    return self.inv_wrapped(node, indent, c);
                }
                return self.fmt_chain_ac(
                    &base,
                    &links,
                    indent,
                    c,
                    acol,
                    self.is_builder_chain(&links),
                );
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
            "{}{}{}{}{}{}",
            obj,
            ta,
            self.type_args_gap(&ta, name),
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
        let prefix = format!(
            "{}{}{}{}{}",
            obj,
            ta,
            self.type_args_gap(&ta, name),
            name,
            gap
        );
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
                    let chain_str =
                        self.fmt_chain(&base, &links, indent, c + 1, self.is_builder_chain(&links));
                    return format!("({})", chain_str);
                }
            }
        }

        let wrap = self.style.call_parameters_wrap;
        if !keep && wrap == WrapStyle::DoNotWrap {
            return flat;
        }

        let inner = indent + 1;
        // `CALL_PARAMETER_INDENT`: an explicit width overrides the
        // continuation indent for call arguments only; `-1` (default)
        // inherits today's `ind(inner)` byte-for-byte.
        let ind = self.construct_ind(indent, self.style.call_parameter_indent, &self.ind(inner));
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
        // `ALIGN_MULTILINE_PARAMETERS_IN_CALLS`: in the arm that keeps the
        // first argument on the header line after `(`, the first argument is
        // glued right after `(` and the remaining argument lines pad to the
        // column after `(` (the record-header model); the arms where every
        // argument begins its own line already share the first argument's
        // column and stay unchanged.
        if !lp && rp && self.style.align_multiline_parameters_in_calls {
            let pref = self.align_prefix(c + 1);
            let mut body = self.expr(args[0], inner, c + 1);
            for &a in args.iter().skip(1) {
                body.push_str(",\n");
                body.push_str(&pref);
                body.push_str(&self.expr(a, inner, c + 1));
            }
            let inner_txt = format!("{}\n{}", body, self.ind(indent));
            return Self::within('(', ')', pad, &inner_txt);
        }
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

    // True when this invocation's argument list would wrap under
    // `args_wrapped` (the flat list overflows from the `(` column). Used by
    // `method_inv` to honour `PREFER_PARAMETERS_WRAP`.
    fn overflowing_args(&self, node: Node<'s>, indent: usize, c: usize) -> bool {
        let Some(a) = self.fld(node, "arguments") else {
            return false;
        };
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
        let prefix = format!(
            "{}{}{}{}{}",
            obj,
            ta,
            self.type_args_gap(&ta, name),
            name,
            gap
        );
        let args_col = self.col_after(c, &prefix);
        !self.fits(args_col, &self.flat_args(a))
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

    fn fmt_chain(
        &self,
        base: &str,
        links: &[Link<'s>],
        indent: usize,
        c: usize,
        builder: bool,
    ) -> String {
        self.fmt_chain_ac(base, links, indent, c, None, builder)
    }

    /// True when `links` is a builder chain: `style.builder_methods` is
    /// non-empty and every link's method name is in it (IntelliJ's
    /// whole-chain-of-builder-methods rule, matching the split/trimmed
    /// names). Default / absent schemes (empty list) never match.
    fn is_builder_chain(&self, links: &[Link<'s>]) -> bool {
        !self.style.builder_methods.is_empty()
            && links.iter().all(|l| {
                self.style
                    .builder_methods
                    .iter()
                    .any(|m| m.as_str() == self.txt(l.name))
            })
    }

    /// [`Self::fmt_chain`] with an inherited alignment column (see
    /// [`Self::binary_ac`]): the wrapped link lines pad to the first link's
    /// dot column when `ALIGN_MULTILINE_CHAINED_METHODS` is on, else to the
    /// inherited `acol` when the chain sits inside an aligned assignment or
    /// parenthesized expression. `builder` selects the `BUILDER_METHODS`
    /// layout: break after the base so every `.call()` — including the
    /// first — starts its own line, at the continuation indent or, with
    /// `KEEP_BUILDER_METHODS_INDENTS`, at the chain's own indent.
    fn fmt_chain_ac(
        &self,
        base: &str,
        links: &[Link<'s>],
        indent: usize,
        c: usize,
        acol: Option<usize>,
        builder: bool,
    ) -> String {
        // `CHAINED_CALL_INDENT`: an explicit width overrides the continuation
        // indent for the chain's link lines only; `-1` (default) inherits
        // today's `cont(indent)` byte-for-byte. The builder layout composes
        // with it via `cont` when stepping the continuation indent
        // (`KEEP_BUILDER_METHODS_INDENTS` off); the keep-indents layout uses
        // the chain's own indent, where no continuation width applies.
        let cont = self.construct_ind(indent, self.style.chained_call_indent, &self.cont(indent));
        let gap = self.sp(self.style.space_before_method_call_parentheses);
        if builder {
            let link_ind = if self.style.keep_builder_methods_indents {
                self.ind(indent)
            } else {
                cont.clone()
            };
            let mut out = String::new();
            for (i, link) in links.iter().enumerate() {
                let ta = link
                    .type_args
                    .map(|n| self.flat_type_args(n))
                    .unwrap_or_default();
                let nm = self.txt(link.name);
                let flat_a = self.flat_args(link.args);
                if i == 0 {
                    // With an empty base the first link opens the header line
                    // as the generic layout does; otherwise the base ends its
                    // own line and the first call starts the builder lines.
                    if base.is_empty() {
                        out = format!(
                            "{}{}{}{}{}",
                            ta,
                            self.type_args_gap(&ta, nm),
                            nm,
                            gap,
                            flat_a
                        );
                    } else {
                        out = format!(
                            "{}\n{}.{}{}{}{}{}",
                            base,
                            link_ind,
                            ta,
                            self.type_args_gap(&ta, nm),
                            nm,
                            gap,
                            flat_a
                        );
                    }
                } else {
                    out.push('\n');
                    out.push_str(&link_ind);
                    out.push('.');
                    out.push_str(&ta);
                    out.push_str(self.type_args_gap(&ta, nm));
                    out.push_str(nm);
                    out.push_str(gap);
                    out.push_str(&flat_a);
                }
            }
            return out;
        }
        let mut out = String::new();
        // `WRAP_FIRST_METHOD_IN_CALL_CHAIN`: the first link also starts a
        // continuation line. With an empty base (a chain without an explicit
        // receiver) there is nothing on the header line to wrap after, so the
        // first link stays where it is.
        let first_next = self.style.wrap_first_method_in_call_chain && !base.is_empty();
        // Continuation prefix for the link lines: the alignment column is the
        // first link's dot column when it stays on the header line.
        let link_pref: String = if !first_next && !base.is_empty() {
            if self.style.align_multiline_chained_methods {
                self.align_prefix(self.col_after(c, base))
            } else if let Some(a) = acol {
                self.align_prefix(a)
            } else {
                cont.clone()
            }
        } else {
            cont.clone()
        };

        for (i, link) in links.iter().enumerate() {
            let ta = link
                .type_args
                .map(|n| self.flat_type_args(n))
                .unwrap_or_default();
            let nm = self.txt(link.name);
            let flat_a = self.flat_args(link.args);

            if i == 0 {
                if base.is_empty() {
                    out = format!(
                        "{}{}{}{}{}",
                        ta,
                        self.type_args_gap(&ta, nm),
                        nm,
                        gap,
                        flat_a
                    );
                } else if first_next {
                    out = format!(
                        "{}\n{}.{}{}{}{}{}",
                        base,
                        cont,
                        ta,
                        self.type_args_gap(&ta, nm),
                        nm,
                        gap,
                        flat_a
                    );
                } else {
                    out = format!(
                        "{}.{}{}{}{}{}",
                        base,
                        ta,
                        self.type_args_gap(&ta, nm),
                        nm,
                        gap,
                        flat_a
                    );
                }
            } else {
                out.push('\n');
                out.push_str(&link_pref);
                out.push('.');
                out.push_str(&ta);
                out.push_str(self.type_args_gap(&ta, nm));
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
        let prefix = format!("new {}{}{}", ta, self.type_args_gap(&ta, &ty), ty);
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

    fn assignment(&self, node: Node<'s>, indent: usize, c: usize, acol: Option<usize>) -> String {
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
                self.assign_expr(r, indent, c, &left, op, self.keep_wrapped(node), acol)
            }
            _ => {
                let sep = self.op_sep(op);
                let right = rhs
                    .map(|n| {
                        let rc = c + left.len() + sep.len() + op.len() + sep.len();
                        self.expr_ac(n, indent, rc, acol)
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
    /// continuation indent. `c` is the column where `prefix` begins. `acol` is
    /// an alignment column inherited from an enclosing aligned construct.
    #[allow(clippy::too_many_arguments)]
    fn assign_expr(
        &self,
        rhs: Node<'s>,
        indent: usize,
        c: usize,
        prefix: &str,
        op: &str,
        keep: bool,
        acol: Option<usize>,
    ) -> String {
        let sep = self.op_sep(op);
        let rhs_col = self.col_after(c, prefix) + sep.len() + op.len() + sep.len();
        // `ALIGN_MULTILINE_ASSIGNMENT`: when on, the RHS — and the
        // continuation lines of anything nested inside it whose own align
        // option is off — aligns to the column where the RHS would sit right
        // after the operator on the header line (the record-header model).
        let eff_acol = if self.style.align_multiline_assignment {
            Some(rhs_col)
        } else {
            acol
        };
        let same = self.expr_ac(rhs, indent, rhs_col, eff_acol);

        // The RHS wrapped internally; leave the operator on the header line.
        if same.contains('\n') {
            return Self::join_sep(&format!("{}{}{}", prefix, sep, op), sep, &same);
        }

        if keep {
            // `KEEP_LINE_BREAKS`: the initialiser's source spans rows, so the
            // RHS moves to the continuation line after the operator even
            // though the flat form fits.
            let cont = self.cont(indent);
            if self.style.place_assignment_sign_on_next_line {
                // `PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE`: the operator starts
                // the continuation line (the layout keeps the continuation
                // indent; see the sign-on-next-line goldens).
                let r = self.expr_ac(rhs, indent, self.col_after(0, &cont), None);
                return format!("{}\n{}{}{}{}", prefix, cont, op, sep, r);
            }
            let pref = if self.style.align_multiline_assignment {
                self.align_prefix(rhs_col)
            } else {
                cont.clone()
            };
            let r = self.expr_ac(rhs, indent, self.col_after(0, &cont), eff_acol);
            return format!("{}{}{}\n{}{}", prefix, sep, op, pref, r);
        }

        let flat = format!("{}{}{}{}{}", prefix, sep, op, sep, same);
        if self.style.assignment_wrap != WrapStyle::WrapAlways && self.fits(c, &flat) {
            return flat;
        }

        let cont = self.cont(indent);
        if self.style.place_assignment_sign_on_next_line {
            // `PLACE_ASSIGNMENT_SIGN_ON_NEXT_LINE`: the operator starts
            // the continuation line (keeps the continuation indent; see the
            // sign-on-next-line goldens).
            let r = self.expr_ac(rhs, indent, self.col_after(0, &cont), None);
            format!("{}\n{}{}{}{}", prefix, cont, op, sep, r)
        } else {
            let pref = if self.style.align_multiline_assignment {
                self.align_prefix(rhs_col)
            } else {
                cont.clone()
            };
            let r = self.expr_ac(rhs, indent, self.col_after(0, &cont), eff_acol);
            format!("{}{}{}\n{}{}", prefix, sep, op, pref, r)
        }
    }

    fn binary_ac(&self, node: Node<'s>, indent: usize, c: usize, acol: Option<usize>) -> String {
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

        let (cont, cont_col) = if self.style.align_multiline_binary_operation {
            // `ALIGN_MULTILINE_BINARY_OPERATION`: continuation lines start at
            // the first operand's column instead of the continuation indent.
            let p = self.align_prefix(c);
            (p, c)
        } else if let Some(a) = acol {
            let p = self.align_prefix(a);
            (p, a)
        } else {
            let cont = self.cont(indent);
            let cont_col = self.col_after(0, &cont);
            (cont, cont_col)
        };
        let sign_next = self.style.binary_operation_sign_on_next_line;
        let mut out = self.binary_operand(operands[0], indent, c, wrap, acol);
        for i in 1..operands.len() {
            let op = &ops[i - 1];
            let sep = self.op_sep(op);
            let operand = self.binary_operand(operands[i], indent, cont_col, wrap, acol);
            if sign_next {
                // `BINARY_OPERATION_SIGN_ON_NEXT_LINE`: the operator starts
                // the continuation line.
                out.push('\n');
                out.push_str(&cont);
                out.push_str(op);
                out.push_str(sep);
                out.push_str(&operand);
            } else {
                // Default: the operator ends the preceding line.
                out.push_str(sep);
                out.push_str(op);
                out.push('\n');
                out.push_str(&cont);
                out.push_str(&operand);
            }
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
    fn binary_operand(
        &self,
        n: Node<'s>,
        indent: usize,
        c: usize,
        wrap: WrapStyle,
        acol: Option<usize>,
    ) -> String {
        if wrap == WrapStyle::ChopDownIfLong && n.kind() == "binary_expression" {
            self.binary_ac(n, indent, c, acol)
        } else {
            self.flat(n).to_string()
        }
    }

    /// [`Self::ternary_ac`] carries an inherited alignment column (see
    /// [`Self::binary_ac`]):
    fn ternary_ac(&self, node: Node<'s>, indent: usize, c: usize, acol: Option<usize>) -> String {
        let wrap = self.style.ternary_operation_wrap;
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

        // DoNotWrap (and the default style) keep today's single-line output;
        // `KEEP_LINE_BREAKS` overrides that when the expression's source
        // spans rows (the same rule as `binary`).
        if !self.keep_wrapped(node)
            && (wrap == WrapStyle::DoNotWrap
                || (wrap != WrapStyle::WrapAlways && self.fits(c, &flat)))
        {
            return flat;
        }

        let (cont, cont_col) = if self.style.align_multiline_ternary_operation {
            // `ALIGN_MULTILINE_TERNARY_OPERATION`: the `?` / `:` continuation
            // lines start at the condition's column.
            let p = self.align_prefix(c);
            (p, c)
        } else if let Some(a) = acol {
            let p = self.align_prefix(a);
            (p, a)
        } else {
            let cont = self.cont(indent);
            let cont_col = self.col_after(0, &cont);
            (cont, cont_col)
        };
        // `TERNARY_OPERATION_SIGNS_ON_NEXT_LINE` steers the `?` / `:` between
        // the two wrapped layouts; the sign halves of the separators that face
        // the newline are dropped so no trailing / leading whitespace appears
        // (R5).
        let signs_next = self.style.ternary_operation_signs_on_next_line;
        let cond = self.ternary_operand(self.fld(node, "condition"), indent, c, wrap, acol);
        let cons =
            self.ternary_operand(self.fld(node, "consequence"), indent, cont_col, wrap, acol);
        let alt = self.ternary_operand(self.fld(node, "alternative"), indent, cont_col, wrap, acol);
        if signs_next {
            // The `?` / `:` start the continuation lines (the layout shipped
            // before the wrap option existed).
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
        } else {
            // Default: the `?` / `:` end the preceding line, consistent with
            // the binary operator-end layout.
            format!(
                "{}{}{}\n{}{}{}{}\n{}{}",
                cond,
                Self::sep(self.style.space_before_quest),
                "?",
                cont,
                cons,
                Self::sep(self.style.space_before_colon),
                ":",
                cont,
                alt
            )
        }
    }

    /// Render one side of a broken ternary. `ChopDownIfLong` recurses into a
    /// side that is itself a ternary expression so a long nested ternary can
    /// wrap further; the other styles keep the sides flat (mirrors
    /// [`Self::binary_operand`]).
    fn ternary_operand(
        &self,
        n: Option<Node<'s>>,
        indent: usize,
        c: usize,
        wrap: WrapStyle,
        acol: Option<usize>,
    ) -> String {
        match n {
            Some(n) if wrap == WrapStyle::ChopDownIfLong && n.kind() == "ternary_expression" => {
                self.ternary_ac(n, indent, c, acol)
            }
            Some(n) => self.flat(n).to_string(),
            None => String::new(),
        }
    }

    fn lambda(&self, node: Node<'s>, indent: usize, c: usize) -> String {
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

        let body_node = self
            .fld(node, "body")
            .unwrap_or_else(|| node.named_child(1).unwrap());
        let sep = Self::sep(self.style.space_around_lambda_arrow);
        let arrow_col = sep.len() + 2 + sep.len();
        // Whether the lambda's configured brace style keeps the opening brace
        // on the `->` line, i.e. a simple block may be kept on one line.
        let inline_lbrace = matches!(
            self.style.lambda_brace_style,
            BraceStyle::EndOfLine | BraceStyle::NextLineIfWrapped
        );
        let body = if body_node.kind() == "block" {
            // check keep_simple
            let stmts = self.named(body_node);
            let inline_ok = inline_lbrace && self.style.keep_simple_lambdas_in_one_line;
            let collapsed = if inline_ok {
                if stmts.is_empty() {
                    // An empty block keeps `flat_block`'s pinned `{}` / `{ }`.
                    let flat = self.flat_block(body_node);
                    if self.fits_lines(c + params.len() + arrow_col, &flat) {
                        Some(flat)
                    } else {
                        None
                    }
                } else {
                    // One-line body presentation: `SPACES_INSIDE_BLOCK_BRACES_`
                    // `WHEN_BODY_IS_PRESENT` (padded / flush) and
                    // `NEW_LINE_WHEN_BODY_IS_PRESENTED` (block on its own
                    // line) apply; the statements are joined like
                    // `flat_block`'s inner text.
                    let inner = stmts
                        .iter()
                        .map(|&s| self.flat(s))
                        .collect::<Vec<_>>()
                        .join("; ");
                    let presented = self.present_block(&inner, indent);
                    if self.fits_lines(c + params.len() + arrow_col, &presented) {
                        Some(presented)
                    } else {
                        None
                    }
                }
            } else {
                None
            };
            match collapsed {
                Some(p) => p,
                None => {
                    let block_str = self.block(body_node, indent, c, 0);
                    // `LAMBDA_BRACE_STYLE`: the NextLine family puts the `{` on
                    // its own line at the statement indent (same arms as
                    // `brace_before_body` / `with_brace`); the arrow's trailing
                    // `sep` then has nothing to join and is dropped.
                    match self.style.lambda_brace_style {
                        BraceStyle::NextLine
                        | BraceStyle::NextLineShifted
                        | BraceStyle::NextLineShifted2 => {
                            format!("\n{}{}", self.ind(indent), block_str)
                        }
                        _ => block_str,
                    }
                }
            }
        } else {
            self.expr(body_node, indent, c + params.len() + arrow_col)
        };

        if body.starts_with('\n') {
            format!("{}{}->{}", params, sep, body)
        } else {
            format!("{}{}->{}{}", params, sep, sep, body)
        }
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
                // Column of the initialiser's `{` within the physical line,
                // for the array-initializer alignment.
                let brace_col = c
                    + self.col_after(0, &format!("new {}{}", ty, dims.join("")))
                    + self
                        .sp(self.style.space_before_array_initializer_lbrace)
                        .len();
                format!(
                    "{}{}",
                    self.sp(self.style.space_before_array_initializer_lbrace),
                    self.array_init(n, indent, c, brace_col)
                )
            })
            .unwrap_or_default();
        format!("new {}{}{}", ty, dims.join(""), init)
    }

    fn array_init(&self, node: Node<'s>, indent: usize, c: usize, brace_col: usize) -> String {
        let wrap = self.style.array_initializer_wrap;
        let flat = self.flat_arr_init(node);

        // DoNotWrap (and the default style) keep today's single-line output;
        // `KEEP_LINE_BREAKS` overrides that when the initialiser's source
        // spans rows (the same rule as `binary`).
        if !self.keep_wrapped(node)
            && (wrap == WrapStyle::DoNotWrap
                || (wrap != WrapStyle::WrapAlways && self.fits(c, &flat)))
        {
            return flat;
        }
        let inner = indent + 1;
        // `ARRAY_ELEMENT_INDENT`: an explicit width overrides the continuation
        // indent for array elements only; `-1` (default) inherits today's
        // `ind(inner)` byte-for-byte.
        let ind = self.construct_ind(indent, self.style.array_element_indent, &self.ind(inner));
        let elems: Vec<Node<'s>> = self.named(node);
        let pad = self.style.space_within_array_initializer_braces;

        // `ARRAY_INITIALIZER_LBRACE/RBRACE_ON_NEXT_LINE` (default false): the
        // brace sits on its own line only when its bool is on — by default
        // `{` ends the preceding line and `}` ends the last element's line.
        // Elements always start on their own line at `ind(indent + 1)`, and
        // the padding toggle never lands next to a newline (R5).
        let l_next = self.style.array_initializer_lbrace_on_next_line;
        let r_next = self.style.array_initializer_rbrace_on_next_line;

        let mut out = String::new();
        if l_next {
            out.push('\n');
            out.push_str(&self.ind(indent));
        }
        out.push('{');
        // `ALIGN_MULTILINE_ARRAY_INITIALIZER_EXPRESSION`: with the lbrace on
        // the header line the first element stays right after `{` and the
        // remaining element lines pad to the column after `{` (the
        // record-header model — the layout this formatter pins for the
        // option). With the lbrace on its own line the elements already
        // begin their own lines and the option leaves them unchanged.
        if self.style.align_multiline_array_initializer_expression && !l_next {
            let elem_col = brace_col + 1 + if pad { 1 } else { 0 };
            let pref = self.align_prefix(elem_col);
            let mut first = true;
            for e in elems {
                if first {
                    if pad {
                        out.push(' ');
                    }
                    first = false;
                } else {
                    out.push_str(",\n");
                    out.push_str(&pref);
                }
                out.push_str(&self.expr(e, inner, elem_col));
            }
        } else {
            out.push('\n');
            let elem_col = self.col_after(0, &ind);
            let mut first = true;
            for e in elems {
                if !first {
                    out.push_str(",\n");
                }
                first = false;
                out.push_str(&ind);
                out.push_str(&self.expr(e, inner, elem_col));
            }
        }
        if r_next {
            out.push('\n');
            out.push_str(&self.ind(indent));
        } else if pad {
            out.push(' ');
        }
        out.push('}');
        out
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
    /// `SPACES_WITHIN_ANGLE_BRACKETS` pads inside the brackets (`< A, B >`,
    /// nested generics at every level via recursion); the empty diamond `<>`
    /// is never padded.
    fn flat_type_args(&self, node: Node<'s>) -> String {
        let inner: Vec<_> = self
            .named(node)
            .iter()
            .map(|n| self.flat_type(*n))
            .collect();
        let body = inner.join(self.comma_sep(self.style.space_after_comma_in_type_arguments));
        if body.is_empty() {
            "<>".to_string()
        } else if self.style.spaces_within_angle_brackets {
            format!("< {} >", body)
        } else {
            format!("<{}>", body)
        }
    }

    /// The gap emitted after a rendered explicit type-argument list `<…>` at
    /// a join where the closing `>` directly abuts a following token: one
    /// space when `SPACE_AFTER_CLOSING_ANGLE_BRACKET_IN_TYPE_ARGUMENT` is on
    /// and both sides are non-empty, nothing otherwise (the default keeps
    /// `a.<T>b()` byte-identical).
    fn type_args_gap(&self, ta: &str, following: &str) -> &'static str {
        if self
            .style
            .space_after_closing_angle_bracket_in_type_argument
            && !ta.is_empty()
            && !following.is_empty()
        {
            " "
        } else {
            ""
        }
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
    /// `SPACES_WITHIN_ANGLE_BRACKETS` pads inside the brackets (`< T, U >`).
    fn flat_type_params(&self, node: Node<'s>) -> String {
        let inner: Vec<_> = self
            .named(node)
            .iter()
            .map(|n| self.flat_type_param(*n))
            .collect();
        let body = inner.join(", ");
        if self.style.spaces_within_angle_brackets && !body.is_empty() {
            format!("< {} >", body)
        } else {
            format!("<{}>", body)
        }
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
    /// The `&` join keeps its spaces per
    /// `SPACE_AROUND_TYPE_BOUNDS_IN_TYPE_PARAMETERS` (default: `A & B`; off
    /// renders `A&B`). The mandatory space after `extends` is applied at the
    /// `flat_type_param` join, never here.
    fn flat_type_bound(&self, node: Node<'s>) -> String {
        let inner: Vec<_> = self
            .named(node)
            .iter()
            .map(|n| self.flat_type(*n))
            .collect();
        let sep = if self.style.space_around_type_bounds_in_type_parameters {
            " & "
        } else {
            "&"
        };
        inner.join(sep)
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

    // ── declaration clause lists (extends / implements / throws) ─────────────

    /// Append the ` extends A, B` / ` implements A, B` tail of a type-declaration
    /// header to `header`. The clause column is `col_after(c, header)` — the
    /// cursor sits on the header's current physical line (annotations or
    /// wrapped record components may already contain newlines). Clauses whose
    /// `type_list` is present and non-empty wrap per `EXTENDS_LIST_WRAP` /
    /// `EXTENDS_KEYWORD_WRAP`; anything else keeps today's verbatim
    /// `flat_type_list` echo (R4).
    fn append_type_clause(
        &self,
        header: &mut String,
        keyword: &str,
        clause: Node<'s>,
        indent: usize,
        c: usize,
    ) {
        match self.type_list_items(clause) {
            Some(items) => {
                let cur = self.col_after(c, header);
                header.push_str(&self.clause_list(
                    keyword,
                    &items,
                    self.style.extends_keyword_wrap,
                    self.style.extends_list_wrap,
                    indent,
                    cur,
                    self.style.align_multiline_extends_list,
                    false,
                ));
            }
            None => {
                header.push(' ');
                header.push_str(keyword);
                header.push(' ');
                header.push_str(&self.flat_type_list(clause));
            }
        }
    }

    /// The flattened elements of a clause's `type_list` (`extends_interfaces`
    /// / `super_interfaces` — both have an anonymous `extends` / `implements`
    /// keyword child followed by a named `type_list`), or `None` when the
    /// clause carries no usable `type_list`.
    fn type_list_items(&self, clause: Node<'s>) -> Option<Vec<String>> {
        let tl = self
            .named(clause)
            .into_iter()
            .find(|n| n.kind() == "type_list")?;
        let items: Vec<String> = self.named(tl).iter().map(|n| self.flat_type(*n)).collect();
        if items.is_empty() {
            None
        } else {
            Some(items)
        }
    }

    /// Render a declaration clause tail — the text appended where the clause
    /// begins at column `cur_col` — for the shared `throws` / `extends` /
    /// `implements` layout. `DoNotWrap` (and single-element lists) produce the
    /// flat ` keyword A, B` clause byte-identical to today's output. When the
    /// list wraps, the keyword stays on the header line (`) throws A,`) unless
    /// `keyword_wrap` moves it to a continuation line at `self.cont(indent)`;
    /// every further element goes on its own `\n<cont>` line. `WrapIfLong` and
    /// `ChopDownIfLong` share the layout: these atomic list elements cannot be
    /// split further.
    ///
    /// `align_list` (`ALIGN_MULTILINE_THROWS_LIST` /
    /// `ALIGN_MULTILINE_EXTENDS_LIST`) pads the wrapped element lines to the
    /// first element's column instead of the continuation indent — the first
    /// element stays on the keyword's line, so the later lines align under it.
    /// `align_keyword` (`ALIGN_THROWS_KEYWORD`) additionally pads the keyword's
    /// own continuation line (when `keyword_wrap` moves it) to the column the
    /// keyword would occupy if it had stayed on the header line after the
    /// preceding token.
    #[allow(clippy::too_many_arguments)]
    fn clause_list(
        &self,
        keyword: &str,
        items: &[String],
        keyword_wrap: bool,
        wrap: WrapStyle,
        indent: usize,
        cur_col: usize,
        align_list: bool,
        align_keyword: bool,
    ) -> String {
        let flat = format!(
            " {} {}",
            keyword,
            items.join(self.comma_sep(self.style.space_after_comma))
        );
        let should_wrap = match wrap {
            WrapStyle::DoNotWrap => false,
            WrapStyle::WrapAlways => items.len() > 1,
            _ => items.len() > 1 && !self.fits(cur_col, &flat),
        };
        if !should_wrap {
            return flat;
        }

        let cont = self.cont(indent);
        let comma = if self.style.space_before_comma {
            " ,"
        } else {
            ","
        };
        let mut lines: Vec<String> = Vec::with_capacity(items.len());
        for (i, it) in items.iter().enumerate() {
            let mut line = it.clone();
            if i + 1 < items.len() {
                line.push_str(comma);
            }
            lines.push(line);
        }

        // Alignment columns: `kw_col` is where the keyword begins, `item_col`
        // where the first element begins right after the keyword — the column
        // the wrapped element lines align under. With the keyword on the
        // header line the keyword starts one column after the clause's leading
        // space; with the keyword wrapped its line starts at `cont` (or at the
        // keyword's natural header column under `align_keyword`).
        let kw_line_pref = if keyword_wrap {
            if align_keyword {
                self.align_prefix(cur_col + 1)
            } else {
                cont.clone()
            }
        } else {
            String::new()
        };
        let kw_col = if keyword_wrap {
            if align_keyword {
                cur_col + 1
            } else {
                self.col_after(0, &cont)
            }
        } else {
            cur_col + 1
        };
        let item_col = kw_col + keyword.len() + 1;
        let item_pref = if align_list {
            self.align_prefix(item_col)
        } else {
            cont.clone()
        };

        let mut s = if keyword_wrap {
            format!("\n{}{} {}", kw_line_pref, keyword, lines[0])
        } else {
            format!(" {} {}", keyword, lines[0])
        };
        for l in lines.iter().skip(1) {
            s.push('\n');
            s.push_str(&item_pref);
            s.push_str(l);
        }
        s
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
                self.ann_eq(k, &v)
            }
            "method_reference" => self.method_ref(node),
            "string_literal" => self.string_literal(node, 0, false),
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
            "new {}{}{}{}{}{}",
            ta,
            self.type_args_gap(&ta, &ty),
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
