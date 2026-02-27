//! Owned module record — converted from the parser's arena-allocated `ModuleRecord<'a>`.

use compact_str::CompactString;
use oxc_span::Span;
use rustc_hash::FxHashMap;

use oxc_syntax::module_record as syntax;

/// Owned version of an import/export name+span pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameSpan {
    pub name: CompactString,
    pub span: Span,
}

impl NameSpan {
    pub fn new(name: CompactString, span: Span) -> Self {
        Self { name, span }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl<'a> From<&syntax::NameSpan<'a>> for NameSpan {
    fn from(other: &syntax::NameSpan<'a>) -> Self {
        Self { name: CompactString::from(other.name.as_str()), span: other.span }
    }
}

/// `ImportName` for `ImportEntry`
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

impl<'a> From<&syntax::ImportImportName<'a>> for ImportImportName {
    fn from(other: &syntax::ImportImportName<'a>) -> Self {
        match other {
            syntax::ImportImportName::Name(name_span) => Self::Name(NameSpan::from(name_span)),
            syntax::ImportImportName::NamespaceObject => Self::NamespaceObject,
            syntax::ImportImportName::Default(span) => Self::Default(*span),
        }
    }
}

/// An import entry from the module record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportEntry {
    pub statement_span: Span,
    pub module_request: NameSpan,
    pub import_name: ImportImportName,
    pub local_name: NameSpan,
    pub is_type: bool,
}

impl<'a> From<&syntax::ImportEntry<'a>> for ImportEntry {
    fn from(other: &syntax::ImportEntry<'a>) -> Self {
        Self {
            statement_span: other.statement_span,
            module_request: NameSpan::from(&other.module_request),
            import_name: ImportImportName::from(&other.import_name),
            local_name: NameSpan::from(&other.local_name),
            is_type: other.is_type,
        }
    }
}

/// `ImportName` for `ExportEntry`
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ExportImportName {
    Name(NameSpan),
    All,
    AllButDefault,
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

impl<'a> From<&syntax::ExportImportName<'a>> for ExportImportName {
    fn from(other: &syntax::ExportImportName<'a>) -> Self {
        match other {
            syntax::ExportImportName::Name(name_span) => Self::Name(NameSpan::from(name_span)),
            syntax::ExportImportName::All => Self::All,
            syntax::ExportImportName::AllButDefault => Self::AllButDefault,
            syntax::ExportImportName::Null => Self::Null,
        }
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

impl<'a> From<&syntax::ExportExportName<'a>> for ExportExportName {
    fn from(other: &syntax::ExportExportName<'a>) -> Self {
        match other {
            syntax::ExportExportName::Name(name_span) => Self::Name(NameSpan::from(name_span)),
            syntax::ExportExportName::Default(span) => Self::Default(*span),
            syntax::ExportExportName::Null => Self::Null,
        }
    }
}

/// `LocalName` for `ExportEntry`
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ExportLocalName {
    Name(NameSpan),
    Default(NameSpan),
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

    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Name(name) | Self::Default(name) => Some(name.name.as_str()),
            Self::Null => None,
        }
    }
}

impl<'a> From<&syntax::ExportLocalName<'a>> for ExportLocalName {
    fn from(other: &syntax::ExportLocalName<'a>) -> Self {
        match other {
            syntax::ExportLocalName::Name(name_span) => Self::Name(NameSpan::from(name_span)),
            syntax::ExportLocalName::Default(name_span) => Self::Default(NameSpan::from(name_span)),
            syntax::ExportLocalName::Null => Self::Null,
        }
    }
}

/// An export entry from the module record.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ExportEntry {
    pub statement_span: Span,
    pub span: Span,
    pub module_request: Option<NameSpan>,
    pub import_name: ExportImportName,
    pub export_name: ExportExportName,
    pub local_name: ExportLocalName,
    pub is_type: bool,
}

impl<'a> From<&syntax::ExportEntry<'a>> for ExportEntry {
    fn from(other: &syntax::ExportEntry<'a>) -> Self {
        Self {
            statement_span: other.statement_span,
            span: other.span,
            module_request: other.module_request.as_ref().map(NameSpan::from),
            import_name: ExportImportName::from(&other.import_name),
            export_name: ExportExportName::from(&other.export_name),
            local_name: ExportLocalName::from(&other.local_name),
            is_type: other.is_type,
        }
    }
}

/// Owned module record — all import/export data from a single parsed module.
///
/// Converted from the arena-allocated `oxc_syntax::module_record::ModuleRecord<'a>`.
#[derive(Debug, Default)]
pub struct ModuleRecord {
    /// Whether this module has ESM syntax (import/export).
    pub has_module_syntax: bool,

    /// `[[RequestedModules]]` — specifier → occurrences.
    pub requested_modules: FxHashMap<CompactString, Vec<syntax::RequestedModule>>,

    /// `[[ImportEntries]]`
    pub import_entries: Vec<ImportEntry>,

    /// `[[LocalExportEntries]]`
    pub local_export_entries: Vec<ExportEntry>,

    /// `[[IndirectExportEntries]]`
    pub indirect_export_entries: Vec<ExportEntry>,

    /// `[[StarExportEntries]]`
    pub star_export_entries: Vec<ExportEntry>,

    /// Local exported bindings: name → span.
    pub exported_bindings: FxHashMap<CompactString, Span>,

    /// Span of `export default`, if present.
    pub export_default: Option<Span>,
}

impl ModuleRecord {
    /// Create an owned `ModuleRecord` from the parser's arena-allocated version.
    pub fn from_syntax(other: &syntax::ModuleRecord) -> Self {
        Self {
            has_module_syntax: other.has_module_syntax,
            requested_modules: other
                .requested_modules
                .iter()
                .map(|(name, modules)| {
                    (CompactString::from(name.as_str()), modules.iter().copied().collect())
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
                .map(|(name, span)| (CompactString::from(name.as_str()), *span))
                .collect(),
            export_default: other
                .local_export_entries
                .iter()
                .filter_map(|e| e.export_name.default_export_span())
                .chain(
                    other
                        .indirect_export_entries
                        .iter()
                        .filter_map(|e| e.export_name.default_export_span()),
                )
                .next(),
        }
    }
}
