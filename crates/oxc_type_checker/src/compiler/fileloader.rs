//! Port of typescript-go's `internal/compiler/fileloader.go`.
//!
//! [`FileLoader::process_all_program_files`] (tsgo `processAllProgramFiles`) parses the root files,
//! follows their references — triple-slash path references, type reference directives, imports,
//! and module augmentations — to load the dependent files in parallel, and collects everything
//! into [`ProcessedFiles`]. Import resolution mirrors `resolveImportsAndModuleAugmentations` —
//! including its `shouldAddFile` gating — but reuses `oxc_resolver`'s TS-aware `resolve_dts`
//! (mode-less: tsgo resolves each usage under its ESM/CJS resolution mode). Lib files and project
//! references are not loaded yet.

use std::path::{Path, PathBuf};

use oxc_index::IndexVec;
use oxc_resolver::{
    ResolveOptions, Resolver, TsconfigDiscovery, TsconfigOptions, TsconfigReferences,
};
use oxc_span::VALID_EXTENSIONS;
use oxc_str::CompactStr;
use rustc_hash::FxHashMap;

use crate::{
    tsoptions::{
        SUPPORTED_TS_EXTENSIONS_WITH_JSON_FLAT, get_allow_js, get_resolve_json_module,
        get_supported_extensions, get_supported_extensions_with_json_flat,
    },
    tspath::{self, file_extension_is, file_extension_is_one_of},
};

use super::{
    filesparser::FilesParser,
    host::CompilerHost,
    program::{FileId, ProgramOptions},
    source_file::SourceFile,
};

/// A resolved reference from one file to another (tsgo `resolvedRef` + the resolution key):
/// the import specifier (`None` for triple-slash references and type reference directives,
/// which have no module name) and the normalized dependency path.
pub(super) type SubTask = (Option<CompactStr>, PathBuf);

/// A program's loaded files — the subset of tsgo's `processedFiles` this step fills.
#[derive(Debug, Default)]
pub(super) struct ProcessedFiles {
    /// Parsed files in tsgo's program order (tsgo `files`): a post-order walk of the import
    /// graph from the root files, so dependencies come before their importers.
    pub(super) files: IndexVec<FileId, SourceFile>,
    /// Normalized path -> file id (tsgo `filesByPath`).
    pub(super) files_by_path: FxHashMap<PathBuf, FileId>,
    /// Root/import paths that produced no source file (tsgo `missingFiles`): unreadable files
    /// and files with unsupported extensions.
    pub(super) missing_files: Vec<PathBuf>,
    /// Each file's resolved module-graph edges: import specifier -> dependency [`FileId`]
    /// (tsgo `resolvedModules`, which keys by path; ours is aligned with `files`).
    pub(super) resolved_modules: IndexVec<FileId, FxHashMap<CompactStr, FileId>>,
}

/// Loads a program's files, mirroring tsgo's `fileLoader`. Holds the host (reads + parses), the
/// module resolver, and the supported-extension tables computed from the compiler options.
/// `Send + Sync`, so rayon workers share one by `&`.
pub(super) struct FileLoader {
    host: CompilerHost,
    resolver: Resolver,
    /// tsgo `supportedExtensions`, grouped by resolution priority (for extensionless
    /// triple-slash references).
    supported_extensions: &'static [&'static [&'static str]],
    /// tsgo `supportedExtensionsWithJsonIfResolveJsonModule`, flattened (the load-time gate).
    supported_extensions_with_json: Vec<&'static str>,
    /// tsgo `CompilerOptions.GetAllowJS()`.
    allow_js: bool,
    /// tsgo `CompilerOptions.GetResolveJsonModule()`.
    resolve_json_module: bool,
    /// tsgo `CompilerOptions.Jsx` is set (gates `.tsx`/`.jsx` resolutions).
    jsx: bool,
}

impl FileLoader {
    fn new(opts: &ProgramOptions) -> Self {
        let default_options = oxc_resolver::CompilerOptions::default();
        let options =
            opts.config.as_ref().map_or(&default_options, |config| &config.compiler_options);
        Self {
            host: CompilerHost::new(opts.current_directory.clone()),
            resolver: build_resolver(opts.config.as_ref().map(|config| config.path.as_path())),
            supported_extensions: get_supported_extensions(options),
            supported_extensions_with_json: get_supported_extensions_with_json_flat(options),
            allow_js: get_allow_js(options),
            resolve_json_module: get_resolve_json_module(options),
            jsx: options.jsx.is_some(),
        }
    }

    pub(super) fn host(&self) -> &CompilerHost {
        &self.host
    }

    pub(super) fn current_directory(&self) -> &Path {
        self.host.current_directory()
    }

