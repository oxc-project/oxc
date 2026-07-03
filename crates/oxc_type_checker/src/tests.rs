//! Behavior tests for the file-enumeration port, pinned against `tsgo` output.

use rustc_hash::FxHashMap;

use crate::vfsmatch::{Entries, FileSystemHost, read_directory};

/// In-memory [`FileSystemHost`]: maps a directory path to its (immediate) files and subdirs.
struct MemFs {
    case_sensitive: bool,
    dirs: FxHashMap<String, (Vec<String>, Vec<String>)>,
}

impl MemFs {
    fn new(case_sensitive: bool, files: &[&str]) -> Self {
        let mut dirs: FxHashMap<String, (Vec<String>, Vec<String>)> = FxHashMap::default();
        for file in files {
            let (mut dir, name) = file.rsplit_once('/').unwrap();
            if dir.is_empty() {
                dir = "/";
            }
            dirs.entry(dir.to_string()).or_default().0.push(name.to_string());
            // Record each ancestor directory's child directory.
            while let Some((parent, child)) = dir.rsplit_once('/') {
                let parent = if parent.is_empty() { "/" } else { parent };
                let subdirs = &mut dirs.entry(parent.to_string()).or_default().1;
                if !subdirs.iter().any(|d| d == child) {
                    subdirs.push(child.to_string());
                }
                if parent == "/" {
                    break;
                }
                dir = parent;
            }
        }
        for (files, subdirs) in dirs.values_mut() {
            files.sort();
            subdirs.sort();
        }
        Self { case_sensitive, dirs }
    }
}

impl FileSystemHost for MemFs {
    fn use_case_sensitive_file_names(&self) -> bool {
        self.case_sensitive
    }

    fn get_accessible_entries(&self, dir: &str) -> Entries {
        self.dirs.get(dir).map_or_else(Entries::default, |(files, dirs)| Entries {
            files: files.clone(),
            directories: dirs.clone(),
            symlinks: None,
        })
    }

    fn realpath(&self, path: &str) -> String {
        path.to_string()
    }
}

fn run(
    fs: &MemFs,
    root: &str,
    extensions: &[&str],
    excludes: &[&str],
    includes: &[&str],
    depth: usize,
) -> Vec<String> {
    let excludes: Vec<String> = excludes.iter().map(ToString::to_string).collect();
    let includes: Vec<String> = includes.iter().map(ToString::to_string).collect();
    read_directory(fs, root, root, extensions, &excludes, &includes, depth)
}

const TS: &[&str] = &[".ts"];

/// Non-ASCII patterns and file names must fail to match, not panic (Go slices bytes and
/// lets the comparison fail).
#[test]
fn non_ascii_wildcard_does_not_panic() {
    let fs = MemFs::new(true, &["/p/src/🎉x.ts", "/p/src/aéx.ts"]);
    assert_eq!(run(&fs, "/p", TS, &[], &["src/*éx.ts"], 0), ["/p/src/aéx.ts"]);

    // `?` against a multi-byte char advances one char; mid-char literal windows fail.
    let fs = MemFs::new(true, &["/p/src/é.ts", "/p/src/ab.ts"]);
    assert_eq!(run(&fs, "/p", TS, &[], &["src/?.ts"], 0), ["/p/src/é.ts"]);

    // Multi-byte name whose ".ts" suffix window is byte-aligned still matches.
    let fs = MemFs::new(false, &["/p/src/abcd🎉xxxx.ts"]);
    assert_eq!(run(&fs, "/p", TS, &[], &["src/*.ts"], 0), ["/p/src/abcd🎉xxxx.ts"]);
}

/// Case-insensitive matching uses Unicode simple folding, like Go's `strings.EqualFold`.
#[test]
fn unicode_case_folding() {
    let fs = MemFs::new(false, &["/p/Ä.ts"]);
    assert_eq!(run(&fs, "/p", TS, &[], &["ä.ts"], 0), ["/p/Ä.ts"]);
    // Kelvin sign folds to `k`.
    let fs = MemFs::new(false, &["/p/\u{212A}.ts"]);
    assert_eq!(run(&fs, "/p", TS, &[], &["k.ts"], 0), ["/p/\u{212A}.ts"]);
    // U+0130 folds only to itself.
    let fs = MemFs::new(false, &["/p/\u{0130}.ts"]);
    assert!(run(&fs, "/p", TS, &[], &["i.ts"], 0).is_empty());
    // On a case-sensitive file system nothing folds.
    let fs = MemFs::new(true, &["/p/Ä.ts"]);
    assert!(run(&fs, "/p", TS, &[], &["ä.ts"], 0).is_empty());
}

/// `depth == 0` means unlimited (Go's signed decrement never re-reaches zero); `1` walks
/// only the top level.
#[test]
fn depth_semantics() {
    let files = &["/p/top.ts", "/p/sub/mid.ts", "/p/sub/deep/bot.ts"];
    let fs = MemFs::new(true, files);
    assert_eq!(run(&fs, "/p", TS, &[], &[], 0).len(), 3);
    assert_eq!(run(&fs, "/p", TS, &[], &[], 1), ["/p/top.ts"]);
    assert_eq!(run(&fs, "/p", TS, &[], &[], 2).len(), 2);
    assert_eq!(run(&fs, "/p", TS, &[], &[], crate::vfsmatch::UNLIMITED_DEPTH).len(), 3);
}

/// A rooted include whose first component is a wildcard produces an empty include base
/// path, which must dissolve into the walked root instead of walking `current_dir` and
/// emitting mangled relative paths.
#[test]
fn rooted_wildcard_include_base_path() {
    let fs = MemFs::new(true, &["/tmp/outside.ts", "/tmp/sub/mid.ts", "/tmp/sub/deep/bot.ts"]);
    let includes = vec!["/**/*.ts".to_string()];
    let files = read_directory(&fs, "/tmp", "/tmp/sub", TS, &[], &includes, 0);
    assert_eq!(files, ["/tmp/sub/mid.ts", "/tmp/sub/deep/bot.ts"]);
}

/// `.min.js` is excluded from wildcard file matches unless the pattern mentions `.min.`,
/// with Unicode-aware suffix detection when case-insensitive.
#[test]
fn min_js_exclusion() {
    let js: &[&str] = &[".js"];
    let fs = MemFs::new(false, &["/p/lib/a.js", "/p/lib/b.MIN.js"]);
    assert_eq!(run(&fs, "/p", js, &[], &["lib/*.js"], 0), ["/p/lib/a.js"]);
    assert_eq!(
        run(&fs, "/p", js, &[], &["lib/*.min.js"], 0),
        ["/p/lib/b.MIN.js"],
        "explicit .min. pattern re-includes, case-folded"
    );
}
