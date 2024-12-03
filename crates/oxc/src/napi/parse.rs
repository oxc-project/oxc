use napi_derive::napi;

use rustc_hash::FxHashMap;

use oxc_span::Span;
use oxc_syntax::module_record::{self, ModuleRecord};

/// Babel Parser Options
#[napi(object)]
#[derive(Default)]
pub struct ParserOptions {
    #[napi(ts_type = "'script' | 'module' | 'unambiguous' | undefined")]
    pub source_type: Option<String>,
    pub source_filename: Option<String>,
    /// Emit `ParenthesizedExpression` in AST.
    ///
    /// If this option is true, parenthesized expressions are represented by
    /// (non-standard) `ParenthesizedExpression` nodes that have a single `expression` property
    /// containing the expression inside parentheses.
    ///
    /// Default: true
    pub preserve_parens: Option<bool>,
}

#[napi(object)]
pub struct ParseResult {
    #[napi(ts_type = "import(\"@oxc-project/types\").Program")]
    pub program: String,
    pub module: EcmaScriptModule,
    pub comments: Vec<Comment>,
    pub errors: Vec<String>,
}

#[napi(object)]
pub struct Comment {
    #[napi(ts_type = "'Line' | 'Block'")]
    pub r#type: &'static str,
    pub value: String,
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
pub struct EcmaScriptModule {
    /// Import Statements.
    pub static_imports: Vec<StaticImport>,
    /// Export Statements.
    pub static_exports: Vec<StaticExport>,
}

#[napi(object)]
pub struct ValueSpan {
    pub value: String,
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
pub struct StaticImport {
    /// Start of import statement.
    pub start: u32,
    /// End of import statement.
    pub end: u32,
    /// Import source.
    ///
    /// ```js
    /// import { foo } from "mod";
    /// //                   ^^^
    /// ```
    pub module_request: ValueSpan,
    /// Import specifiers.
    ///
    /// Empty for `import "mod"`.
    pub entries: Vec<ImportEntry>,
}

#[napi(object)]
pub struct ImportEntry {
    /// The name under which the desired binding is exported by the module.
    ///
    /// ```js
    /// import { foo } from "mod";
    /// //       ^^^
    /// import { foo as bar } from "mod";
    /// //       ^^^
    /// ```
    pub import_name: ImportName,
    /// The name that is used to locally access the imported value from within the importing module.
    /// ```js
    /// import { foo } from "mod";
    /// //       ^^^
    /// import { foo as bar } from "mod";
    /// //              ^^^
    /// ```
    pub local_name: ValueSpan,

    /// Whether this binding is for a TypeScript type-only import.
    ///
    /// `true` for the following imports:
    /// ```ts
    /// import type { foo } from "mod";
    /// import { type foo } from "mod";
    /// ```
    pub is_type: bool,
}

#[napi(string_enum)]
pub enum ImportNameKind {
    /// `import { x } from "mod"`
    Name,
    /// `import * as ns from "mod"`
    NamespaceObject,
    /// `import defaultExport from "mod"`
    Default,
}

#[napi(object)]
pub struct ImportName {
    pub kind: ImportNameKind,
    pub name: Option<String>,
    pub start: Option<u32>,
    pub end: Option<u32>,
}

#[napi(object)]
pub struct StaticExport {
    pub start: u32,
    pub end: u32,
    pub entries: Vec<ExportEntry>,
}

#[napi(object)]
pub struct ExportEntry {
    pub start: u32,
    pub end: u32,
    pub module_request: Option<ValueSpan>,
    /// The name under which the desired binding is exported by the module`.
    pub import_name: ExportImportName,
    /// The name used to export this binding by this module.
    pub export_name: ExportExportName,
    /// The name that is used to locally access the exported value from within the importing module.
    pub local_name: ExportLocalName,
}

#[napi(string_enum)]
pub enum ExportImportNameKind {
    /// `export { name }
    Name,
    /// `export * as ns from "mod"`
    All,
    /// `export * from "mod"`
    AllButDefault,
    /// Does not have a specifier.
    None,
}

#[napi(object)]
pub struct ExportImportName {
    pub kind: ExportImportNameKind,
    pub name: Option<String>,
    pub start: Option<u32>,
    pub end: Option<u32>,
}

#[napi(string_enum)]
pub enum ExportExportNameKind {
    /// `export { name }
    Name,
    /// `export default expression`
    Default,
    /// `export * from "mod"
    None,
}

#[napi(object)]
pub struct ExportExportName {
    pub kind: ExportExportNameKind,
    pub name: Option<String>,
    pub start: Option<u32>,
    pub end: Option<u32>,
}

#[napi(object)]
pub struct ExportLocalName {
    pub kind: ExportLocalNameKind,
    pub name: Option<String>,
    pub start: Option<u32>,
    pub end: Option<u32>,
}

#[napi(string_enum)]
pub enum ExportLocalNameKind {
    /// `export { name }
    Name,
    /// `export default expression`
    Default,
    /// If the exported value is not locally accessible from within the module.
    /// `export default function () {}`
    None,
}

impl From<&ModuleRecord<'_>> for EcmaScriptModule {
    fn from(record: &ModuleRecord<'_>) -> Self {
        let mut static_imports = record
            .requested_modules
            .iter()
            .flat_map(|(name, requested_modules)| {
                requested_modules.iter().filter(|m| m.is_import).map(|m| {
                    let entries = record
                        .import_entries
                        .iter()
                        .filter(|e| e.statement_span == m.statement_span)
                        .map(ImportEntry::from)
                        .collect::<Vec<_>>();
                    {
                        StaticImport {
                            start: m.statement_span.start,
                            end: m.statement_span.end,
                            module_request: ValueSpan {
                                value: name.to_string(),
                                start: m.span.start,
                                end: m.span.end,
                            },
                            entries,
                        }
                    }
                })
            })
            .collect::<Vec<_>>();
        static_imports.sort_unstable_by_key(|e| e.start);

        let mut static_exports = record
            .local_export_entries
            .iter()
            .chain(record.indirect_export_entries.iter())
            .chain(record.star_export_entries.iter())
            .map(|e| (e.statement_span, ExportEntry::from(e)))
            .collect::<Vec<_>>()
            .into_iter()
            .fold(FxHashMap::<Span, Vec<ExportEntry>>::default(), |mut acc, (span, e)| {
                acc.entry(span).or_default().push(e);
                acc
            })
            .into_iter()
            .map(|(span, entries)| StaticExport { start: span.start, end: span.end, entries })
            .collect::<Vec<_>>();
        static_exports.sort_unstable_by_key(|e| e.start);

        Self { static_imports, static_exports }
    }
}

impl From<&module_record::ImportEntry<'_>> for ImportEntry {
    fn from(e: &module_record::ImportEntry<'_>) -> Self {
        Self {
            import_name: ImportName::from(&e.import_name),
            local_name: ValueSpan::from(&e.local_name),
            is_type: e.is_type,
        }
    }
}

