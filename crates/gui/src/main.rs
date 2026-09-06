//! Desktop GUI for editing IntelliJ codestyle files.
//!
//! Renders every option in the core [`OPTIONS`] registry with a control
//! matching its type (bool → checkbox, `u32` → drag value, wrap/brace/force →
//! labeled combo of the IntelliJ meaning, line separator → labeled combo of
//! the separator choices), shows a live formatting preview,
//! and saves a minimal `<code_scheme>` via [`serialize_codestyle`].

use eframe::egui;
use java_formatter_core::config::{
    parse_codestyle, serialize_codestyle, BraceStyle, ForceStyle, JavaStyle, LineSeparator,
    OptionDef, OptionValue, WrapStyle, OPTIONS,
};
use java_formatter_core::formatter::format_java;
use std::path::Path;

const DEFAULT_SOURCE: &str = r#"public class Demo {
    public static void main(String[] args) {
        System.out.println("hello, world");
    }
}
"#;

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
}

impl Default for CodestyleApp {
    fn default() -> Self {
        Self {
            path: String::new(),
            style: JavaStyle::default(),
            source: DEFAULT_SOURCE.to_owned(),
            message: None,
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

    fn option_row(&mut self, ui: &mut egui::Ui, def: &'static OptionDef) {
        let mut value = (def.get)(&self.style);
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
            (def.set)(&mut self.style, value);
            let response = ui.label(egui::RichText::new(def.xml_name).monospace());
            response.on_hover_text(def.description);
        });
    }

    fn options_panel(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut current_group: Option<&str> = None;
            for def in OPTIONS {
                if current_group != Some(def.group) {
                    current_group = Some(def.group);
                    ui.add_space(8.0);
                    ui.heading(def.group);
                    ui.separator();
                }
                self.option_row(ui, def);
            }
        });
    }

    fn preview_panel(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            columns[0].heading("Java source");
            columns[0].add(
                egui::TextEdit::multiline(&mut self.source)
                    .code_editor()
                    .desired_rows(24)
                    .desired_width(f32::INFINITY),
            );

            columns[1].heading("Formatted preview");
            let preview = format_java(&self.source, &self.style);
            egui::ScrollArea::horizontal()
                .id_salt("preview_scroll")
                .show(&mut columns[1], |ui| {
                    ui.add(
                        egui::Label::new(egui::RichText::new(preview).monospace())
                            .wrap_mode(egui::TextWrapMode::Extend),
                    );
                });
        });
    }
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
