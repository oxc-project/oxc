//! Tests for the discovery of the `tsgolint` executable.
//!
//! These tests need runs in which `tsgolint` is reachable from some linted files but not from
//! others, which cannot be expressed with a fixture used in place: this repository has
//! `node_modules/.bin/tsgolint` installed at its root, and the lookup walks up to the file
//! system root. They therefore copy the fixture into a temporary directory outside of the
//! repository and run the real `oxlint` binary there, with `OXLINT_TSGOLINT_PATH` unset and a
//! sanitized `PATH`.

use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use tempfile::TempDir;

/// Copy `from` into `to` recursively.
fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// Copy `fixtures/cli/<fixture>` into a fresh temporary directory outside of this repository.
fn isolated_copy_of(fixture: &str) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let cwd = temp.path().join("project");
    copy_dir(&Path::new("fixtures/cli").join(fixture), &cwd);
    (temp, cwd)
}

/// Install `script` as the `tsgolint` of `package_root`.
///
/// Only shell builtins may be used, because these tests run with a sanitized `PATH`.
#[cfg(unix)]
fn install_tsgolint(package_root: &Path, script: &str) {
    use std::os::unix::fs::PermissionsExt;

    let bin_dir = package_root.join("node_modules").join(".bin");
    std::fs::create_dir_all(&bin_dir).unwrap();

    let executable = bin_dir.join("tsgolint");
    std::fs::write(&executable, script).unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// A `tsgolint` which produces no output: it drains its stdin and exits successfully, which
/// `oxlint` reads as "no diagnostics for these files".
#[cfg(unix)]
fn install_fake_tsgolint(package_root: &Path) {
    install_tsgolint(package_root, "#!/bin/sh\nwhile IFS= read -r _line; do :; done\nexit 0\n");
}

/// Run `oxlint` in `cwd` with an environment in which no `tsgolint` can be found, so that only
/// the executables installed by the test itself are discovered. Returns its stdout and whether
/// it succeeded.
fn run_isolated(temp: &TempDir, cwd: &Path, args: &[&str]) -> (String, bool) {
    // An empty directory, so that `PATH` resolves nothing at all.
    let empty_path = temp.path().join("empty-path");
    std::fs::create_dir_all(&empty_path).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_oxlint"))
        .args(args)
        .current_dir(cwd)
        .env_remove("OXLINT_TSGOLINT_PATH")
        .env("PATH", &empty_path)
        .stdin(Stdio::null())
        .output()
        .unwrap();

    (String::from_utf8(output.stdout).unwrap(), output.status.success())
}

const MISSING_EXECUTABLE: &str = "Could not find a `tsgolint` executable";

/// The case of <https://github.com/oxc-project/oxc/issues/18995>: `tsgolint` is installed in one
/// package only, and `oxlint` runs from the workspace root.
#[cfg(unix)]
#[test]
fn lints_the_package_which_has_tsgolint_and_reports_the_one_which_does_not() {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");
    install_fake_tsgolint(&cwd.join("packages").join("a"));

    let (stdout, _) = run_isolated(&temp, &cwd, &["--type-aware", "."]);

    // `packages/a` resolved its own `tsgolint`, so it is not reported, and the run went ahead.
    assert_eq!(
        stdout.matches(MISSING_EXECUTABLE).count(),
        1,
        "expected exactly one warning about the missing executable, got:\n{stdout}"
    );
    assert!(
        stdout.contains("packages/b/src/index.ts"),
        "expected the file of `packages/b` to be named, got:\n{stdout}"
    );
    // Paths are relative to the working directory, like every other diagnostic.
    assert!(
        !stdout.contains(&format!("{}/packages", cwd.display())),
        "expected relative paths in the warning, got:\n{stdout}"
    );

    // Regular rules keep working for every package, including the reported one.
    assert_eq!(
        stdout.matches("eslint(no-debugger)").count(),
        2,
        "expected both packages to be linted with the regular rules, got:\n{stdout}"
    );
}

/// Nothing could be linted with type-aware rules, so the run must fail: `--type-aware` was
/// requested and no file was linted with it.
#[test]
fn nothing_resolved_is_an_error_before_linting() {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");

    let (stdout, succeeded) = run_isolated(&temp, &cwd, &["--type-aware", "."]);

    assert!(!succeeded, "type-aware linting ran nothing, the run must fail, got:\n{stdout}");
    assert_eq!(
        stdout.matches(MISSING_EXECUTABLE).count(),
        1,
        "expected exactly one message about the missing executable, got:\n{stdout}"
    );
    assert!(stdout.contains("2 file(s)"), "expected the files to be accounted for, got:\n{stdout}");
    assert!(stdout.contains("oxlint-tsgolint"), "expected the package name, got:\n{stdout}");
    assert!(
        stdout.contains("OXLINT_TSGOLINT_PATH"),
        "expected the environment variable name, got:\n{stdout}"
    );

    // The run failed before linting anything, so nothing was buffered into a diagnostic
    // channel which would never be drained.
    assert!(
        !stdout.contains("eslint(no-debugger)"),
        "no file should have been linted, got:\n{stdout}"
    );
    // A run which did not happen is not summarized as a clean one.
    assert!(
        !stdout.contains("Found 0 warnings and 0 errors"),
        "a run which never started should not report an all-clear, got:\n{stdout}"
    );
}

/// A `tsgolint` which answers with a rule diagnostic that has no rule, framed exactly like a
/// real one: `| size u32 LE | kind u8 | payload |`.
///
/// `tsgolint` never sends this. oxlint used to unwrap the missing field and abort the whole
/// process.
#[cfg(unix)]
fn install_lying_tsgolint(package_root: &Path) {
    let payload = concat!(
        r#"{"kind":0,"range":{"pos":0,"end":1},"#,
        r#""message":{"id":"x","description":"a diagnostic with no rule","help":null},"#,
        r#""rule":null,"file_path":"/x.ts"}"#,
    );
    let size = u32::try_from(payload.len()).unwrap().to_le_bytes();
    // Octal escapes, because that is what POSIX `printf` is required to understand.
    let mut header = String::new();
    for byte in size {
        let _ = write!(header, "\\{byte:03o}");
    }

    install_tsgolint(
        package_root,
        &format!(
            "#!/bin/sh\nwhile IFS= read -r _line; do :; done\nprintf '{header}\\001'\nprintf '%s' '{payload}'\nexit 0\n"
        ),
    );
}

/// Malformed output fails the group it came from, not the whole run, and never panics.
#[cfg(unix)]
#[test]
fn reports_malformed_tsgolint_output_as_a_group_failure() {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");
    install_fake_tsgolint(&cwd.join("packages").join("a"));
    install_lying_tsgolint(&cwd.join("packages").join("b"));

    let (stdout, succeeded) = run_isolated(&temp, &cwd, &["--type-aware", "."]);

    assert!(!succeeded, "a group which sent nonsense must fail the run, got:\n{stdout}");
    assert!(!stdout.contains("panicked"), "malformed output must not panic, got:\n{stdout}");
    assert!(
        stdout.contains("rule diagnostic without its `rule`"),
        "the failure should say what was wrong, got:\n{stdout}"
    );
    // The package which answered properly keeps its diagnostics.
    assert_eq!(
        stdout.matches("eslint(no-debugger)").count(),
        2,
        "the regular diagnostics must survive, got:\n{stdout}"
    );
}

/// A `tsgolint` which frames its message properly but announces a wrong length: the header
/// announces `u32::MAX` bytes, followed by a handful of junk.
///
/// Trusting that size would allocate 4 GiB for nine bytes of output, which turns a
/// desynchronized stream into an out-of-memory abort.
#[cfg(unix)]
fn install_oversized_tsgolint(package_root: &Path) {
    // `\377` four times is `u32::MAX` little-endian, `\001` is the diagnostic kind. Octal
    // escapes, because that is what POSIX `printf` is required to understand.
    install_tsgolint(
        package_root,
        "#!/bin/sh\nwhile IFS= read -r _line; do :; done\nprintf '\\377\\377\\377\\377\\001'\nprintf 'junk'\nexit 0\n",
    );
}

/// An impossible message size fails its group like any other malformed output, instead of
/// being trusted.
#[cfg(unix)]
#[test]
fn reports_an_oversized_tsgolint_message_as_a_group_failure() {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");
    install_fake_tsgolint(&cwd.join("packages").join("a"));
    install_oversized_tsgolint(&cwd.join("packages").join("b"));

    let (stdout, succeeded) = run_isolated(&temp, &cwd, &["--type-aware", "."]);

    assert!(!succeeded, "a group which announced nonsense must fail the run, got:\n{stdout}");
    assert!(!stdout.contains("panicked"), "an impossible size must not panic, got:\n{stdout}");
    assert!(
        stdout.contains("announced a 4294967295 byte message"),
        "the failure should say what was announced, got:\n{stdout}"
    );
    // The package which answered properly keeps its diagnostics.
    assert_eq!(
        stdout.matches("eslint(no-debugger)").count(),
        2,
        "the regular diagnostics must survive, got:\n{stdout}"
    );
}

/// A `tsgolint` which fails to run, standing in for a broken or incompatible install.
#[cfg(unix)]
fn install_broken_tsgolint(package_root: &Path) {
    install_tsgolint(package_root, "#!/bin/sh\nexit 1\n");
}

/// A failing `tsgolint` must not discard what the rest of the run already produced.
#[cfg(unix)]
#[test]
fn prints_the_diagnostics_produced_before_a_group_failed() {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");
    install_fake_tsgolint(&cwd.join("packages").join("a"));
    install_broken_tsgolint(&cwd.join("packages").join("b"));

    let (stdout, succeeded) = run_isolated(&temp, &cwd, &["--type-aware", "."]);

    assert!(!succeeded, "a group which could not run must fail the run, got:\n{stdout}");

    // The regular diagnostics of both packages survive the failure.
    assert_eq!(
        stdout.matches("eslint(no-debugger)").count(),
        2,
        "the regular diagnostics must still be printed, got:\n{stdout}"
    );
    // The failure names the executable which could not be run.
    assert!(
        stdout.contains("packages/b/node_modules/.bin/tsgolint"),
        "the failing executable should be named, got:\n{stdout}"
    );
    // `packages/a` resolved fine, so it is not reported as missing or failing.
    assert!(
        !stdout.contains("packages/a/node_modules/.bin/tsgolint"),
        "the working executable should not be reported, got:\n{stdout}"
    );
    assert!(!stdout.contains(MISSING_EXECUTABLE), "nothing is missing here, got:\n{stdout}");
}

/// Linting a sibling package from `packages/b`: the file is outside the working directory, so
/// the walk has no boundary and climbs to the package which owns it.
///
/// The path has to be absolute, because the CLI rejects one containing `..`.
#[cfg(unix)]
#[test]
fn lints_a_package_outside_the_working_directory() {
    let (temp, root) = isolated_copy_of("tsgolint_monorepo");
    let sibling = root.join("packages").join("a");
    install_fake_tsgolint(&sibling);

    let cwd = root.join("packages").join("b");
    let (stdout, _) = run_isolated(
        &temp,
        &cwd,
        &["--type-aware", "-c", "../../.oxlintrc.json", sibling.to_str().unwrap()],
    );

    // `packages/a` was linted, and found its own `tsgolint` even from outside the cwd.
    assert!(
        stdout.contains("eslint(no-debugger)"),
        "`packages/a` should have been linted, got:\n{stdout}"
    );
    assert!(
        !stdout.contains(MISSING_EXECUTABLE),
        "its own installation should have been found, got:\n{stdout}"
    );
}

/// The reporters which are not the graphical one print the message and its position, and
/// nothing else. A diagnostic without a source renders as an empty line there.
#[cfg(unix)]
#[test]
fn reports_the_missing_executable_in_every_format() {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");
    install_fake_tsgolint(&cwd.join("packages").join("a"));

    for format in ["unix", "checkstyle"] {
        let (stdout, _) = run_isolated(&temp, &cwd, &["--format", format, "--type-aware", "."]);

        assert!(stdout.contains("tsgolint"), "{format}: the warning is missing:\n{stdout}");
        assert!(
            stdout.contains("packages/b/src/index.ts"),
            "{format}: the warning should name the file it is about:\n{stdout}"
        );
        assert!(
            stdout.contains("oxlint-tsgolint") && stdout.contains("OXLINT_TSGOLINT_PATH"),
            "{format}: the remedy should survive a format which drops `help`:\n{stdout}"
        );
        // The position comes from the file the diagnostic was attached to.
        assert!(
            !stdout.contains(":0:0:"),
            "{format}: the warning should be placed in a file:\n{stdout}"
        );
    }
}

/// Same for the failure of a group which could not be run.
#[cfg(unix)]
#[test]
fn reports_a_group_failure_in_every_format() {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");
    install_fake_tsgolint(&cwd.join("packages").join("a"));
    install_broken_tsgolint(&cwd.join("packages").join("b"));

    for format in ["unix", "checkstyle"] {
        let (stdout, _) = run_isolated(&temp, &cwd, &["--format", format, "--type-aware", "."]);

        assert!(
            stdout.contains("packages/b/node_modules/.bin/tsgolint"),
            "{format}: the failing executable should be named:\n{stdout}"
        );
        assert!(
            stdout.contains("packages/b/src/index.ts"),
            "{format}: the failure should be attached to a file of the group:\n{stdout}"
        );
        assert!(
            stdout.contains("oxlint-tsgolint"),
            "{format}: the remedy should survive a format which drops `help`:\n{stdout}"
        );
        assert!(
            !stdout.contains(":0:0:"),
            "{format}: the failure should be placed in a file:\n{stdout}"
        );
    }
}

/// A directive for a rule whose pass did not run cannot be reported as unused. Reporting it
/// would prompt the user to delete a directive which is still needed.
#[cfg(unix)]
#[test]
fn keeps_the_type_aware_directives_of_the_files_which_could_not_be_linted() {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");
    install_fake_tsgolint(&cwd.join("packages").join("a"));

    let file = cwd.join("packages").join("b").join("src").join("index.ts");
    let source = std::fs::read_to_string(&file).unwrap();
    std::fs::write(
        &file,
        format!("// eslint-disable-next-line typescript/no-floating-promises\n{source}"),
    )
    .unwrap();

    let (stdout, _) =
        run_isolated(&temp, &cwd, &["--type-aware", "--report-unused-disable-directives", "."]);

    assert!(
        !stdout.contains("Unused eslint-disable"),
        "the directive of a rule which never ran is not unused, got:\n{stdout}"
    );
    // The file is still reported as not type-aware linted, so the user knows why.
    assert!(stdout.contains(MISSING_EXECUTABLE), "got:\n{stdout}");
}

/// A directive naming both a rule which ran and one which did not is not entirely unused: the
/// rule which ran is reported on its own, and the comment is not offered for deletion.
#[cfg(unix)]
#[test]
fn reports_only_the_rules_which_ran_of_a_mixed_directive() {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");
    install_fake_tsgolint(&cwd.join("packages").join("a"));

    let file = cwd.join("packages").join("b").join("src").join("index.ts");
    let source = std::fs::read_to_string(&file).unwrap();
    std::fs::write(
        &file,
        format!("// eslint-disable-next-line no-empty, typescript/no-floating-promises\n{source}"),
    )
    .unwrap();

    let (stdout, _) =
        run_isolated(&temp, &cwd, &["--type-aware", "--report-unused-disable-directives", "."]);

    // The rule which ran and did not fire is reported by name.
    assert!(
        stdout.contains("no problems were reported from no-empty"),
        "the rule which ran should be reported, got:\n{stdout}"
    );
    // The rule which never ran is not reported.
    assert!(
        !stdout.contains("no-floating-promises)."),
        "the rule which never ran should not be reported, got:\n{stdout}"
    );
    // The whole comment is not reported as unused, which would invite deleting it.
    assert!(
        !stdout.contains("Unused eslint-disable directive (no problems were reported)."),
        "the comment still covers a rule which never ran, got:\n{stdout}"
    );
}

/// A group whose `tsgolint` failed did not lint its files either, so their directives are
/// treated like those of a file which had no `tsgolint` at all.
#[cfg(unix)]
#[test]
fn keeps_the_type_aware_directives_of_a_group_which_failed() {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");
    install_fake_tsgolint(&cwd.join("packages").join("a"));
    install_broken_tsgolint(&cwd.join("packages").join("b"));

    let file = cwd.join("packages").join("b").join("src").join("index.ts");
    let source = std::fs::read_to_string(&file).unwrap();
    std::fs::write(
        &file,
        format!("// eslint-disable-next-line typescript/no-floating-promises\n{source}"),
    )
    .unwrap();

    let (stdout, _) =
        run_isolated(&temp, &cwd, &["--type-aware", "--report-unused-disable-directives", "."]);

    assert!(
        !stdout.contains("Unused eslint-disable"),
        "the directive of a rule whose tsgolint could not run is not unused, got:\n{stdout}"
    );
    // The failure itself is still reported, so the user knows why.
    assert!(stdout.contains("could not lint this file"), "got:\n{stdout}");
}

/// The regular pass ran, so the directives it found are still reported; a broken `tsgolint`
/// must not discard them.
#[cfg(unix)]
#[test]
fn reports_unused_disable_directives_even_when_a_group_failed() {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");
    install_fake_tsgolint(&cwd.join("packages").join("a"));
    install_broken_tsgolint(&cwd.join("packages").join("b"));

    // A directive which disables nothing, in the package whose `tsgolint` works, so the
    // unused-directive report cannot be confused with the failure itself.
    let file = cwd.join("packages").join("a").join("src").join("index.ts");
    let source = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, format!("// eslint-disable-next-line no-empty\n{source}")).unwrap();

    let (stdout, _) =
        run_isolated(&temp, &cwd, &["--type-aware", "--report-unused-disable-directives", "."]);

    assert!(
        stdout.contains("Unused eslint-disable directive (no problems were reported)."),
        "the unused directive should still be reported, got:\n{stdout}"
    );
    assert!(
        stdout.contains("packages/b/node_modules/.bin/tsgolint"),
        "and so should the failure, got:\n{stdout}"
    );
}

