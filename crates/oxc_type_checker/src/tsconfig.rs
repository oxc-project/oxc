//! The file-enumeration half of typescript-go `internal/tsoptions/tsconfigparsing.go`.
//!
//! tsconfig *parsing* (JSONC, `extends` chains, `references`) is delegated to
//! [`oxc_resolver`]; this module ports the parts `oxc_resolver` does not do: computing the
//! default include/exclude specs, validating them, substituting `${configDir}`, and
//! expanding `files`/`include`/`exclude` into the concrete project file list (with
//! extension-priority dedup) via [`crate::vfsmatch`].
//!
//! Known divergence from tsgo: relative `files`/`include`/`exclude` specs inherited from a
//! base config in a *different directory* are not re-based against that base's directory
//! (`oxc_resolver` merges them verbatim); same-directory `extends` is exact.

use indexmap::IndexMap;
use oxc_diagnostics::OxcDiagnostic;
use oxc_resolver::{CompilerOptions, TsConfig};
use rustc_hash::FxBuildHasher;

use crate::{extension, tspath::TsPath, vfsmatch};

const DEFAULT_INCLUDE_SPEC: &str = "**/*";

/// The `${configDir}` template variable (tsgo `configDirTemplate`).
const CONFIG_DIR_TEMPLATE: &str = "${configDir}";

/// tsgo `startsWithConfigDirTemplate`: a case-*insensitive* prefix check.
fn starts_with_config_dir_template(value: &str) -> bool {
    value
        .get(..CONFIG_DIR_TEMPLATE.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(CONFIG_DIR_TEMPLATE))
}

/// tsgo `getSubstitutedPathWithConfigDirTemplate`: replace the first *exact-case*
/// occurrence (so `${CONFIGDIR}/x` passes the prefix check but is left unreplaced,
/// faithfully reproducing tsgo) and resolve against `base_path`.
fn substitute_config_dir(value: &str, base_path: &str) -> String {
    let replaced = match value.find(CONFIG_DIR_TEMPLATE) {
        Some(index) => {
            let mut out = String::with_capacity(value.len());
            out.push_str(&value[..index]);
            out.push_str("./");
            out.push_str(&value[index + CONFIG_DIR_TEMPLATE.len()..]);
            out
        }
        None => value.to_string(),
    };
    TsPath::from(replaced.as_str()).normalized_absolute(base_path).into_string()
}

fn substitute_config_dir_in_specs(specs: &mut [String], base_path: &str) {
    for spec in specs {
        if starts_with_config_dir_template(spec) {
            *spec = substitute_config_dir(spec, base_path);
        }
    }
}

/// Compact JSON encoding of a string array, for the "No inputs were found" diagnostic
/// (tsgo `core.StringifyJson`; HTML escaping of `<`/`>`/`&` is not reproduced).
fn json_string_array(specs: &[String]) -> String {
    use std::fmt::Write;

    let mut out = String::from("[");
    for (i, spec) in specs.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('"');
        for c in spec.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => {
                    write!(out, "\\u{:04x}", c as u32).unwrap();
                }
                c => out.push(c),
            }
        }
        out.push('"');
    }
    out.push(']');
    out
}

/// Insertion-ordered map used to collect and dedup matched files, mirroring tsgo's
/// `collections.OrderedMap`. Deletion uses `shift_remove` to preserve insertion order.
type FileMap = IndexMap<String, String, FxBuildHasher>;

/// The compiler options the file matcher cares about, resolved from
/// [`oxc_resolver::CompilerOptions`].
#[derive(Debug, Default, Clone)]
pub struct CompilerOptionsView {
    pub allow_js: bool,
    pub resolve_json_module: bool,
    pub out_dir: Option<String>,
    pub declaration_dir: Option<String>,
}

