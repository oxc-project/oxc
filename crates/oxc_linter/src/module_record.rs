//! [ECMAScript Module Record](https://tc39.es/ecma262/#sec-abstract-module-records)

use std::{
    fmt,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock, RwLock},
};

use rustc_hash::FxHashMap;

use oxc_semantic::Semantic;
use oxc_span::{CompactStr, Span};
pub use oxc_syntax::module_record::RequestedModule;

/// ESM Module Record
///
/// All data inside this data structure are for ESM, no commonjs data is allowed.
///
/// See
/// * <https://tc39.es/ecma262/#table-additional-fields-of-source-text-module-records>
/// * <https://tc39.es/ecma262/#cyclic-module-record>
#[derive(Default)]
pub struct ModuleRecord {
    /// This module has ESM syntax: `import` and `export`.
    pub has_module_syntax: bool,

    /// Resolved absolute path to this module record
    pub resolved_absolute_path: PathBuf,

    /// `[[RequestedModules]]`
    ///
    /// A List of all the ModuleSpecifier strings used by the module represented by this record to request the importation of a module. The List is in source text occurrence order.
    ///
    /// Module requests from:
    ///   import ImportClause FromClause
    ///   import ModuleSpecifier
    ///   export ExportFromClause FromClause
    /// Keyed by ModuleSpecifier, valued by all node occurrences
    pub requested_modules: FxHashMap<CompactStr, Vec<RequestedModule>>,

    /// `[[LoadedModules]]`
    ///
    /// A map from the specifier strings used by the module represented by this record to request
    /// the importation of a module to the resolved Module Record. The list does not contain two
    /// different Records with the same `[[Specifier]]`.
    ///
    /// Note that Oxc does not support cross-file analysis, so this map will be empty after
    /// [`ModuleRecord`] is created. You must link the module records yourself.
    pub loaded_modules: RwLock<FxHashMap<CompactStr, Arc<ModuleRecord>>>,

    /// `[[ImportEntries]]`
    ///
    /// A List of `ImportEntry` records derived from the code of this module
    pub import_entries: Vec<ImportEntry>,

    /// `[[LocalExportEntries]]`
    ///
    /// A List of `ExportEntry` records derived from the code of this module
    /// that correspond to declarations that occur within the module
    pub local_export_entries: Vec<ExportEntry>,

    /// `[[IndirectExportEntries]]`
    ///
    /// A List of `ExportEntry` records derived from the code of this module
    /// that correspond to reexported imports that occur within the module
    /// or exports from `export * as namespace` declarations.
    pub indirect_export_entries: Vec<ExportEntry>,

    /// `[[StarExportEntries]]`
    ///
    /// A List of `ExportEntry` records derived from the code of this module
    /// that correspond to `export *` declarations that occur within the module,
    /// not including `export * as namespace` declarations.
    pub star_export_entries: Vec<ExportEntry>,

    /// Local exported bindings
    pub exported_bindings: FxHashMap<CompactStr, Span>,

    /// Reexported bindings from `export * from 'specifier'`
    /// Keyed by resolved path
    exported_bindings_from_star_export: OnceLock<FxHashMap<PathBuf, Vec<CompactStr>>>,

    /// `export default name`
    ///         ^^^^^^^ span
    pub export_default: Option<Span>,
}

impl fmt::Debug for ModuleRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        // recursively formatting loaded modules can crash when the module graph is cyclic
        let loaded_modules = self
            .loaded_modules
            .read()
            .unwrap()
            .iter()
            .map(|(key, _)| (key.to_string()))
            .reduce(|acc, key| format!("{acc}, {key}"))
            .unwrap_or_default();
        let loaded_modules = format!("{{ {loaded_modules} }}");
        f.debug_struct("ModuleRecord")
            .field("has_module_syntax", &self.has_module_syntax)
            .field("resolved_absolute_path", &self.resolved_absolute_path)
            .field("requested_modules", &self.requested_modules)
            .field("loaded_modules", &loaded_modules)
            .field("import_entries", &self.import_entries)
            .field("local_export_entries", &self.local_export_entries)
            .field("indirect_export_entries", &self.indirect_export_entries)
            .field("star_export_entries", &self.star_export_entries)
            .field("exported_bindings", &self.exported_bindings)
            .field("exported_bindings_from_star_export", &self.exported_bindings_from_star_export)
            .field("export_default", &self.export_default)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameSpan {
    pub name: CompactStr,
    span: Span,
}

