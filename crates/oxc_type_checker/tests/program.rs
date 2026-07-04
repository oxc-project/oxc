//! Integration tests for `oxc_type_checker::compiler`: parsing root files into a [`Program`].

use std::path::{Path, PathBuf};

use oxc_type_checker::compiler::{CompilerHost, FileId, Program};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn fixture(name: &str) -> PathBuf {
    fixtures_dir().join(name)
}

fn build_program(root_files: &[PathBuf]) -> Program {
    Program::new(CompilerHost::new(fixtures_dir()), root_files)
}

fn base_name(path: &Path) -> &str {
    path.file_name().unwrap().to_str().unwrap()
}

fn base_names(program: &Program) -> Vec<&str> {
    program.source_files().iter().map(|source_file| base_name(source_file.file_name())).collect()
}

#[test]
fn collects_root_files_in_order() {
    let program = build_program(&[fixture("a.ts"), fixture("b.ts")]);

    assert_eq!(program.len(), 2);
    assert!(!program.is_empty());
    assert!(program.missing_files().is_empty());
    assert_eq!(base_names(&program), ["a.ts", "b.ts"]);
}

#[test]
fn deduplicates_repeated_root_files() {
    // The same file listed twice (and out of order) collapses to a single `SourceFile`; the
    // first occurrence fixes its position.
    let program = build_program(&[fixture("a.ts"), fixture("b.ts"), fixture("a.ts")]);

    assert_eq!(program.len(), 2);
    assert_eq!(base_names(&program), ["a.ts", "b.ts"]);
}

#[test]
fn looks_up_files_by_path() {
    let a = fixture("a.ts");
    let program = build_program(std::slice::from_ref(&a));

    let source_file = program.get_source_file_by_path(&a).expect("a.ts should be found by path");
    assert_eq!(base_name(source_file.file_name()), "a.ts");
    assert!(program.get_source_file_by_path(&fixture("missing.ts")).is_none());
}

#[test]
fn builds_a_semantic_model_for_each_file() {
    let program = build_program(&[fixture("a.ts")]);
    let source_file = program.source_file(FileId::from_usize(0));

    let symbol_names: Vec<String> =
        source_file.with_semantic(|s| s.scoping().symbol_names().map(String::from).collect());
    assert!(symbol_names.iter().any(|n| n == "a"), "expected symbol `a`, got {symbol_names:?}");
    assert!(
        symbol_names.iter().any(|n| n == "greet"),
        "expected symbol `greet`, got {symbol_names:?}"
    );
}

#[test]
fn records_missing_root_files() {
    let missing = fixture("does-not-exist.ts");
    let program = build_program(&[missing]);

    assert!(program.is_empty());
    assert_eq!(program.missing_files().len(), 1);
}
