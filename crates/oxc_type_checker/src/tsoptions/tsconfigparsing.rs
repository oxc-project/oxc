//! Port of typescript-go's `internal/tsoptions/tsconfigparsing.go`.
//!
//! Reads and parses a `tsconfig.json` into a [`ParsedCommandLine`]: the raw JSONC is parsed with
//! `serde_json` (comments and trailing commas stripped in place), every user-settable
//! `compilerOptions` field is converted through the declarations table
//! ([`for_each_compiler_option!`]), the `extends` chain is resolved and merged with tsc's
//! semantics (per-option atomic, child wins, explicit `null` resets), `${configDir}` template
//! values are substituted against the root config's directory, and `files`/`include`/`exclude`
//! are expanded into the concrete root-file list (tsgo `getFileNamesFromConfigSpecs`), stored
//! as [`ParsedOptions::file_names`].
//!
//! Unlike tsc this produces hard errors instead of config diagnostics (missing/cyclic `extends`,
//! unreadable or malformed files); invalid *values* are tolerated the way tsc tolerates them —
//! diagnosed-and-ignored, except we don't diagnose yet — so an unknown enum value or a
//! wrong-typed option is simply left unset.
//!
//! [`for_each_compiler_option!`]: for_each_compiler_option

use std::{
    cell::OnceCell,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use cow_utils::CowUtils;
use indexmap::IndexMap;
use oxc_resolver::{ResolveOptions, Resolver};
use rustc_hash::FxHashSet;
use serde_json::{Map, Value};

use crate::{
    core::{
        CompilerOptions, CompilerOptionsPathsMap, JsxEmit, ModuleDetectionKind, ModuleKind,
        ModuleResolutionKind, NewLineKind, ParsedOptions, ScriptTarget, for_each_compiler_option,
    },
    tspath::{change_extension, file_extension_is, file_extension_is_one_of, to_path},
    vfs::vfsmatch::{SpecMatcher, read_directory},
};

use super::parsinghelpers::merge_compiler_options;

type JsonObject = Map<String, Value>;

/// tsgo `tsoptions.ParsedCommandLine`: a parsed project tsconfig.
///
/// Wraps the parse result (tsgo `ParsedConfig`); the file specs are already expanded into
/// [`ParsedOptions::file_names`] at parse time, as in tsgo. tsgo's `ConfigFile` source file,
/// its diagnostics, and its watch/glob caches are not ported — `config_path` stands in for
/// the config file's identity.
///
/// Everything is anchored absolute: option paths are resolved against the directory of the
/// config file that defined them, `${configDir}`-template values and the file specs against
/// the root config's directory.
#[derive(Debug)]
pub struct ParsedCommandLine {
    /// The parse result (tsgo `ParsedConfig`).
    pub parsed_config: ParsedOptions,
    config_path: PathBuf,
}

impl ParsedCommandLine {
    /// The root config file this command line was parsed from.
    pub fn path(&self) -> &Path {
        &self.config_path
    }

    /// The root config file's directory — what `${configDir}` and relative specs resolve
    /// against.
    ///
    /// # Panics
    ///
    /// * When the config path is misconfigured (has no parent directory).
    pub fn directory(&self) -> &Path {
        self.config_path.parent().expect("config path is an absolute file path")
    }

    /// tsgo `ParsedCommandLine.CompilerOptions`: the merged `compilerOptions`.
    pub fn compiler_options(&self) -> &CompilerOptions {
        &self.parsed_config.compiler_options
    }

    /// tsgo `ParsedCommandLine.FileNames`: the project's root files, expanded from the
    /// config's `files`/`include`/`exclude` at parse time.
    pub fn file_names(&self) -> &[PathBuf] {
        &self.parsed_config.file_names
    }

    /// tsgo `ParsedCommandLine.ProjectReferences`. Not consumed yet — project references
    /// are a later program-loading step.
    pub fn project_references(&self) -> &[PathBuf] {
        &self.parsed_config.project_references
    }
}

/// Parse the `tsconfig.json` at `config_file` (a config file or a directory containing
/// `tsconfig.json`), resolving and merging its `extends` chain.
///
/// Mirrors tsgo's `GetParsedCommandLineOfConfigFile` / `parseJsonConfigFileContentWorker` for
/// the option and file-spec surface (type acquisition, watch options, and config diagnostics
/// are not ported).
///
/// # Errors
///
/// Returns an error when a config file is missing or malformed, or when the `extends` chain
/// cannot be resolved (missing target, circularity).
#[expect(
    clippy::missing_panics_doc,
    reason = "config_path is made absolute (with a file name) above, so parent() cannot fail"
)]
pub fn parse_config_file(config_file: &Path) -> Result<Arc<ParsedCommandLine>> {
    let cwd =
        std::env::current_dir().context("Unable to determine the current working directory")?;
    let mut config_path = to_path(&cwd, config_file);
    if config_path.is_dir() {
        config_path.push("tsconfig.json");
    }

    let mut resolution_stack = Vec::new();
    // The node_modules resolver for bare `extends` specifiers, built on first use and shared
    // down the whole `extends` chain.
    let extends_resolver = OnceCell::new();
    let parsed = parse_config(&config_path, &mut resolution_stack, &extends_resolver)?;

    // Finalization on the root config (tsgo `parseJsonConfigFileContentWorker`): stamp the
    // config path, substitute `${configDir}` values against the root config's directory, and
    // expand the file specs into the root file list.
    let config_dir = config_path.parent().expect("config path is an absolute file path");
    let mut compiler_options = parsed.options;
    compiler_options.config_file_path = Some(config_path.clone());
    handle_option_config_dir_template_substitution(&mut compiler_options, config_dir);

    let files = parsed.files.map(|specs| finalize_file_specs(specs, config_dir));
    let include = parsed.include.map(|specs| finalize_glob_specs(specs, config_dir));
    let exclude = parsed.exclude.map(|specs| finalize_glob_specs(specs, config_dir));
    let file_names =
        get_file_names_from_config_specs(files, include, exclude, config_dir, &compiler_options);

    Ok(Arc::new(ParsedCommandLine {
        parsed_config: ParsedOptions {
            compiler_options,
            file_names,
            project_references: parsed.references,
        },
        config_path,
    }))
}

