//! Port of typescript-go's `internal/tsoptions/tsconfigparsing.go`.
//!
//! Reads and parses a `tsconfig.json`, then expands its `files`/`include`/`exclude` into the
//! concrete list of root files. The JSONC parse and the `extends`/`references` search are
//! delegated to `oxc_resolver`; the file-glob expansion mirrors tsgo (see the `vfs` module).

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use indexmap::IndexMap;
use oxc_resolver::{CompilerOptions, ResolveOptions, Resolver, TsConfig};
use rustc_hash::FxHashSet;

use crate::{
    tspath::{change_extension, file_extension_is, file_extension_is_one_of},
    vfs::vfsmatch::{SpecMatcher, read_directory},
};

/// tsgo `tspath.SupportedTSExtensions`, grouped by resolution priority.
const SUPPORTED_TS_EXTENSIONS: &[&[&str]] =
    &[&[".ts", ".tsx", ".d.ts"], &[".cts", ".d.cts"], &[".mts", ".d.mts"]];

/// tsgo `tspath.AllSupportedExtensions` (TS + JS), grouped by resolution priority.
const ALL_SUPPORTED_EXTENSIONS: &[&[&str]] = &[
    &[".ts", ".tsx", ".d.ts", ".js", ".jsx"],
    &[".cts", ".d.cts", ".cjs"],
    &[".mts", ".d.mts", ".mjs"],
];

/// Parse the `tsconfig.json` at `config_file`, resolving its `extends` chain.
///
/// Mirrors tsgo's `GetParsedCommandLineOfConfigFile`, but delegates the JSONC parse and the
/// `extends`/`references` search to `oxc_resolver`. `config_file` may be a path to a config
/// file or to a directory (in which case `tsconfig.json` is assumed).
///
/// # Errors
///
/// Returns the `oxc_resolver` error text when the file is missing, or when the tsconfig (or
/// any config it `extends`) is invalid.
pub fn parse_config_file(config_file: &Path) -> Result<Arc<TsConfig>, String> {
    let resolver = Resolver::new(ResolveOptions::default());
    resolver.resolve_tsconfig(config_file).map_err(|error| error.to_string())
}

/// Expand a parsed `tsconfig` into the list of root files to type check.
///
/// Faithful port of tsgo's `getFileNamesFromConfigSpecs`: literal `files` are always kept;
/// `include` globs (default `**/*` when neither `files` nor `include` is given) are walked
/// from the tsconfig directory via `read_directory`; `.json` files require an include spec
/// that targets JSON; and higher-priority extensions win over lower ones (`a.ts` over a
/// sibling `a.d.ts`/`a.js`).
#[must_use]
pub fn get_file_names(tsconfig: &TsConfig) -> Vec<PathBuf> {
    let base = tsconfig.directory();
    let options = &tsconfig.compiler_options;
    let extension_groups = supported_extension_groups(options);

    // Flat suffix list for the directory walk, plus `.json` when `resolveJsonModule` is on.
    let mut walk_extensions: Vec<&str> =
        extension_groups.iter().flat_map(|group| group.iter().copied()).collect();
    if options.resolve_json_module == Some(true) {
        walk_extensions.push(".json");
    }

    // Literal `files` are always included and cannot be removed by `include`/`exclude`.
    let mut literal_files = Vec::new();
    let mut literal_keys = FxHashSet::default();
    if let Some(files) = &tsconfig.files {
        for file in files {
            let file = if file.is_absolute() { file.clone() } else { base.join(file) };
            if literal_keys.insert(file.to_string_lossy().into_owned()) {
                literal_files.push(file);
            }
        }
    }

    // `include` specs. Default to `**/*` only when neither `files` nor `include` is present.
    let include_specs: Vec<String> = match &tsconfig.include {
        Some(include) => include.iter().map(|path| path_to_string(path)).collect(),
        None if tsconfig.files.is_none() => vec![base.join("**/*").to_string_lossy().into_owned()],
        None => Vec::new(),
    };
    let exclude_specs: Vec<String> =
        tsconfig.exclude.iter().flatten().map(|path| path_to_string(path)).collect();

    let mut wildcard_files: IndexMap<String, PathBuf> = IndexMap::new();
    let mut json_files: IndexMap<String, PathBuf> = IndexMap::new();

    if !include_specs.is_empty() {
        // `.json` files are only picked up by an include spec that specifically targets JSON.
        let json_specs: Vec<String> =
            include_specs.iter().filter(|spec| is_json_spec(spec)).cloned().collect();
        let json_matcher = SpecMatcher::new(&json_specs);

        for file in read_directory(base, &walk_extensions, &exclude_specs, &include_specs) {
            let key = file.to_string_lossy().into_owned();

            if file_extension_is(&key, ".json") {
                if json_matcher.matches(&file)
                    && !literal_keys.contains(&key)
                    && !json_files.contains_key(&key)
                {
                    json_files.insert(key, file);
                }
                continue;
            }

            // Skip when a higher-priority extension of the same file was already included.
            if has_file_with_higher_priority_extension(
                &key,
                extension_groups,
                &literal_keys,
                &wildcard_files,
            ) {
                continue;
            }
            // Drop any lower-priority extension of the same file added earlier.
            remove_wildcard_files_with_lower_priority_extension(
                &key,
                &mut wildcard_files,
                extension_groups,
            );
            if !literal_keys.contains(&key) && !wildcard_files.contains_key(&key) {
                wildcard_files.insert(key, file);
            }
        }
    }

    let mut result =
        Vec::with_capacity(literal_files.len() + wildcard_files.len() + json_files.len());
    result.extend(literal_files);
    result.extend(wildcard_files.into_values());
    result.extend(json_files.into_values());
    result
}

