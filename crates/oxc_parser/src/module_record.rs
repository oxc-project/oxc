use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::BoundNames;
use oxc_span::{GetSpan, Span};
use oxc_syntax::module_record::*;

use crate::diagnostics;

pub struct ModuleRecordBuilder<'a> {
    allocator: &'a Allocator,
    module_record: ModuleRecord<'a>,
    export_entries: Vec<ExportEntry<'a>>,
    exported_bindings_duplicated: Vec<NameSpan<'a>>,
}

impl<'a> ModuleRecordBuilder<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            module_record: ModuleRecord::new(allocator),
            export_entries: vec![],
            exported_bindings_duplicated: vec![],
        }
    }

    pub fn build(mut self) -> (ModuleRecord<'a>, Vec<OxcDiagnostic>) {
        // The `ParseModule` algorithm requires `importedBoundNames` (import entries) to be
        // resolved before resolving export entries.
        self.resolve_export_entries();
        let errors = self.errors();
        (self.module_record, errors)
    }

    pub fn errors(&self) -> Vec<OxcDiagnostic> {
        let mut errors = vec![];

        let module_record = &self.module_record;

        // It is a Syntax Error if the ExportedNames of ModuleItemList contains any duplicate entries.
        for name_span in &self.exported_bindings_duplicated {
            let old_span = module_record.exported_bindings[&name_span.name];
            errors.push(diagnostics::duplicate_export(&name_span.name, name_span.span, old_span));
        }

        // Multiple default exports
        // `export default foo`
        // `export { default }`
        let default_exports = module_record
            .local_export_entries
            .iter()
            .filter_map(|export_entry| export_entry.export_name.default_export_span())
            .chain(
                module_record
                    .indirect_export_entries
                    .iter()
                    .filter_map(|export_entry| export_entry.export_name.default_export_span()),
            )
            .collect::<Vec<_>>();
        if default_exports.len() > 1 {
            errors.push(
                OxcDiagnostic::error("Duplicated default export").with_labels(default_exports),
            );
        }
        errors
    }

    fn add_module_request(&mut self, name: Atom<'a>, requested_module: RequestedModule) {
        self.module_record
            .requested_modules
            .entry(name)
            .or_insert_with(|| oxc_allocator::Vec::new_in(self.allocator))
            .push(requested_module);
    }

    fn add_import_entry(&mut self, entry: ImportEntry<'a>) {
        self.module_record.import_entries.push(entry);
    }

    fn add_export_entry(&mut self, entry: ExportEntry<'a>) {
        self.export_entries.push(entry);
    }

    fn append_local_export_entry(&mut self, entry: ExportEntry<'a>) {
        self.module_record.local_export_entries.push(entry);
    }

    fn append_indirect_export_entry(&mut self, entry: ExportEntry<'a>) {
        self.module_record.indirect_export_entries.push(entry);
    }

    fn append_star_export_entry(&mut self, entry: ExportEntry<'a>) {
        self.module_record.star_export_entries.push(entry);
    }

    fn add_export_binding(&mut self, name: Atom<'a>, span: Span) {
        if let Some(old_node) = self.module_record.exported_bindings.insert(name, span) {
            self.exported_bindings_duplicated.push(NameSpan::new(name, old_node));
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
                        .find(|entry| entry.local_name.name == name.name),
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
                                    statement_span: ie.statement_span,
                                    span: ee.span,
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
                                    local_name: ExportLocalName::default(),
                                    is_type: ie.is_type,
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

    pub fn visit_import_expression(&mut self, e: &ImportExpression<'a>) {
        self.module_record
            .dynamic_imports
            .push(DynamicImport { span: e.span, module_request: e.source.span() });
    }

    pub fn visit_import_meta(&mut self, span: Span) {
        self.module_record.has_module_syntax = true;
        self.module_record.import_metas.push(span);
    }

    pub fn visit_module_declaration(&mut self, module_decl: &ModuleDeclaration<'a>) {
        self.module_record.has_module_syntax = true;
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

    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        let module_request = NameSpan::new(decl.source.value, decl.source.span);

        if let Some(specifiers) = &decl.specifiers {
            for specifier in specifiers {
                let (import_name, local_name, is_type) = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => (
                        ImportImportName::Name(NameSpan::new(
                            specifier.imported.name(),
                            specifier.imported.span(),
                        )),
                        NameSpan::new(specifier.local.name, specifier.local.span),
                        decl.import_kind.is_type() || specifier.import_kind.is_type(),
                    ),
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => (
                        ImportImportName::NamespaceObject,
                        NameSpan::new(specifier.local.name, specifier.local.span),
                        decl.import_kind.is_type(),
                    ),
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => (
                        ImportImportName::Default(specifier.span),
                        NameSpan::new(specifier.local.name, specifier.local.span),
                        decl.import_kind.is_type(),
                    ),
                };
                self.add_import_entry(ImportEntry {
                    statement_span: decl.span,
                    module_request: module_request.clone(),
                    import_name,
                    local_name,
                    is_type,
                });
            }
        }
        self.add_module_request(
            module_request.name,
            RequestedModule {
                statement_span: decl.span,
                span: module_request.span,
                is_type: decl.import_kind.is_type(),
                is_import: true,
            },
        );
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        let module_request = NameSpan::new(decl.source.value, decl.source.span);
        let export_entry = ExportEntry {
            statement_span: decl.span,
            span: decl.span,
            module_request: Some(module_request.clone()),
            import_name: decl
                .exported
                .as_ref()
                .map_or(ExportImportName::AllButDefault, |_| ExportImportName::All),
            export_name: decl.exported.as_ref().map_or(ExportExportName::Null, |exported_name| {
                ExportExportName::Name(NameSpan::new(exported_name.name(), exported_name.span()))
            }),
            local_name: ExportLocalName::default(),
            is_type: decl.export_kind.is_type(),
        };
        self.add_export_entry(export_entry);
        if let Some(exported_name) = &decl.exported {
            self.add_export_binding(exported_name.name(), exported_name.span());
        }
        self.add_module_request(
            module_request.name,
            RequestedModule {
                statement_span: decl.span,
                span: module_request.span,
                is_type: decl.export_kind.is_type(),
                is_import: false,
            },
        );
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        // ignore all TypeScript syntax as they overload
        if decl.declaration.is_typescript_syntax() {
            return;
        }
        let exported_name = &decl.exported;

        let local_name = match &decl.declaration {
            ExportDefaultDeclarationKind::Identifier(ident) => {
                ExportLocalName::Default(NameSpan::new(ident.name, ident.span))
            }
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                func.id.as_ref().map_or_else(
                    || ExportLocalName::Null,
                    |id| ExportLocalName::Name(NameSpan::new(id.name, id.span)),
                )
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => class.id.as_ref().map_or_else(
                || ExportLocalName::Null,
                |id| ExportLocalName::Name(NameSpan::new(id.name, id.span)),
            ),
            _ => ExportLocalName::Null,
        };
        let export_entry = ExportEntry {
            statement_span: decl.span,
            span: decl.declaration.span(),
            module_request: None,
            import_name: ExportImportName::default(),
            export_name: ExportExportName::Default(exported_name.span()),
            local_name,
            is_type: false,
        };
        self.add_export_entry(export_entry);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if decl.export_kind.is_type() {
            return;
        }
        // ignore all TypeScript syntax as they overload
        if decl.is_typescript_syntax() {
            return;
        }

        let module_request =
            decl.source.as_ref().map(|source| NameSpan::new(source.value, source.span));

        if let Some(module_request) = &module_request {
            self.add_module_request(
                module_request.name,
                RequestedModule {
                    statement_span: decl.span,
                    span: module_request.span,
                    is_type: decl.export_kind.is_type(),
                    is_import: false,
                },
            );
        }

        if let Some(d) = &decl.declaration {
            d.bound_names(&mut |ident| {
                let export_name = ExportExportName::Name(NameSpan::new(ident.name, ident.span));
                let local_name = ExportLocalName::Name(NameSpan::new(ident.name, ident.span));
                let export_entry = ExportEntry {
                    statement_span: decl.span,
                    span: d.span(),
                    module_request: module_request.clone(),
                    import_name: ExportImportName::Null,
                    export_name,
                    local_name,
                    is_type: decl.export_kind.is_type(),
                };
                self.add_export_entry(export_entry);
                self.add_export_binding(ident.name, ident.span);
            });
        }

        for specifier in &decl.specifiers {
            let export_name = ExportExportName::Name(NameSpan::new(
                specifier.exported.name(),
                specifier.exported.span(),
            ));
            let import_name = if module_request.is_some() {
                ExportImportName::Name(NameSpan::new(
                    specifier.local.name(),
                    specifier.local.span(),
                ))
            } else {
                ExportImportName::Null
            };
            let local_name = if module_request.is_some() {
                ExportLocalName::Null
            } else {
                ExportLocalName::Name(NameSpan::new(specifier.local.name(), specifier.local.span()))
            };
            let export_entry = ExportEntry {
                statement_span: decl.span,
                span: specifier.span,
                module_request: module_request.clone(),
                import_name,
                export_name,
                local_name,
                is_type: specifier.export_kind.is_type() || decl.export_kind.is_type(),
            };
            self.add_export_entry(export_entry);
            self.add_export_binding(specifier.exported.name(), specifier.exported.span());
        }
    }
}

