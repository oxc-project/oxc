//! [ECMAScript Module Record](https://tc39.es/ecma262/#sec-abstract-module-records)

use oxc_span::{Atom, Span};
use rustc_hash::FxHashMap;

/// [Source Text Module Record](https://tc39.es/ecma262/#table-additional-fields-of-source-text-module-records)
#[derive(Debug, Default)]
pub struct ModuleRecord {
    /// <https://tc39.es/ecma262/#sec-static-semantics-modulerequests>
    /// Module requests from:
    ///   import ImportClause FromClause
    ///   import ModuleSpecifier
    ///   export ExportFromClause FromClause
    /// Keyed by FromClause, valued by all node occurrences
    pub module_requests: FxHashMap<Atom, Vec<Span>>,

    /// A List of ImportEntry records derived from the code of this module
    pub import_entries: Vec<ImportEntry>,

    /// A List of ExportEntry records derived from the code of this module
    /// that correspond to declarations that occur within the module
    pub local_export_entries: Vec<ExportEntry>,

    /// A List of ExportEntry records derived from the code of this module
    /// that correspond to reexported imports that occur within the module
    /// or exports from export * as namespace declarations.
    pub indirect_export_entries: Vec<ExportEntry>,

    /// A List of ExportEntry records derived from the code of this module
    /// that correspond to export * declarations that occur within the module,
    /// not including export * as namespace declarations.
    pub star_export_entries: Vec<ExportEntry>,

    pub exported_bindings: FxHashMap<Atom, Span>,
    pub exported_bindings_duplicated: Vec<NameSpan>,

    pub export_default: Option<Span>,
    pub export_default_duplicated: Vec<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameSpan {
    name: Atom,
    span: Span,
}

impl NameSpan {
    pub fn new(name: Atom, span: Span) -> Self {
        Self { name, span }
    }

    pub fn name(&self) -> &Atom {
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

/// [`ExportEntry`](https://tc39.es/ecma262/#importentry-record)
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

        assert!(!ImportImportName::Name(name).is_namespace_object());
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
