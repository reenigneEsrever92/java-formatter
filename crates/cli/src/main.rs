use java_formatter_core::{config, formatter};

use clap::Parser;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "java-formatter",
    about = "Format Java source files using IntelliJ IDEA codestyle rules"
)]
struct Args {
    /// Path to the Java source file to format. Reads from standard input when
    /// omitted or when '-' is given.
    file: Option<PathBuf>,

    /// Path to an IntelliJ codestyle XML file (e.g. .idea/codeStyles/Project.xml).
    /// Defaults to IntelliJ built-in settings when omitted.
    #[arg(short, long)]
    style: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

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

    let style = match args.style {
        Some(ref path) => {
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
    };

    let (formatted, issues) = formatter::format_java_diagnosed(&source, &style);
    for issue in &issues {
        eprintln!("warning: {}", issue);
    }
    if !issues.is_empty() {
        eprintln!("warning: input is not valid Java; output is best-effort");
    }
    print!("{}", formatted);
}
