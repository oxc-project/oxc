//! Port of typescript-go's `internal/tsoptions/tsconfigparsing.go`.
//!
//! Reads and parses a `tsconfig.json` into a [`ParsedConfig`]: the raw JSONC is parsed with
//! `serde_json` (comments and trailing commas stripped in place), every user-settable
//! `compilerOptions` field is converted through the declarations table
//! ([`for_each_compiler_option!`]), the `extends` chain is resolved and merged with tsc's
//! semantics (per-option atomic, child wins, explicit `null` resets), `${configDir}` template
//! values are substituted against the root config's directory, and `files`/`include`/`exclude`
//! are expanded into the concrete root-file list (tsgo `getFileNamesFromConfigSpecs`), stored
//! as [`ParsedOptions::file_names`].
//!
//! Unlike tsc this produces hard errors instead of config diagnostics (missing/cyclic `extends`,
//! unreadable or malformed files — where tsgo diagnoses and still yields a usable config);
//! invalid *values* are tolerated the way tsc tolerates them — diagnosed-and-ignored, except we
//! don't diagnose yet — so an unknown enum value or a wrong-typed option is simply left unset.
//! `compileOnSave`, `typeAcquisition`, and `watchOptions` are not ported.
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
        ModuleResolutionKind, NewLineKind, ParsedOptions, ProjectReference, ScriptTarget,
        for_each_compiler_option,
    },
    tspath::{
        change_extension, file_extension_is, file_extension_is_one_of, is_rooted_disk_path, to_path,
    },
    vfs::vfsmatch::{SpecMatcher, read_directory},
};

use super::{
    enummaps::get_lib_file_name,
    parsinghelpers::{merge_compiler_options, parse_number},
};

type JsonObject = Map<String, Value>;

/// A parsed project configuration.
///
/// Port of tsgo `tsoptions.ParsedCommandLine`, renamed: the tsc name is historical — command-
/// line arguments and tsconfig files parse into this same shape, and tsc merges CLI overrides
/// into the config's parse result (tsgo `execute.tscCompilation`). `ParsedConfig` names what
/// the value is; when `oxcheck` grows tsc-style option flags, they will merge into this type
/// exactly as in tsgo. The accessors keep their tsgo names.
///
/// Wraps the parse result ([`ParsedOptions`]); the file specs are already expanded into
/// [`ParsedOptions::file_names`] at parse time, as in tsgo. tsgo's `ConfigFile` source file,
/// its diagnostics, and its watch/glob caches are not ported — `config_path` stands in for
/// the config file's identity.
///
/// Everything is anchored absolute: option paths are resolved against the directory of the
/// config file that defined them, `${configDir}`-template values and the file specs against
/// the root config's directory.
#[derive(Debug)]
pub struct ParsedConfig {
    /// The parse result (tsgo's `ParsedConfig` field).
    pub parsed_options: ParsedOptions,
    config_path: PathBuf,
}

impl ParsedConfig {
    /// The root config file this configuration was parsed from.
    pub fn path(&self) -> &Path {
        &self.config_path
    }

    /// tsgo `ParsedCommandLine.CompilerOptions`: the merged `compilerOptions`.
    pub fn compiler_options(&self) -> &CompilerOptions {
        &self.parsed_options.compiler_options
    }

    /// tsgo `ParsedCommandLine.FileNames`: the project's root files, expanded from the
    /// config's `files`/`include`/`exclude` at parse time.
    pub fn file_names(&self) -> &[PathBuf] {
        &self.parsed_options.file_names
    }