    /// tsgo `fileLoader.toPath`: normalize a file name into its identity key.
    pub(super) fn to_path(&self, file_name: &Path) -> PathBuf {
        tspath::to_path(self.current_directory(), file_name)
    }

    /// tsgo `fileLoader.isSupportedExtension` (against
    /// `supportedExtensionsWithJsonIfResolveJsonModule`).
    pub(super) fn is_supported_extension(&self, file_name: &str) -> bool {
        file_extension_is_one_of(file_name, &self.supported_extensions_with_json)
    }

    /// Resolve everything a file references into sub-tasks, in tsgo `parseTask.load` order:
    /// triple-slash path references, type reference directives, (lib references — not loaded
    /// yet), then imports and module augmentations.
    pub(super) fn resolve_references(&self, source_file: &SourceFile) -> Vec<SubTask> {
        let importing_file = source_file.file_name();
        let mut resolutions = Vec::new();

        for reference in source_file.referenced_files() {
            if let Some(path) = self.resolve_tripleslash_path_reference(reference, importing_file) {
                resolutions.push((None, path));
            }
        }
        // tsgo `resolveTypeReferenceDirectives` (`resolver.ResolveTypeReferenceDirective`,
        // approximated with `resolve_dts`: relative directives resolve as declaration files,
        // bare names through `@types`).
        for directive in source_file.type_reference_directives() {
            if directive.is_empty() {
                continue;
            }
            if let Ok(resolution) = self.resolver.resolve_dts(importing_file, directive) {
                resolutions.push((None, self.to_path(resolution.path())));
            }
        }

        self.resolve_imports_and_module_augmentations(source_file, &mut resolutions);
        resolutions
    }

    /// tsgo `resolveTripleslashPathReference` + `getSourceFileFromReference`: resolve the
    /// reference against the containing file's directory; a name with an extension must be
    /// supported and exist, an extensionless name tries the primary supported extensions.
    fn resolve_tripleslash_path_reference(
        &self,
        module_name: &str,
        containing_file: &Path,
    ) -> Option<PathBuf> {
        let base_path = containing_file.parent()?;
        let referenced = if Path::new(module_name).is_absolute() {
            PathBuf::from(module_name)
        } else {
            base_path.join(module_name)
        };
        let normalized = self.to_path(&referenced);
        let file_name = normalized.to_string_lossy();
        if tspath::has_extension(&file_name) {
            if self.is_supported_extension(&file_name) && self.host.file_exists(&normalized) {
                return Some(normalized);
            }
            return None;
        }
        for extension in self.supported_extensions[0] {
            let mut candidate = normalized.clone().into_os_string();
            candidate.push(extension);
            let candidate = PathBuf::from(candidate);
            if self.host.file_exists(&candidate) {
                return Some(candidate);
            }
        }
        None
    }

    /// tsgo `resolveImportsAndModuleAugmentations`: resolve the file's imports and module
    /// augmentations to normalized dependency paths, applying `shouldAddFile`'s gates —
    /// resolutions a `GetResolutionDiagnostic` would reject (`.json` without `resolveJsonModule`,
    /// `.tsx`/`.jsx` without `jsx`, arbitrary extensions outside declaration files), JavaScript
    /// files without `allowJs`, and JavaScript files under `node_modules` (tsgo elides those
    /// beyond `maxNodeModuleJsDepth`, which defaults to 0; the option itself is not exposed by
    /// `oxc_resolver`, so depths above 0 are not supported). Specifiers that don't resolve
    /// (e.g. uninstalled packages) are skipped.
    fn resolve_imports_and_module_augmentations(
        &self,
        source_file: &SourceFile,
        resolutions: &mut Vec<SubTask>,
    ) {
        let importing_file = source_file.file_name();
        let is_declaration_file = source_file.source_type().is_typescript_definition();
        let module_names = source_file.imports().iter().chain(source_file.module_augmentations());

        for module_name in module_names {
            if module_name.is_empty() {
                continue;
            }
            let Some(resolution) = self.resolve_module_name(importing_file, module_name) else {
                continue;
            };
            let resolved_file_name = resolution.path().to_string_lossy();

            // tsgo `shouldAddFile`: `!(isJsFile && !GetAllowJS())`, and JS files found under
            // node_modules are elided (`elideOnDepth` with the default depth limit of 0).
            // Checked by path component so Windows separators match too.
            let is_js_file = !file_extension_is_one_of(
                &resolved_file_name,
                SUPPORTED_TS_EXTENSIONS_WITH_JSON_FLAT,
            );
            if is_js_file
                && (!self.allow_js
                    || resolution
                        .path()
                        .components()
                        .any(|component| component.as_os_str() == "node_modules"))
            {
                continue;
            }
            // tsgo `module.GetResolutionDiagnostic`: resolutions that would error are not added.
            if !self.resolution_allowed(&resolved_file_name, is_declaration_file) {
                continue;
            }
            resolutions.push((Some(module_name.clone()), self.to_path(resolution.path())));
        }
    }

