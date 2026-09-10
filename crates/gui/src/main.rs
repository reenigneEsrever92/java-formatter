//! Desktop GUI for editing IntelliJ codestyle files.
//!
//! Renders every option in the core [`OPTIONS`] registry with a control
//! matching its type (bool → checkbox, `u32` → drag value, wrap/brace/force →
//! labeled combo of the IntelliJ meaning, line separator → labeled combo of
//! the separator choices), shows a live formatting preview, and saves a
//! minimal `<code_scheme>` via [`serialize_codestyle`].
//!
//! The Java source and the formatted preview are syntax-highlighted with
//! tree-sitter ([`highlight_java`]) and scroll in both directions; the preview
//! draws a vertical guide at the configured right margin.

use eframe::egui;
use java_formatter_core::config::{
    parse_codestyle, serialize_codestyle, BraceStyle, ForceStyle, Group, GroupDef, JavaStyle,
    LineSeparator, OptionDef, OptionValue, WrapStyle, GROUPS, OPTIONS,
};
use java_formatter_core::formatter::format_java;
use java_formatter_core::highlight::{highlight_java, HighlightKind, HighlightSpan};
use std::path::Path;

const DEFAULT_SOURCE: &str = r#"public class Demo {
    public static void main(String[] args) {
        System.out.println("hello, world");
    }
}
"#;

/// Highlight spans cached against the exact text they were computed for.
type HighlightCache = Option<(String, Vec<HighlightSpan>)>;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("java-formatter codestyle editor"),
        ..Default::default()
    };
    eframe::run_native(
        "java-formatter codestyle editor",
        options,
        Box::new(|_cc| Ok(Box::new(CodestyleApp::default()))),
    )
}

struct CodestyleApp {
    /// Path of the codestyle file being edited (set by New / Open… / drag-and-drop).
    path: String,
    /// The style being edited, kept in sync with the option controls.
    style: JavaStyle,
    /// Java source shown in the editor and formatted live in the preview.
    source: String,
    /// Last status / error message, shown in the top bar.
    message: Option<String>,
    /// Cached highlight spans for the editor buffer, keyed by its text.
    editor_highlight: HighlightCache,
    /// Cached highlight spans for the formatted preview, keyed by its text.
    preview_highlight: HighlightCache,
    /// Filter text for the options panel; empty shows every option.
    options_filter: String,
}

impl Default for CodestyleApp {
    fn default() -> Self {
        Self {
            path: String::new(),
            style: JavaStyle::default(),
            source: DEFAULT_SOURCE.to_owned(),
            message: None,
            editor_highlight: None,
            preview_highlight: None,
            options_filter: String::new(),
        }
    }
}

impl CodestyleApp {
    fn open(&mut self, path: &Path) {
        match std::fs::read_to_string(path) {
            Ok(xml) => match parse_codestyle(&xml) {
                Ok(style) => {
                    self.style = style;
                    self.path = path.display().to_string();
                    self.message = Some(format!("Loaded {}", path.display()));
                }
                Err(e) => {
                    self.message = Some(format!("Could not parse {}: {}", path.display(), e));
                }
            },
            Err(e) => {
                self.message = Some(format!("Could not read {}: {}", path.display(), e));
            }
        }
    }

