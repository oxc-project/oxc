use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::BoundNames;
use oxc_span::{CompactStr, GetSpan, Span};
use oxc_syntax::module_record::*;

use crate::diagnostics;

#[derive(Default)]
pub struct ModuleRecordBuilder {
    pub module_record: ModuleRecord,
    export_entries: Vec<ExportEntry>,
}

impl ModuleRecordBuilder {
    pub fn build(mut self) -> ModuleRecord {
        // The `ParseModule` algorithm requires `importedBoundNames` (import entries) to be
        // resolved before resolving export entries.
        self.resolve_export_entries();
        self.module_record
    }

    pub fn errors(&self) -> Vec<OxcDiagnostic> {
        let mut errors = vec![];

        let module_record = &self.module_record;

        // It is a Syntax Error if the ExportedNames of ModuleItemList contains any duplicate entries.
        for name_span in &module_record.exported_bindings_duplicated {
            let old_span = module_record.exported_bindings[name_span.name()];
            errors.push(diagnostics::duplicate_export(
                name_span.name(),
                name_span.span(),
                old_span,
            ));
        }

        for span in &module_record.export_default_duplicated {
            let old_span = module_record.export_default.unwrap();
            errors.push(diagnostics::duplicate_export("default", *span, old_span));
        }

        // `export default x;`
        // `export { y as default };`
        if let (Some(span), Some(default_span)) =
            (module_record.exported_bindings.get("default"), &module_record.export_default)
        {
            errors.push(diagnostics::duplicate_export("default", *default_span, *span));
        }

        errors
    }

    fn add_module_request(&mut self, name_span: &NameSpan, is_type: bool, is_import: bool) {
        self.module_record
            .requested_modules
            .entry(name_span.name().clone())
            .or_default()
            .push(RequestedModule::new(name_span.span(), is_type, is_import));
    }

    fn add_import_entry(&mut self, entry: ImportEntry) {
        self.module_record.import_entries.push(entry);
    }

    fn add_export_entry(&mut self, entry: ExportEntry) {
        self.export_entries.push(entry);
    }

    fn append_local_export_entry(&mut self, entry: ExportEntry) {
        self.module_record.local_export_entries.push(entry);
    }

    fn append_indirect_export_entry(&mut self, entry: ExportEntry) {
        self.module_record.indirect_export_entries.push(entry);
    }

    fn append_star_export_entry(&mut self, entry: ExportEntry) {
        self.module_record.star_export_entries.push(entry);
    }

    fn add_export_binding(&mut self, name: CompactStr, span: Span) {
        if let Some(old_node) = self.module_record.exported_bindings.insert(name.clone(), span) {
            self.module_record.exported_bindings_duplicated.push(NameSpan::new(name, old_node));
        }
    }

    fn add_default_export(&mut self, span: Span) {
        if let Some(old_node) = self.module_record.export_default.replace(span) {
            self.module_record.export_default_duplicated.push(old_node);
        }
    }

