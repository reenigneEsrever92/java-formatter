//! Integration tests for directory mode: `-d` / `--dir` formats every
//! `*.java` file under a directory in place, `-r` / `--recursive` descends
//! into subdirectories (skipping hidden dot-directories), per-file failures
//! are logged to stderr without stopping the run, and the exit code is 1 when
//! any file failed.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

/// The built `java-formatter` binary, exposed by Cargo to integration tests.
const BIN: &str = env!("CARGO_BIN_EXE_java-formatter");

/// A `.java` fixture the default style demonstrably reformats.
const MESSY_JAVA: &str = "class Foo{int x=1;}\n";

/// Write the given relative-path → content pairs under a fresh temp dir.
fn write_tree(files: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    for (rel, content) in files {
        let path = dir.path().join(rel);
        fs::create_dir_all(path.parent().expect("fixture path has a parent"))
            .expect("failed to create parent dirs");
        fs::write(&path, content).expect("failed to write fixture");
    }
    dir
}

fn run(args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .output()
        .expect("failed to execute java-formatter")
}

fn read(dir: &Path, rel: &str) -> String {
    fs::read_to_string(dir.join(rel)).expect("failed to read fixture")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn dir_formats_top_level_java_files_in_place() {
    let dir = write_tree(&[
        ("a.java", MESSY_JAVA),
        ("b.java", MESSY_JAVA),
        ("notes.txt", "not java\n"),
        ("sub/c.java", MESSY_JAVA),
    ]);

    let out = run(&["-d", dir.path().to_str().unwrap()]);

    assert_eq!(out.status.code(), Some(0));
    assert!(
        out.stdout.is_empty(),
        "directory mode must not write stdout"
    );
    assert_ne!(read(dir.path(), "a.java"), MESSY_JAVA);
    assert_ne!(read(dir.path(), "b.java"), MESSY_JAVA);
    // Non-Java files and subdirectory files are untouched without -r.
    assert_eq!(read(dir.path(), "notes.txt"), "not java\n");
    assert_eq!(read(dir.path(), "sub/c.java"), MESSY_JAVA);
}

#[test]
fn dir_output_matches_single_file_mode_and_is_idempotent() {
    let dir = write_tree(&[("a.java", MESSY_JAVA)]);

    let expected = run(&[dir.path().join("a.java").to_str().unwrap()]);
    assert_eq!(expected.status.code(), Some(0));
    assert_ne!(String::from_utf8_lossy(&expected.stdout), MESSY_JAVA);

    let first = run(&["-d", dir.path().to_str().unwrap()]);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(
        read(dir.path(), "a.java"),
        String::from_utf8_lossy(&expected.stdout)
    );

    // A second run is a no-op: still exits 0 and does not change the file.
    let second = run(&["-d", dir.path().to_str().unwrap()]);
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(
        read(dir.path(), "a.java"),
        String::from_utf8_lossy(&expected.stdout)
    );
}

#[test]
fn recursive_formats_subdirectories_and_skips_hidden_directories() {
    let dir = write_tree(&[
        ("a.java", MESSY_JAVA),
        ("sub/b.java", MESSY_JAVA),
        ("sub/deep/c.java", MESSY_JAVA),
        (".hidden/d.java", MESSY_JAVA),
        ("sub/.git/e.java", MESSY_JAVA),
    ]);

    let out = run(&["-d", dir.path().to_str().unwrap(), "-r"]);

    assert_eq!(out.status.code(), Some(0));
    assert_ne!(read(dir.path(), "a.java"), MESSY_JAVA);
    assert_ne!(read(dir.path(), "sub/b.java"), MESSY_JAVA);
    assert_ne!(read(dir.path(), "sub/deep/c.java"), MESSY_JAVA);
    // Hidden dot-directories are skipped.
    assert_eq!(read(dir.path(), ".hidden/d.java"), MESSY_JAVA);
    assert_eq!(read(dir.path(), "sub/.git/e.java"), MESSY_JAVA);
}

#[test]
fn style_scheme_is_applied_across_the_run() {
    let scheme = r#"<code_scheme name="tabbed" version="173">
  <codeStyleSettings language="JAVA">
    <indentOptions>
      <option name="USE_TAB_CHARACTER" value="true" />
      <option name="TAB_SIZE" value="4" />
    </indentOptions>
  </codeStyleSettings>
</code_scheme>
"#;
    let dir = write_tree(&[("a.java", MESSY_JAVA), ("b.java", MESSY_JAVA)]);
    let scheme_path = dir.path().join("scheme.xml");
    fs::write(&scheme_path, scheme).expect("failed to write scheme");
    let scheme_arg = scheme_path.to_str().unwrap();

    let out = run(&["-d", dir.path().to_str().unwrap(), "--style", scheme_arg]);

    assert_eq!(out.status.code(), Some(0));
    for file in ["a.java", "b.java"] {
        let formatted = read(dir.path(), file);
        assert!(
            formatted.contains('\t'),
            "tab-indent scheme should be applied to {file}: {formatted:?}"
        );
    }
}

#[test]
fn missing_directory_exits_1_with_a_message() {
    let dir = write_tree(&[]);
    let missing = dir.path().join("nope");

    let out = run(&["-d", missing.to_str().unwrap()]);

    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("could not read directory"));
}