    fn save(&mut self) {
        if self.path.is_empty() {
            self.message = Some("Enter a file path before saving.".to_owned());
            return;
        }
        let xml = serialize_codestyle(&self.style);
        match std::fs::write(&self.path, xml) {
            Ok(()) => self.message = Some(format!("Saved {}", self.path)),
            Err(e) => {
                self.message = Some(format!("Could not write {}: {}", self.path, e));
            }
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Path:");
            ui.add(
                egui::TextEdit::singleline(&mut self.path)
                    .hint_text("e.g. .idea/codeStyles/Project.xml (save target)")
                    .desired_width(340.0),
            );
            if ui.button("New").clicked() {
                self.style = JavaStyle::default();
                self.message = Some("New style: IntelliJ defaults.".to_owned());
            }
            if ui.button("Open…").clicked() {
                let picked = rfd::FileDialog::new()
                    .add_filter("IntelliJ codestyle (*.xml)", &["xml"])
                    .set_title("Open codestyle")
                    .pick_file();
                if let Some(path) = picked {
                    self.open(&path);
                }
            }
            if ui.button("Save").clicked() {
                self.save();
            }
        });
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("Drop a codestyle.xml anywhere in the window to open it.")
                .weak()
                .small(),
        );
        if let Some(msg) = &self.message {
            ui.add_space(2.0);
            ui.label(msg);
        }
        ui.add_space(4.0);
        ui.separator();
    }

    fn option_row(ui: &mut egui::Ui, style: &mut JavaStyle, def: &'static OptionDef) {
        let mut value = (def.get)(style);
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            match &mut value {
                OptionValue::Bool(b) => {
                    ui.checkbox(b, "");
                }
                OptionValue::UInt(n) => {
                    ui.add(egui::DragValue::new(n).speed(1).range(0..=u32::MAX));
                }
                // Signed options (`-1` = inherit for the per-construct indent
                // widths): a drag value covering `-1` and reasonable widths.
                OptionValue::Int(n) => {
                    ui.add(egui::DragValue::new(n).speed(1).range(-1..=1024));
                }
                OptionValue::Wrap(w) => {
                    egui::ComboBox::from_id_salt(def.xml_name)
                        .selected_text(wrap_label(*w))
                        .width(190.0)
                        .show_ui(ui, |ui| {
                            for candidate in [
                                WrapStyle::DoNotWrap,
                                WrapStyle::WrapIfLong,
                                WrapStyle::WrapAlways,
                                WrapStyle::ChopDownIfLong,
                            ] {
                                ui.selectable_value(w, candidate, wrap_label(candidate));
                            }
                        });
                }
                OptionValue::Brace(b) => {
                    egui::ComboBox::from_id_salt(def.xml_name)
                        .selected_text(brace_label(*b))
                        .width(190.0)
                        .show_ui(ui, |ui| {
                            for candidate in [
                                BraceStyle::EndOfLine,
                                BraceStyle::NextLine,
                                BraceStyle::NextLineShifted,
                                BraceStyle::NextLineShifted2,
                                BraceStyle::NextLineIfWrapped,
                            ] {
                                ui.selectable_value(b, candidate, brace_label(candidate));
                            }
                        });
                }
                OptionValue::Force(f) => {
                    egui::ComboBox::from_id_salt(def.xml_name)
                        .selected_text(force_label(*f))
                        .width(190.0)
                        .show_ui(ui, |ui| {
                            for candidate in [
                                ForceStyle::DoNotForce,
                                ForceStyle::ForceIfMultiline,
                                ForceStyle::ForceAlways,
                            ] {
                                ui.selectable_value(f, candidate, force_label(candidate));
                            }
                        });
                }
                OptionValue::LineSep(s) => {
                    egui::ComboBox::from_id_salt(def.xml_name)
                        .selected_text(line_sep_label(*s))
                        .width(190.0)
                        .show_ui(ui, |ui| {
                            for candidate in [
                                LineSeparator::System,
                                LineSeparator::Lf,
                                LineSeparator::Crlf,
                                LineSeparator::Cr,
                            ] {
                                ui.selectable_value(s, candidate, line_sep_label(candidate));
                            }
                        });
                }
                OptionValue::ImportLayout(entries) => {
                    // Read-only summary: a full table editor is out of scope.
                    // The value is passed back to `set` unchanged below.
                    let n = entries.len();
                    ui.label(egui::RichText::new(format!("{n} entries (read-only)")).weak());
                }
                OptionValue::Packages(packages) => {
                    // One package per line; the joined/edited text is written
                    // back into the list so `set` below persists edits.
                    let mut text = packages.join("\n");
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut text)
                            .desired_rows(2)
                            .desired_width(200.0),
                    );
                    if response.changed() {
                        *packages = text
                            .lines()
                            .map(str::trim)
                            .filter(|l| !l.is_empty())
                            .map(str::to_string)
                            .collect();
                    }
                }
                OptionValue::String(s) => {
                    // A single-line text edit over the raw comma-separated
                    // value; the edited text is written back so `set` below
                    // splits it.
                    let mut text = s.clone();
                    let response =
                        ui.add(egui::TextEdit::singleline(&mut text).desired_width(200.0));
                    if response.changed() {
                        *s = text;
                    }
                }
            }
            (def.set)(style, value);
            let response = ui.label(egui::RichText::new(def.xml_name).monospace());
            response.on_hover_text(def.description);
        });
    }

    fn options_panel(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::TextEdit::singleline(&mut self.options_filter)
                .hint_text("Filter options…")
                .desired_width(f32::INFINITY),
        );
        ui.add_space(4.0);

        let filter = self.options_filter.trim().to_lowercase();
        let layout = panel_layout(&filter);
        if layout.is_empty() {
            ui.label(egui::RichText::new("No option matches the filter.").weak());
            return;
        }

        let style = &mut self.style;
        egui::ScrollArea::vertical()
            .id_salt("options_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for section in &layout {
                    egui::CollapsingHeader::new(section.group.title)
                        .id_salt(section.group.title)
                        .default_open(true)
                        .show(ui, |ui| {
                            for def in &section.options {
                                Self::option_row(ui, style, def);
                            }
                            for (sub, options) in &section.subs {
                                ui.add_space(2.0);
                                egui::CollapsingHeader::new(sub.title)
                                    .id_salt(sub.title)
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        for def in options {
                                            Self::option_row(ui, style, def);
                                        }
                                    });
                            }
                        });
                }
            });
    }

    fn preview_panel(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            self.editor_pane(&mut columns[0]);
            self.preview_pane(&mut columns[1]);
        });
    }

    /// The editable Java source, syntax-highlighted and scrollable in both
    /// directions. Soft wrap is off so lines keep their real columns.
    fn editor_pane(&mut self, ui: &mut egui::Ui) {
        ui.heading("Java source");
        let cache = &mut self.editor_highlight;
        let source = &mut self.source;
        let mut layouter = |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, _wrap_width: f32| {
            let job = highlight_job(ui, buffer.as_str(), cache);
            ui.fonts_mut(|fonts| fonts.layout_job(job))
        };
        egui::ScrollArea::both()
            .id_salt("editor_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(source)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
    }

    /// A read-only, syntax-highlighted rendering of the formatted output, with
    /// a vertical guide at the configured right margin.
    fn preview_pane(&mut self, ui: &mut egui::Ui) {
        ui.heading("Formatted preview");
        let preview = format_java(&self.source, &self.style);
        let margin = self.style.right_margin;

        egui::ScrollArea::both()
            .id_salt("preview_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let job = highlight_job(ui, &preview, &mut self.preview_highlight);
                let response = ui.add(
                    egui::Label::new(job)
                        .wrap_mode(egui::TextWrapMode::Extend)
                        .selectable(true),
                );

                draw_margin_guide(ui, response.rect, margin);
            });
    }
}