/// One config file's parse result with its own `extends` chain already merged in
/// (tsgo `parsedTsconfig`).
struct ParsedTsconfig {
    options: CompilerOptions,
    /// The config's *own* explicitly-`null` options (from its raw map, not the merged result) —
    /// what an inheriting config's merge consumes to apply tsc's null-resets rule.
    own_explicit_nulls: FxHashSet<&'static str>,
    /// `files`/`include`/`exclude` specs. Relative specs belong to the *root* config's
    /// directory; specs inherited through `extends` have already been rebased to absolute
    /// paths against the config that defined them.
    files: Option<Vec<String>>,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    /// Own `references[].path`, absolute (never inherited).
    references: Vec<PathBuf>,
}

/// tsgo `parseConfig`: parse one config file and merge its `extends` chain — extended configs
/// first (in `extends` array order, later entries winning), the config's own values last.
fn parse_config(
    config_path: &Path,
    resolution_stack: &mut Vec<PathBuf>,
    extends_resolver: &OnceCell<Resolver>,
) -> Result<ParsedTsconfig> {
    if resolution_stack.iter().any(|path| path == config_path) {
        // tsgo diagnostic `Circularity_detected_while_resolving_configuration_Colon_0`.
        let chain = resolution_stack
            .iter()
            .chain(std::iter::once(&config_path.to_path_buf()))
            .map(|path| path.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" -> ");
        bail!("Circularity detected while resolving configuration: {chain}");
    }

    let json = read_json_config_file(config_path)?;
    let Value::Object(root) = json else {
        bail!("The root value of '{}' must be an object.", config_path.display());
    };
    let config_dir = config_path.parent().expect("config path is an absolute file path");

    let (own_options, own_explicit_nulls) =
        convert_compiler_options_from_json_worker(root.get("compilerOptions"), config_dir);
    let own_files = top_level_specs(&root, "files");
    let own_include = top_level_specs(&root, "include");
    let own_exclude = top_level_specs(&root, "exclude");
    let references = project_references(&root, config_dir);
    let extended_paths = get_extends_config_path_or_array(&root, config_dir, extends_resolver)?;

    if extended_paths.is_empty() {
        return Ok(ParsedTsconfig {
            options: own_options,
            own_explicit_nulls,
            files: own_files,
            include: own_include,
            exclude: own_exclude,
            references,
        });
    }

    resolution_stack.push(config_path.to_path_buf());
    let mut options = CompilerOptions::default();
    let mut inherited_files: Option<Vec<String>> = None;
    let mut inherited_include: Option<Vec<String>> = None;
    let mut inherited_exclude: Option<Vec<String>> = None;
    for extended_path in &extended_paths {
        let extended = parse_config(extended_path, resolution_stack, extends_resolver)?;
        merge_compiler_options(&mut options, &extended.options, &extended.own_explicit_nulls);
        // tsgo `applyExtendedConfig`: file specs are inherited wholesale, each relative spec
        // staying anchored to the config that defined it.
        let extended_dir = extended_path.parent().expect("config path is an absolute file path");
        if let Some(specs) = &extended.files {
            inherited_files = Some(rebase_inherited_specs(specs, extended_dir));
        }
        if let Some(specs) = &extended.include {
            inherited_include = Some(rebase_inherited_specs(specs, extended_dir));
        }
        if let Some(specs) = &extended.exclude {
            inherited_exclude = Some(rebase_inherited_specs(specs, extended_dir));
        }
    }
    resolution_stack.pop();
    merge_compiler_options(&mut options, &own_options, &own_explicit_nulls);

    Ok(ParsedTsconfig {
        options,
        own_explicit_nulls,
        files: own_files.or(inherited_files),
        include: own_include.or(inherited_include),
        exclude: own_exclude.or(inherited_exclude),
        references,
    })
}

