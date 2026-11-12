use oxc_allocator::{Allocator, ArenaHashMap, ArenaVec};
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::BoundNames;
use oxc_span::{GetSpan, Span};
use oxc_syntax::module_record::*;

use crate::diagnostics;

pub struct ModuleRecordBuilder<'a> {
    allocator: &'a Allocator,
    source_type: SourceType,
    module_record: ModuleRecord<'a>,
    export_entries: ArenaVec<'a, ExportEntry<'a>>,
    exported_bindings_duplicated: ArenaVec<'a, NameSpan<'a>>,
    /// `default` exports of declarations skipped because their module request is not UTF-8.
    /// They still count towards the duplicate default export check.
    skipped_default_exports: ArenaVec<'a, Span>,
}

impl<'a> ModuleRecordBuilder<'a> {
    pub fn new(allocator: &'a Allocator, source_type: SourceType) -> Self {
        Self {
            allocator,
            source_type,
            module_record: ModuleRecord::new(allocator),
            export_entries: ArenaVec::new_in(&allocator),
            exported_bindings_duplicated: ArenaVec::new_in(&allocator),
            skipped_default_exports: ArenaVec::new_in(&allocator),
        }
    }

    pub fn build(mut self) -> (ModuleRecord<'a>, Vec<OxcDiagnostic>) {
        // The `ParseModule` algorithm requires `importedBoundNames` (import entries) to be
        // resolved before resolving export entries.
        self.resolve_export_entries();
        let errors = self.errors();
        (self.module_record, errors)
    }

    /// Returns true if the file contains module syntax (import/export declarations or import.meta).
    pub fn has_module_syntax(&self) -> bool {
        self.module_record.has_module_syntax
    }

