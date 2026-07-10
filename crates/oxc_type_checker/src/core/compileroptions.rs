//! Port of typescript-go's `internal/core/compileroptions.go`.
//!
//! [`CompilerOptions`] models every user-settable tsconfig `compilerOptions` field with tsgo's
//! types: tsgo's `Tristate` maps to `Option<bool>` (`TSUnknown` = `None`), its `int32` enums map
//! to Rust enums wrapped in `Option` (the zero "unknown/none" value = `None`), and path-typed
//! options are [`PathBuf`]s. tsgo-internal and CLI-only fields (`watch`, `build`, `locale`,
//! `diagnostics`, ...) are not modeled; `config_file_path` and `paths_base_path` are ported
//! because option semantics depend on them.
//!
//! The [`for_each_compiler_option!`] callback macro is the single source of truth for the field
//! list — the struct itself, the `extends` merge, and the JSON conversion are all generated from
//! it, so a field cannot be parsed but forgotten in the merge (or vice versa).
//!
//! [`for_each_compiler_option!`]: for_each_compiler_option

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

/// tsgo `CompilerOptions.Paths`: the tsconfig `paths` mapping, insertion-ordered.
///
/// Values are kept as written (relative specs stay relative); they resolve against
/// [`CompilerOptions::paths_base_path`], mirroring tsgo's `PathsBasePath`.
pub type CompilerOptionsPathsMap = IndexMap<String, Vec<String>, FxBuildHasher>;

/// tsgo `core.ModuleKind`. Variants are declared in tsgo's numeric order so that range
/// comparisons (`Node16 <= kind && kind <= NodeNext`) work through `PartialOrd`.
///
/// tsconfig `"module": "none"` parses to the tsgo zero value, which is indistinguishable from an
/// unset option everywhere (merging skips zero values, getters test against zero) — so here it
/// maps to `Option::None` rather than a variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModuleKind {
    CommonJs,
    /// Deprecated in tsc: valid only in options parsing and validation.
    Amd,
    /// Deprecated in tsc: valid only in options parsing and validation.
    Umd,
    /// Deprecated in tsc: valid only in options parsing and validation.
    System,
    // ES module kinds are contiguous (tsgo relies on this to range-check "any ES module kind").
    Es2015,
    Es2020,
    Es2022,
    EsNext,
    // Node16+ is an amalgam of (updated) commonjs and es2022+ (tsgo keeps these contiguous too).
    Node16,
    Node18,
    Node20,
    NodeNext,
    /// Emit as written.
    Preserve,
}

/// tsgo `core.ModuleResolutionKind`. tsc's `moduleResolution` option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleResolutionKind {
    /// Deprecated in tsc: valid only in options parsing and validation.
    Classic,
    /// Deprecated in tsc: valid only in options parsing and validation.
    Node10,
    Node16,
    NodeNext,
    Bundler,
}

/// tsgo `core.ScriptTarget`. Variants are declared in tsgo's numeric order for `PartialOrd`
/// comparisons (`target >= ES2022`); note `Json` sorts *above* `EsNext`, as in tsgo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScriptTarget {
    /// Deprecated in tsc: valid only in options parsing and validation.
    Es5,
    Es2015,
    Es2016,
    Es2017,
    Es2018,
    Es2019,
    Es2020,
    Es2021,
    Es2022,
    Es2023,
    Es2024,
    Es2025,
    EsNext,
    /// Not settable from tsconfig — the per-file script target of `.json` sources.
    Json,
}

impl ScriptTarget {
    /// tsgo `ScriptTargetLatestStandard`.
    pub const LATEST_STANDARD: Self = Self::Es2025;
}

/// tsgo `core.ModuleDetectionKind`. tsc's `moduleDetection` option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleDetectionKind {
    Auto,
    Legacy,
    Force,
}

/// tsgo `core.JsxEmit`. tsc's `jsx` option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsxEmit {
    Preserve,
    ReactNative,
    React,
    ReactJsx,
    ReactJsxDev,
}

/// tsgo `core.NewLineKind`. tsc's `newLine` option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewLineKind {
    CarriageReturnLineFeed,
    LineFeed,
}