/// One top-level section of the options panel: the options that sit directly
/// under it, plus its sub-sections, each carrying the options that matched.
struct PanelSection {
    group: &'static GroupDef,
    options: Vec<&'static OptionDef>,
    subs: Vec<(&'static GroupDef, Vec<&'static OptionDef>)>,
}

/// Whether `def` is kept by a (already lower-cased) filter query.
fn option_matches(def: &OptionDef, filter: &str) -> bool {
    filter.is_empty()
        || def.xml_name.to_lowercase().contains(filter)
        || def.description.to_lowercase().contains(filter)
}

/// The options of `group` that survive `filter`, in registry order.
fn matching_options(group: Group, filter: &str) -> Vec<&'static OptionDef> {
    OPTIONS
        .iter()
        .filter(|def| def.group == group && option_matches(def, filter))
        .collect()
}

/// The sections to render, in [`GROUPS`] display order, with options filtered
/// by `filter` (lower-cased; empty means "everything"). Sections with no
/// matching option and no matching sub-section are dropped. Deriving the
/// layout from `GROUPS` — never from the `OPTIONS` array order — is what keeps
/// every section title to exactly one heading.
fn panel_layout(filter: &str) -> Vec<PanelSection> {
    GROUPS
        .iter()
        .filter(|group| group.parent.is_none())
        .filter_map(|group| {
            let options = matching_options(group.id, filter);
            let subs: Vec<(&'static GroupDef, Vec<&'static OptionDef>)> = GROUPS
                .iter()
                .filter(|sub| sub.parent == Some(group.id))
                .map(|sub| (sub, matching_options(sub.id, filter)))
                .filter(|(_, options)| !options.is_empty())
                .collect();
            if options.is_empty() && subs.is_empty() {
                None
            } else {
                Some(PanelSection {
                    group,
                    options,
                    subs,
                })
            }
        })
        .collect()
}

impl eframe::App for CodestyleApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let dropped: Vec<_> = ui.ctx().input(|i| i.raw.dropped_files.clone());
        for file in dropped {
            let path = file.path();
            if path.extension().is_some_and(|e| e == "xml") {
                self.open(path);
            } else {
                self.message = Some(format!("Not a codestyle file: {}", path.display()));
            }
        }

        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));

        egui::Panel::left("options")
            .resizable(true)
            .default_size(360.0)
            .show(ui, |ui| {
                ui.heading("Code style options");
                ui.add_space(4.0);
                self.options_panel(ui);
            });

        egui::CentralPanel::default().show(ui, |ui| self.preview_panel(ui));
    }
}

