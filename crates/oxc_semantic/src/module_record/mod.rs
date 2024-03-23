mod builder;

pub use builder::ModuleRecordBuilder;

#[cfg(test)]
mod module_record_tests {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::{SourceType, Span};
    #[allow(clippy::wildcard_imports)]
    use oxc_syntax::module_record::*;
    use std::{path::PathBuf, sync::Arc};

    use crate::SemanticBuilder;

    fn build(source_text: &str) -> Arc<ModuleRecord> {
        let source_type = SourceType::default().with_module(true);
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        let program = allocator.alloc(ret.program);
        let semantic_ret = SemanticBuilder::new(source_text, source_type)
            .with_trivias(ret.trivias)
            .build_module_record(PathBuf::new(), program)
            .build(program);
        Arc::clone(&semantic_ret.semantic.module_record)
    }

    // Table 55 gives examples of ImportEntry records fields used to represent the syntactic import forms:
    // `https://tc39.es/ecma262/#table-import-forms-mapping-to-importentry-records`

    #[test]
    fn import_default() {
        let module_record = build("import v from 'mod'");
        let import_entry = ImportEntry {
            module_request: NameSpan::new("mod".into(), Span::new(14, 19)),
            import_name: ImportImportName::Default(Span::new(7, 8)),
            local_name: NameSpan::new("v".into(), Span::new(7, 8)),
            is_type: false,
        };
        assert_eq!(module_record.import_entries.len(), 1);
        assert_eq!(module_record.import_entries[0], import_entry);
    }

    #[test]
    fn import_namespace() {
        let module_record = build("import * as ns from 'mod'");
        let import_entry = ImportEntry {
            module_request: NameSpan::new("mod".into(), Span::new(20, 25)),
            import_name: ImportImportName::NamespaceObject,
            local_name: NameSpan::new("ns".into(), Span::new(12, 14)),
            is_type: false,
        };
        assert_eq!(module_record.import_entries.len(), 1);
        assert_eq!(module_record.import_entries[0], import_entry);
    }

    #[test]
    fn import_specifier() {
        let module_record = build("import { x } from 'mod'");
        let import_entry = ImportEntry {
            module_request: NameSpan::new("mod".into(), Span::new(18, 23)),
            import_name: ImportImportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            local_name: NameSpan::new("x".into(), Span::new(9, 10)),
            is_type: false,
        };
        assert_eq!(module_record.import_entries.len(), 1);
        assert_eq!(module_record.import_entries[0], import_entry);
    }

    #[test]
    fn import_specifier_alias() {
        let module_record = build("import { x as v } from 'mod'");
        let import_entry = ImportEntry {
            module_request: NameSpan::new("mod".into(), Span::new(23, 28)),
            import_name: ImportImportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            local_name: NameSpan::new("v".into(), Span::new(14, 15)),
            is_type: false,
        };
        assert_eq!(module_record.import_entries.len(), 1);
        assert_eq!(module_record.import_entries[0], import_entry);
    }

    #[test]
    fn import_without_binding() {
        let module_record = build("import 'mod'");
        assert!(module_record.import_entries.is_empty());
    }

    // Table 57 gives examples of the ExportEntry record fields used to represent the syntactic export forms
    // `https://tc39.es/ecma262/#table-export-forms-mapping-to-exportentry-records`

    #[test]
    fn export_star() {
        // ExportDeclaration : export ExportFromClause FromClause ;
        // ExportFromClause : *
        let module_record = build("export * from 'mod'");
        let export_entry = ExportEntry {
            module_request: Some(NameSpan::new("mod".into(), Span::new(14, 19))),
            import_name: ExportImportName::AllButDefault,
            span: Span::new(0, 19),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.star_export_entries.len(), 1);
        assert_eq!(module_record.star_export_entries[0], export_entry);
    }

    #[test]
    fn export_star_as_namespace() {
        // ExportDeclaration : export ExportFromClause FromClause ;
        // ExportFromClause : * as ModuleExportName
        let module_record = build("export * as ns from 'mod'");
        let export_entry = ExportEntry {
            module_request: Some(NameSpan::new("mod".into(), Span::new(20, 25))),
            import_name: ExportImportName::All,
            export_name: ExportExportName::Name(NameSpan::new("ns".into(), Span::new(12, 14))),
            span: Span::new(0, 25),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.indirect_export_entries.len(), 1);
        assert_eq!(module_record.indirect_export_entries[0], export_entry);
    }