/// The failure is emitted together with the diagnostics: a machine reading `--format json`
/// must see both, and nothing else on stdout.
#[cfg(unix)]
#[test]
fn prints_json_diagnostics_and_the_failure_together() {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");
    install_fake_tsgolint(&cwd.join("packages").join("a"));
    install_broken_tsgolint(&cwd.join("packages").join("b"));

    let (stdout, succeeded) = run_isolated(&temp, &cwd, &["--format", "json", "--type-aware", "."]);

    assert!(!succeeded, "a group which could not run must fail the run, got:\n{stdout}");

    let report: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|error| panic!("stdout should be valid JSON ({error}):\n{stdout}"));
    let diagnostics = report["diagnostics"].as_array().expect("an array of diagnostics");

    let messages = diagnostics
        .iter()
        .map(|diagnostic| diagnostic["message"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        messages.iter().filter(|message| message.contains("debugger")).count(),
        2,
        "the regular diagnostics should be in the JSON, got: {messages:?}"
    );
    assert!(
        messages.iter().any(|message| message.contains("packages/b/node_modules/.bin/tsgolint")),
        "the tsgolint failure should be in the JSON, got: {messages:?}"
    );
}

/// A rule `tsgolint` runs, reported as `typescript/<rule>` like every other rule of that plugin.
const TYPE_AWARE_RULE: &str = "typescript/no-floating-promises";
/// A rule oxlint runs itself, under that same plugin.
const NATIVE_TYPESCRIPT_RULE: &str = "typescript/no-explicit-any";
const STALE_SUPPRESSIONS: &str = "suppressions that do not occur anymore";