impl NameSpan {
    pub fn new(name: CompactStr, span: Span) -> Self {
        Self { name, span }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl<'a> From<&oxc_syntax::module_record::NameSpan<'a>> for NameSpan {
    fn from(other: &oxc_syntax::module_record::NameSpan<'a>) -> Self {
        Self { name: CompactStr::from(other.name.as_str()), span: other.span }
    }
}

/// [`ImportEntry`](https://tc39.es/ecma262/#importentry-record)
///
/// ## Examples
///
/// ```ts
/// //     _ local_name
/// import v from "mod";
/// //             ^^^ module_request
///
/// //     ____ is_type will be `true`
/// import type { foo as bar } from "mod";
/// // import_name^^^    ^^^ local_name
///
/// import * as ns from "mod";
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportEntry {
    /// Span of the import statement.
    ///
    /// ## Examples
    ///
    /// ```ts
    /// import { foo } from "mod";
    /// ^^^^^^^^^^^^^^^^^^^^^^^^^
    /// ```
    pub statement_span: Span,

    /// String value of the ModuleSpecifier of the ImportDeclaration.
    ///
    /// ## Examples
    ///
    /// ```ts
    /// import { foo } from "mod";
    /// //                   ^^^
    /// ```
    pub module_request: NameSpan,

    /// The name under which the desired binding is exported by the module identified by `[[ModuleRequest]]`.
    ///
    /// ## Examples
    ///
    /// ```ts
    /// import { foo } from "mod";
    /// //       ^^^
    /// import { foo as bar } from "mod";
    /// //       ^^^
    /// ```
    pub import_name: ImportImportName,

    /// The name that is used to locally access the imported value from within the importing module.
    ///
    /// ## Examples
    ///
    /// ```ts
    /// import { foo } from "mod";
    /// //       ^^^
    /// import { foo as bar } from "mod";
    /// //              ^^^
    /// ```
    pub local_name: NameSpan,

    /// Whether this binding is for a TypeScript type-only import. This is a non-standard field.
    /// When creating a [`ModuleRecord`] for a JavaScript file, this will always be false.
    ///
    /// ## Examples
    ///
    /// `is_type` will be `true` for the following imports:
    /// ```ts
    /// import type { foo } from "mod";
    /// import { type foo } from "mod";
    /// ```
    ///
    /// and will be `false` for these imports:
    /// ```ts
    /// import { foo } from "mod";
    /// import { foo as type } from "mod";
    /// ```
    pub is_type: bool,
}

impl<'a> From<&oxc_syntax::module_record::ImportEntry<'a>> for ImportEntry {
    fn from(other: &oxc_syntax::module_record::ImportEntry<'a>) -> Self {
        Self {
            statement_span: other.statement_span,
            module_request: NameSpan::from(&other.module_request),
            import_name: ImportImportName::from(&other.import_name),
            local_name: NameSpan::from(&other.local_name),
            is_type: other.is_type,
        }
    }
}

/// `ImportName` For `ImportEntry`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportImportName {
    Name(NameSpan),
    NamespaceObject,
    Default(Span),
}

impl ImportImportName {
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default(_))
    }

    pub fn is_namespace_object(&self) -> bool {
        matches!(self, Self::NamespaceObject)
    }
}

impl<'a> From<&oxc_syntax::module_record::ImportImportName<'a>> for ImportImportName {
    fn from(other: &oxc_syntax::module_record::ImportImportName<'a>) -> Self {
        match other {
            oxc_syntax::module_record::ImportImportName::Name(name_span) => {
                Self::Name(NameSpan::from(name_span))
            }
            oxc_syntax::module_record::ImportImportName::NamespaceObject => Self::NamespaceObject,
            oxc_syntax::module_record::ImportImportName::Default(span) => Self::Default(*span),
        }
    }
}