    #[test]
    fn named_exports() {
        // ExportDeclaration : export NamedExports ;
        // ExportSpecifier : ModuleExportName
        let module_record = build("export { x }");
        let export_entry = ExportEntry {
            export_name: ExportExportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            local_name: ExportLocalName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            span: Span::new(9, 10),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn named_exports_alias() {
        // ExportDeclaration : export NamedExports ;
        // ExportSpecifier : ModuleExportName as ModuleExportName
        let module_record = build("export { x as v }");
        let export_entry = ExportEntry {
            export_name: ExportExportName::Name(NameSpan::new("v".into(), Span::new(14, 15))),
            local_name: ExportLocalName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            span: Span::new(9, 15),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn named_exports_from() {
        // ExportDeclaration : export ExportFromClause FromClause ;
        // ExportSpecifier : ModuleExportName
        let module_record = build("export { x } from 'mod'");
        let export_entry = ExportEntry {
            module_request: Some(NameSpan::new("mod".into(), Span::new(18, 23))),
            export_name: ExportExportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            import_name: ExportImportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            span: Span::new(9, 10),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.indirect_export_entries.len(), 1);
        assert_eq!(module_record.indirect_export_entries[0], export_entry);
    }

    #[test]
    fn named_exports_alias_from() {
        // ExportDeclaration : export ExportFromClause FromClause ;
        // ExportSpecifier : ModuleExportName as ModuleExportName
        let module_record = build("export { x as v } from 'mod'");
        let export_entry = ExportEntry {
            module_request: Some(NameSpan::new("mod".into(), Span::new(23, 28))),
            export_name: ExportExportName::Name(NameSpan::new("v".into(), Span::new(14, 15))),
            import_name: ExportImportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            span: Span::new(9, 15),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.indirect_export_entries.len(), 1);
        assert_eq!(module_record.indirect_export_entries[0], export_entry);
    }

    #[test]
    fn export_declaration() {
        // ExportDeclaration : export VariableStatement
        let module_record = build("export var v");
        let export_entry = ExportEntry {
            export_name: ExportExportName::Name(NameSpan::new("v".into(), Span::new(11, 12))),
            local_name: ExportLocalName::Name(NameSpan::new("v".into(), Span::new(11, 12))),
            span: Span::new(7, 12),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn export_default_declaration() {
        // ExportDeclaration : export default HoistableDeclaration
        let module_record = build("export default function f() {}");
        let export_entry = ExportEntry {
            export_name: ExportExportName::Default(Span::new(7, 14)),
            local_name: ExportLocalName::Name(NameSpan::new("f".into(), Span::new(24, 25))),
            span: Span::new(15, 30),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn export_default_function_expression() {
        // ExportDeclaration : export default HoistableDeclaration
        let module_record = build("export default function() {}");
        let export_entry = ExportEntry {
            export_name: ExportExportName::Default(Span::new(7, 14)),
            local_name: ExportLocalName::Default(Span::new(7, 14)),
            span: Span::new(15, 28),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn export_default_expression() {
        // ExportDeclaration : export default HoistableDeclaration
        let module_record = build("export default 42");
        let export_entry = ExportEntry {
            export_name: ExportExportName::Default(Span::new(7, 14)),
            local_name: ExportLocalName::Default(Span::new(7, 14)),
            span: Span::new(15, 17),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn export_named_default() {
        let module_record = build("export { default }");
        let export_entry = ExportEntry {
            export_name: ExportExportName::Name(NameSpan::new("default".into(), Span::new(9, 16))),
            local_name: ExportLocalName::Name(NameSpan::new("default".into(), Span::new(9, 16))),
            span: Span::new(9, 16),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn indirect_export_entries() {
        let module_record =
            build("import { x } from 'mod';export { x };export * as ns from 'mod';");
        assert_eq!(module_record.indirect_export_entries.len(), 2);
        assert_eq!(
            module_record.indirect_export_entries[0],
            ExportEntry {
                module_request: Some(NameSpan::new("mod".into(), Span::new(18, 23))),
                span: Span::new(33, 34),
                import_name: ExportImportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
                export_name: ExportExportName::Name(NameSpan::new("x".into(), Span::new(33, 34))),
                local_name: ExportLocalName::Null,
            }
        );
        assert_eq!(
            module_record.indirect_export_entries[1],
            ExportEntry {
                module_request: Some(NameSpan::new("mod".into(), Span::new(57, 62))),
                span: Span::new(37, 63),
                import_name: ExportImportName::All,
                export_name: ExportExportName::Name(NameSpan::new("ns".into(), Span::new(49, 51))),
                local_name: ExportLocalName::Null,
            }
        );
    }
}