    /// [ParseModule](https://tc39.es/ecma262/#sec-parsemodule)
    /// Step 10.
    fn resolve_export_entries(&mut self) {
        let export_entries = self.export_entries.drain(..).collect::<Vec<_>>();
        // 10. For each ExportEntry Record ee of exportEntries, do
        for ee in export_entries {
            // a. If ee.[[ModuleRequest]] is null, then
            if ee.module_request.is_none() {
                let found_import_entry = match &ee.local_name {
                    ExportLocalName::Name(name) => self
                        .module_record
                        .import_entries
                        .iter()
                        .find(|entry| entry.local_name.name() == name.name()),
                    _ => None,
                };
                match found_import_entry {
                    // i. If ee.[[LocalName]] is not an element of importedBoundNames, then
                    None => {
                        // 1. Append ee to localExportEntries.
                        self.append_local_export_entry(ee);
                    }
                    // ii. Else,
                    // 1. Let ie be the element of importEntries whose [[LocalName]] is the same as ee.[[LocalName]].
                    Some(ie) => {
                        match &ie.import_name {
                            // 2. If ie.[[ImportName]] is namespace-object, then
                            ImportImportName::NamespaceObject => {
                                // a. NOTE: This is a re-export of an imported module namespace object.
                                // b. Append ee to localExportEntries.
                                self.append_local_export_entry(ee);
                            }
                            // 3. Else,
                            // a. NOTE: This is a re-export of a single name.
                            // Append the ExportEntry Record { [[ModuleRequest]]: ie.[[ModuleRequest]], [[ImportName]]: ie.[[ImportName]], [[LocalName]]: null, [[ExportName]]: ee.[[ExportName]] }
                            // to indirectExportEntries.
                            ImportImportName::Name(_) | ImportImportName::Default(_) => {
                                let export_entry = ExportEntry {
                                    module_request: Some(ie.module_request.clone()),
                                    import_name: match &ie.import_name {
                                        ImportImportName::Name(name) => {
                                            ExportImportName::Name(name.clone())
                                        }
                                        // `import d from "mod"`
                                        // `export { d }`
                                        //           ^ this is local_name of ie
                                        ImportImportName::Default(_) => {
                                            ExportImportName::Name(ie.local_name.clone())
                                        }
                                        ImportImportName::NamespaceObject => unreachable!(),
                                    },
                                    export_name: ee.export_name.clone(),
                                    span: ee.span,
                                    ..ExportEntry::default()
                                };
                                self.append_indirect_export_entry(export_entry);
                            }
                        }
                    }
                }
                // b. Else if ee.[[ImportName]] is all-but-default, then
            } else if ee.import_name.is_all_but_default() {
                // i. Assert: ee.[[ExportName]] is null.
                debug_assert!(ee.export_name.is_null());
                self.append_star_export_entry(ee);
                // c. Else,
            } else {
                // i. Append ee to indirectExportEntries.
                self.append_indirect_export_entry(ee);
            }
        }
    }

    pub fn visit_module_declaration(&mut self, module_decl: &ModuleDeclaration) {
        self.module_record.not_esm = false;
        match module_decl {
            ModuleDeclaration::ImportDeclaration(import_decl) => {
                self.visit_import_declaration(import_decl);
            }
            ModuleDeclaration::ExportAllDeclaration(export_all_decl) => {
                self.visit_export_all_declaration(export_all_decl);
            }
            ModuleDeclaration::ExportDefaultDeclaration(export_default_decl) => {
                self.visit_export_default_declaration(export_default_decl);
            }
            ModuleDeclaration::ExportNamedDeclaration(export_named_decl) => {
                self.visit_export_named_declaration(export_named_decl);
            }
            ModuleDeclaration::TSExportAssignment(_)
            | ModuleDeclaration::TSNamespaceExportDeclaration(_) => { /* noop */ }
        }
    }