    pub fn errors(&self) -> Vec<OxcDiagnostic> {
        let mut errors = vec![];

        let module_record = &self.module_record;

        // Skip checking for exports in TypeScript
        if !self.source_type.is_typescript() {
            // It is a Syntax Error if the ExportedNames of ModuleItemList contains any duplicate entries.
            for name_span in &self.exported_bindings_duplicated {
                let old_span = module_record.exported_bindings[&name_span.name];
                errors.push(diagnostics::duplicate_export(
                    &name_span.name,
                    name_span.span,
                    old_span,
                ));
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
                .chain(self.skipped_default_exports.iter().copied());
            if default_exports.clone().count() > 1 {
                errors.push(diagnostics::duplicate_default_export(default_exports.collect()));
            }
        }

        errors
    }

    fn add_module_request(&mut self, name: Str<'a>, requested_module: RequestedModule) {
        self.module_record
            .requested_modules
            .entry(name)
            .or_insert_with(|| ArenaVec::new_in(&self.allocator))
            .push(requested_module);
    }

    fn add_import_entry(&mut self, entry: ImportEntry<'a>) {
        self.module_record.import_entries.push(entry);
    }

    fn add_export_entry(&mut self, entry: ExportEntry<'a>) {
        self.export_entries.push(entry);
    }

    fn add_export_binding(&mut self, name: Str<'a>, span: Span) {
        if let Some(old_node) = self.module_record.exported_bindings.insert(name, span) {
            self.exported_bindings_duplicated.push(NameSpan::new(name, old_node));
        }
    }

    /// [ParseModule](https://tc39.es/ecma262/#sec-parsemodule)
    /// Step 10.
    fn resolve_export_entries(&mut self) {
        let mut export_entries =
            std::mem::replace(&mut self.export_entries, ArenaVec::new_in(&self.allocator));
        let module_record = &mut self.module_record;

        if let Some(first) = export_entries.first() {
            let is_star = first.import_name.is_all_but_default();
            if export_entries.iter().all(|entry| {
                entry.module_request.is_some() && entry.import_name.is_all_but_default() == is_star
            }) {
                // Reuse the original buffer when every export has the same non-local destination.
                if is_star {
                    debug_assert!(export_entries.iter().all(|entry| entry.export_name.is_null()));
                    module_record.star_export_entries = export_entries;
                } else {
                    module_record.indirect_export_entries = export_entries;
                }
                return;
            }
        }

        if module_record.import_entries.len() >= 64
            && export_entries.len() >= 16
            && module_record.import_entries.len().saturating_mul(export_entries.len()) >= 4096
        {
            let local_exports =
                resolve_large_exports(self.allocator, module_record, export_entries);
            module_record.local_export_entries = local_exports;
            return;
        }

        // Retain local exports in their original buffer, avoiding a second arena allocation.
        // 10. For each ExportEntry Record ee of exportEntries, do
        export_entries.retain_mut(|entry| {
            // a. If ee.[[ModuleRequest]] is null, then
            if entry.module_request.is_none() {
                let ExportLocalName::Name(local_name) = &entry.local_name else {
                    // i. If ee.[[LocalName]] is not an element of importedBoundNames, then
                    // 1. Append ee to localExportEntries.
                    return true;
                };
                let Some(import) = module_record
                    .import_entries
                    .iter()
                    .find(|import| import.local_name.name == local_name.name)
                else {
                    // i. If ee.[[LocalName]] is not an element of importedBoundNames, then
                    // 1. Append ee to localExportEntries.
                    return true;
                };
                // ii. Else,
                // 1. Let ie be the element of importEntries whose [[LocalName]] is the same as ee.[[LocalName]].
                let import_name = match &import.import_name {
                    ImportImportName::Name(name) => ExportImportName::Name(name.clone()),
                    ImportImportName::Default(span) => ExportImportName::Name(NameSpan::new(
                        self.allocator.alloc_str("default").into(),
                        *span,
                    )),
                    // ParseModule 10.a.ii.3: normalize to `export * as name from 'mod'`.
                    ImportImportName::NamespaceObject => ExportImportName::All,
                };
                // Append the ExportEntry Record { [[ModuleRequest]]: ie.[[ModuleRequest]], [[ImportName]]: ie.[[ImportName]], [[LocalName]]: null, [[ExportName]]: ee.[[ExportName]] }
                // to indirectExportEntries.
                entry.statement_span = import.statement_span;
                entry.module_request = Some(import.module_request.clone());
                entry.import_name = import_name;
                entry.local_name = ExportLocalName::default();
                entry.is_type |= import.is_type;
            }

            // b. Else if ee.[[ImportName]] is all-but-default, then
            if entry.import_name.is_all_but_default() {
                // i. Assert: ee.[[ExportName]] is null.
                debug_assert!(entry.export_name.is_null());
                module_record.star_export_entries.push(std::mem::take(entry));
            } else {
                // c. Else,
                // i. Append ee to indirectExportEntries.
                module_record.indirect_export_entries.push(std::mem::take(entry));
            }
            false
        });
        module_record.local_export_entries = export_entries;
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

    pub fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        self.module_record.has_module_syntax = true;
        // Module records still require UTF-8 module specifiers. Keep other values in the AST only.
        let Some(source) = decl.source.value.as_str() else { return };
        let module_request = NameSpan::new(source.into(), decl.source.span);

        if let Some(specifiers) = &decl.specifiers {
            reserve_entries(&mut self.module_record.import_entries, specifiers.len());
            for specifier in specifiers {
                let local = specifier.local();
                let local_name = NameSpan::new(local.name.into(), local.span);
                let (import_name, is_type) = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => (
                        ImportImportName::Name(NameSpan::new(
                            specifier.imported.name(),
                            specifier.imported.span(),
                        )),
                        decl.import_kind.is_type() || specifier.import_kind.is_type(),
                    ),
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                        (ImportImportName::NamespaceObject, decl.import_kind.is_type())
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        (ImportImportName::Default(specifier.span), decl.import_kind.is_type())
                    }
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

    pub fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        self.module_record.has_module_syntax = true;
        let exported = decl.exported.as_ref().map(|name| NameSpan::new(name.name(), name.span()));
        // The duplicate-export check does not depend on the module request,
        // so the exported name is registered before the request is examined.
        if let Some(exported) = &exported {
            self.add_export_binding(exported.name, exported.span);
        }
        // Module records still require UTF-8 module specifiers. Keep other values in the AST only.
        let Some(source) = decl.source.value.as_str() else {
            if let Some(exported) = exported.filter(|exported| exported.name == "default") {
                self.skipped_default_exports.push(exported.span);
            }
            return;
        };
        let module_request = NameSpan::new(source.into(), decl.source.span);
        self.add_export_entry(ExportEntry {
            statement_span: decl.span,
            span: decl.span,
            module_request: Some(module_request.clone()),
            import_name: if exported.is_some() {
                ExportImportName::All
            } else {
                ExportImportName::AllButDefault
            },
            export_name: exported.map_or(ExportExportName::Null, ExportExportName::Name),
            local_name: ExportLocalName::default(),
            is_type: decl.export_kind.is_type(),
        });
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

    pub fn visit_export_default_declaration(
        &mut self,
        decl: &ExportDefaultDeclaration<'a>,
        default_keyword_span: Span,
    ) {
        let local_name = match &decl.declaration {
            ExportDefaultDeclarationKind::Identifier(ident) => {
                ExportLocalName::Default(NameSpan::new(ident.name.into(), ident.span))
            }
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                func.id.as_ref().map_or_else(
                    || ExportLocalName::Null,
                    |id| ExportLocalName::Name(NameSpan::new(id.name.into(), id.span)),
                )
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => class.id.as_ref().map_or_else(
                || ExportLocalName::Null,
                |id| ExportLocalName::Name(NameSpan::new(id.name.into(), id.span)),
            ),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(t) => {
                ExportLocalName::Name(NameSpan::new(t.id.name.into(), t.id.span))
            }
            _ => ExportLocalName::Null,
        };
        let export_entry = ExportEntry {
            statement_span: decl.span,
            span: decl.declaration.span(),
            module_request: None,
            import_name: ExportImportName::default(),
            export_name: ExportExportName::Default(default_keyword_span),
            local_name,
            is_type: decl.is_typescript_syntax(),
        };
        self.add_export_entry(export_entry);
        self.module_record.has_module_syntax = true;
    }

    pub fn visit_export_declaration(&mut self, decl: &ExportDeclaration<'a>) {
        let export_kind = decl.export_kind();
        let declaration = &decl.declaration;
        iter_binding_identifiers_of_declaration(declaration, &mut |ident| {
            let export_name = ExportExportName::Name(NameSpan::new(ident.name.into(), ident.span));
            let local_name = ExportLocalName::Name(NameSpan::new(ident.name.into(), ident.span));
            let export_entry = ExportEntry {
                statement_span: decl.span,
                span: declaration.span(),
                module_request: None,
                import_name: ExportImportName::Null,
                export_name,
                local_name,
                is_type: export_kind.is_type(),
            };
            self.add_export_entry(export_entry);
            self.add_export_binding(ident.name.into(), ident.span);
        });
        self.module_record.has_module_syntax = true;
    }

    pub fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        reserve_entries(&mut self.export_entries, decl.specifiers.len());
        for specifier in &decl.specifiers {
            let export_name = ExportExportName::Name(NameSpan::new(
                specifier.exported.name(),
                specifier.exported.span(),
            ));
            let export_entry = ExportEntry {
                statement_span: decl.span,
                span: specifier.span,
                module_request: None,
                import_name: ExportImportName::Null,
                export_name,
                local_name: ExportLocalName::Name(NameSpan::new(
                    specifier.local.name(),
                    specifier.local.span(),
                )),
                is_type: specifier.export_kind.is_type() || decl.export_kind.is_type(),
            };
            self.add_export_entry(export_entry);
            self.add_export_binding(specifier.exported.name(), specifier.exported.span());
        }
        self.module_record.has_module_syntax = true;
    }

    pub fn visit_export_from_declaration(&mut self, decl: &ExportFromDeclaration<'a>) {
        self.module_record.has_module_syntax = true;
        // The duplicate-export check does not depend on the module request,
        // so exported names are registered before the request is examined.
        for specifier in &decl.specifiers {
            self.add_export_binding(specifier.exported.name(), specifier.exported.span());
        }
        // Module records still require UTF-8 module specifiers. Keep other values in the AST only.
        let Some(source) = decl.source.value.as_str() else {
            self.skipped_default_exports.extend(
                decl.specifiers
                    .iter()
                    .filter(|specifier| specifier.exported.name() == "default")
                    .map(|specifier| specifier.exported.span()),
            );
            return;
        };
        let module_request = NameSpan::new(source.into(), decl.source.span);
        self.add_module_request(
            module_request.name,
            RequestedModule {
                statement_span: decl.span,
                span: module_request.span,
                is_type: decl.export_kind.is_type(),
                is_import: false,
            },
        );

        reserve_entries(&mut self.export_entries, decl.specifiers.len());
        for specifier in &decl.specifiers {
            let export_name = ExportExportName::Name(NameSpan::new(
                specifier.exported.name(),
                specifier.exported.span(),
            ));
            let import_name = ExportImportName::Name(NameSpan::new(
                specifier.local.name(),
                specifier.local.span(),
            ));
            let export_entry = ExportEntry {
                statement_span: decl.span,
                span: specifier.span,
                module_request: Some(module_request.clone()),
                import_name,
                export_name,
                local_name: ExportLocalName::Null,
                is_type: specifier.export_kind.is_type() || decl.export_kind.is_type(),
            };
            self.add_export_entry(export_entry);
        }
    }

    pub fn set_module_syntax(&mut self) {
        self.module_record.has_module_syntax = true;
    }
}

fn reserve_entries<T>(entries: &mut ArenaVec<'_, T>, additional: usize) {
    let len = entries.len();
    let required = len + additional;
    if required > entries.capacity() {
        // Match normal push growth so reserving a batch does not change later capacities.
        let capacity = required.next_power_of_two();
        entries.reserve(capacity - len);
    }
}

fn iter_binding_identifiers_of_declaration<'a, F>(decl: &Declaration<'a>, f: &mut F)
where
    F: FnMut(&BindingIdentifier<'a>),
{
    if let Declaration::VariableDeclaration(decl) = decl {
        decl.bound_names(f);
    } else if let Some(ident) = decl.id() {
        f(ident);
    }
}

// Keep the adaptive machinery out of the ordinary module-resolution loop.
#[inline(never)]
fn resolve_large_exports<'a>(
    allocator: &'a Allocator,
    module_record: &mut ModuleRecord<'a>,
    mut exports: ArenaVec<'a, ExportEntry<'a>>,
) -> ArenaVec<'a, ExportEntry<'a>> {
    let imports = &module_record.import_entries;
    let first = &imports[0];
    let first_name = first.local_name.name;
    let mut index = None;
    let mut can_index = true;
    let mut searched = 0usize;
    let budget = imports.len().saturating_sub(1).saturating_mul(8);
    let mut retained = 0;
    for read in 0..exports.len() {
        let entry = &exports[read];
        let name = if entry.module_request.is_none() {
            match &entry.local_name {
                ExportLocalName::Name(name) => Some(name.name),
                _ => None,
            }
        } else {
            None
        };
        let import = if let Some(name) = name {
            if name == first_name {
                Some(first)
            } else {
                if can_index && searched >= budget && exports.len() - read >= 8 {
                    let queries_left = exports[read..].iter().filter(|entry| {
                        entry.module_request.is_none()
                            && matches!(&entry.local_name, ExportLocalName::Name(name) if name.name != first_name)
                    }).count();
                    if queries_left >= 8 {
                        index = Some(index_pending_exports(
                            allocator,
                            imports,
                            &exports[read..],
                            queries_left,
                        ));
                    }
                    can_index = false;
                }
                if let Some(index) = &index {
                    find_indexed_import(index, imports, name)
                } else {
                    let position = imports[1..]
                        .iter()
                        .position(|import| import.local_name.name == name)
                        .map(|position| position + 1);
                    let work = position.unwrap_or(imports.len());
                    if work > 2 {
                        searched = searched.saturating_add(work);
                    }
                    position.map(|position| &imports[position])
                }
            }
        } else {
            None
        };
        let entry = &mut exports[read];
        if entry.module_request.is_none() {
            let Some(import) = import else {
                // The unprocessed suffix remains available for indexing; the prefix preserves order.
                if retained != read {
                    exports.swap(retained, read);
                }
                retained += 1;
                continue;
            };
            entry.statement_span = import.statement_span;
            entry.module_request = Some(import.module_request.clone());
            entry.import_name = match &import.import_name {
                ImportImportName::Name(name) => ExportImportName::Name(name.clone()),
                ImportImportName::Default(span) => ExportImportName::Name(NameSpan::new(
                    allocator.alloc_str("default").into(),
                    *span,
                )),
                ImportImportName::NamespaceObject => ExportImportName::All,
            };
            entry.local_name = ExportLocalName::default();
            entry.is_type |= import.is_type;
        }
        if entry.import_name.is_all_but_default() {
            debug_assert!(entry.export_name.is_null());
            module_record.star_export_entries.push(std::mem::take(entry));
        } else {
            module_record.indirect_export_entries.push(std::mem::take(entry));
        }
    }
    exports.truncate(retained);
    exports
}

#[inline(never)]
fn index_pending_exports<'a>(
    allocator: &'a Allocator,
    imports: &[ImportEntry<'a>],
    exports: &[ExportEntry<'a>],
    queries_left: usize,
) -> ArenaHashMap<'a, Str<'a>, usize> {
    let mut index = ArenaHashMap::with_capacity_in(queries_left.min(imports.len()), allocator);
    if imports.len() < queries_left {
        for (position, import) in imports.iter().enumerate() {
            index.entry(import.local_name.name).or_insert(position);
        }
        return index;
    }
    for entry in exports {
        if entry.module_request.is_none()
            && let ExportLocalName::Name(name) = &entry.local_name
            && name.name != imports[0].local_name.name
        {
            index.entry(name.name).or_insert(usize::MAX);
        }
    }
    for (position, import) in imports.iter().enumerate() {
        if let Some(slot) = index.get_mut(&import.local_name.name)
            && *slot == usize::MAX
        {
            *slot = position;
        }
    }
    index
}

#[inline(never)]
fn find_indexed_import<'a, 'b>(
    index: &ArenaHashMap<'a, Str<'a>, usize>,
    imports: &'b [ImportEntry<'a>],
    name: Str<'a>,
) -> Option<&'b ImportEntry<'a>> {
    index.get(&name).and_then(|&position| imports.get(position))
}

#[cfg(test)]
mod module_record_tests {
    use oxc_allocator::Allocator;
    use oxc_span::{SourceType, Span};
    use oxc_syntax::module_record::*;

