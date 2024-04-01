use std::path::PathBuf;

#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, syntax_directed_operations::BoundNames};
use oxc_span::{CompactStr, GetSpan, Span};
#[allow(clippy::wildcard_imports)]
use oxc_syntax::module_record::*;

#[derive(Default)]
pub struct ModuleRecordBuilder {
    pub module_record: ModuleRecord,
    export_entries: Vec<ExportEntry>,
}

impl ModuleRecordBuilder {
    pub fn new(resolved_absolute_path: PathBuf) -> Self {
        Self { module_record: ModuleRecord::new(resolved_absolute_path), ..Self::default() }
    }

    pub fn visit(&mut self, program: &Program) {
        self.module_record.not_esm = true;
        // This avoids additional checks on TypeScript `TsModuleBlock` which
        // also has `ModuleDeclaration`s.
        for stmt in &program.body {
            if let Statement::ModuleDeclaration(module_decl) = stmt {
                self.module_record.not_esm = false;
                self.visit_module_declaration(module_decl);
            }
        }

        // The `ParseModule` algorithm requires `importedBoundNames` (import entries) to be
        // resolved before resolving export entries.
        self.resolve_export_entries();
    }

    pub fn build(self) -> ModuleRecord {
        self.module_record
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
        self.add_default_export(exported_name.span());

        let id = match &decl.declaration {
            ExportDefaultDeclarationKind::Expression(_) => None,
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => func.id.as_ref(),
            ExportDefaultDeclarationKind::ClassDeclaration(class) => class.id.as_ref(),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(_)
            | ExportDefaultDeclarationKind::TSEnumDeclaration(_) => return,
        };
        let export_entry = ExportEntry {
            export_name: ExportExportName::Default(exported_name.span()),
            local_name: id.as_ref().map_or_else(
                || ExportLocalName::Default(exported_name.span()),
                |ident| {
                    ExportLocalName::Name(NameSpan::new(ident.name.to_compact_str(), ident.span))
                },
            ),
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