impl CompilerOptionsView {
    pub fn from_resolver(options: &CompilerOptions, base_path: &str) -> Self {
        // `allowJs` defaults to the value of `checkJs` (tsgo `GetAllowJS`).
        let check_js = options.check_js.unwrap_or(false);
        // `${configDir}`-prefixed dirs are left untouched by `oxc_resolver` (everything
        // else arrives absolutized); substitute them here (tsgo
        // `handleOptionConfigDirTemplateSubstitution`).
        let dir_option = |dir: Option<&std::path::PathBuf>| {
            dir.map(|p| {
                let s = TsPath::from_slashes(&p.to_string_lossy()).into_string();
                if starts_with_config_dir_template(&s) {
                    substitute_config_dir(&s, base_path)
                } else {
                    s
                }
            })
        };
        Self {
            allow_js: options.allow_js.unwrap_or(check_js),
            resolve_json_module: Self::effective_resolve_json_module(options),
            out_dir: dir_option(options.out_dir.as_ref()),
            declaration_dir: dir_option(options.declaration_dir.as_ref()),
        }
    }

    /// tsgo `GetResolveJsonModule`: an explicit value wins; otherwise `node20`/`nodenext`
    /// modules resolve JSON, `node16`/`node18` do not, and every other module kind
    /// (including unset, whose target-derived kinds are never `node*`) falls through to
    /// the default bundler module resolution, which does.
    ///
    /// `oxc_resolver` does not expose `moduleResolution`, so an explicit non-bundler
    /// override there is not honored yet.
    fn effective_resolve_json_module(options: &CompilerOptions) -> bool {
        options.resolve_json_module.unwrap_or_else(|| match options.module.as_deref() {
            Some(module) => {
                !module.eq_ignore_ascii_case("node16") && !module.eq_ignore_ascii_case("node18")
            }
            None => true,
        })
    }

    /// The supported extension priority groups (`.js` groups only when `allowJs`).
    fn supported_extensions(&self) -> &'static [&'static [&'static str]] {
        if self.allow_js {
            extension::ALL_SUPPORTED_EXTENSIONS
        } else {
            extension::SUPPORTED_TS_EXTENSIONS
        }
    }

    /// [`Self::supported_extensions`] plus a `.json` group when `resolveJsonModule` is on.
    fn supported_extensions_with_json(&self) -> Vec<&'static [&'static str]> {
        let mut extensions = self.supported_extensions().to_vec();
        if self.resolve_json_module {
            extensions.push(extension::JSON_GROUP);
        }
        extensions
    }
}

/// Validated `files`/`include`/`exclude` specs, with defaults already applied.
#[derive(Debug, Default, Clone)]
pub struct ConfigFileSpecs {
    pub validated_files_spec: Vec<String>,
    pub validated_include_specs: Vec<String>,
    pub validated_exclude_specs: Vec<String>,
    /// Raw `include` specs, with the `**/*` default applied (tsgo `includeSpecs`); used by
    /// the "No inputs were found" diagnostic.
    include_specs: Vec<String>,
    /// Raw `exclude` specs, with the `outDir`/`declarationDir` default applied.
    exclude_specs: Vec<String>,
    /// tsgo `canJsonReportNoInputFiles`: neither `files` nor `references` was present.
    /// (A present-but-empty `references: []` is indistinguishable from an absent one via
    /// `oxc_resolver`, so it does not suppress the diagnostic as it would in tsgo.)
    can_report_no_input_files: bool,
}