/// Read a config file into a JSON value, stripping the BOM, comments, and trailing commas
/// (tsgo parses tsconfig with its scanner in JSON mode; a whitespace-only file is an empty
/// config).
fn read_json_config_file(config_file: &Path) -> Result<Value> {
    let text = fs::read_to_string(config_file)
        .with_context(|| format!("Cannot read file '{}'.", config_file.display()))?;
    let mut json = text.into_bytes();
    if json.starts_with(&[0xEF, 0xBB, 0xBF]) {
        json[..3].fill(b' ');
    }
    // Replaces comments and trailing commas with whitespace in place; on malformed input the
    // JSON parse below reports the error.
    _ = json_strip_comments::strip_slice(&mut json);
    if json.iter().all(u8::is_ascii_whitespace) {
        return Ok(Value::Object(JsonObject::new()));
    }
    serde_json::from_slice(&json)
        .with_context(|| format!("Failed to parse '{}'", config_file.display()))
}

/// A top-level string-array property (`files`/`include`/`exclude`), kept as raw specs.
/// Non-array values and non-string elements are ignored (tsc diagnoses-and-ignores them).
fn top_level_specs(root: &JsonObject, key: &str) -> Option<Vec<String>> {
    let items = root.get(key)?.as_array()?;
    Some(items.iter().filter_map(Value::as_str).map(str::to_string).collect())
}

/// Own `references[].path` resolved against the config's directory (tsgo reads `references`
/// from the raw config only — it does not flow through `extends`).
fn project_references(root: &JsonObject, config_dir: &Path) -> Vec<PathBuf> {
    root.get("references")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_object)
                .filter_map(|reference| reference.get("path").and_then(Value::as_str))
                .map(|path| to_path(config_dir, Path::new(path)))
                .collect()
        })
        .unwrap_or_default()
}

/// tsgo `getExtendsConfigPathOrArray`: the `extends` value as resolved config paths. A single
/// specifier or an array (array order preserved — later entries win the merge). tsgo skips a
/// top-level empty-string `extends` without error; an empty string *inside* an array errors.
fn get_extends_config_path_or_array(
    root: &JsonObject,
    config_dir: &Path,
    extends_resolver: &OnceCell<Resolver>,
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    match root.get("extends") {
        Some(Value::String(specifier)) if !specifier.is_empty() => {
            paths.push(get_extends_config_path(specifier, config_dir, extends_resolver)?);
        }
        Some(Value::Array(items)) => {
            for item in items {
                if let Some(specifier) = item.as_str() {
                    paths.push(get_extends_config_path(specifier, config_dir, extends_resolver)?);
                }
            }
        }
        _ => {}
    }
    Ok(paths)
}