#[test]
fn directory_with_no_java_files_is_success() {
    let dir = write_tree(&[("notes.txt", "not java\n")]);

    let out = run(&["-d", dir.path().to_str().unwrap()]);

    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
    assert_eq!(read(dir.path(), "notes.txt"), "not java\n");
}

#[test]
fn invalid_java_warns_formats_best_effort_and_exits_1() {
    let bad = "class Foo {\n";
    let dir = write_tree(&[("bad.java", bad), ("good.java", MESSY_JAVA)]);

    let out = run(&["-d", dir.path().to_str().unwrap()]);

    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(
        err.contains("bad.java"),
        "stderr should name the failing file: {err}"
    );
    assert!(err.contains("not valid Java"));
    // The other file is still formatted and the invalid one is rewritten
    // best-effort, matching single-file mode's output.
    assert_ne!(read(dir.path(), "good.java"), MESSY_JAVA);
    let single = run(&[dir.path().join("bad.java").to_str().unwrap()]);
    assert_eq!(
        read(dir.path(), "bad.java"),
        String::from_utf8_lossy(&single.stdout)
    );
}

#[cfg(unix)]
#[test]
fn unreadable_file_is_logged_other_files_are_formatted_and_exit_is_1() {
    use std::os::unix::fs::PermissionsExt;

    let dir = write_tree(&[("bad.java", MESSY_JAVA), ("good.java", MESSY_JAVA)]);
    fs::set_permissions(
        dir.path().join("bad.java"),
        fs::Permissions::from_mode(0o000),
    )
    .expect("failed to make fixture unreadable");

    let out = run(&["-d", dir.path().to_str().unwrap()]);

    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(
        err.contains("bad.java"),
        "stderr should name the failing file: {err}"
    );
    assert!(err.contains("could not read"));
    assert_ne!(read(dir.path(), "good.java"), MESSY_JAVA);
}

#[cfg(unix)]
#[test]
fn write_failure_is_logged_other_files_are_formatted_and_exit_is_1() {
    use std::os::unix::fs::PermissionsExt;

    let dir = write_tree(&[("bad.java", MESSY_JAVA), ("good.java", MESSY_JAVA)]);
    // Readable but not writable, so formatting succeeds and the write fails.
    fs::set_permissions(
        dir.path().join("bad.java"),
        fs::Permissions::from_mode(0o444),
    )
    .expect("failed to make fixture read-only");

    let out = run(&["-d", dir.path().to_str().unwrap()]);

    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(
        err.contains("bad.java"),
        "stderr should name the failing file: {err}"
    );
    assert!(err.contains("could not write"));
    assert_ne!(read(dir.path(), "good.java"), MESSY_JAVA);
}

#[test]
fn recursive_without_dir_is_a_usage_error() {
    let out = run(&["-r"]);

    assert_eq!(out.status.code(), Some(2));
    assert!(!out.stderr.is_empty());
}

#[test]
fn dir_conflicts_with_positional_file() {
    let dir = write_tree(&[]);

    let out = run(&[
        "-d",
        dir.path().to_str().unwrap(),
        dir.path().join("x.java").to_str().unwrap(),
    ]);

    assert_eq!(out.status.code(), Some(2));
    assert!(!out.stderr.is_empty());
}

#[test]
fn single_file_mode_still_exits_0_with_warning_on_invalid_java() {
    let dir = write_tree(&[("bad.java", "class Foo {\n")]);

    let out = run(&[dir.path().join("bad.java").to_str().unwrap()]);

    // R15 contract for single-file mode: warned, best-effort, exit 0.
    assert_eq!(out.status.code(), Some(0));
    assert!(stderr(&out).contains("not valid Java"));
    assert!(!out.stdout.is_empty());
}