    use crate::Parser;

    fn build<'a>(allocator: &'a Allocator, source_text: &'a str) -> ModuleRecord<'a> {
        let source_type = SourceType::ts();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        assert!(ret.diagnostics.is_empty());
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
        let module_record = build(&allocator, "export { x }; var x");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 13),
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
        let module_record = build(&allocator, "export { x as v }; var x");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 18),
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
        let module_record = build(&allocator, "export { default } from 'mod'");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 29),
            module_request: Some(NameSpan::new("mod".into(), Span::new(24, 29))),
            span: Span::new(9, 16),
            export_name: ExportExportName::Name(NameSpan::new("default".into(), Span::new(9, 16))),
            import_name: ExportImportName::Name(NameSpan::new("default".into(), Span::new(9, 16))),
            local_name: ExportLocalName::Null,
            ..ExportEntry::default()
        };
        assert_eq!(module_record.indirect_export_entries.len(), 1);
        assert_eq!(module_record.indirect_export_entries[0], export_entry);
    }

    #[test]
    fn reexport_imported_default() {
        for (source, exported) in [
            ("import d from 'mod'; export { d };", "d"),
            ("export { d as renamed }; import d from 'mod';", "renamed"),
            ("import d from 'mod'; export { d as default };", "default"),
        ] {
            let allocator = Allocator::default();
            let ret = Parser::new(&allocator, source, SourceType::mjs()).parse();
            assert!(ret.diagnostics.is_empty());
            assert!(ret.module_record.local_export_entries.is_empty());
            assert_eq!(ret.module_record.indirect_export_entries.len(), 1);
            let entry = &ret.module_record.indirect_export_entries[0];
            let import_start = u32::try_from(source.find("import d").unwrap()).unwrap() + 7;
            assert_eq!(
                entry.import_name,
                ExportImportName::Name(NameSpan::new(
                    "default".into(),
                    Span::sized(import_start, 1),
                ))
            );
            let ExportExportName::Name(name) = &entry.export_name else { panic!() };
            assert_eq!(name.name, exported);
            assert_eq!(entry.module_request.as_ref().unwrap().name, "mod");
            assert!(matches!(entry.local_name, ExportLocalName::Null));
        }
    }

    #[test]
    fn reexported_import_preserves_type_only_flags() {
        let allocator = Allocator::default();
        for (source, is_type) in [
            ("import { T } from 'mod'; export { T };", false),
            ("import { T } from 'mod'; export type { T };", true),
            ("import { T } from 'mod'; export { type T };", true),
            ("import type { T } from 'mod'; export { T };", true),
            ("import { type T } from 'mod'; export { T };", true),
            ("import type { T } from 'mod'; export type { T };", true),
            ("import T from 'mod'; export type { T as default };", true),
        ] {
            let record = build(&allocator, source);
            assert!(record.local_export_entries.is_empty(), "{source}");
            assert_eq!(record.indirect_export_entries.len(), 1, "{source}");
            assert_eq!(record.indirect_export_entries[0].is_type, is_type, "{source}");
        }
    }

    #[test]
    fn reexported_namespace_is_indirect() {
        let allocator = Allocator::default();
        for (source, exported, is_type) in [
            ("import * as ns from 'mod'; export { ns };", "ns", false),
            ("export { ns as renamed }; import * as ns from 'mod';", "renamed", false),
            ("import * as ns from 'mod'; export { ns as default };", "default", false),
            ("import * as ns from 'mod'; export type { ns };", "ns", true),
            ("import * as ns from 'mod'; export { type ns };", "ns", true),
            ("import type * as ns from 'mod'; export { ns };", "ns", true),
        ] {
            let record = build(&allocator, source);
            assert!(record.local_export_entries.is_empty(), "{source}");
            assert_eq!(record.indirect_export_entries.len(), 1, "{source}");
            let import = &record.import_entries[0];
            let entry = &record.indirect_export_entries[0];
            assert_eq!(entry.statement_span, import.statement_span);
            assert_eq!(entry.module_request.as_ref(), Some(&import.module_request));
            assert_eq!(entry.import_name, ExportImportName::All);
            assert_eq!(entry.local_name, ExportLocalName::Null);
            let ExportExportName::Name(name) = &entry.export_name else { panic!() };
            assert_eq!(name.name, exported);
            assert_eq!(entry.is_type, is_type);
        }

        // A default export expression creates a local binding, unlike `export { ns as default }`.
        let record = build(&allocator, "import * as ns from 'mod'; export default ns;");
        assert_eq!(record.local_export_entries.len(), 1);
        assert!(record.indirect_export_entries.is_empty());
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
    fn mixed_export_entries_preserve_source_order() {
        let allocator = Allocator::default();
        let source = "
            export const first = 1;
            export * from 'star1';
            export { imported as indirect1 };
            export const second = 2;
            export { value as indirect2 } from 'other';
            export * from 'star2';
            export { ns };
            export const third = 3;
            import { imported } from 'mod';
            import * as ns from 'namespace';
        ";
        let ret = Parser::new(&allocator, source, SourceType::mjs()).parse();
        assert!(ret.diagnostics.is_empty());
        let module_record = ret.module_record;
        assert_eq!(
            module_record
                .local_export_entries
                .iter()
                .map(|entry| entry.local_name.name().unwrap())
                .collect::<Vec<_>>(),
            ["first", "second", "third"]
        );
        assert_eq!(
            module_record
                .indirect_export_entries
                .iter()
                .map(|entry| {
                    let ExportExportName::Name(name) = &entry.export_name else { panic!() };
                    name.name
                })
                .collect::<Vec<_>>(),
            ["indirect1", "indirect2", "ns"]
        );
        assert_eq!(
            module_record
                .star_export_entries
                .iter()
                .map(|entry| entry.module_request.as_ref().unwrap().name)
                .collect::<Vec<_>>(),
            ["star1", "star2"]
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

    #[test]
    fn export_default_interface() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export default interface Foo {}");
        assert_eq!(module_record.local_export_entries.len(), 1);
        let entry = &module_record.local_export_entries[0];
        assert_eq!(entry.local_name.name().unwrap().as_str(), "Foo");
        assert!(entry.export_name.is_default());
        assert!(entry.is_type);
    }
}
