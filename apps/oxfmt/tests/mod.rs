mod tester;

use std::path::PathBuf;

use tester::Tester;

#[test]
fn single_file() {
    // Test different flags on the same file
    Tester::new().with_cwd(PathBuf::from("tests/fixtures/single_file")).test_and_snapshot_multiple(
        &[&["--check", "simple.js"], &["--list-different", "simple.js"]],
    );
}

#[test]
fn multiple_files() {
    // Test different ways to specify multiple files
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/multiple_files"))
        .test_and_snapshot_multiple(&[
            // Explicit file list
            &["--check", "simple.js", "arrow.js"],
            // Directory
            &["--check", "."],
            // Default to current directory
            &["--check"],
        ]);
}

#[test]
fn no_error_on_unmatched_pattern() {
    // Test both with and without --no-error-on-unmatched-pattern flag
    Tester::new().test_and_snapshot_multiple(&[
        &["--check", "--no-error-on-unmatched-pattern", "__non__existent__file.js"],
        &["--check", "__non__existent__file.js"],
    ]);
}

#[test]
fn supported_extensions() {
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/extensions"))
        .test_and_snapshot_multiple(&[&["--check"]]);
}

#[test]
fn write_mode() {
    let before = "  class                 Foo {}";
    let after = "class Foo {}\n";
    Tester::test_write("tests/fixtures/temp.js", before, after);
}