impl From<&module_record::ImportImportName<'_>> for ImportName {
    fn from(e: &module_record::ImportImportName<'_>) -> Self {
        let (kind, name, start, end) = match e {
            module_record::ImportImportName::Name(name_span) => (
                ImportNameKind::Name,
                Some(name_span.name.to_string()),
                Some(name_span.span.start),
                Some(name_span.span.end),
            ),
            module_record::ImportImportName::NamespaceObject => {
                (ImportNameKind::NamespaceObject, None, None, None)
            }
            module_record::ImportImportName::Default(span) => {
                (ImportNameKind::Default, None, Some(span.start), Some(span.end))
            }
        };
        Self { kind, name, start, end }
    }
}

impl From<&module_record::NameSpan<'_>> for ValueSpan {
    fn from(name_span: &module_record::NameSpan) -> Self {
        Self {
            value: name_span.name.to_string(),
            start: name_span.span.start,
            end: name_span.span.end,
        }
    }
}

impl From<&module_record::ExportEntry<'_>> for ExportEntry {
    fn from(e: &module_record::ExportEntry) -> Self {
        Self {
            start: e.span.start,
            end: e.span.end,
            module_request: e.module_request.as_ref().map(ValueSpan::from),
            import_name: ExportImportName::from(&e.import_name),
            export_name: ExportExportName::from(&e.export_name),
            local_name: ExportLocalName::from(&e.local_name),
        }
    }
}

impl From<&module_record::ExportImportName<'_>> for ExportImportName {
    fn from(e: &module_record::ExportImportName<'_>) -> Self {
        let (kind, name, start, end) = match e {
            module_record::ExportImportName::Name(name_span) => (
                ExportImportNameKind::Name,
                Some(name_span.name.to_string()),
                Some(name_span.span.start),
                Some(name_span.span.end),
            ),
            module_record::ExportImportName::All => (ExportImportNameKind::All, None, None, None),
            module_record::ExportImportName::AllButDefault => {
                (ExportImportNameKind::AllButDefault, None, None, None)
            }
            module_record::ExportImportName::Null => (ExportImportNameKind::None, None, None, None),
        };
        Self { kind, name, start, end }
    }
}

impl From<&module_record::ExportExportName<'_>> for ExportExportName {
    fn from(e: &module_record::ExportExportName<'_>) -> Self {
        let (kind, name, start, end) = match e {
            module_record::ExportExportName::Name(name_span) => (
                ExportExportNameKind::Name,
                Some(name_span.name.to_string()),
                Some(name_span.span.start),
                Some(name_span.span.end),
            ),
            module_record::ExportExportName::Default(span) => {
                (ExportExportNameKind::Default, None, Some(span.start), Some(span.end))
            }
            module_record::ExportExportName::Null => (ExportExportNameKind::None, None, None, None),
        };
        Self { kind, name, start, end }
    }
}

impl From<&module_record::ExportLocalName<'_>> for ExportLocalName {
    fn from(e: &module_record::ExportLocalName<'_>) -> Self {
        let (kind, name, start, end) = match e {
            module_record::ExportLocalName::Name(name_span) => (
                ExportLocalNameKind::Name,
                Some(name_span.name.to_string()),
                Some(name_span.span.start),
                Some(name_span.span.end),
            ),
            module_record::ExportLocalName::Default(name_span) => (
                ExportLocalNameKind::Default,
                Some(name_span.name.to_string()),
                Some(name_span.span.start),
                Some(name_span.span.end),
            ),
            module_record::ExportLocalName::Null => (ExportLocalNameKind::None, None, None, None),
        };
        Self { kind, name, start, end }
    }
}
