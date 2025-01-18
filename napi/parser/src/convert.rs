use rustc_hash::FxHashMap;

use oxc::syntax::module_record::{self, ModuleRecord};

use crate::types::{
    DynamicImport, EcmaScriptModule, ExportExportName, ExportExportNameKind, ExportImportName,
    ExportImportNameKind, ExportLocalName, ExportLocalNameKind, ImportName, ImportNameKind, Span,
    StaticExport, StaticExportEntry, StaticImport, StaticImportEntry, ValueSpan,
};

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
                        .map(StaticImportEntry::from)
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
            .map(|e| (e.statement_span, StaticExportEntry::from(e)))
            .collect::<Vec<_>>()
            .into_iter()
            .fold(FxHashMap::<_, Vec<StaticExportEntry>>::default(), |mut acc, (span, e)| {
                acc.entry(span).or_default().push(e);
                acc
            })
            .into_iter()
            .map(|(span, entries)| StaticExport { start: span.start, end: span.end, entries })
            .collect::<Vec<_>>();
        static_exports.sort_unstable_by_key(|e| e.start);

        let dynamic_imports = record
            .dynamic_imports
            .iter()
            .map(|i| DynamicImport {
                start: i.span.start,
                end: i.span.end,
                module_request: Span::from(&i.module_request),
            })
            .collect::<Vec<_>>();

        let import_metas = record.import_metas.iter().map(Span::from).collect();

        Self {
            has_module_syntax: record.has_module_syntax,
            static_imports,
            static_exports,
            dynamic_imports,
            import_metas,
        }
    }
}

impl From<&oxc::span::Span> for Span {
    fn from(span: &oxc::span::Span) -> Self {
        Self { start: span.start, end: span.end }
    }
}

impl From<&module_record::ExportEntry<'_>> for StaticExportEntry {
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

impl From<&module_record::ImportEntry<'_>> for StaticImportEntry {
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