fn wrap_label(w: WrapStyle) -> &'static str {
    match w {
        WrapStyle::DoNotWrap => "Do not wrap",
        WrapStyle::WrapIfLong => "Wrap if long",
        WrapStyle::WrapAlways => "Wrap always",
        WrapStyle::ChopDownIfLong => "Chop down if long",
    }
}

fn brace_label(b: BraceStyle) -> &'static str {
    match b {
        BraceStyle::EndOfLine => "End of line",
        BraceStyle::NextLine => "Next line",
        BraceStyle::NextLineShifted => "Next line, shifted",
        BraceStyle::NextLineShifted2 => "Next line, shifted (2)",
        BraceStyle::NextLineIfWrapped => "Next line if wrapped",
    }
}

fn force_label(f: ForceStyle) -> &'static str {
    match f {
        ForceStyle::DoNotForce => "Do not force",
        ForceStyle::ForceIfMultiline => "Force braces if multiline",
        ForceStyle::ForceAlways => "Force braces always",
    }
}

fn line_sep_label(s: LineSeparator) -> &'static str {
    match s {
        LineSeparator::System => "System",
        LineSeparator::Lf => "LF (\\n)",
        LineSeparator::Crlf => "CRLF (\\r\\n)",
        LineSeparator::Cr => "CR (\\r)",
    }
}

/// Build the layout job for `text`, reusing `cache` when the text is unchanged.
/// The job never wraps, so both panes keep the source's real columns.
fn highlight_job(ui: &egui::Ui, text: &str, cache: &mut HighlightCache) -> egui::text::LayoutJob {
    let stale = match cache.as_ref() {
        Some((cached, _)) => cached != text,
        None => true,
    };
    if stale {
        *cache = Some((text.to_owned(), highlight_java(text)));
    }
    let spans = &cache.as_ref().expect("populated above").1;

    let font_id = egui::TextStyle::Monospace.resolve(ui.style());
    let default = egui::TextFormat {
        font_id: font_id.clone(),
        color: ui.visuals().text_color(),
        ..Default::default()
    };

    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    job.keep_trailing_whitespace = true;

    let mut pos = 0;
    for span in spans {
        if span.end <= pos {
            continue;
        }
        let start = span.start.max(pos);
        if start > pos {
            job.append(&text[pos..start], 0.0, default.clone());
        }
        let format = egui::TextFormat {
            font_id: font_id.clone(),
            color: highlight_color(ui, span.kind),
            ..Default::default()
        };
        job.append(&text[start..span.end], 0.0, format);
        pos = span.end;
    }
    if pos < text.len() {
        job.append(&text[pos..], 0.0, default);
    }
    job
}

/// Draw the right-margin guide at `margin` monospace columns from the text's
/// left edge, and widen the surrounding scroll content so the guide stays
/// reachable behind a horizontal scrollbar even when no line reaches it.
/// A margin of `0` draws nothing.
fn draw_margin_guide(ui: &mut egui::Ui, text_rect: egui::Rect, margin: u32) {
    if margin == 0 {
        return;
    }
    let font_id = egui::TextStyle::Monospace.resolve(ui.style());
    let char_width = ui.fonts_mut(|fonts| fonts.glyph_width(&font_id, ' '));
    let guide_x = text_rect.left() + margin as f32 * char_width;
    ui.expand_to_include_rect(egui::Rect::from_min_max(
        egui::pos2(guide_x, text_rect.top()),
        egui::pos2(guide_x + 1.0, text_rect.bottom()),
    ));
    let stroke = egui::Stroke::new(1.0, ui.visuals().weak_text_color());
    let y_range = ui.clip_rect().y_range();
    ui.painter().vline(guide_x, y_range, stroke);
}

