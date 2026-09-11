use java_formatter_core::{config, formatter};

use clap::Parser;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
#[command(
    name = "java-formatter",
    about = "Format Java source files using IntelliJ IDEA codestyle rules"
)]
struct Args {
    /// Path to the Java source file to format. Reads from standard input when
    /// omitted or when '-' is given.
    #[arg(conflicts_with = "dir")]
    file: Option<PathBuf>,

    /// Path to an IntelliJ codestyle XML file (e.g. .idea/codeStyles/Project.xml).
    /// Defaults to IntelliJ built-in settings when omitted.
    #[arg(short, long)]
    style: Option<PathBuf>,

    /// Format every *.java file in this directory in place.
    #[arg(short, long, value_name = "DIR")]
    dir: Option<PathBuf>,

    /// Recurse into subdirectories (requires --dir).
    #[arg(short, long, requires = "dir")]
    recursive: bool,
}

fn main() {
    let args = Args::parse();

    if let Some(dir) = &args.dir {
        format_directory(dir, args.recursive, &args.style);
    } else {
        format_file(&args);
    }
}

/// Load the style scheme, or the IntelliJ built-in defaults when `--style` is
/// absent. Exits 1 on a missing/unreadable/malformed scheme.
fn load_style(style: &Option<PathBuf>) -> config::JavaStyle {
    match style {
        Some(path) => {
            let xml = fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!(
                    "error: could not read style file '{}': {}",
                    path.display(),
                    e
                );
                process::exit(1);
            });
            config::parse_codestyle(&xml).unwrap_or_else(|e| {
                eprintln!("error: could not parse codestyle XML: {}", e);
                process::exit(1);
            })
        }
        None => config::JavaStyle::default(),
    }
}

/// Format a single file or stdin to stdout (the original CLI behaviour).
fn format_file(args: &Args) {
    let source = match &args.file {
        Some(path) if path.to_str() != Some("-") => fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("error: could not read '{}': {}", path.display(), e);
            process::exit(1);
        }),
        _ => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .unwrap_or_else(|e| {
                    eprintln!("error: could not read from stdin: {}", e);
                    process::exit(1);
                });
            buf
        }
    };

    let style = load_style(&args.style);

    let (formatted, issues) = formatter::format_java_diagnosed(&source, &style);
    for issue in &issues {
        eprintln!("warning: {}", issue);
    }
    if !issues.is_empty() {
        eprintln!("warning: input is not valid Java; output is best-effort");
    }
    print!("{}", formatted);
}

/// Format every `*.java` file under `dir` in place. Per-file problems are
/// logged to stderr and the remaining files are still formatted; the process
/// exits 1 if any file failed, 0 otherwise.
fn format_directory(dir: &Path, recursive: bool, style_arg: &Option<PathBuf>) {
    let style = load_style(style_arg);

    let mut files = Vec::new();
    if let Err(msg) = collect_java_files(dir, recursive, &mut files) {
        eprintln!("error: {}", msg);
        process::exit(1);
    }

    let mut failed = false;
    for path in files {
        let source = match fs::read_to_string(&path) {
            Err(e) => {
                eprintln!("error: could not read '{}': {}", path.display(), e);
                failed = true;
                continue;
            }
            Ok(source) => source,
        };

        let (formatted, issues) = formatter::format_java_diagnosed(&source, &style);
        for issue in &issues {
            eprintln!("warning: {}: {}", path.display(), issue);
        }
        if !issues.is_empty() {
            eprintln!(
                "warning: {}: input is not valid Java; output is best-effort",
                path.display()
            );
            failed = true;
        }

        // Write only when the output differs, so already-formatted files keep
        // their timestamps.
        if formatted != source {
            if let Err(e) = fs::write(&path, &formatted) {
                eprintln!("error: could not write '{}': {}", path.display(), e);
                failed = true;
            }
        }
    }

    process::exit(if failed { 1 } else { 0 });
}

/// Collect every `*.java` file under `dir` into `out`, in deterministic order.
///
/// Without `recursive` only the directory's own files are collected. With
/// `recursive` subdirectories are walked depth-first, skipping hidden
/// dot-directories (e.g. `.git`); directory symlinks are never followed (they
/// could introduce cycles).
fn collect_java_files(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("could not read directory '{}': {}", dir.display(), e))?;

    let mut paths: Vec<_> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("could not read directory entry: {}", e))?;
        paths.push(entry);
    }
    paths.sort_by_key(|e| e.file_name());

    for entry in paths {
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(e) => return Err(format!("could not inspect '{}': {}", path.display(), e)),
        };

        if file_type.is_dir() {
            let name = entry.file_name();
            if recursive && !name.to_string_lossy().starts_with('.') {
                collect_java_files(&path, recursive, out)?;
            }
        } else if file_type.is_symlink() {
            // Follow file symlinks; skip symlinks whose target is a directory.
            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(e) => return Err(format!("could not inspect '{}': {}", path.display(), e)),
            };
            if metadata.is_file() && is_java_file(&path) {
                out.push(path);
            }
        } else if file_type.is_file() && is_java_file(&path) {
            out.push(path);
        }
    }

    Ok(())
}

fn is_java_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "java")
}
