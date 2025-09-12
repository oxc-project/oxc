mod tester;

use tester::Tester;

#[test]
fn single_file() {
    // Test different flags on the same file
    Tester::new().test_and_snapshot_multiple(&[
        &["-c", "tests/fixtures/single_file/simple.js"],
        &["tests/fixtures/single_file/simple.js"], // Without flag (defaults to -c)
        &["-l", "tests/fixtures/single_file/simple.js"],
    ]);
}

#[test]
fn multiple_files() {
    // Test different ways to specify multiple files
    Tester::new().test_and_snapshot_multiple(&[
        // Explicit file list
        &["tests/fixtures/multiple_files/simple.js", "tests/fixtures/multiple_files/arrow.js"],
        // Directory
        &["tests/fixtures/multiple_files"],
        // Glob pattern (not expanded in tests, but usually expanded by the shell)
        &["tests/fixtures/multiple_files/*.js"],
        // Quoted glob pattern
        // TODO: Implement glob expansion w/ `fast-glob`
        &["'tests/fixtures/multiple_files/*.js'"],
    ]);
}

#[test]
fn no_error_on_unmatched_pattern() {
    // Test both with and without --no-error-on-unmatched-pattern flag
    Tester::new().test_and_snapshot_multiple(&[
        &["--no-error-on-unmatched-pattern", "__non__existent__file.js"],
        &["__non__existent__file.js"],
    ]);
}

#[test]
fn supported_extensions() {
    let args = &["tests/fixtures/extensions"];
    Tester::new().test_and_snapshot(args);
}

#[test]
fn write_mode() {
    let before = "  class                 Foo {}";
    let after = "class Foo {}\n";
    Tester::test_write("tests/fixtures/temp.js", before, after);
}