/// Set up a monorepo where only `packages/a` has a `tsgolint`, with a suppressions file
/// generated by oxlint itself so that its shape is the real one, plus the suppressions
/// `extra_for_b` records against `packages/b` (the package which cannot be type-aware linted).
///
/// The fake `tsgolint` reports nothing, so the type-aware entries have to be added rather than
/// observed.
#[cfg(unix)]
fn monorepo_with_suppressions(extra_for_b: &[&str]) -> (TempDir, PathBuf, PathBuf) {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");
    install_fake_tsgolint(&cwd.join("packages").join("a"));

    run_isolated(&temp, &cwd, &["--type-aware", "--suppress-all", "."]);

    let path = cwd.join("oxlint-suppressions.json");
    let mut suppressions: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(
        suppressions["packages/b/src/index.ts"]["no-debugger"].is_object(),
        "`--suppress-all` should have recorded the regular violations: {suppressions}"
    );
    for rule in extra_for_b {
        suppressions["packages/b/src/index.ts"][*rule] = serde_json::json!({ "count": 1 });
    }
    std::fs::write(&path, serde_json::to_string_pretty(&suppressions).unwrap()).unwrap();

    (temp, cwd, path)
}

/// A suppression recorded for a rule `tsgolint` would have run, on a file whose package has no
/// `tsgolint`, must not be reported as stale nor pruned away, because that rule never ran
/// rather than stopping to fire.
#[cfg(unix)]
#[test]
fn keeps_the_type_aware_suppressions_of_the_files_which_could_not_be_linted() {
    let (temp, cwd, path) = monorepo_with_suppressions(&[TYPE_AWARE_RULE]);

    let (stdout, succeeded) = run_isolated(&temp, &cwd, &["--type-aware", "."]);

    assert!(
        !stdout.contains(STALE_SUPPRESSIONS),
        "the suppression of a file which was never type-aware linted is not stale, got:\n{stdout}"
    );
    assert!(succeeded, "only the missing-executable warning should remain, got:\n{stdout}");

    // Pruning must leave it in place as well.
    let (prune_stdout, _) =
        run_isolated(&temp, &cwd, &["--type-aware", "--prune-suppressions", "."]);
    let pruned = std::fs::read_to_string(&path).unwrap();
    assert!(
        pruned.contains(TYPE_AWARE_RULE),
        "the suppression should survive pruning, got:\n{pruned}\n{prune_stdout}"
    );
}