/// tsgo `getExtendsConfigPath`: resolve one `extends` specifier. Rooted and `./`/`../`
/// specifiers resolve against the config's directory, retrying with `.json` appended;
/// anything else resolves like a module through `node_modules` (tsgo `module.ResolveConfig`).
fn get_extends_config_path(
    specifier: &str,
    config_dir: &Path,
    extends_resolver: &OnceCell<Resolver>,
) -> Result<PathBuf> {
    if specifier.is_empty() {
        // TS18051
        bail!("Compiler option 'extends' cannot be given an empty string.");
    }
    // tsgo normalizes slashes first, so the Windows-style `.\` form counts as relative.
    let normalized = specifier.cow_replace('\\', "/");
    if Path::new(normalized.as_ref()).is_absolute()
        || normalized.starts_with("./")
        || normalized.starts_with("../")
    {
        let mut extended = to_path(config_dir, Path::new(normalized.as_ref()));
        if !extended.is_file() {
            if extended.to_string_lossy().ends_with(".json") {
                // TS6053
                bail!("File '{specifier}' not found.");
            }
            let mut with_json = extended.into_os_string();
            with_json.push(".json");
            extended = PathBuf::from(with_json);
            if !extended.is_file() {
                bail!("File '{specifier}' not found.");
            }
        }
        return Ok(extended);
    }
    // Bare (or `#`-prefixed) specifier: node-style config lookup, sharing one resolver (and
    // its filesystem cache) across the `extends` chain. The lookup mirrors tsgo's
    // `module.ResolveConfig` — `.json` extension, the package.json `tsconfig` field as the
    // entry point, `tsconfig.json` as the directory default.
    let resolver = extends_resolver.get_or_init(|| {
        Resolver::new(ResolveOptions {
            condition_names: vec!["node".to_string(), "import".to_string()],
            extensions: vec![".json".to_string()],
            main_fields: vec!["tsconfig".to_string()],
            main_files: vec!["tsconfig".to_string()],
            ..ResolveOptions::default()
        })
    });
    match resolver.resolve(config_dir, specifier) {
        Ok(resolution) => Ok(to_path(config_dir, resolution.path())),
        Err(_) => bail!("File '{specifier}' not found."),
    }
}

/// tsgo `applyExtendedConfig`'s spec rebasing: an inherited relative spec stays anchored to
/// the directory of the config that defined it; rooted and `${configDir}` specs pass through.
fn rebase_inherited_specs(specs: &[String], extended_dir: &Path) -> Vec<String> {
    specs
        .iter()
        .map(|spec| {
            if starts_with_config_dir_template(spec) || Path::new(spec).is_absolute() {
                spec.clone()
            } else {
                to_path(extended_dir, Path::new(spec)).to_string_lossy().into_owned()
            }
        })
        .collect()
}

/// Resolve the root config's literal `files` to absolute paths.
fn finalize_file_specs(specs: Vec<String>, config_dir: &Path) -> Vec<PathBuf> {
    specs.into_iter().map(|spec| finalize_spec(&spec, config_dir)).collect()
}

/// The same anchoring for `include`/`exclude`, which stay glob-spec strings (they are
/// patterns, not paths — tsgo keeps its validated specs as strings too).
fn finalize_glob_specs(specs: Vec<String>, config_dir: &Path) -> Vec<String> {
    specs
        .into_iter()
        .map(|spec| finalize_spec(&spec, config_dir).to_string_lossy().into_owned())
        .collect()
}

/// Anchor one root-config spec absolute: `${configDir}` and still-relative (own) specs
/// resolve against the root config's directory.
fn finalize_spec(spec: &str, config_dir: &Path) -> PathBuf {
    if starts_with_config_dir_template(spec) {
        get_substituted_path_with_config_dir_template(spec, config_dir)
    } else {
        to_path(config_dir, Path::new(spec))
    }
}

