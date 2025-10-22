mod tester;

use std::path::PathBuf;

use tester::Tester;

#[test]
fn single_file() {
    // Test different flags on the same file
    Tester::new().with_cwd(PathBuf::from("tests/fixtures/single_file")).test_and_snapshot_multiple(
        "single_file",
        &[&["--check", "simple.js"], &["--list-different", "simple.js"]],
    );
}

#[test]
fn multiple_files() {
    // Test different ways to specify multiple files
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/multiple_files"))
        .test_and_snapshot_multiple(
            "multiple_files",
            &[
                // Explicit file list
                &["--check", "simple.js", "arrow.js"],
                // Default to current directory
                &["--check"],
                // Explicit cwd
                &["--check", "."],
                &["--check", "./"],
                &["--check", "!*.{ts,tsx}"],
            ],
        );
}

#[test]
fn no_error_on_unmatched_pattern() {
    // Test both with and without --no-error-on-unmatched-pattern flag
    Tester::new().test_and_snapshot_multiple(
        "no_error_on_unmatched_pattern",
        &[
            &["--check", "--no-error-on-unmatched-pattern", "__non__existent__file.js"],
            &["--check", "__non__existent__file.js"],
        ],
    );
}

#[test]
fn supported_extensions() {
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/extensions"))
        .test_and_snapshot_multiple("supported_extensions", &[&["--check"]]);
}

#[test]
fn write_mode() {
    let before = "  class                 Foo {}";
    let after = "class Foo {}\n";
    Tester::test_write("tests/fixtures/temp.js", before, after);
}

#[test]
fn config_file_auto_discovery() {
    // Tests .json takes priority over .jsonc when both exist
    // config_file/ has both .oxfmtrc.json (semi: true) and .oxfmtrc.jsonc (semi: false)
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/config_file"))
        .test_and_snapshot_multiple("config_file_auto_discovery", &[&["--check"]]);

    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/config_file/nested"))
        .test_and_snapshot_multiple("config_file_auto_discovery_nested", &[&["--check"]]);

    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/config_file/nested/deep"))
        .test_and_snapshot_multiple("config_file_auto_discovery_nested_deep", &[&["--check"]]);
}

#[test]
fn config_file_explicit() {
    Tester::new().with_cwd(PathBuf::from("tests/fixtures/config_file")).test_and_snapshot_multiple(
        "config_file_explicit",
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
        .test_and_snapshot_multiple("vcs_dirs_ignored", &[&["--check"]]);
}

#[test]
fn node_modules_ignored() {
    // Test that `node_modules` directories are ignored by default
    // but can be included with `--with-node-modules` flag
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/node_modules_dirs"))
        .test_and_snapshot_multiple(
            "node_modules_ignored",
            &[&["--check"], &["--check", "--with-node-modules"]],
        );
}

#[test]
fn exclude_nested_paths() {
    // Test that nested path exclusion works correctly
    // See: https://github.com/oxc-project/oxc/issues/14684
    // All these cases should not report parse error from `foo/bar/error.js`
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/exclude_nested"))
        .test_and_snapshot_multiple(
            "exclude_nested_paths",
            &[
                &["--check", "!foo/bar/error.js"],
                &["--check", "!foo/bar"],
                &["--check", "!foo"],
                &["--check", "!**/error.js"],
                &["--check", "foo", "!foo/bar/error.js"],
                &["--check", "foo", "!foo/bar"],
                &["--check", "foo", "!**/bar/error.js"],
                &["--check", "foo", "!**/bar/*"],
            ],
        );
}
#[test]
fn exclude_nested_paths_with_dot() {
    // All these cases should not report parse error from `foo/bar/error.js`
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/exclude_nested"))
        .test_and_snapshot_multiple(
            "exclude_nested_paths_with_dot_1",
            &[
                &["--check", ".", "!foo/bar/error.js"],
                &["--check", ".", "!foo/bar"],
                &["--check", ".", "!foo"],
                &["--check", ".", "!**/error.js"],
            ],
        );
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/exclude_nested"))
        .test_and_snapshot_multiple(
            "exclude_nested_paths_with_dot_2",
            &[
                &["--check", "./foo", "!**/bar/error.js"],
                &["--check", "./foo", "!**/error.js"],
                &["--check", "./foo", "!**/bar/*"],
                &["--check", "./foo", "!foo/bar/error.js"],
                &["--check", "./foo", "!foo/bar"],
            ],
        );
}

#[test]
fn ignore_patterns() {
    // Test ignore file handling with different configurations
    // .prettierignore (cwd) contains: not-formatted/
    // not-formatted/.prettierignore (subdirectory) should be ignored
    // gitignore.txt contains: ignored/
    // custom.ignore contains: ignored/ (only)
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/ignore_patterns"))
        .test_and_snapshot_multiple(
            "ignore_patterns",
            &[
                // Default: auto-detects only cwd/.prettierignore (ignores not-formatted/ dir)
                // Note: not-formatted/.prettierignore exists but should be ignored
                &["--check"],
                // Explicit: uses gitignore.txt (ignores ignored/ dir, checks not-formatted/)
                &["--check", "--ignore-path", "gitignore.txt"],
                // Multiple files: ignores both dirs
                &["--check", "--ignore-path", "gitignore.txt", "--ignore-path", ".prettierignore"],
                // Nonexistent file should error
                &["--check", "--ignore-path", "nonexistent.ignore"],
            ],
        );
}

#[test]
fn config_ignore_patterns() {
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/config_ignore_patterns"))
        .test_and_snapshot_multiple(
            "config_ignore_patterns",
            &[
                // .oxfmtrc.json contains: ignorePatterns: ["not-formatted/"]
                // Should not find any files
                &["--check"],
                // fmtrc.jsonc also ignores every .js files
                &["--check", "--config", "fmtrc.jsonc"],
            ],
        );
}

#[test]
fn ignore_and_override() {
    Tester::new()
        .with_cwd(PathBuf::from("tests/fixtures/ignore_and_override"))
        .test_and_snapshot_multiple(
            "ignore_and_override",
            &[
                // Ignore err.js
                &["--check", "!**/err.js"],
                // Ignore every files
                &["--check", "--ignore-path", "ignore1"],
                // Override ignore for should_format/ok.js
                &["--check", "--ignore-path", "ignore1", "should_format/ok.js"],
                // ! prefixed line for should_format/ok.js
                &["--check", "--ignore-path", "ignore2"],
            ],
        );
}