/// tsgo `GetSupportedExtensions`: JS extensions are added when `allowJs` (or `checkJs`) is on.
fn supported_extension_groups(options: &CompilerOptions) -> &'static [&'static [&'static str]] {
    // tsgo `GetAllowJS`: use `allowJs` if set, otherwise fall back to `checkJs`.
    let allow_js = options.allow_js.unwrap_or(options.check_js == Some(true));
    if allow_js { ALL_SUPPORTED_EXTENSIONS } else { SUPPORTED_TS_EXTENSIONS }
}

/// tsgo `hasFileWithHigherPriorityExtension`.
fn has_file_with_higher_priority_extension(
    file: &str,
    extension_groups: &[&[&str]],
    literal_keys: &FxHashSet<String>,
    wildcard_files: &IndexMap<String, PathBuf>,
) -> bool {
    let group = extension_group_for(file, extension_groups);
    for &ext in &group {
        // Reached the file's own extension without finding a higher-priority sibling.
        // `.d.ts` also ends with `.ts`, so don't let it satisfy the `.ts` slot.
        if file_extension_is(file, ext) && (ext != ".ts" || !file_extension_is(file, ".d.ts")) {
            return false;
        }
        let sibling = change_extension(file, ext);
        if literal_keys.contains(&sibling) || wildcard_files.contains_key(&sibling) {
            // Legacy: allow a `.d.ts` to be loaded alongside its `.js`/`.jsx` counterpart.
            if ext == ".d.ts" && (file_extension_is(file, ".js") || file_extension_is(file, ".jsx"))
            {
                continue;
            }
            return true;
        }
    }
    false
}

/// tsgo `removeWildcardFilesWithLowerPriorityExtension`.
fn remove_wildcard_files_with_lower_priority_extension(
    file: &str,
    wildcard_files: &mut IndexMap<String, PathBuf>,
    extension_groups: &[&[&str]],
) {
    let group = extension_group_for(file, extension_groups);
    for &ext in group.iter().rev() {
        if file_extension_is(file, ext) {
            return;
        }
        wildcard_files.shift_remove(&change_extension(file, ext));
    }
}

/// The flattened extension group(s) `file` belongs to (tsgo unions all matching groups).
fn extension_group_for<'a>(file: &str, extension_groups: &[&'a [&'a str]]) -> Vec<&'a str> {
    let mut group = Vec::new();
    for &candidate in extension_groups {
        if file_extension_is_one_of(file, candidate) {
            group.extend_from_slice(candidate);
        }
    }
    group
}

/// Whether an include spec specifically targets `.json` files (e.g. `**/*.json`, `a.json`).
fn is_json_spec(spec: &str) -> bool {
    Path::new(spec).extension().is_some_and(|ext| ext == "json")
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