/// The rules oxlint runs itself did run, even under the `typescript` plugin, so their
/// suppressions are still reported and pruned as usual.
#[cfg(unix)]
#[test]
fn still_reports_the_stale_native_suppressions_of_those_files() {
    let (temp, cwd, path) = monorepo_with_suppressions(&[TYPE_AWARE_RULE, NATIVE_TYPESCRIPT_RULE]);

    let (stdout, _) = run_isolated(&temp, &cwd, &["--type-aware", "."]);

    assert!(
        stdout.contains(STALE_SUPPRESSIONS),
        "the stale `{NATIVE_TYPESCRIPT_RULE}` suppression should be reported, got:\n{stdout}"
    );

    let (_, _) = run_isolated(&temp, &cwd, &["--type-aware", "--prune-suppressions", "."]);
    let pruned = std::fs::read_to_string(&path).unwrap();
    assert!(
        !pruned.contains(NATIVE_TYPESCRIPT_RULE),
        "the stale native suppression should be pruned, got:\n{pruned}"
    );
    assert!(
        pruned.contains(TYPE_AWARE_RULE),
        "the type-aware suppression should survive, got:\n{pruned}"
    );
}

#[test]
fn does_not_report_when_type_aware_linting_is_disabled() {
    let (temp, cwd) = isolated_copy_of("tsgolint_monorepo");

    let (stdout, _) = run_isolated(&temp, &cwd, &["-c", "config-no-type-aware.json", "."]);

    assert!(!stdout.contains("tsgolint"), "expected no mention of tsgolint at all, got:\n{stdout}");
}