    fn visit_import_declaration(&mut self, decl: &ImportDeclaration) {
        let module_request = NameSpan::new(decl.source.value.to_compact_str(), decl.source.span);

        if let Some(specifiers) = &decl.specifiers {
            for specifier in specifiers {
                let (import_name, local_name, is_type) = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => (
                        ImportImportName::Name(NameSpan::new(
                            specifier.imported.name().to_compact_str(),
                            specifier.imported.span(),
                        )),
                        NameSpan::new(specifier.local.name.to_compact_str(), specifier.local.span),
                        decl.import_kind.is_type() || specifier.import_kind.is_type(),
                    ),
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => (
                        ImportImportName::NamespaceObject,
                        NameSpan::new(specifier.local.name.to_compact_str(), specifier.local.span),
                        decl.import_kind.is_type(),
                    ),
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => (
                        ImportImportName::Default(specifier.span),
                        NameSpan::new(specifier.local.name.to_compact_str(), specifier.local.span),
                        decl.import_kind.is_type(),
                    ),
                };
                self.add_import_entry(ImportEntry {
                    module_request: module_request.clone(),
                    import_name,
                    local_name,
                    is_type,
                });
            }
        }
        self.add_module_request(
            &module_request,
            decl.import_kind.is_type(),
            /* is_import */ true,
        );
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration) {
        let module_request = NameSpan::new(decl.source.value.to_compact_str(), decl.source.span);
        let export_entry = ExportEntry {
            module_request: Some(module_request.clone()),
            import_name: decl
                .exported
                .as_ref()
                .map_or(ExportImportName::AllButDefault, |_| ExportImportName::All),
            export_name: decl.exported.as_ref().map_or(ExportExportName::Null, |exported_name| {
                ExportExportName::Name(NameSpan::new(
                    exported_name.name().to_compact_str(),
                    exported_name.span(),
                ))
            }),
            span: decl.span,
            ..ExportEntry::default()
        };
        self.add_export_entry(export_entry);
        if let Some(exported_name) = &decl.exported {
            self.add_export_binding(exported_name.name().to_compact_str(), exported_name.span());
        }
        self.add_module_request(
            &module_request,
            decl.export_kind.is_type(),
            /* is_import */ false,
        );
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration) {
        // ignore all TypeScript syntax as they overload
        if decl.declaration.is_typescript_syntax() {
            return;
        }
        let exported_name = &decl.exported;
        let exported_name_span = decl.exported.span();
        self.add_default_export(exported_name_span);

        let local_name = match &decl.declaration {
            ExportDefaultDeclarationKind::Identifier(ident) => {
                ExportLocalName::Default(NameSpan::new(ident.name.to_compact_str(), ident.span))
            }
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                func.id.as_ref().map_or_else(
                    || ExportLocalName::Null,
                    |id| ExportLocalName::Name(NameSpan::new(id.name.to_compact_str(), id.span)),
                )
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => class.id.as_ref().map_or_else(
                || ExportLocalName::Null,
                |id| ExportLocalName::Name(NameSpan::new(id.name.to_compact_str(), id.span)),
            ),
            _ => ExportLocalName::Null,
        };
        let export_entry = ExportEntry {
            export_name: ExportExportName::Default(exported_name.span()),
            local_name,
            span: decl.declaration.span(),
            ..ExportEntry::default()
        };
        self.add_export_entry(export_entry);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration) {
        if decl.export_kind.is_type() {
            return;
        }
        // ignore all TypeScript syntax as they overload
        if decl.is_typescript_syntax() {
            return;
        }

        let module_request = decl
            .source
            .as_ref()
            .map(|source| NameSpan::new(source.value.to_compact_str(), source.span));

        if let Some(module_request) = &module_request {
            self.add_module_request(
                module_request,
                decl.export_kind.is_type(),
                /* is_import */ false,
            );
        }

        if let Some(decl) = &decl.declaration {
            decl.bound_names(&mut |ident| {
                let export_name =
                    ExportExportName::Name(NameSpan::new(ident.name.to_compact_str(), ident.span));
                let local_name =
                    ExportLocalName::Name(NameSpan::new(ident.name.to_compact_str(), ident.span));
                let export_entry = ExportEntry {
                    span: decl.span(),
                    module_request: module_request.clone(),
                    import_name: ExportImportName::Null,
                    export_name,
                    local_name,
                };
                self.add_export_entry(export_entry);
                self.add_export_binding(ident.name.to_compact_str(), ident.span);
            });
        }

        for specifier in &decl.specifiers {
            let export_name = ExportExportName::Name(NameSpan::new(
                specifier.exported.name().to_compact_str(),
                specifier.exported.span(),
            ));
            let import_name = if module_request.is_some() {
                ExportImportName::Name(NameSpan::new(
                    specifier.local.name().to_compact_str(),
                    specifier.local.span(),
                ))
            } else {
                ExportImportName::Null
            };
            let local_name = if module_request.is_some() {
                ExportLocalName::Null
            } else {
                ExportLocalName::Name(NameSpan::new(
                    specifier.local.name().to_compact_str(),
                    specifier.local.span(),
                ))
            };
            let export_entry = ExportEntry {
                span: specifier.span,
                module_request: module_request.clone(),
                import_name,
                export_name,
                local_name,
            };
            self.add_export_entry(export_entry);
            self.add_export_binding(
                specifier.exported.name().to_compact_str(),
                specifier.exported.span(),
            );
        }
    }
}

#[cfg(test)]
mod module_record_tests {
    use oxc_allocator::Allocator;
    use oxc_span::{SourceType, Span};
    use oxc_syntax::module_record::*;

    use crate::Parser;

    fn build(source_text: &str) -> ModuleRecord {
        let source_type = SourceType::mjs();
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        ret.module_record
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
            local_name: ExportLocalName::Null,
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
            local_name: ExportLocalName::Null,
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