/// tsgo `getFileNamesFromConfigSpecs`: expand `files`/`include`/`exclude` into the project's
/// root files. Literal `files` are always kept; `include` globs (default `**/*` when neither
/// `files` nor `include` is given) are walked from the tsconfig directory via
/// `read_directory`; `.json` files require an include spec that targets JSON; and
/// higher-priority extensions win over lower ones (`a.ts` over a sibling `a.d.ts`/`a.js`).
fn get_file_names_from_config_specs(
    files: Option<Vec<PathBuf>>,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    base: &Path,
    options: &CompilerOptions,
) -> Vec<PathBuf> {
    let extension_groups = get_supported_extensions(options);

    // Flat suffix list for the directory walk, plus `.json` when JSON modules resolve
    // (tsgo walks with `GetSupportedExtensionsWithJsonIfResolveJsonModule`).
    let walk_extensions = get_supported_extensions_with_json_flat(options);

    // Literal `files` (already absolute) are always included and cannot be removed by
    // `include`/`exclude`.
    let has_files = files.is_some();
    let mut literal_files = Vec::new();
    let mut literal_keys = FxHashSet::default();
    for file in files.into_iter().flatten() {
        if literal_keys.insert(file.to_string_lossy().into_owned()) {
            literal_files.push(file);
        }
    }

    // `include` specs. Default to `**/*` only when neither `files` nor `include` is present.
    let include_specs: Vec<String> = match include {
        Some(include) => include,
        None if !has_files => vec![base.join("**/*").to_string_lossy().into_owned()],
        None => Vec::new(),
    };
    let exclude_specs: Vec<String> = exclude.unwrap_or_default();

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

/// tsgo's `${configDir}` template variable (`tsconfigparsing.go`), recognized only as a prefix,
/// case-insensitively.
const TEMPLATE_VARIABLE: &str = "${configDir}";

fn starts_with_config_dir_template(value: &str) -> bool {
    value
        .get(..TEMPLATE_VARIABLE.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(TEMPLATE_VARIABLE))
}

/// tsgo `getSubstitutedPathWithConfigDirTemplate`: replace the `${configDir}` prefix with the
/// root config's directory and normalize.
fn get_substituted_path_with_config_dir_template(value: &str, config_dir: &Path) -> PathBuf {
    let rest = value[TEMPLATE_VARIABLE.len()..].trim_start_matches(['/', '\\']);
    to_path(config_dir, Path::new(rest))
}

/// tsgo `normalizeNonListOptionValue` for path-typed options: `${configDir}`-prefixed values
/// survive as written until the post-merge substitution; everything else is absolutized
/// against the directory of the config file that defined it.
fn convert_path_value(value: &str, config_dir: &Path) -> PathBuf {
    if starts_with_config_dir_template(value) {
        PathBuf::from(value)
    } else {
        to_path(config_dir, Path::new(value))
    }
}

/// tsconfig `paths`: pattern -> substitutions, kept as written (`${configDir}` substitution
/// happens post-merge; relative substitutions resolve against `paths_base_path`).
fn convert_paths_map(object: &JsonObject) -> CompilerOptionsPathsMap {
    let mut paths = CompilerOptionsPathsMap::default();
    for (pattern, substitutions) in object {
        let Some(substitutions) = substitutions.as_array() else { continue };
        paths.insert(
            pattern.clone(),
            substitutions.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        );
    }
    paths
}

/// One conversion expression per option kind. A wrong-typed value yields `None` — tsc reports
/// a diagnostic and leaves the option unset; we just leave it unset.
macro_rules! convert_value {
    ($value:ident, $config_dir:ident, bool) => {
        $value.as_bool()
    };
    ($value:ident, $config_dir:ident, string) => {
        $value.as_str().map(str::to_string)
    };
    ($value:ident, $config_dir:ident, path) => {
        $value.as_str().map(|value| convert_path_value(value, $config_dir))
    };
    ($value:ident, $config_dir:ident, string_list) => {
        $value.as_array().map(|items| {
            items.iter().filter_map(Value::as_str).map(str::to_string).collect::<Vec<_>>()
        })
    };
    ($value:ident, $config_dir:ident, path_list) => {
        $value.as_array().map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(|value| convert_path_value(value, $config_dir))
                .collect::<Vec<_>>()
        })
    };
    ($value:ident, $config_dir:ident, paths_map) => {
        $value.as_object().map(convert_paths_map)
    };
    ($value:ident, $config_dir:ident, number) => {
        $value.as_i64().and_then(|number| i32::try_from(number).ok())
    };
    ($value:ident, $config_dir:ident, enum($ty:ty)) => {
        $value.as_str().and_then(<$ty>::from_str_ignore_case)
    };
}

macro_rules! define_convert_compiler_options {
    ($(($field:ident, $json:literal, $($kind:tt)+)),* $(,)?) => {
        /// tsgo `convertCompilerOptionsFromJsonWorker`: convert a raw `compilerOptions` object
        /// through the declarations table. Returns the options plus the set of options that
        /// were explicitly `null` (tsc's "reset, don't inherit" marker for the `extends`
        /// merge).
        fn convert_compiler_options_from_json_worker(
            json: Option<&Value>,
            config_dir: &Path,
        ) -> (CompilerOptions, FxHashSet<&'static str>) {
            let mut options = CompilerOptions::default();
            let mut explicit_nulls = FxHashSet::default();
            let Some(json) = json.and_then(Value::as_object) else {
                return (options, explicit_nulls);
            };
            $(
                match json.get($json) {
                    None => {}
                    Some(Value::Null) => {
                        explicit_nulls.insert($json);
                    }
                    Some(value) => {
                        options.$field = convert_value!(value, config_dir, $($kind)+);
                    }
                }
            )*
            if options.paths.is_some() {
                // tsgo `parseOwnConfigOfJson`: `paths` resolve against the directory of the
                // config that defined them.
                options.paths_base_path = Some(config_dir.to_path_buf());
            }
            (options, explicit_nulls)
        }
    };
}
for_each_compiler_option!(define_convert_compiler_options);