/// Invokes the callback macro with every user-settable compilerOptions field as
/// `(rust_field, "jsonName", kind)`, where `kind` is one of `bool`, `string`, `path`,
/// `string_list`, `path_list`, `paths_map`, `number`, or `enum(Type)`.
///
/// This is the Rust rendering of tsgo's option declarations table
/// (`internal/tsoptions/declscompiler.go`): the struct below, the `extends` merge
/// (`tsoptions/parsinghelpers.rs`), and the JSON conversion (`tsoptions/tsconfigparsing.rs`)
/// are all driven by this one list. Path-kind entries (`path`, `path_list`, `paths_map`)
/// are exactly the options tsgo rebases against config directories and substitutes
/// `${configDir}` into.
macro_rules! for_each_compiler_option {
    ($apply:ident) => {
        $apply! {
            (allow_arbitrary_extensions, "allowArbitraryExtensions", bool),
            (allow_importing_ts_extensions, "allowImportingTsExtensions", bool),
            (allow_js, "allowJs", bool),
            (allow_synthetic_default_imports, "allowSyntheticDefaultImports", bool),
            (allow_umd_global_access, "allowUmdGlobalAccess", bool),
            (allow_unreachable_code, "allowUnreachableCode", bool),
            (allow_unused_labels, "allowUnusedLabels", bool),
            (always_strict, "alwaysStrict", bool),
            (assume_changes_only_affect_direct_dependencies, "assumeChangesOnlyAffectDirectDependencies", bool),
            (base_url, "baseUrl", path),
            (check_js, "checkJs", bool),
            (composite, "composite", bool),
            (custom_conditions, "customConditions", string_list),
            (declaration, "declaration", bool),
            (declaration_dir, "declarationDir", path),
            (declaration_map, "declarationMap", bool),
            (deduplicate_packages, "deduplicatePackages", bool),
            (disable_referenced_project_load, "disableReferencedProjectLoad", bool),
            (disable_size_limit, "disableSizeLimit", bool),
            (disable_solution_searching, "disableSolutionSearching", bool),
            (disable_source_of_project_reference_redirect, "disableSourceOfProjectReferenceRedirect", bool),
            (downlevel_iteration, "downlevelIteration", bool),
            (emit_bom, "emitBOM", bool),
            (emit_declaration_only, "emitDeclarationOnly", bool),
            (emit_decorator_metadata, "emitDecoratorMetadata", bool),
            (erasable_syntax_only, "erasableSyntaxOnly", bool),
            (es_module_interop, "esModuleInterop", bool),
            (exact_optional_property_types, "exactOptionalPropertyTypes", bool),
            (experimental_decorators, "experimentalDecorators", bool),
            (force_consistent_casing_in_file_names, "forceConsistentCasingInFileNames", bool),
            (ignore_deprecations, "ignoreDeprecations", string),
            (import_helpers, "importHelpers", bool),
            (incremental, "incremental", bool),
            (inline_source_map, "inlineSourceMap", bool),
            (inline_sources, "inlineSources", bool),
            (isolated_declarations, "isolatedDeclarations", bool),
            (isolated_modules, "isolatedModules", bool),
            (jsx, "jsx", enum(JsxEmit)),
            (jsx_factory, "jsxFactory", string),
            (jsx_fragment_factory, "jsxFragmentFactory", string),
            (jsx_import_source, "jsxImportSource", string),
            // `lib` entries are validated against tsgo's `LibMap` and stored as canonical
            // lowercased `lib.*.d.ts` file names (invalid names dropped), as in tsgo.
            (lib, "lib", lib_list),
            (lib_replacement, "libReplacement", bool),
            (map_root, "mapRoot", string),
            (max_node_module_js_depth, "maxNodeModuleJsDepth", number),
            (module, "module", enum(ModuleKind)),
            (module_detection, "moduleDetection", enum(ModuleDetectionKind)),
            (module_resolution, "moduleResolution", enum(ModuleResolutionKind)),
            // The one list option where empty strings are meaningful (tsgo
            // `listPreserveFalsyValues`): `""` means "also try the bare name".
            (module_suffixes, "moduleSuffixes", string_list_preserving_falsy),
            (new_line, "newLine", enum(NewLineKind)),
            (no_check, "noCheck", bool),
            (no_emit, "noEmit", bool),
            (no_emit_helpers, "noEmitHelpers", bool),
            (no_emit_on_error, "noEmitOnError", bool),
            (no_error_truncation, "noErrorTruncation", bool),
            (no_fallthrough_cases_in_switch, "noFallthroughCasesInSwitch", bool),
            (no_implicit_any, "noImplicitAny", bool),
            (no_implicit_override, "noImplicitOverride", bool),
            (no_implicit_returns, "noImplicitReturns", bool),
            (no_implicit_this, "noImplicitThis", bool),
            (no_lib, "noLib", bool),
            (no_property_access_from_index_signature, "noPropertyAccessFromIndexSignature", bool),
            (no_resolve, "noResolve", bool),
            (no_unchecked_indexed_access, "noUncheckedIndexedAccess", bool),
            (no_unchecked_side_effect_imports, "noUncheckedSideEffectImports", bool),
            (no_unused_locals, "noUnusedLocals", bool),
            (no_unused_parameters, "noUnusedParameters", bool),
            (out_dir, "outDir", path),
            (out_file, "outFile", path),
            (paths, "paths", paths_map),
            (preserve_const_enums, "preserveConstEnums", bool),
            (preserve_symlinks, "preserveSymlinks", bool),
            (react_namespace, "reactNamespace", string),
            (remove_comments, "removeComments", bool),
            (resolve_json_module, "resolveJsonModule", bool),
            (resolve_package_json_exports, "resolvePackageJsonExports", bool),
            (resolve_package_json_imports, "resolvePackageJsonImports", bool),
            (rewrite_relative_import_extensions, "rewriteRelativeImportExtensions", bool),
            (root_dir, "rootDir", path),
            (root_dirs, "rootDirs", path_list),
            (skip_default_lib_check, "skipDefaultLibCheck", bool),
            (skip_lib_check, "skipLibCheck", bool),
            (source_map, "sourceMap", bool),
            (source_root, "sourceRoot", string),
            (stable_type_ordering, "stableTypeOrdering", bool),
            (strict, "strict", bool),
            (strict_bind_call_apply, "strictBindCallApply", bool),
            (strict_builtin_iterator_return, "strictBuiltinIteratorReturn", bool),
            (strict_function_types, "strictFunctionTypes", bool),
            (strict_null_checks, "strictNullChecks", bool),
            (strict_property_initialization, "strictPropertyInitialization", bool),
            (strip_internal, "stripInternal", bool),
            (target, "target", enum(ScriptTarget)),
            (trace_resolution, "traceResolution", bool),
            (ts_build_info_file, "tsBuildInfoFile", path),
            (type_roots, "typeRoots", path_list),
            (types, "types", string_list),
            (use_define_for_class_fields, "useDefineForClassFields", bool),
            (use_unknown_in_catch_variables, "useUnknownInCatchVariables", bool),
            (verbatim_module_syntax, "verbatimModuleSyntax", bool),
        }
    };
}
pub(crate) use for_each_compiler_option;