impl ConfigFileSpecs {
    /// Build the specs from a parsed tsconfig, applying tsgo's default include/exclude rules,
    /// validating them, and substituting `${configDir}`. Returned diagnostics flag invalid
    /// specs and an empty `files` list.
    pub fn from_tsconfig(
        tsconfig: &TsConfig,
        options: &CompilerOptionsView,
        base_path: &str,
    ) -> (Self, Vec<OxcDiagnostic>) {
        let to_strings = |specs: &Vec<std::path::PathBuf>| -> Vec<String> {
            specs.iter().map(|p| p.to_string_lossy().into_owned()).collect()
        };
        let files_spec = tsconfig.files.as_ref().map(&to_strings);
        let mut include_specs = tsconfig.include.as_ref().map(&to_strings);
        let mut exclude_specs = tsconfig.exclude.as_ref().map(&to_strings);

        let mut diagnostics = Vec::new();

        // An explicitly empty `files` with nothing else to compile is an error (tsgo
        // "The files list in config file is empty" check; TS18002).
        if files_spec.as_ref().is_some_and(Vec::is_empty)
            && tsconfig.references.is_empty()
            && tsconfig.extends.is_none()
        {
            let config = TsPath::from_slashes(&tsconfig.path().to_string_lossy());
            diagnostics.push(OxcDiagnostic::error(format!(
                "The 'files' list in config file '{config}' is empty."
            )));
        }

        // When `exclude` is absent, default it to `outDir`/`declarationDir` (so build outputs are
        // not re-included). `node_modules`/`bower_components`/`jspm_packages` are handled
        // implicitly by the matcher, not here.
        if exclude_specs.is_none() {
            let mut values = Vec::new();
            if let Some(out_dir) = &options.out_dir
                && !out_dir.is_empty()
            {
                values.push(out_dir.clone());
            }
            if let Some(declaration_dir) = &options.declaration_dir
                && !declaration_dir.is_empty()
            {
                values.push(declaration_dir.clone());
            }
            if !values.is_empty() {
                exclude_specs = Some(values);
            }
        }

        // When neither `files` nor `include` is present, default `include` to `**/*`.
        if files_spec.is_none() && include_specs.is_none() {
            include_specs = Some(vec![DEFAULT_INCLUDE_SPEC.to_string()]);
        }

        let can_report_no_input_files = files_spec.is_none() && tsconfig.references.is_empty();
        let raw_include_specs = include_specs.clone().unwrap_or_default();
        let raw_exclude_specs = exclude_specs.clone().unwrap_or_default();

        let mut validate = |specs: Option<Vec<String>>, disallow_trailing_recursion: bool| {
            specs.map_or_else(Vec::new, |specs| {
                let (validated, errors) = Self::validate_specs(&specs, disallow_trailing_recursion);
                diagnostics.extend(errors);
                validated
            })
        };
        let mut validated_files_spec = files_spec.unwrap_or_default();
        let mut validated_include_specs = validate(include_specs, true);
        let mut validated_exclude_specs = validate(exclude_specs, false);

        // `${configDir}` substitution happens after validation, on the validated specs
        // (tsgo `getSubstitutedStringArrayWithConfigDirTemplate` call sites).
        substitute_config_dir_in_specs(&mut validated_files_spec, base_path);
        substitute_config_dir_in_specs(&mut validated_include_specs, base_path);
        substitute_config_dir_in_specs(&mut validated_exclude_specs, base_path);

        let specs = Self {
            validated_files_spec,
            validated_include_specs,
            validated_exclude_specs,
            include_specs: raw_include_specs,
            exclude_specs: raw_exclude_specs,
            can_report_no_input_files,
        };
        (specs, diagnostics)
    }

    /// tsgo `shouldReportNoInputFiles`: the expanded file list is empty and the config had
    /// no `files` or `references` to justify that.
    pub fn should_report_no_input_files(&self, file_names: &[String]) -> bool {
        file_names.is_empty() && self.can_report_no_input_files
    }

    /// The TS18003 "No inputs were found" diagnostic for this config.
    pub fn no_inputs_diagnostic(&self, config_file: &str) -> OxcDiagnostic {
        OxcDiagnostic::error(format!(
            "No inputs were found in config file '{config_file}'. Specified 'include' paths were '{}' and 'exclude' paths were '{}'.",
            json_string_array(&self.include_specs),
            json_string_array(&self.exclude_specs),
        ))
    }

    /// Expand the specs into the concrete, absolute project file list, resolving relative specs
    /// against `base_path` and walking the filesystem via `host`.
    pub fn file_names(
        &self,
        base_path: &str,
        options: &CompilerOptionsView,
        host: &dyn vfsmatch::FileSystemHost,
    ) -> Vec<String> {
        FileExpander::new(options, host).expand(self, base_path)
    }