/// The post-merge `${configDir}` pass for one option, keyed by kind: only path-typed options
/// participate (tsgo `handleOptionConfigDirTemplateSubstitution` covers exactly the path
/// options plus `paths` values).
macro_rules! substitute_field {
    ($options:ident, $config_dir:ident, $field:ident, path) => {
        if let Some(value) = &mut $options.$field {
            let substituted = value
                .to_str()
                .filter(|text| starts_with_config_dir_template(text))
                .map(|text| get_substituted_path_with_config_dir_template(text, $config_dir));
            if let Some(substituted) = substituted {
                *value = substituted;
            }
        }
    };
    ($options:ident, $config_dir:ident, $field:ident, path_list) => {
        if let Some(values) = &mut $options.$field {
            for value in values.iter_mut() {
                let substituted = value
                    .to_str()
                    .filter(|text| starts_with_config_dir_template(text))
                    .map(|text| get_substituted_path_with_config_dir_template(text, $config_dir));
                if let Some(substituted) = substituted {
                    *value = substituted;
                }
            }
        }
    };
    ($options:ident, $config_dir:ident, $field:ident, paths_map) => {
        if let Some(paths) = &mut $options.$field {
            for substitutions in paths.values_mut() {
                for substitution in substitutions.iter_mut() {
                    if starts_with_config_dir_template(substitution) {
                        let substituted = get_substituted_path_with_config_dir_template(
                            substitution,
                            $config_dir,
                        );
                        *substitution = substituted.to_string_lossy().into_owned();
                    }
                }
            }
        }
    };
    ($options:ident, $config_dir:ident, $field:ident, $($other:tt)+) => {};
}

macro_rules! define_substitute_config_dir {
    ($(($field:ident, $json:literal, $($kind:tt)+)),* $(,)?) => {
        /// tsgo `handleOptionConfigDirTemplateSubstitution`: after the `extends` merge,
        /// substitute `${configDir}`-prefixed path options against the *root* config's
        /// directory (which is why parse-time absolutization skips them).
        fn handle_option_config_dir_template_substitution(options: &mut CompilerOptions, config_dir: &Path) {
            $( substitute_field!(options, config_dir, $field, $($kind)+); )*
        }
    };
}
for_each_compiler_option!(define_substitute_config_dir);

/// tsgo `tspath.SupportedTSExtensions`, grouped by resolution priority.
const SUPPORTED_TS_EXTENSIONS: &[&[&str]] =
    &[&[".ts", ".tsx", ".d.ts"], &[".cts", ".d.cts"], &[".mts", ".d.mts"]];

/// tsgo `tspath.AllSupportedExtensions` (TS + JS), grouped by resolution priority.
const ALL_SUPPORTED_EXTENSIONS: &[&[&str]] = &[
    &[".ts", ".tsx", ".d.ts", ".js", ".jsx"],
    &[".cts", ".d.cts", ".cjs"],
    &[".mts", ".d.mts", ".mjs"],
];

/// tsgo `tspath.SupportedTSExtensionsWithJsonFlat`: the extensions of resolved imports that are
/// *not* considered JavaScript files by `resolveImportsAndModuleAugmentations`'s gating.
pub const SUPPORTED_TS_EXTENSIONS_WITH_JSON_FLAT: &[&str] =
    &[".ts", ".tsx", ".d.ts", ".cts", ".d.cts", ".mts", ".d.mts", ".json"];

/// tsgo `GetSupportedExtensions`: JS extensions are added when `allowJs` (or `checkJs`) is on.
pub fn get_supported_extensions(options: &CompilerOptions) -> &'static [&'static [&'static str]] {
    if options.get_allow_js() { ALL_SUPPORTED_EXTENSIONS } else { SUPPORTED_TS_EXTENSIONS }
}

/// tsgo `GetSupportedExtensionsWithJsonIfResolveJsonModule`, flattened: every supported
/// extension, plus `.json` when JSON modules resolve.
pub fn get_supported_extensions_with_json_flat(options: &CompilerOptions) -> Vec<&'static str> {
    let mut extensions: Vec<&'static str> =
        get_supported_extensions(options).iter().flat_map(|group| group.iter().copied()).collect();
    if options.get_resolve_json_module() {
        extensions.push(".json");
    }
    extensions
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