/// The Rust type of a compilerOptions field, keyed by its kind token.
macro_rules! option_field_type {
    (bool) => { Option<bool> };
    (string) => { Option<String> };
    (path) => { Option<PathBuf> };
    (string_list) => { Option<Vec<String>> };
    (string_list_preserving_falsy) => { Option<Vec<String>> };
    (lib_list) => { Option<Vec<String>> };
    (path_list) => { Option<Vec<PathBuf>> };
    (paths_map) => { Option<CompilerOptionsPathsMap> };
    (number) => { Option<i64> };
    (enum($ty:ty)) => { Option<$ty> };
}

macro_rules! define_compiler_options {
    ($(($field:ident, $json:literal, $($kind:tt)+)),* $(,)?) => {
        /// tsgo `core.CompilerOptions`: every user-settable tsconfig `compilerOptions` field.
        ///
        /// `None` means "not set" (tsgo's `Tristate` `TSUnknown` / enum zero values). Path-typed
        /// values are absolute — the tsconfig parser resolves them against the directory of the
        /// config file that defined them (`${configDir}`-prefixed values against the root
        /// config's directory). See `crate::tsoptions` for parsing and `extends` merging.
        #[derive(Debug, Default, Clone)]
        pub struct CompilerOptions {
            $(
                #[doc = concat!("<https://www.typescriptlang.org/tsconfig/#", $json, ">")]
                pub $field: option_field_type!($($kind)+),
            )*
            /// tsgo `ConfigFilePath` (internal): the root config file this options object came
            /// from, set by the parser. Option semantics depend on it (e.g. the default
            /// `typeRoots` walk starts from it).
            pub config_file_path: Option<PathBuf>,
            /// tsgo `PathsBasePath` (internal): the directory [`Self::paths`] entries resolve
            /// against — the directory of the config that defined `paths`, set by the parser.
            pub paths_base_path: Option<PathBuf>,
        }
    };
}
for_each_compiler_option!(define_compiler_options);

impl CompilerOptions {
    /// tsgo `GetEmitScriptTarget`: `target`, defaulting to [`ScriptTarget::LATEST_STANDARD`].
    pub fn get_emit_script_target(&self) -> ScriptTarget {
        self.target.unwrap_or(ScriptTarget::LATEST_STANDARD)
    }

    /// tsgo `GetEmitModuleKind`: `module`, defaulting by the emit script target.
    pub fn get_emit_module_kind(&self) -> ModuleKind {
        if let Some(module) = self.module {
            return module;
        }
        let target = self.get_emit_script_target();
        if target == ScriptTarget::EsNext {
            ModuleKind::EsNext
        } else if target >= ScriptTarget::Es2022 {
            ModuleKind::Es2022
        } else if target >= ScriptTarget::Es2020 {
            ModuleKind::Es2020
        } else if target >= ScriptTarget::Es2015 {
            ModuleKind::Es2015
        } else {
            ModuleKind::CommonJs
        }
    }