/// [`ExportEntry`](https://tc39.es/ecma262/#exportentry-record)
///
/// Describes a single exported binding from a module. Named export statements that contain more
/// than one binding produce multiple ExportEntry records.
///
/// ## Examples
///
/// ```ts
/// // foo's ExportEntry nas no `module_request` or `import_name.
/// //       ___ local_name
/// export { foo };
/// //       ^^^ export_name. Since there's no alias, it's the same as local_name.
///
/// // re-exports do not produce local bindings, so `local_name` is null.
/// //       ___ import_name    __ module_request
/// export { foo as bar } from "mod";
/// //              ^^^ export_name
///
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ExportEntry {
    /// Span for the entire export entry
    pub span: Span,

    /// The String value of the ModuleSpecifier of the ExportDeclaration.
    /// null if the ExportDeclaration does not have a ModuleSpecifier.
    pub module_request: Option<NameSpan>,

    /// The name under which the desired binding is exported by the module identified by `[[ModuleRequest]]`.
    /// null if the ExportDeclaration does not have a ModuleSpecifier.
    /// "all" is used for `export * as ns from "mod"`` declarations.
    /// "all-but-default" is used for `export * from "mod" declarations`.
    pub import_name: ExportImportName,

    /// The name used to export this binding by this module.
    pub export_name: ExportExportName,

    /// The name that is used to locally access the exported value from within the importing module.
    /// null if the exported value is not locally accessible from within the module.
    pub local_name: ExportLocalName,

    /// Whether the export is a TypeScript `export type`.
    ///
    /// Examples:
    ///
    /// ```ts
    /// export type * from 'mod'
    /// export type * as ns from 'mod'
    /// export type { foo }
    /// export { type foo }
    /// export type { foo } from 'mod'
    /// ```
    pub is_type: bool,
}

impl<'a> From<&oxc_syntax::module_record::ExportEntry<'a>> for ExportEntry {
    fn from(other: &oxc_syntax::module_record::ExportEntry<'a>) -> Self {
        Self {
            span: other.span,
            module_request: other.module_request.as_ref().map(NameSpan::from),
            import_name: ExportImportName::from(&other.import_name),
            export_name: ExportExportName::from(&other.export_name),
            local_name: ExportLocalName::from(&other.local_name),
            is_type: other.is_type,
        }
    }
}

/// `ImportName` for `ExportEntry`
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ExportImportName {
    Name(NameSpan),
    /// all is used for export * as ns from "mod" declarations.
    All,
    /// all-but-default is used for export * from "mod" declarations.
    AllButDefault,
    /// the ExportDeclaration does not have a ModuleSpecifier
    #[default]
    Null,
}

impl<'a> From<&oxc_syntax::module_record::ExportImportName<'a>> for ExportImportName {
    fn from(other: &oxc_syntax::module_record::ExportImportName<'a>) -> Self {
        match other {
            oxc_syntax::module_record::ExportImportName::Name(name_span) => {
                Self::Name(NameSpan::from(name_span))
            }
            oxc_syntax::module_record::ExportImportName::All => Self::All,
            oxc_syntax::module_record::ExportImportName::AllButDefault => Self::AllButDefault,
            oxc_syntax::module_record::ExportImportName::Null => Self::Null,
        }
    }
}

impl ExportImportName {
    pub fn is_all(&self) -> bool {
        matches!(self, Self::All)
    }

    pub fn is_all_but_default(&self) -> bool {
        matches!(self, Self::AllButDefault)
    }
}

/// `ExportName` for `ExportEntry`
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ExportExportName {
    Name(NameSpan),
    Default(Span),
    #[default]
    Null,
}

impl ExportExportName {
    /// Returns `true` if this is [`ExportExportName::Default`].
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default(_))
    }

    /// Returns `true` if this is [`ExportExportName::Null`].
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Attempt to get the [`Span`] of this export name.
    pub fn span(&self) -> Option<Span> {
        match self {
            Self::Name(name) => Some(name.span()),
            Self::Default(span) => Some(*span),
            Self::Null => None,
        }
    }
}

