//! [ECMAScript Module Record](https://tc39.es/ecma262/#sec-abstract-module-records)

use std::{fmt, hash::BuildHasherDefault, path::PathBuf, sync::Arc};

use dashmap::DashMap;
use indexmap::IndexMap;
use rustc_hash::{FxHashMap, FxHasher};

use oxc_span::{CompactStr, Span};

/// ESM Module Record
///
/// All data inside this data structure are for ESM, no commonjs data is allowed.
///
/// See
/// * <https://tc39.es/ecma262/#table-additional-fields-of-source-text-module-records>
/// * <https://tc39.es/ecma262/#cyclic-module-record>
#[derive(Default)]
pub struct ModuleRecord {
    /// This module has no import / export statements
    pub not_esm: bool,

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
    pub requested_modules: IndexMap<CompactStr, Vec<RequestedModule>, BuildHasherDefault<FxHasher>>,

    /// `[[LoadedModules]]`
    ///
    /// A map from the specifier strings used by the module represented by this record to request the importation of a module to the resolved Module Record.
    /// The list does not contain two different Records with the same `[[Specifier]]`.
    pub loaded_modules: DashMap<CompactStr, Arc<ModuleRecord>, BuildHasherDefault<FxHasher>>,

    /// `[[ImportEntries]]`
    ///
    /// A List of ImportEntry records derived from the code of this module
    pub import_entries: Vec<ImportEntry>,

    /// `[[LocalExportEntries]]`
    ///
    /// A List of ExportEntry records derived from the code of this module
    /// that correspond to declarations that occur within the module
    pub local_export_entries: Vec<ExportEntry>,

    /// `[[IndirectExportEntries]]`
    ///
    /// A List of ExportEntry records derived from the code of this module
    /// that correspond to reexported imports that occur within the module
    /// or exports from export * as namespace declarations.
    pub indirect_export_entries: Vec<ExportEntry>,

    /// `[[StarExportEntries]]`
    ///
    /// A List of ExportEntry records derived from the code of this module
    /// that correspond to export * declarations that occur within the module,
    /// not including export * as namespace declarations.
    pub star_export_entries: Vec<ExportEntry>,

    /// Local exported bindings
    pub exported_bindings: FxHashMap<CompactStr, Span>,

    /// Local duplicated exported bindings, for diagnostics
    pub exported_bindings_duplicated: Vec<NameSpan>,

    /// Reexported bindings from `export * from 'specifier'`
    /// Keyed by resolved path
    pub exported_bindings_from_star_export: DashMap<PathBuf, Vec<CompactStr>>,

    /// `export default name`
    ///         ^^^^^^^ span
    pub export_default: Option<Span>,

    /// Duplicated span of `export default` for diagnostics
    pub export_default_duplicated: Vec<Span>,
}

impl ModuleRecord {
    pub fn new(resolved_absolute_path: PathBuf) -> Self {
        Self { resolved_absolute_path, ..Self::default() }
    }
}

impl fmt::Debug for ModuleRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        // recursively formatting loaded modules can crash when the module graph is cyclic
        let loaded_modules = self
            .loaded_modules
            .iter()
            .map(|entry| (entry.key().to_string()))
            .reduce(|acc, key| format!("{acc}, {key}"))
            .unwrap_or_default();
        let loaded_modules = format!("{{ {loaded_modules} }}");
        f.debug_struct("ModuleRecord")
            .field("not_esm", &self.not_esm)
            .field("resolved_absolute_path", &self.resolved_absolute_path)
            .field("requested_modules", &self.requested_modules)
            .field("loaded_modules", &loaded_modules)
            .field("import_entries", &self.import_entries)
            .field("local_export_entries", &self.local_export_entries)
            .field("indirect_export_entries", &self.indirect_export_entries)
            .field("star_export_entries", &self.star_export_entries)
            .field("exported_bindings", &self.exported_bindings)
            .field("exported_bindings_duplicated", &self.exported_bindings_duplicated)
            .field("exported_bindings_from_star_export", &self.exported_bindings_from_star_export)
            .field("export_default", &self.export_default)
            .field("export_default_duplicated", &self.export_default_duplicated)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameSpan {
    name: CompactStr,
    span: Span,
}

impl NameSpan {
    pub fn new(name: CompactStr, span: Span) -> Self {
        Self { name, span }
    }

    pub fn name(&self) -> &CompactStr {
        &self.name
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

/// [`ImportEntry`](https://tc39.es/ecma262/#importentry-record)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportEntry {
    /// String value of the ModuleSpecifier of the ImportDeclaration.
    pub module_request: NameSpan,

    /// The name under which the desired binding is exported by the module identified by `[[ModuleRequest]]`.
    pub import_name: ImportImportName,

    /// The name that is used to locally access the imported value from within the importing module.
    pub local_name: NameSpan,

    pub is_type: bool,
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

/// [`ExportEntry`](https://tc39.es/ecma262/#exportentry-record)
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
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            Self::Name(name) => Some(name.span()),
            Self::Default(span) => Some(*span),
            Self::Null => None,
        }
    }
}

/// `LocalName` for `ExportEntry`
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ExportLocalName {
    Name(NameSpan),
    Default(Span),
    #[default]
    Null,
}

impl ExportLocalName {
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

pub struct FunctionMeta {
    pub deprecated: bool,
}

#[derive(Debug, Clone)]
pub struct RequestedModule {
    span: Span,
    is_type: bool,
    /// is_import is true if the module is requested by an import statement.
    is_import: bool,
}

impl RequestedModule {
    pub fn new(span: Span, is_type: bool, is_import: bool) -> Self {
        Self { span, is_type, is_import }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn is_type(&self) -> bool {
        self.is_type
    }

    pub fn is_import(&self) -> bool {
        self.is_import
    }
}

#[cfg(test)]
mod test {
    use super::{ExportExportName, ExportLocalName, ImportImportName, NameSpan};
    use oxc_span::Span;

    #[test]
    fn import_import_name() {
        let name = NameSpan::new("name".into(), Span::new(0, 0));
        assert!(!ImportImportName::Name(name.clone()).is_default());
        assert!(!ImportImportName::NamespaceObject.is_default());
        assert!(ImportImportName::Default(Span::new(0, 0)).is_default());

        assert!(!ImportImportName::Name(name.clone()).is_namespace_object());
        assert!(ImportImportName::NamespaceObject.is_namespace_object());
        assert!(!ImportImportName::Default(Span::new(0, 0)).is_namespace_object());
    }

    #[test]
    fn export_import_name() {
        let name = NameSpan::new("name".into(), Span::new(0, 0));
        assert!(!ExportExportName::Name(name.clone()).is_default());
        assert!(ExportExportName::Default(Span::new(0, 0)).is_default());
        assert!(!ExportExportName::Null.is_default());

        assert!(!ExportExportName::Name(name).is_null());
        assert!(!ExportExportName::Default(Span::new(0, 0)).is_null());
        assert!(ExportExportName::Null.is_null());
    }

    #[test]
    fn export_local_name() {
        let name = NameSpan::new("name".into(), Span::new(0, 0));
        assert!(!ExportLocalName::Name(name.clone()).is_default());
        assert!(ExportLocalName::Default(Span::new(0, 0)).is_default());
        assert!(!ExportLocalName::Null.is_default());

        assert!(!ExportLocalName::Name(name).is_null());
        assert!(!ExportLocalName::Default(Span::new(0, 0)).is_null());
        assert!(ExportLocalName::Null.is_null());
    }
}
