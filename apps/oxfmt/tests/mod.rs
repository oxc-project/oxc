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
            // Default to current directory
            &["--check"],
            // Explicit cwd
            &["--check", "."],
            &["--check", "./"],
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

#[test]
fn config_file_auto_discovery() {
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/config_file"))
        .test_and_snapshot_multiple(&[&["--check"]]);

    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/config_file/nested"))
        .test_and_snapshot_multiple(&[&["--check"]]);

    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/config_file/nested/deep"))
        .test_and_snapshot_multiple(&[&["--check"]]);
}

#[test]
fn config_file_explicit() {
    Tester::new().with_cwd(PathBuf::from("tests/fixtures/config_file")).test_and_snapshot_multiple(
        &[
            &["--check", "--config", "./fmt.json"],
            &["--check", "--config", "./fmt.jsonc"],
            &["--check", "--config", "NOT_EXISTS.json"],
        ],
    );
}

#[test]
fn vcs_dirs_ignored() {
    // Test that VCS directories (.git, .jj, .sl, .svn, .hg) are ignored
    // but regular directories and root files are processed
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/vcs_dirs"))
        .test_and_snapshot_multiple(&[&["--check"]]);
}

#[test]
fn exclude_nested_paths() {
    // Test that nested path exclusion works correctly
    // See: https://github.com/oxc-project/oxc/issues/14684
    // All these cases should not report parse error from `foo/bar/error.js`
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/exclude_nested"))
        .test_and_snapshot_multiple(&[
            &["--check", "!foo/bar/error.js"],
            &["--check", "!foo/bar"],
            &["--check", "!foo"],
            &["--check", "!**/error.js"],
            &["--check", "foo", "!foo/bar/error.js"],
            &["--check", "foo", "!foo/bar"],
            &["--check", "foo", "!**/bar/error.js"],
            &["--check", "foo", "!**/bar/*"],
        ]);
}
#[test]
fn exclude_nested_paths_with_dot() {
    // All these cases should not report parse error from `foo/bar/error.js`
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/exclude_nested"))
        .test_and_snapshot_multiple(&[
            &["--check", ".", "!foo/bar/error.js"],
            &["--check", ".", "!foo/bar"],
            &["--check", ".", "!foo"],
            &["--check", ".", "!**/error.js"],
        ]);
    // Split to avoid too long file name error...
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/exclude_nested"))
        .test_and_snapshot_multiple(&[
            &["--check", "./foo", "!**/bar/error.js"],
            &["--check", "./foo", "!**/error.js"],
            &["--check", "./foo", "!**/bar/*"],
            &["--check", "./foo", "!foo/bar/error.js"],
            &["--check", "./foo", "!foo/bar"],
        ]);
}