impl<'a> From<&oxc_syntax::module_record::ExportExportName<'a>> for ExportExportName {
    fn from(other: &oxc_syntax::module_record::ExportExportName<'a>) -> Self {
        match other {
            oxc_syntax::module_record::ExportExportName::Name(name_span) => {
                Self::Name(NameSpan::from(name_span))
            }
            oxc_syntax::module_record::ExportExportName::Default(span) => Self::Default(*span),
            oxc_syntax::module_record::ExportExportName::Null => Self::Null,
        }
    }
}

/// `LocalName` for `ExportEntry`
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ExportLocalName {
    Name(NameSpan),
    /// `export default name_span`
    Default(NameSpan),
    #[default]
    Null,
}

impl ExportLocalName {
    /// `true` if this is a [`ExportLocalName::Default`].
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default(_))
    }

    /// `true` if this is a [`ExportLocalName::Null`].
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Get the bound name of this export. [`None`] for [`ExportLocalName::Null`].
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Name(name) | Self::Default(name) => Some(name.name.as_str()),
            Self::Null => None,
        }
    }
}

impl<'a> From<&oxc_syntax::module_record::ExportLocalName<'a>> for ExportLocalName {
    fn from(other: &oxc_syntax::module_record::ExportLocalName<'a>) -> Self {
        match other {
            oxc_syntax::module_record::ExportLocalName::Name(name_span) => {
                Self::Name(NameSpan::from(name_span))
            }
            oxc_syntax::module_record::ExportLocalName::Default(name_span) => {
                Self::Default(NameSpan::from(name_span))
            }
            oxc_syntax::module_record::ExportLocalName::Null => Self::Null,
        }
    }
}

impl ModuleRecord {
    pub fn new(
        path: &Path,
        other: &oxc_syntax::module_record::ModuleRecord,
        _semantic: &Semantic,
    ) -> Self {
        Self {
            has_module_syntax: other.has_module_syntax,
            resolved_absolute_path: path.to_path_buf(),
            requested_modules: other
                .requested_modules
                .iter()
                .map(|(name, requested_modules)| {
                    (
                        CompactStr::from(name.as_str()),
                        requested_modules.iter().copied().collect::<Vec<_>>(),
                    )
                })
                .collect(),
            import_entries: other.import_entries.iter().map(ImportEntry::from).collect(),

            local_export_entries: other
                .local_export_entries
                .iter()
                .map(ExportEntry::from)
                .collect(),
            indirect_export_entries: other
                .indirect_export_entries
                .iter()
                .map(ExportEntry::from)
                .collect(),
            star_export_entries: other.star_export_entries.iter().map(ExportEntry::from).collect(),
            exported_bindings: other
                .exported_bindings
                .iter()
                .map(|(name, span)| (CompactStr::from(name.as_str()), *span))
                .collect(),
            export_default: other
                .local_export_entries
                .iter()
                .filter_map(|export_entry| export_entry.export_name.default_export_span())
                .chain(
                    other
                        .indirect_export_entries
                        .iter()
                        .filter_map(|export_entry| export_entry.export_name.default_export_span()),
                )
                .next(),
            ..ModuleRecord::default()
        }
    }

    pub(crate) fn exported_bindings_from_star_export(
        &self,
    ) -> &FxHashMap<PathBuf, Vec<CompactStr>> {
        self.exported_bindings_from_star_export.get_or_init(|| {
            let mut exported_bindings_from_star_export: FxHashMap<PathBuf, Vec<CompactStr>> =
                FxHashMap::default();
            let loaded_modules = self.loaded_modules.read().unwrap();
            for export_entry in &self.star_export_entries {
                let Some(module_request) = &export_entry.module_request else {
                    continue;
                };
                let Some(remote_module_record) = loaded_modules.get(module_request.name()) else {
                    continue;
                };
                // Append both remote `bindings` and `exported_bindings_from_star_export`
                let remote_exported_bindings_from_star_export = remote_module_record
                    .exported_bindings_from_star_export()
                    .iter()
                    .flat_map(|(_, value)| value.clone());
                let remote_bindings = remote_module_record
                    .exported_bindings
                    .keys()
                    .cloned()
                    .chain(remote_exported_bindings_from_star_export);
                exported_bindings_from_star_export
                    .entry(remote_module_record.resolved_absolute_path.clone())
                    .or_default()
                    .extend(remote_bindings);
            }
            exported_bindings_from_star_export
        })
    }
}