    /// tsgo `GetModuleResolutionKind`: `moduleResolution`, with the unset/`classic`/`node10`
    /// cases derived from the emit module kind (node16-family -> `Node16`, `nodenext` ->
    /// `NodeNext`, everything else -> `Bundler`).
    pub fn get_module_resolution_kind(&self) -> ModuleResolutionKind {
        match self.module_resolution {
            None | Some(ModuleResolutionKind::Classic | ModuleResolutionKind::Node10) => {
                match self.get_emit_module_kind() {
                    ModuleKind::Node16 | ModuleKind::Node18 | ModuleKind::Node20 => {
                        ModuleResolutionKind::Node16
                    }
                    ModuleKind::NodeNext => ModuleResolutionKind::NodeNext,
                    _ => ModuleResolutionKind::Bundler,
                }
            }
            Some(kind) => kind,
        }
    }

    /// tsgo `GetEmitModuleDetectionKind`: `moduleDetection`, defaulting to `Force` for the
    /// node16..nodenext module kinds and `Auto` otherwise.
    pub fn get_emit_module_detection_kind(&self) -> ModuleDetectionKind {
        if let Some(kind) = self.module_detection {
            return kind;
        }
        let module = self.get_emit_module_kind();
        if module >= ModuleKind::Node16 && module <= ModuleKind::NodeNext {
            ModuleDetectionKind::Force
        } else {
            ModuleDetectionKind::Auto
        }
    }

    /// tsgo `GetResolvePackageJsonExports`: true unless explicitly disabled.
    pub fn get_resolve_package_json_exports(&self) -> bool {
        self.resolve_package_json_exports != Some(false)
    }

    /// tsgo `GetResolvePackageJsonImports`: true unless explicitly disabled.
    pub fn get_resolve_package_json_imports(&self) -> bool {
        self.resolve_package_json_imports != Some(false)
    }

    /// tsgo `GetAllowImportingTsExtensions`: `allowImportingTsExtensions` or
    /// `rewriteRelativeImportExtensions`.
    pub fn get_allow_importing_ts_extensions(&self) -> bool {
        self.allow_importing_ts_extensions == Some(true)
            || self.rewrite_relative_import_extensions == Some(true)
    }

    /// tsgo `GetResolveJsonModule`: the explicit value when set; otherwise `true` for the
    /// `node20`/`nodenext` emit module kinds or when the module-resolution kind is `Bundler`.
    pub fn get_resolve_json_module(&self) -> bool {
        if let Some(explicit) = self.resolve_json_module {
            return explicit;
        }
        // tsgo: "TODO in 6.0: add Node16/Node18".
        matches!(self.get_emit_module_kind(), ModuleKind::Node20 | ModuleKind::NodeNext)
            || self.get_module_resolution_kind() == ModuleResolutionKind::Bundler
    }

    /// tsgo `GetAllowJS`: `allowJs` if set, otherwise fall back to `checkJs`.
    pub fn get_allow_js(&self) -> bool {
        self.allow_js.unwrap_or(self.check_js == Some(true))
    }

    /// tsgo `GetStrictOptionValue`: a strict-family flag's value, defaulting to `strict` being
    /// anything but explicitly `false`.
    pub fn get_strict_option_value(&self, value: Option<bool>) -> bool {
        value.unwrap_or(self.strict != Some(false))
    }

    /// tsgo `GetEffectiveTypeRoots`: `typeRoots` when set (`from_config` = true); otherwise
    /// every ancestor directory's `node_modules/@types`, walking up from the config file's
    /// directory (or `current_directory` without one).
    ///
    /// # Panics
    ///
    /// * As in tsgo: when there is neither a config file path nor a current directory to
    ///   walk from.
    pub fn get_effective_type_roots(&self, current_directory: &Path) -> (Vec<PathBuf>, bool) {
        if let Some(type_roots) = &self.type_roots {
            return (type_roots.clone(), true);
        }
        let base_dir =
            self.config_file_path.as_deref().and_then(Path::parent).unwrap_or(current_directory);
        assert!(
            !base_dir.as_os_str().is_empty(),
            "cannot get effective type roots without a config file path or current directory"
        );
        let type_roots =
            base_dir.ancestors().map(|dir| dir.join("node_modules").join("@types")).collect();
        (type_roots, false)
    }

    /// tsgo `UsesWildcardTypes`: the `types` array contains `"*"`.
    pub fn uses_wildcard_types(&self) -> bool {
        self.types.as_ref().is_some_and(|types| types.iter().any(|name| name == "*"))
    }

    /// tsgo `GetIsolatedModules`: `isolatedModules` or `verbatimModuleSyntax`.
    pub fn get_isolated_modules(&self) -> bool {
        self.isolated_modules == Some(true) || self.verbatim_module_syntax == Some(true)
    }
}
