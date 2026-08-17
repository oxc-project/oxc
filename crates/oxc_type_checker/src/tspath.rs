//! Small subset of typescript-go's `internal/tspath` needed by the file-matching port.
//!
//! The string helpers below operate on absolute, already-normalized POSIX-style paths (this is what
//! `oxc_resolver` hands us) and so do not re-implement `.`/`..` reduction; [`to_path`] is the
//! normalizer that produces such a path from an arbitrary input.

use std::path::{Path, PathBuf};

use oxc_resolver::PathUtil;

/// The extensions recognized when stripping/replacing a file extension, longest first so
/// `.d.ts` is matched before `.ts`. Mirrors tsgo's declaration-aware `ChangeExtension`.
const EXTENSIONS_TO_REMOVE: &[&str] = &[
    ".d.ts", ".d.cts", ".d.mts", ".tsx", ".ts", ".cts", ".mts", ".jsx", ".js", ".cjs", ".mjs",
    ".json",
];

/// tsgo `FileExtensionIs`: `path` ends with `extension` (and is longer than it).
pub fn file_extension_is(path: &str, extension: &str) -> bool {
    path.len() > extension.len() && path.ends_with(extension)
}

/// tsgo `FileExtensionIsOneOf`.
pub fn file_extension_is_one_of(path: &str, extensions: &[&str]) -> bool {
    extensions.iter().any(|extension| file_extension_is(path, extension))
}

/// tsgo `GetBaseFileName`: the final path segment.
pub fn get_base_file_name(path: &str) -> &str {
    match path.rfind('/') {
        Some(index) => &path[index + 1..],
        None => path,
    }
}

/// tsgo `HasExtension`: the base name contains a `.`.
pub fn has_extension(file_name: &str) -> bool {
    get_base_file_name(file_name).contains('.')
}

/// tsgo `GetDirectoryPath`: everything up to (not including) the final separator.
pub fn get_directory_path(path: &str) -> &str {
    match path.rfind('/') {
        Some(0) => "/",
        Some(index) => &path[..index],
        None => "",
    }
}

/// tsgo `ChangeExtension`: replace the file's recognized extension with `new_extension`.
pub fn change_extension(path: &str, new_extension: &str) -> String {
    for extension in EXTENSIONS_TO_REMOVE {
        if file_extension_is(path, extension) {
            return format!("{}{new_extension}", &path[..path.len() - extension.len()]);
        }
    }
    format!("{path}{new_extension}")
}

/// tsgo `IsExternalModuleNameRelative`: the module name starts with `./`, `../`, or is `.`/`..`
/// (`tspath.PathIsRelative`, including the Windows `.\` forms).
pub fn is_external_module_name_relative(module_name: &str) -> bool {
    let rest = if let Some(rest) = module_name.strip_prefix("..") {
        rest
    } else if let Some(rest) = module_name.strip_prefix('.') {
        rest
    } else {
        return false;
    };
    rest.is_empty() || rest.starts_with('/') || rest.starts_with('\\')
}

/// tsgo `ToPath` / `GetNormalizedAbsolutePath`: resolve `file_name` against
/// `current_directory` into an absolute, lexically-normalized path (collapsing `.`/`..`).
///
/// Reuses `oxc_resolver`'s [`PathUtil`]: `normalize_with` makes the path absolute (an absolute
/// `file_name` replaces the base), and `normalize` then collapses `.`/`..` — including for an
/// absolute `file_name`, which `normalize_with` passes through unchanged. Unlike tsgo's `ToPath`,
/// this does not case-fold on case-insensitive file systems — case canonicalization is deferred,
/// matching the case-sensitive matching used elsewhere.
pub fn to_path(current_directory: &Path, file_name: &Path) -> PathBuf {
    current_directory.normalize_with(file_name).normalize()
}