    fn validate_specs(
        specs: &[String],
        disallow_trailing_recursion: bool,
    ) -> (Vec<String>, Vec<OxcDiagnostic>) {
        let mut errors = Vec::new();
        let mut final_specs = Vec::new();
        for spec in specs {
            if let Some(message) = Self::spec_to_diagnostic(spec, disallow_trailing_recursion) {
                errors.push(OxcDiagnostic::error(message));
            } else {
                final_specs.push(spec.clone());
            }
        }
        (final_specs, errors)
    }

    fn spec_to_diagnostic(spec: &str, disallow_trailing_recursion: bool) -> Option<String> {
        if disallow_trailing_recursion && Self::invalid_trailing_recursion(spec) {
            return Some(format!(
                "File specification cannot end in a recursive directory wildcard ('**'): '{spec}'."
            ));
        }
        if Self::invalid_dot_dot_after_recursive_wildcard(spec) {
            return Some(format!(
                "File specification cannot contain a parent directory ('..') that appears after a recursive directory wildcard ('**'): '{spec}'."
            ));
        }
        None
    }

    fn invalid_trailing_recursion(spec: &str) -> bool {
        // Matches `**`, `/**`, `**/`, and `/**/`, but not `a**b`.
        let s = spec.strip_suffix('/').unwrap_or(spec);
        s == "**" || s.ends_with("/**")
    }

    fn invalid_dot_dot_after_recursive_wildcard(s: &str) -> bool {
        // True when some `/..` segment appears after some `**/` segment.
        let wildcard_index = if s.starts_with("**/") { Some(0) } else { s.find("/**/") };
        let Some(wildcard_index) = wildcard_index else {
            return false;
        };
        let last_dot_index = if s.ends_with("/..") { Some(s.len()) } else { s.rfind("/../") };
        last_dot_index.is_some_and(|last| last > wildcard_index)
    }
}

/// Collects + de-duplicates the files matched by a project's specs. Mirrors the three ordered
/// maps in tsgo's `getFileNamesFromConfigSpecs` (literal / wildcard / wildcard-json).
struct FileExpander<'a> {
    options: &'a CompilerOptionsView,
    host: &'a dyn vfsmatch::FileSystemHost,
    use_case_sensitive: bool,
    /// Extension priority groups (no `.json`), used for the sibling-shadowing dedup.
    supported_extensions: &'static [&'static [&'static str]],
    literal: FileMap,
    wildcard: FileMap,
    wildcard_json: FileMap,
}

impl<'a> FileExpander<'a> {
    fn new(options: &'a CompilerOptionsView, host: &'a dyn vfsmatch::FileSystemHost) -> Self {
        Self {
            options,
            host,
            use_case_sensitive: host.use_case_sensitive_file_names(),
            supported_extensions: options.supported_extensions(),
            literal: FileMap::default(),
            wildcard: FileMap::default(),
            wildcard_json: FileMap::default(),
        }
    }

    /// Case-folded de-dup key for `value` (the file path or a raw spec).
    fn key(&self, value: &str) -> String {
        TsPath::from(value).canonical(self.use_case_sensitive)
    }

    /// Order mirrors tsgo: literal (`files`) entries first, then wildcard matches, then wildcard
    /// `.json` matches. Literal files are always included and immune to include/exclude.
    fn expand(mut self, specs: &ConfigFileSpecs, base_path: &str) -> Vec<String> {
        let base_path = TsPath::from(base_path).normalized().into_string();

        for file_name in &specs.validated_files_spec {
            let file =
                TsPath::from(file_name.as_str()).normalized_absolute(&base_path).into_string();
            let key = self.key(file_name);
            self.literal.insert(key, file);
        }

        if !specs.validated_include_specs.is_empty() {
            let extensions_flat =
                extension::flatten(&self.options.supported_extensions_with_json());
            let files = vfsmatch::read_directory(
                self.host,
                &base_path,
                &base_path,
                &extensions_flat,
                &specs.validated_exclude_specs,
                &specs.validated_include_specs,
                vfsmatch::UNLIMITED_DEPTH,
            );

            let json_matcher = self.json_matcher(specs, &base_path);
            for file in files {
                if TsPath::from(file.as_str()).file_extension_is(extension::EXTENSION_JSON) {
                    self.include_json(json_matcher.as_ref(), file);
                    continue;
                }
                if self.has_higher_priority_sibling(&file) {
                    continue;
                }
                self.remove_lower_priority_siblings(&file);
                let key = self.key(&file);
                if !self.literal.contains_key(&key) && !self.wildcard.contains_key(&key) {
                    self.wildcard.insert(key, file);
                }
            }
        }

        let mut result =
            Vec::with_capacity(self.literal.len() + self.wildcard.len() + self.wildcard_json.len());
        result.extend(self.literal.into_values());
        result.extend(self.wildcard.into_values());
        result.extend(self.wildcard_json.into_values());
        result
    }

