use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use serde_json::Value;

use oxc_diagnostics::{DiagnosticService, Error, LabeledSpan, OxcDiagnostic};
use oxc_span::Span;

use super::{diagnostic::CachedDiagnostic, store::cache_path};
use crate::cli::{CliRunResult, CliRunner, lint_command};

const DEBUGGER: &str = "debugger;\n";
const CLEAN: &str = "export {};\n";

fn run(cwd: &Path, args: &[&str]) -> (String, CliRunResult) {
    // Single thread, like `Tester`, so output order is deterministic.
    let _ = rayon::ThreadPoolBuilder::new().num_threads(1).build_global();
    let mut all_args = vec!["-f", "unix", "-D", "no-debugger"];
    all_args.extend(args);
    let options = lint_command().run_inner(all_args.as_slice()).unwrap();
    let mut output = Vec::new();
    let result = CliRunner::new(options, None).with_cwd(cwd.to_path_buf()).run(&mut output);
    (String::from_utf8(output).unwrap(), result)
}

fn setup() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.js"), DEBUGGER).unwrap();
    fs::write(dir.path().join("b.js"), CLEAN).unwrap();
    dir
}

fn read_cache(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// Replace the cached message of every diagnostic, so the output shows whether the cached
/// result was replayed or the file was linted.
fn tamper(path: &Path) {
    let mut cache = read_cache(path);
    for entry in cache["files"].as_object_mut().unwrap().values_mut() {
        if let Some(diagnostics) = entry.get_mut("diagnostics") {
            for diagnostic in diagnostics.as_array_mut().unwrap() {
                diagnostic["message"] = Value::from("FROM CACHE");
            }
        }
    }
    fs::write(path, serde_json::to_vec(&cache).unwrap()).unwrap();
}

fn set_mtime(path: &Path, mtime: SystemTime) {
    fs::File::options().write(true).open(path).unwrap().set_modified(mtime).unwrap();
}

#[test]
fn caches_and_replays() {
    let dir = setup();
    let cache_file = dir.path().join(".oxlintcache");

    let (uncached, _) = run(dir.path(), &[]);
    let (cold, result) = run(dir.path(), &["--cache"]);
    assert!(matches!(result, CliRunResult::LintFoundErrors), "{result:?}");
    assert_eq!(cold, uncached);

    let cache = read_cache(&cache_file);
    assert_eq!(cache["files"]["a.js"]["diagnostics"].as_array().unwrap().len(), 1);
    assert!(cache["files"]["b.js"].get("diagnostics").is_none());

    let (warm, result) = run(dir.path(), &["--cache"]);
    assert!(matches!(result, CliRunResult::LintFoundErrors), "{result:?}");
    assert_eq!(warm, cold);

    tamper(&cache_file);
    let (output, _) = run(dir.path(), &["--cache"]);
    assert!(output.contains("FROM CACHE"), "{output}");
}

#[test]
fn changed_file_is_linted_again() {
    let dir = setup();
    let cache_file = dir.path().join(".oxlintcache");
    run(dir.path(), &["--cache"]);
    tamper(&cache_file);

    fs::write(dir.path().join("a.js"), "debugger;\ndebugger;\n").unwrap();
    let (output, _) = run(dir.path(), &["--cache"]);
    assert!(!output.contains("FROM CACHE"), "{output}");
    assert!(output.contains("2 problems"), "{output}");

    fs::write(dir.path().join("a.js"), CLEAN).unwrap();
    let (output, result) = run(dir.path(), &["--cache"]);
    assert!(matches!(result, CliRunResult::LintSucceeded), "{result:?}: {output}");
}

#[test]
fn metadata_strategy_compares_size_and_mtime() {
    let dir = setup();
    let file = dir.path().join("a.js");
    let cache_file = dir.path().join(".oxlintcache");
    run(dir.path(), &["--cache"]);
    tamper(&cache_file);

    // Same size and modification time: the cached result is used.
    let mtime = fs::metadata(&file).unwrap().modified().unwrap();
    fs::write(&file, "// hello\n\n").unwrap();
    set_mtime(&file, mtime);
    let (output, _) = run(dir.path(), &["--cache"]);
    assert!(output.contains("FROM CACHE"), "{output}");

    // Only the modification time changed: the file is linted again.
    fs::write(&file, DEBUGGER).unwrap();
    set_mtime(&file, mtime + Duration::from_secs(10));
    tamper(&cache_file);
    let (output, _) = run(dir.path(), &["--cache"]);
    assert!(!output.contains("FROM CACHE"), "{output}");
}

#[test]
fn content_strategy_compares_content() {
    let dir = setup();
    let file = dir.path().join("a.js");
    let cache_file = dir.path().join(".oxlintcache");
    let args = ["--cache", "--cache-strategy", "content"];
    run(dir.path(), &args);
    tamper(&cache_file);

    // Only the modification time changed: the cached result is used.
    let mtime = fs::metadata(&file).unwrap().modified().unwrap();
    set_mtime(&file, mtime + Duration::from_secs(10));
    let (output, _) = run(dir.path(), &args);
    assert!(output.contains("FROM CACHE"), "{output}");

    // Same size and modification time but different content: the file is linted again.
    fs::write(&file, "// hello\n\n").unwrap();
    set_mtime(&file, mtime + Duration::from_secs(10));
    let (output, result) = run(dir.path(), &args);
    assert!(matches!(result, CliRunResult::LintSucceeded), "{result:?}: {output}");
}

#[test]
fn settings_change_invalidates_cache() {
    let dir = setup();
    let cache_file = dir.path().join(".oxlintcache");
    run(dir.path(), &["--cache"]);
    tamper(&cache_file);

    // A different rule severity from the command line.
    let (output, result) = run(dir.path(), &["--cache", "-W", "no-debugger"]);
    assert!(!output.contains("FROM CACHE"), "{output}");
    assert!(matches!(result, CliRunResult::LintSucceeded), "{result:?}: {output}");

    // A config file.
    run(dir.path(), &["--cache"]);
    tamper(&cache_file);
    fs::write(dir.path().join(".oxlintrc.json"), r#"{ "rules": { "no-console": "error" } }"#)
        .unwrap();
    let (output, _) = run(dir.path(), &["--cache"]);
    assert!(!output.contains("FROM CACHE"), "{output}");

    // A config file change.
    tamper(&cache_file);
    fs::write(dir.path().join(".oxlintrc.json"), r#"{ "rules": { "no-console": "warn" } }"#)
        .unwrap();
    let (output, _) = run(dir.path(), &["--cache"]);
    assert!(!output.contains("FROM CACHE"), "{output}");
}

#[test]
fn output_options_do_not_invalidate_cache() {
    let dir = setup();
    let cache_file = dir.path().join(".oxlintcache");
    run(dir.path(), &["--cache"]);
    tamper(&cache_file);

    let (output, _) = run(dir.path(), &["--cache", "--quiet", "--max-warnings", "3"]);
    assert!(output.contains("FROM CACHE"), "{output}");
}

#[test]
fn running_without_cache_deletes_cache_file() {
    let dir = setup();
    let cache_file = dir.path().join(".oxlintcache");
    run(dir.path(), &["--cache"]);
    assert!(cache_file.is_file());

    run(dir.path(), &[]);
    assert!(!cache_file.exists());
}

#[test]
fn fixed_files_are_not_cached() {
    let dir = setup();
    let file = dir.path().join("a.js");
    let cache_file = dir.path().join(".oxlintcache");

    let (output, result) = run(dir.path(), &["--cache", "--fix-suggestions"]);
    assert!(matches!(result, CliRunResult::LintSucceeded), "{result:?}: {output}");
    assert_ne!(fs::read_to_string(&file).unwrap(), DEBUGGER);
    let cache = read_cache(&cache_file);
    assert!(cache["files"].get("a.js").is_none(), "{cache}");
    assert!(cache["files"].get("b.js").is_some(), "{cache}");

    run(dir.path(), &["--cache"]);
    assert!(read_cache(&cache_file)["files"].get("a.js").is_some());
}

#[test]
fn files_with_problems_are_linted_again_when_fixing() {
    let dir = setup();
    let cache_file = dir.path().join(".oxlintcache");
    run(dir.path(), &["--cache"]);
    tamper(&cache_file);

    let (output, result) = run(dir.path(), &["--cache", "--fix-suggestions"]);
    assert!(!output.contains("FROM CACHE"), "{output}");
    assert!(matches!(result, CliRunResult::LintSucceeded), "{result:?}: {output}");
    assert_ne!(fs::read_to_string(dir.path().join("a.js")).unwrap(), DEBUGGER);
}

#[test]
fn cache_location() {
    let dir = setup();

    run(dir.path(), &["--cache", "--cache-location", "custom-cache"]);
    assert!(dir.path().join("custom-cache").is_file());
    assert!(!dir.path().join(".oxlintcache").exists());

    run(dir.path(), &["--cache", "--cache-location", "cache-dir/"]);
    let files = fs::read_dir(dir.path().join("cache-dir"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(files.len(), 1, "{files:?}");
    assert!(files[0].starts_with(".cache_"), "{files:?}");

    // Running without `--cache` deletes the cache file, but never the directory.
    run(dir.path(), &["--cache-location", "cache-dir/"]);
    assert!(dir.path().join("cache-dir").is_dir());
    assert_eq!(fs::read_dir(dir.path().join("cache-dir")).unwrap().count(), 0);
}

#[test]
fn cache_path_for_directory_depends_on_cwd() {
    let dir = tempfile::tempdir().unwrap();
    let a = cache_path(Some(dir.path()), Path::new("/a"));
    let b = cache_path(Some(dir.path()), Path::new("/b"));
    assert_eq!(a.parent(), Some(dir.path()));
    assert_ne!(a, b);
    assert_eq!(a, cache_path(Some(dir.path()), Path::new("/a")));
    assert_eq!(cache_path(None, Path::new("/a")), PathBuf::from("/a/.oxlintcache"));
}

#[test]
fn invalid_cache_file_is_ignored() {
    let dir = setup();
    let cache_file = dir.path().join(".oxlintcache");
    fs::write(&cache_file, "not json").unwrap();

    let (output, result) = run(dir.path(), &["--cache"]);
    assert!(matches!(result, CliRunResult::LintFoundErrors), "{result:?}: {output}");
    assert!(read_cache(&cache_file)["files"].get("a.js").is_some());
}

#[test]
fn deleted_files_are_removed_from_cache() {
    let dir = setup();
    let cache_file = dir.path().join(".oxlintcache");
    run(dir.path(), &["--cache"]);

    // Files not linted in this run are kept while they exist.
    fs::remove_file(dir.path().join("b.js")).unwrap();
    run(dir.path(), &["--cache", "a.js"]);
    let cache = read_cache(&cache_file);
    assert!(cache["files"].get("a.js").is_some(), "{cache}");
    assert!(cache["files"].get("b.js").is_none(), "{cache}");
}

#[test]
fn cache_is_bypassed_with_suppressions() {
    let dir = setup();
    let cache_file = dir.path().join(".oxlintcache");

    run(dir.path(), &["--cache", "--suppress-all"]);
    assert!(dir.path().join("oxlint-suppressions.json").is_file());
    assert!(!cache_file.exists());

    let (output, result) = run(dir.path(), &["--cache"]);
    assert!(matches!(result, CliRunResult::LintSucceeded), "{result:?}: {output}");
    assert!(!cache_file.exists());
}

#[cfg(unix)]
#[test]
fn nothing_is_cached_after_an_unattributed_error() {
    use std::os::unix::fs::PermissionsExt;

    let dir = setup();
    let unreadable = dir.path().join("c.js");
    fs::write(&unreadable, CLEAN).unwrap();
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read(&unreadable).is_ok() {
        // Running as root.
        return;
    }

    run(dir.path(), &["--cache"]);
    let cache = read_cache(&dir.path().join(".oxlintcache"));
    assert_eq!(cache["files"].as_object().unwrap().len(), 0, "{cache}");
}

#[test]
fn replayed_diagnostic_renders_like_original() {
    let source_text = "let a = 1;\nlet b = a;\n";
    let diagnostic = OxcDiagnostic::warn("message")
        .with_error_code("eslint", "rule")
        .with_help("help")
        .with_note("note")
        .with_url("https://oxc.rs")
        .with_labels([
            LabeledSpan::new_primary_with_span(Some("primary".into()), Span::new(4, 5)),
            LabeledSpan::new_with_span(None, Span::new(15, 16)),
        ]);
    let original: Error =
        diagnostic.with_source_code(DiagnosticService::named_source("/", "/a.js", source_text));

    let cached = CachedDiagnostic::from_diagnostic(original.as_ref());
    let json = serde_json::to_string(&cached).unwrap();
    let cached: CachedDiagnostic = serde_json::from_str(&json).unwrap();
    let replayed = cached.replay(DiagnosticService::named_source("/", "/a.js", source_text));

    assert_eq!(render(replayed.as_ref()), render(original.as_ref()));
    assert_eq!(original.code(), replayed.code());
    assert_eq!(original.severity(), replayed.severity());
    assert_eq!(original.url(), replayed.url());
    assert_eq!(original.labels(), replayed.labels());
}

fn render(diagnostic: &dyn oxc_diagnostics::Diagnostic) -> String {
    let mut output = String::new();
    oxc_diagnostics::GraphicalReportHandler::new_themed(oxc_diagnostics::GraphicalTheme::none())
        .with_width(80)
        .with_links(false)
        .render_report(&mut output, diagnostic)
        .unwrap();
    output
}
