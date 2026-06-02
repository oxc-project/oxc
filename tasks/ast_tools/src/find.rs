//! Finding crates and source files containing AST types via `cargo metadata`.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

/// Name of `oxc_ast_macros` crate.
const AST_MACROS_CRATE: &str = "oxc_ast_macros";

/// Prefix for `oxc_ast` crate source files.
const AST_FILES_PREFIX: &str = "crates/oxc_ast/src/";

/// Files which must appear first in the file list, in this order.
///
/// These are the core AST definition files. Keeping them at the top of the list
/// produces generated code in a readable order, with `Program` first.
///
/// Paths are relative to [`AST_FILES_PREFIX`].
const AST_PRIORITY_PATHS: &[&str] =
    &["ast/js.rs", "ast/literal.rs", "ast/jsx.rs", "ast/ts.rs", "ast/comment.rs"];

/// Find all crates and source files containing AST types.
///
/// Runs `cargo metadata` to find all workspace crates that depend on `oxc_ast_macros`,
/// then finds all `.rs` files in their `src` directories (excluding `src/generated`).
pub fn find_crates_and_files() -> (
    // Crate paths, relative to workspace root
    Vec<String>,
    // File paths of `.rs` files in these crates, relative to workspace root
    Vec<String>,
    // Workspace root path
    PathBuf,
) {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .expect("Failed to run cargo metadata");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("cargo metadata failed: {stderr}");
    }

    let metadata: Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse cargo metadata JSON");

    // Find crates that depend on `oxc_ast_macros`
    let root_path =
        metadata["workspace_root"].as_str().expect("Missing workspace_root in cargo metadata");
    let root_path = PathBuf::from(root_path);

    let packages = metadata["packages"].as_array().expect("Missing packages in cargo metadata");

    // Get paths of crates that depend on `oxc_ast_macros`, and all `.rs` files in those crates
    let mut crate_paths = Vec::new();
    let mut file_paths = Vec::new();

    for package in packages {
        let has_dependency = package["dependencies"]
            .as_array()
            .expect("Missing dependencies array")
            .iter()
            .any(|dep| dep["name"].as_str().unwrap() == AST_MACROS_CRATE);

        if !has_dependency {
            continue;
        }

        let manifest_path = package["manifest_path"].as_str().expect("Missing manifest_path");
        let crate_absolute_path = Path::new(manifest_path).parent().expect("Invalid manifest path");

        // Find all `.rs` files in this crate's `src` directory
        let src_dir = crate_absolute_path.join("src");
        assert!(src_dir.is_dir(), "`src` directory not found");
        find_rs_files(&src_dir, &root_path, true, &mut file_paths);

        let crate_path = path_to_normalized_string(
            crate_absolute_path.strip_prefix(&root_path).expect("Crate path not in workspace"),
        );
        crate_paths.push(crate_path);
    }

    crate_paths.sort_unstable();

    // Sort file paths: `oxc_ast` crate first (with priority paths at the top),
    // then remaining files alphabetically.
    #[expect(clippy::items_after_statements)]
    fn rank(path: &str) -> (usize, &str) {
        match path.strip_prefix(AST_FILES_PREFIX) {
            Some(s) => {
                (AST_PRIORITY_PATHS.iter().position(|&p| p == s).unwrap_or(usize::MAX - 1), path)
            }
            None => (usize::MAX, path),
        }
    }
    file_paths.sort_unstable_by(|a, b| rank(a).cmp(&rank(b)));

    (crate_paths, file_paths, root_path)
}

/// Recursively find all `.rs` files in a directory.
///
/// If `skip_generated` is `true`, skips subdirectory named `generated`.
/// This is used to exclude `src/generated` (which contains codegen output, not input).
fn find_rs_files(dir: &Path, root_path: &Path, skip_generated: bool, paths: &mut Vec<String>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("Failed to read directory {}: {err}", dir.display()));

    for entry in entries {
        let entry = entry.unwrap_or_else(|err| panic!("Failed to read directory entry: {err}"));

        let path = entry.path();
        if path.is_dir() {
            if skip_generated && entry.file_name() == "generated" {
                continue;
            }
            find_rs_files(&path, root_path, false, paths);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let relative_path = path.strip_prefix(root_path).expect("File not in workspace");
            paths.push(path_to_normalized_string(relative_path));
        }
    }
}

/// Convert a `Path` to a string using linux-style forward slashes.
///
/// On Linux/Mac, just converts to a string.
/// On Windows, backslashes are replaced with forward slashes, so that paths are consistent across platforms.
fn path_to_normalized_string(path: &Path) -> String {
    if cfg!(windows) {
        #[expect(clippy::disallowed_methods)]
        path.to_str().expect("Path is not valid UTF-8").replace('\\', "/")
    } else {
        path.to_string_lossy().into_owned()
    }
}