    /// tsgo `resolver.ResolveModuleName`, via `oxc_resolver`. `resolve_dts` mirrors TS's
    /// code-module resolution, which never loads a literal `.json` file (only its `.d.json.ts`
    /// substitution) — JSON modules resolve through tsgo's separate `tryResolveJsonModule`
    /// branch, stood in for here by the general-purpose resolver.
    fn resolve_module_name(
        &self,
        importing_file: &Path,
        module_name: &str,
    ) -> Option<oxc_resolver::Resolution> {
        // tsc resolves specifiers literally — a bundler-style query or fragment suffix
        // (`./worker.ts?worker`) never matches a file. `oxc_resolver` would strip and resolve
        // it, so guard here (a leading `#` is a package import, not a fragment).
        if module_name.char_indices().any(|(i, c)| i > 0 && (c == '?' || c == '#')) {
            return None;
        }
        match self.resolver.resolve_dts(importing_file, module_name) {
            Ok(resolution) => Some(resolution),
            Err(_) => {
                if self.resolve_json_module && file_extension_is(module_name, ".json") {
                    let importing_dir = importing_file.parent()?;
                    self.resolver.resolve(importing_dir, module_name).ok()
                } else {
                    None
                }
            }
        }
    }

    /// tsgo `module.GetResolutionDiagnostic` returning no diagnostic. TS extensions are always
    /// allowed; `.tsx`/`.jsx` need the `jsx` option; `.js`-family extensions are handled by the
    /// caller's `allowJs` gate; `.json` needs `resolveJsonModule`; anything else is only allowed
    /// from declaration files (`allowArbitraryExtensions` is not exposed by `oxc_resolver`).
    fn resolution_allowed(&self, resolved_file_name: &str, is_declaration_file: bool) -> bool {
        if file_extension_is_one_of(
            resolved_file_name,
            &[".d.ts", ".d.cts", ".d.mts", ".ts", ".cts", ".mts"],
        ) || file_extension_is_one_of(resolved_file_name, &[".js", ".mjs", ".cjs"])
        {
            true
        } else if file_extension_is_one_of(resolved_file_name, &[".tsx", ".jsx"]) {
            self.jsx
        } else if file_extension_is(resolved_file_name, ".json") {
            self.resolve_json_module
        } else {
            is_declaration_file
        }
    }

    /// tsgo `processAllProgramFiles`: parse the roots, load their transitive imports in parallel,
    /// and collect the result.
    pub(super) fn process_all_program_files(opts: &ProgramOptions) -> ProcessedFiles {
        let loader = FileLoader::new(opts);
        let mut files_parser = FilesParser::default();
        files_parser.parse(&loader, &opts.root_files);
        files_parser.get_processed_files()
    }
}

/// Build the module resolver for TS declaration resolution (`resolve_dts`): TS extensions,
/// TS-style `.js`->`.ts`/`.tsx`/`.d.ts` extension substitution (mirroring the order of tsgo's
/// `tryAddingExtensions`, used by the resolver paths that fall back to the general algorithm,
/// e.g. package `imports`), and the project's tsconfig (for `paths`/`baseUrl`).
fn build_resolver(tsconfig: Option<&Path>) -> Resolver {
    let tsconfig = tsconfig.map(|path| {
        TsconfigDiscovery::Manual(TsconfigOptions {
            config_file: path.to_path_buf(),
            references: TsconfigReferences::Auto,
        })
    });
    let alias = |extensions: &[&str]| extensions.iter().map(ToString::to_string).collect();
    Resolver::new(ResolveOptions {
        extensions: VALID_EXTENSIONS.iter().map(|ext| format!(".{ext}")).collect(),
        main_fields: vec!["module".to_string(), "main".to_string()],
        condition_names: vec!["module".to_string(), "import".to_string()],
        extension_alias: vec![
            (".js".to_string(), alias(&[".ts", ".tsx", ".d.ts", ".js", ".jsx"])),
            (".jsx".to_string(), alias(&[".tsx", ".ts", ".d.ts", ".jsx", ".js"])),
            (".mjs".to_string(), alias(&[".mts", ".d.mts", ".mjs"])),
            (".cjs".to_string(), alias(&[".cts", ".d.cts", ".cjs"])),
        ],
        tsconfig,
        ..ResolveOptions::default()
    })
}