/// Colours for each [`HighlightKind`], chosen for the active light/dark theme.
fn highlight_color(ui: &egui::Ui, kind: HighlightKind) -> egui::Color32 {
    let dark = ui.visuals().dark_mode;
    let (dark_rgb, light_rgb) = match kind {
        HighlightKind::Keyword => ((197, 134, 192), (153, 0, 153)),
        HighlightKind::String => ((152, 195, 121), (0, 128, 0)),
        HighlightKind::Number => ((209, 154, 102), (160, 96, 0)),
        HighlightKind::Comment => ((128, 138, 138), (128, 128, 128)),
        HighlightKind::Annotation => ((220, 220, 170), (128, 128, 0)),
        HighlightKind::Type => ((120, 200, 200), (0, 100, 120)),
        HighlightKind::Method => ((120, 170, 240), (0, 0, 200)),
    };
    let (r, g, b) = if dark { dark_rgb } else { light_rgb };
    egui::Color32::from_rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run `body` inside a 400x600 viewport with a `ScrollArea::both` and
    /// return the scroll area's content size.
    fn content_size_after(mut body: impl FnMut(&mut egui::Ui)) -> egui::Vec2 {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(400.0, 600.0),
            )),
            ..Default::default()
        };

        let mut content_size = None;
        let mut full = ctx.run_ui(input, |ui| {
            let output = egui::ScrollArea::both().show(ui, |ui| {
                body(ui);
            });
            content_size = Some(output.content_size);
        });
        full.textures_delta.clear();
        content_size.expect("the frame ran")
    }

    /// A 120-column margin is wider than the preview pane, so the guide must
    /// widen the scroll content; otherwise it sits off-screen and cannot be
    /// scrolled to.
    #[test]
    fn margin_guide_widens_the_scroll_content() {
        let content_size = content_size_after(|ui| {
            let response = ui.add(egui::Label::new("short"));
            draw_margin_guide(ui, response.rect, 120);
        });
        assert!(
            content_size.x > 120.0,
            "expected the guide to widen the scroll content, got {content_size:?}"
        );
    }

    #[test]
    fn zero_margin_leaves_the_content_narrow() {
        let content_size = content_size_after(|ui| {
            let response = ui.add(egui::Label::new("short"));
            draw_margin_guide(ui, response.rect, 0);
        });
        assert!(
            content_size.x < 120.0,
            "a zero margin should not widen the content, got {content_size:?}"
        );
    }

    /// Every section and sub-section title rendered by the panel.
    fn section_titles(layout: &[PanelSection]) -> Vec<&str> {
        layout
            .iter()
            .flat_map(|section| {
                std::iter::once(section.group.title)
                    .chain(section.subs.iter().map(|(sub, _)| sub.title))
            })
            .collect()
    }

    /// Every option rendered by the panel.
    fn rendered_options(layout: &[PanelSection]) -> Vec<&'static OptionDef> {
        layout
            .iter()
            .flat_map(|section| {
                section.options.iter().copied().chain(
                    section
                        .subs
                        .iter()
                        .flat_map(|(_, options)| options.iter().copied()),
                )
            })
            .collect()
    }

    /// The bug this change fixes: a group whose entries were not contiguous in
    /// `OPTIONS` produced one heading per run. Titles are now unique.
    #[test]
    fn section_titles_render_exactly_once() {
        let layout = panel_layout("");
        let titles = section_titles(&layout);
        let unique: std::collections::BTreeSet<_> = titles.iter().collect();
        assert_eq!(unique.len(), titles.len(), "duplicate titles: {titles:?}");
        assert_eq!(titles.iter().filter(|t| **t == "Wrapping").count(), 1);
        assert_eq!(titles.iter().filter(|t| **t == "Enums").count(), 1);
        assert_eq!(titles.iter().filter(|t| **t == "Spaces").count(), 1);
    }

    /// Sections follow the `GROUPS` order, independent of `OPTIONS` order.
    #[test]
    fn sections_follow_the_declared_order() {
        let layout = panel_layout("");
        let rendered: Vec<Group> = layout.iter().map(|section| section.group.id).collect();
        let declared: Vec<Group> = GROUPS
            .iter()
            .filter(|group| group.parent.is_none())
            .map(|group| group.id)
            .collect();
        assert_eq!(rendered, declared);
    }

    /// No option is lost or duplicated by the sectioned layout.
    #[test]
    fn every_option_is_rendered_once() {
        let names: std::collections::BTreeSet<_> = rendered_options(&panel_layout(""))
            .into_iter()
            .map(|def| def.xml_name)
            .collect();
        assert_eq!(names.len(), OPTIONS.len());
        assert!(OPTIONS.iter().all(|def| names.contains(def.xml_name)));
    }

    /// The filter narrows the panel to matching options and drops empty
    /// sections.
    #[test]
    fn filter_keeps_only_matching_sections() {
        let layout = panel_layout("method_call_chain_wrap");
        let rendered = rendered_options(&layout);
        assert_eq!(rendered.len(), 1);
        assert_eq!(rendered[0].xml_name, "METHOD_CALL_CHAIN_WRAP");
        assert_eq!(layout.len(), 1);
        assert_eq!(layout[0].group.title, "Wrapping");

        assert!(panel_layout("no such option").is_empty());
    }
}