#[cfg(test)]
mod module_record_tests {
    use oxc_allocator::Allocator;
    use oxc_span::{SourceType, Span};
    use oxc_syntax::module_record::*;

    use crate::Parser;

    fn build<'a>(allocator: &'a Allocator, source_text: &'a str) -> ModuleRecord<'a> {
        let source_type = SourceType::mjs();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        ret.module_record
    }

    // Table 55 gives examples of ImportEntry records fields used to represent the syntactic import forms:
    // `https://tc39.es/ecma262/#table-import-forms-mapping-to-importentry-records`

    #[test]
    fn import_default() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import v from 'mod'");
        let import_entry = ImportEntry {
            statement_span: Span::new(0, 19),
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
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import * as ns from 'mod'");
        let import_entry = ImportEntry {
            statement_span: Span::new(0, 25),
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
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import { x } from 'mod'");
        let import_entry = ImportEntry {
            statement_span: Span::new(0, 23),
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
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import { x as v } from 'mod'");
        let import_entry = ImportEntry {
            statement_span: Span::new(0, 28),
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
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import 'mod'");
        assert!(module_record.import_entries.is_empty());
    }

    // Table 57 gives examples of the ExportEntry record fields used to represent the syntactic export forms
    // `https://tc39.es/ecma262/#table-export-forms-mapping-to-exportentry-records`

    #[test]
    fn export_star() {
        // ExportDeclaration : export ExportFromClause FromClause ;
        // ExportFromClause : *
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export * from 'mod'");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 19),
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
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export * as ns from 'mod'");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 25),
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
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export { x }");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 12),
            span: Span::new(9, 10),
            export_name: ExportExportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            local_name: ExportLocalName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn named_exports_alias() {
        // ExportDeclaration : export NamedExports ;
        // ExportSpecifier : ModuleExportName as ModuleExportName
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export { x as v }");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 17),
            span: Span::new(9, 15),
            export_name: ExportExportName::Name(NameSpan::new("v".into(), Span::new(14, 15))),
            local_name: ExportLocalName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn named_exports_from() {
        // ExportDeclaration : export ExportFromClause FromClause ;
        // ExportSpecifier : ModuleExportName
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export { x } from 'mod'");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 23),
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
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export { x as v } from 'mod'");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 28),
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
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export var v");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 12),
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
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export default function f() {}");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 30),
            span: Span::new(15, 30),
            export_name: ExportExportName::Default(Span::new(7, 14)),
            local_name: ExportLocalName::Name(NameSpan::new("f".into(), Span::new(24, 25))),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn export_default_function_expression() {
        // ExportDeclaration : export default HoistableDeclaration
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export default function() {}");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 28),
            span: Span::new(15, 28),
            export_name: ExportExportName::Default(Span::new(7, 14)),
            local_name: ExportLocalName::Null,
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn export_default_expression() {
        // ExportDeclaration : export default HoistableDeclaration
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export default 42");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 17),
            span: Span::new(15, 17),
            export_name: ExportExportName::Default(Span::new(7, 14)),
            local_name: ExportLocalName::Null,
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn export_named_default() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export { default }");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 18),
            span: Span::new(9, 16),
            export_name: ExportExportName::Name(NameSpan::new("default".into(), Span::new(9, 16))),
            local_name: ExportLocalName::Name(NameSpan::new("default".into(), Span::new(9, 16))),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn indirect_export_entries() {
        let allocator = Allocator::default();
        let module_record =
            build(&allocator, "import { x } from 'mod';export { x };export * as ns from 'mod';");
        assert_eq!(module_record.indirect_export_entries.len(), 2);
        assert_eq!(
            module_record.indirect_export_entries[0],
            ExportEntry {
                statement_span: Span::new(0, 24),
                span: Span::new(33, 34),
                module_request: Some(NameSpan::new("mod".into(), Span::new(18, 23))),
                import_name: ExportImportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
                export_name: ExportExportName::Name(NameSpan::new("x".into(), Span::new(33, 34))),
                local_name: ExportLocalName::Null,
                is_type: false
            }
        );
        assert_eq!(
            module_record.indirect_export_entries[1],
            ExportEntry {
                statement_span: Span::new(37, 63),
                span: Span::new(37, 63),
                module_request: Some(NameSpan::new("mod".into(), Span::new(57, 62))),
                import_name: ExportImportName::All,
                export_name: ExportExportName::Name(NameSpan::new("ns".into(), Span::new(49, 51))),
                local_name: ExportLocalName::Null,
                is_type: false
            }
        );
    }

    #[test]
    fn import_meta() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import.meta.foo; import.meta.bar");
        assert_eq!(module_record.import_metas.len(), 2);
        assert_eq!(module_record.import_metas[0], Span::new(0, 11));
        assert_eq!(module_record.import_metas[1], Span::new(17, 28));
    }

    #[test]
    fn dynamic_imports() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import('foo')");
        assert_eq!(module_record.dynamic_imports.len(), 1);
        assert_eq!(module_record.dynamic_imports[0].span, Span::new(0, 13));
        assert_eq!(module_record.dynamic_imports[0].module_request, Span::new(7, 12));
    }
}