    /// The matcher for `.json` include specs (those explicitly ending in `.json`), if any.
    fn json_matcher(
        &self,
        specs: &ConfigFileSpecs,
        base_path: &str,
    ) -> Option<vfsmatch::SpecMatcher> {
        let json_includes: Vec<String> = specs
            .validated_include_specs
            .iter()
            .filter(|include| include.ends_with(extension::EXTENSION_JSON))
            .cloned()
            .collect();
        vfsmatch::SpecMatcher::new(
            &json_includes,
            base_path,
            vfsmatch::Usage::Files,
            self.use_case_sensitive,
        )
    }

    /// Add a `.json` file if it matches a `.json` include spec.
    fn include_json(&mut self, json_matcher: Option<&vfsmatch::SpecMatcher>, file: String) {
        if json_matcher.and_then(|m| m.match_index(&file)).is_some() {
            let key = self.key(&file);
            if !self.literal.contains_key(&key) && !self.wildcard_json.contains_key(&key) {
                self.wildcard_json.insert(key, file);
            }
        }
    }

    /// Whether a higher-priority-extension sibling of `file` was already included (so `file`
    /// should be skipped). E.g. `foo.ts` shadows `foo.d.ts`/`foo.js` in the same directory.
    fn has_higher_priority_sibling(&self, file: &str) -> bool {
        let file_path = TsPath::from(file);
        let extension_group = Self::matched_extension_group(&file_path, self.supported_extensions);
        if extension_group.is_empty() {
            return false;
        }
        for &ext in &extension_group {
            // A `.d.ts` file also matches the `.ts` suffix; don't treat them as the same file.
            if file_path.file_extension_is(ext)
                && (ext != extension::EXTENSION_TS
                    || !file_path.file_extension_is(extension::EXTENSION_DTS))
            {
                return false;
            }
            let sibling = self.key(file_path.change_extension(ext).as_str());
            if self.literal.contains_key(&sibling) || self.wildcard.contains_key(&sibling) {
                // LEGACY: a `.d.ts` may coexist with its `.js`/`.jsx` counterpart.
                if ext == extension::EXTENSION_DTS
                    && (file_path.file_extension_is(extension::EXTENSION_JS)
                        || file_path.file_extension_is(extension::EXTENSION_JSX))
                {
                    continue;
                }
                return true;
            }
        }
        false
    }

    /// Remove already-included wildcard files with a lower extension priority than `file`.
    fn remove_lower_priority_siblings(&mut self, file: &str) {
        let file_path = TsPath::from(file);
        let extension_group = Self::matched_extension_group(&file_path, self.supported_extensions);
        for &ext in extension_group.iter().rev() {
            if file_path.file_extension_is(ext) {
                return;
            }
            let lower_priority = self.key(file_path.change_extension(ext).as_str());
            self.wildcard.shift_remove(&lower_priority);
        }
    }

    /// The concatenation of every priority group whose extensions `file` matches.
    fn matched_extension_group<'e>(file: &TsPath, extensions: &[&'e [&'e str]]) -> Vec<&'e str> {
        let mut group = Vec::new();
        for &priority_group in extensions {
            if file.file_extension_is_one_of(priority_group) {
                group.extend_from_slice(priority_group);
            }
        }
        group
    }
}