    /// tsgo `ParsedCommandLine.ProjectReferences`. Not consumed yet — project references
    /// are a later program-loading step.
    pub fn project_references(&self) -> &[ProjectReference] {
        &self.parsed_options.project_references
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
pub fn parse_config_file(config_file: &Path) -> Result<Arc<ParsedConfig>> {
    let cwd =
        std::env::current_dir().context("Unable to determine the current working directory")?;
    let mut config_path = to_path(&cwd, config_file);
    if config_path.is_dir() {
        config_path.push("tsconfig.json");
    }

    let config_dir = config_path.parent().expect("config path is an absolute file path");
    let json = read_json_config_file(&config_path)?;
    // tsgo `getProjectReferences`: references never inherit through `extends`, so they are
    // read once here at the worker level, from the root raw config only.
    let project_references = match json.as_object() {
        Some(root) => get_project_references(root, config_dir),
        None => Vec::new(),
    };

    let mut resolution_stack = Vec::new();
    // The node_modules resolver for bare `extends` specifiers, built on first use and shared
    // down the whole `extends` chain.
    let extends_resolver = OnceCell::new();
    let parsed = parse_config(&config_path, json, &mut resolution_stack, &extends_resolver)?;

    // Finalization on the root config (tsgo `parseJsonConfigFileContentWorker`): stamp the
    // config path, substitute `${configDir}` values against the root config's directory, and
    // expand the file specs into the root file list.
    let mut compiler_options = parsed.options;
    compiler_options.config_file_path = Some(config_path.clone());
    handle_option_config_dir_template_substitution(&mut compiler_options, config_dir);

    let files = match parsed.files {
        SpecsFromRaw::Specs(specs) => Some(finalize_file_specs(specs, config_dir)),
        _ => None,
    };
    let include = match parsed.include {
        SpecsFromRaw::Specs(specs) => Some(finalize_glob_specs(specs, config_dir)),
        _ => None,
    };
    let exclude = match parsed.exclude {
        SpecsFromRaw::Specs(specs) => Some(finalize_glob_specs(specs, config_dir)),
        // tsgo `getConfigFileSpecs`: when `exclude` is absent (or `null`), `outDir` and
        // `declarationDir` are excluded by default. A wrong-typed `exclude` gets no default.
        SpecsFromRaw::NoProp | SpecsFromRaw::NullValue => {
            let injected: Vec<String> =
                [&compiler_options.out_dir, &compiler_options.declaration_dir]
                    .into_iter()
                    .flatten()
                    .map(|dir| dir.to_string_lossy().into_owned())
                    .collect();
            (!injected.is_empty()).then_some(injected)
        }
        SpecsFromRaw::NotArray => None,
    };
    let file_names =
        get_file_names_from_config_specs(files, include, exclude, config_dir, &compiler_options);

    Ok(Arc::new(ParsedConfig {
        parsed_options: ParsedOptions { compiler_options, file_names, project_references },
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
    files: SpecsFromRaw,
    include: SpecsFromRaw,
    exclude: SpecsFromRaw,
}

/// tsgo `propOfRaw` for the top-level spec properties (`files`/`include`/`exclude`). The
/// distinctions matter: *any* present key — even `null` or wrong-typed — blocks `extends`
/// inheritance (tsgo `setPropertyValue` checks raw presence), but only an array contributes
/// specs, and the `outDir`/`declarationDir` default-exclude applies to absent-or-`null`
/// (tsgo's `"no-prop"`) but not to a wrong-typed value.
enum SpecsFromRaw {
    /// The key is absent (tsgo `wrongValue: "no-prop"`).
    NoProp,
    /// The key is present with a JSON `null` (also tsgo `"no-prop"`, but raw presence still
    /// blocks inheritance).
    NullValue,
    /// The key is present with a non-array value (tsgo `wrongValue: "not-array"`).
    NotArray,
    /// The key is a string array; non-string elements are dropped (tsc diagnoses them).
    Specs(Vec<String>),
}

/// tsgo `parseConfig`: parse one config's JSON (already read — the caller reads the file, as
/// tsgo's `getExtendedConfig` does) and merge its `extends` chain — extended configs first
/// (in `extends` array order, later entries winning), the config's own values last.
fn parse_config(
    config_path: &Path,
    json: Value,
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

    let Value::Object(root) = json else {
        bail!("The root value of '{}' must be an object.", config_path.display());
    };
    let config_dir = config_path.parent().expect("config path is an absolute file path");

    let (own_options, own_explicit_nulls) = convert_compiler_options_from_json_worker(
        root.get("compilerOptions"),
        config_dir,
        config_path,
    );
    let own_files = get_specs_from_raw(&root, "files");
    let own_include = get_specs_from_raw(&root, "include");
    let own_exclude = get_specs_from_raw(&root, "exclude");
    let extended_paths = get_extends_config_path_or_array(&root, config_dir, extends_resolver)?;

    if extended_paths.is_empty() {
        return Ok(ParsedTsconfig {
            options: own_options,
            own_explicit_nulls,
            files: own_files,
            include: own_include,
            exclude: own_exclude,
        });
    }

    resolution_stack.push(config_path.to_path_buf());
    let mut options = CompilerOptions::default();
    let mut inherited_files = SpecsFromRaw::NoProp;
    let mut inherited_include = SpecsFromRaw::NoProp;
    let mut inherited_exclude = SpecsFromRaw::NoProp;
    for extended_path in &extended_paths {
        let extended_json = read_json_config_file(extended_path)?;
        let extended =
            parse_config(extended_path, extended_json, resolution_stack, extends_resolver)?;
        merge_compiler_options(&mut options, &extended.options, &extended.own_explicit_nulls);
        // tsgo `applyExtendedConfig`: file specs are inherited wholesale (only array values —
        // a parent's `null`/wrong-typed spec contributes nothing), each relative spec staying
        // anchored to the config that defined it.
        let extended_dir = extended_path.parent().expect("config path is an absolute file path");
        if let SpecsFromRaw::Specs(specs) = &extended.files {
            inherited_files = SpecsFromRaw::Specs(rebase_inherited_specs(specs, extended_dir));
        }
        if let SpecsFromRaw::Specs(specs) = &extended.include {
            inherited_include =
                SpecsFromRaw::Specs(rebase_inherited_specs(&validate_specs(specs), extended_dir));
        }
        if let SpecsFromRaw::Specs(specs) = &extended.exclude {
            inherited_exclude =
                SpecsFromRaw::Specs(rebase_inherited_specs(&validate_specs(specs), extended_dir));
        }
    }
    resolution_stack.pop();
    merge_compiler_options(&mut options, &own_options, &own_explicit_nulls);

    // tsgo `setPropertyValue`: a key present on the child — even `null` or wrong-typed —
    // blocks inheritance of that property.
    Ok(ParsedTsconfig {
        options,
        own_explicit_nulls,
        files: own_files.or_inherited(inherited_files),
        include: own_include.or_inherited(inherited_include),
        exclude: own_exclude.or_inherited(inherited_exclude),
    })
}

/// Read a config file into a JSON value, decoding UTF-16 (tsgo `decodeBytes`) and stripping
/// the BOM, comments, and trailing commas (tsgo parses tsconfig with its scanner in JSON
/// mode; a whitespace-only file is an empty config).
fn read_json_config_file(config_file: &Path) -> Result<Value> {
    let bytes = fs::read(config_file)
        .with_context(|| format!("Cannot read file '{}'.", config_file.display()))?;
    let mut json = decode_bytes(bytes)
        .with_context(|| format!("Cannot read file '{}'.", config_file.display()))?
        .into_bytes();
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

/// tsgo `decodeBytes` (`internal/vfs`): a UTF-16 BOM selects UTF-16 LE/BE decoding; anything
/// else is treated as UTF-8.
fn decode_bytes(bytes: Vec<u8>) -> Result<String> {
    match bytes.first_chunk::<2>() {
        Some([0xFF, 0xFE]) => decode_utf16(&bytes[2..], u16::from_le_bytes),
        Some([0xFE, 0xFF]) => decode_utf16(&bytes[2..], u16::from_be_bytes),
        _ => String::from_utf8(bytes).context("File is not valid UTF-8"),
    }
}

fn decode_utf16(bytes: &[u8], from_bytes: fn([u8; 2]) -> u16) -> Result<String> {
    let units: Vec<u16> =
        bytes.chunks_exact(2).map(|pair| from_bytes([pair[0], pair[1]])).collect();
    String::from_utf16(&units).context("File is not valid UTF-16")
}

impl SpecsFromRaw {
    /// The inheritance rule: own raw presence (any variant but [`Self::NoProp`]) wins.
    fn or_inherited(self, inherited: Self) -> Self {
        match self {
            Self::NoProp => inherited,
            own => own,
        }
    }
}

/// tsgo `getSpecsFromRaw`/`getPropFromRaw` for a top-level spec property, kept as raw specs.
fn get_specs_from_raw(root: &JsonObject, key: &str) -> SpecsFromRaw {
    match root.get(key) {
        None => SpecsFromRaw::NoProp,
        Some(Value::Null) => SpecsFromRaw::NullValue,
        Some(Value::Array(items)) => SpecsFromRaw::Specs(
            items.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        ),
        Some(_) => SpecsFromRaw::NotArray,
    }
}

/// tsgo `getProjectReferences` (in `parseJsonConfigFileContentWorker`): the root config's
/// `references`, each `path` resolved against the root config's directory. Entries without a
/// non-empty string `path` are skipped, as in tsgo (which diagnoses them; we don't diagnose
/// yet). Deliberately safer than tsgo on malformed elements: tsgo's `parseProjectReference`
/// panics on a non-string `path` or non-bool `circular`; we skip/default instead.
fn get_project_references(root: &JsonObject, base_path: &Path) -> Vec<ProjectReference> {
    root.get("references")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_object)
                .filter_map(|reference| {
                    let path = reference.get("path")?.as_str()?;
                    if path.is_empty() {
                        return None;
                    }
                    Some(ProjectReference {
                        path: to_path(base_path, Path::new(path)),
                        original_path: path.to_string(),
                        circular: reference
                            .get("circular")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// tsgo `getExtendsConfigPathOrArray`: the `extends` value as resolved config paths. A single
/// specifier or an array (array order preserved — later entries win the merge). Empty-string
/// specifiers are skipped: tsgo ignores a top-level one and diagnoses-then-continues past one
/// inside an array (TS18051), so neither aborts the parse.
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
                if let Some(specifier) = item.as_str()
                    && !specifier.is_empty()
                {
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
    // tsgo normalizes slashes first, so the Windows-style `.\` form counts as relative;
    // `IsRootedDiskPath` recognizes DOS drive roots on every platform.
    let normalized = specifier.cow_replace('\\', "/");
    if is_rooted_disk_path(&normalized)
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
    // entry point, `tsconfig.json` as the directory default, and CommonJS-mode conditions
    // (tsgo resolves configs with `ResolutionModeCommonJS`, so `GetConditions` yields
    // `require`/`types`/`node`).
    let resolver = extends_resolver.get_or_init(|| {
        Resolver::new(ResolveOptions {
            condition_names: vec!["require".to_string(), "types".to_string(), "node".to_string()],
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
/// patterns, not paths — tsgo keeps its validated specs as strings too). Invalid specs are
/// dropped first, as tsgo's `validateSpecs` does.
fn finalize_glob_specs(specs: Vec<String>, config_dir: &Path) -> Vec<String> {
    specs
        .into_iter()
        .filter(|spec| !invalid_dot_dot_after_recursive_wildcard(spec))
        .map(|spec| finalize_spec(&spec, config_dir).to_string_lossy().into_owned())
        .collect()
}

/// tsgo `validateSpecs` for inherited `include`/`exclude` specs, which must be validated
/// *before* rebasing lexically collapses their `..` segments.
fn validate_specs(specs: &[String]) -> Vec<String> {
    specs.iter().filter(|spec| !invalid_dot_dot_after_recursive_wildcard(spec)).cloned().collect()
}

/// tsgo `invalidDotDotAfterRecursiveWildcard` (`validateSpecs`): a `**` segment followed by a
/// later `..` segment — such include/exclude specs are dropped (tsc diagnoses them). tsgo's
/// other validation, the invalid *trailing* `**` in an include, needs no counterpart here:
/// the glob compiler already turns it into a match-nothing pattern, tsgo's net effect.
fn invalid_dot_dot_after_recursive_wildcard(spec: &str) -> bool {
    let mut seen_recursive_wildcard = false;
    for segment in spec.split('/') {
        if segment == "**" {
            seen_recursive_wildcard = true;
        } else if seen_recursive_wildcard && segment == ".." {
            return true;
        }
    }
    false
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

/// The in-place form of the above for an already-parsed path option value; a no-op for
/// values without the `${configDir}` prefix.
fn substitute_config_dir_in_path(value: &mut PathBuf, config_dir: &Path) {
    let substituted = value
        .to_str()
        .filter(|text| starts_with_config_dir_template(text))
        .map(|text| get_substituted_path_with_config_dir_template(text, config_dir));
    if let Some(substituted) = substituted {
        *value = substituted;
    }
}

/// tsgo `getDefaultCompilerOptions`: a `jsconfig.json` seeds JavaScript-project defaults,
/// which explicit values in the config then override.
fn get_default_compiler_options(config_file_name: &Path) -> CompilerOptions {
    let mut options = CompilerOptions::default();
    if config_file_name.file_name().is_some_and(|name| name == "jsconfig.json") {
        options.allow_js = Some(true);
        options.max_node_module_js_depth = Some(2);
        options.skip_lib_check = Some(true);
        options.no_emit = Some(true);
    }
    options
}

/// tsgo `normalizeNonListOptionValue` for path-typed options: slashes are normalized first,
/// then `${configDir}`-prefixed values survive as written until the post-merge substitution;
/// everything else is absolutized against the directory of the config file that defined it.
fn convert_path_value(value: &str, config_dir: &Path) -> PathBuf {
    let value = value.cow_replace('\\', "/");
    if starts_with_config_dir_template(&value) {
        PathBuf::from(value.into_owned())
    } else {
        to_path(config_dir, Path::new(value.as_ref()))
    }
}

/// tsconfig `paths`: pattern -> substitutions, kept as written (`${configDir}` substitution
/// happens post-merge; relative substitutions resolve against `paths_base_path`). A pattern
/// whose value is not an array keeps its key with no substitutions, as in tsgo
/// (`parseStringMap` sets every key).
fn convert_paths_map(object: &JsonObject) -> CompilerOptionsPathsMap {
    let mut paths = CompilerOptionsPathsMap::default();
    for (pattern, substitutions) in object {
        let substitutions = substitutions.as_array().map_or_else(Vec::new, |items| {
            items.iter().filter_map(Value::as_str).map(str::to_string).collect()
        });
        paths.insert(pattern.clone(), substitutions);
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
        // tsgo filters falsy values (empty strings) out of list options unless the option
        // sets `listPreserveFalsyValues`.
        $value.as_array().map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
    };
    ($value:ident, $config_dir:ident, string_list_preserving_falsy) => {
        $value.as_array().map(|items| {
            items.iter().filter_map(Value::as_str).map(str::to_string).collect::<Vec<_>>()
        })
    };
    ($value:ident, $config_dir:ident, lib_list) => {
        // tsgo validates each `lib` entry against `LibMap` (an enum-typed list element) and
        // stores the canonical `lib.*.d.ts` file name; invalid entries are dropped.
        $value.as_array().map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .filter_map(get_lib_file_name)
                .map(str::to_string)
                .collect::<Vec<_>>()
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
        parse_number($value)
    };
    ($value:ident, $config_dir:ident, enum($ty:ty)) => {
        $value.as_str().and_then(<$ty>::from_str_ignore_case)
    };
}

macro_rules! define_convert_compiler_options {
    ($(($field:ident, $json:literal, $($kind:tt)+)),* $(,)?) => {
        /// tsgo `convertCompilerOptionsFromJsonWorker`: convert a raw `compilerOptions` object
        /// through the declarations table, over the config file's default options (`jsconfig`
        /// seeds). Returns the options plus the set of options that were explicitly `null`
        /// (tsc's "reset, don't inherit" marker for the `extends` merge).
        fn convert_compiler_options_from_json_worker(
            json: Option<&Value>,
            config_dir: &Path,
            config_file_name: &Path,
        ) -> (CompilerOptions, FxHashSet<&'static str>) {
            let mut options = get_default_compiler_options(config_file_name);
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
            substitute_config_dir_in_path(value, $config_dir);
        }
    };
    ($options:ident, $config_dir:ident, $field:ident, path_list) => {
        if let Some(values) = &mut $options.$field {
            for value in values.iter_mut() {
                substitute_config_dir_in_path(value, $config_dir);
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

/// Whether an include spec specifically targets `.json` files (e.g. `**/*.json`, `a.json`) —
/// tsgo checks `HasSuffix(include, tspath.ExtensionJson)`.
fn is_json_spec(spec: &str) -> bool {
    file_extension_is(spec, ".json")
}
