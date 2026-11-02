use oxc_allocator::{ArenaHashMap, ArenaVec, CloneIn, GetAllocator};
use oxc_ast::ast::*;
use oxc_compat::ESFeature;
use oxc_span::ContentEq;
use oxc_str::JSStr;
use oxc_syntax::symbol::SymbolId;

use crate::TraverseCtx;

use crate::peephole::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// Check whether two import declarations can be merged.
    fn can_merge_imports(first: &ImportDeclaration<'a>, second: &ImportDeclaration<'a>) -> bool {
        if first.source.value != second.source.value
            || first.phase != second.phase
            || first.phase == Some(ImportPhase::Source)
            || first.import_kind != second.import_kind
            || first.import_kind.is_type()
            || first.with_clause.content_ne(&second.with_clause)
        {
            return false;
        }

        // Additional default specifiers can be represented as named imports of
        // `default`. Namespace and named specifiers cannot coexist.
        let mut default_count = 0;
        let mut has_namespace = false;
        let mut has_named = false;
        for specifier in first
            .specifiers
            .iter()
            .flat_map(|specifiers| specifiers.iter())
            .chain(second.specifiers.iter().flat_map(|specifiers| specifiers.iter()))
        {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(_) => has_named = true,
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => default_count += 1,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                    if has_namespace {
                        return false;
                    }
                    has_namespace = true;
                }
            }
        }
        !(has_namespace && (has_named || default_count > 1))
    }

    /// Convert a default import into a named import of the module's `default`
    /// export. This allows multiple default bindings in one import clause.
    fn default_to_named_import(
        default_specifier: ImportDeclarationSpecifier<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> ImportDeclarationSpecifier<'a> {
        let ImportDeclarationSpecifier::ImportDefaultSpecifier(default_specifier) =
            default_specifier
        else {
            unreachable!();
        };
        let ImportDefaultSpecifier { span, local, .. } = default_specifier.unbox();
        ImportDeclarationSpecifier::new_import_specifier(
            span,
            ModuleExportName::IdentifierName(IdentifierName::new(span, "default", ctx)),
            local,
            ImportOrExportKind::Value,
            ctx,
        )
    }

    /// Merge import declarations that load the same module request.
    ///
    /// ```js
    /// import { foo } from "module";
    /// import { bar } from "other";
    /// import { baz } from "module";
    /// // becomes
    /// import { foo, baz } from "module";
    /// import { bar } from "other";
    /// ```
    pub fn merge_imports(stmts: &mut ArenaVec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        let mut import_cache: ArenaHashMap<'a, JSStr<'a>, ArenaVec<'a, usize>> =
            ArenaHashMap::new_in(ctx.allocator());
        let mut merges = ArenaVec::new_in(ctx);

        // Merge into the target immediately so later declarations are checked
        // against the combined import clause. Keep every unmerged declaration
        // in a source group because import attributes, phases, and type/value
        // mode may differ between declarations with the same source string.
        for statement_index in 0..stmts.len() {
            let Statement::ImportDeclaration(import_decl) = stmts.get(statement_index).unwrap()
            else {
                continue;
            };
            let candidates = import_cache
                .entry(import_decl.source.value)
                .or_insert_with(|| ArenaVec::new_in(ctx));
            let target_index = candidates.iter().find_map(|&target_index| {
                let Some(Statement::ImportDeclaration(target)) = stmts.get(target_index) else {
                    return None;
                };
                Self::can_merge_imports(target, import_decl).then_some(target_index)
            });

            let Some(target_index) = target_index else {
                candidates.push(statement_index);
                continue;
            };

            let Statement::ImportDeclaration(source_import) =
                stmts.get_mut(statement_index).unwrap()
            else {
                unreachable!();
            };
            let Some(mut source_specifiers) = source_import.specifiers.take() else {
                merges.push(statement_index);
                continue;
            };

            let Statement::ImportDeclaration(target_import) = stmts.get_mut(target_index).unwrap()
            else {
                unreachable!();
            };
            // default must be first specifier in import
            let is_default_specifier = |s: &ImportDeclarationSpecifier| {
                matches!(s, ImportDeclarationSpecifier::ImportDefaultSpecifier(_))
            };
            if let Some(target_specifiers) = &mut target_import.specifiers {
                if source_specifiers.first().is_some_and(is_default_specifier) {
                    let default_specifier = source_specifiers.remove(0);
                    if target_specifiers.first().is_some_and(is_default_specifier) {
                        target_specifiers
                            .push(Self::default_to_named_import(default_specifier, ctx));
                    } else {
                        target_specifiers.insert(0, default_specifier);
                    }
                }
                target_specifiers.append(&mut source_specifiers);
            } else {
                target_import.specifiers = Some(source_specifiers);
            }
            merges.push(statement_index);
        }

        // Remove imports in reverse order so removing a later import does not
        // shift the indices of any merge that is still waiting to be applied.
        for &source_index in merges.iter().rev() {
            let source_stmt = stmts.remove(source_index);
            ctx.drop_statement(&source_stmt);
        }
    }

    /// Check whether an import can be merged with a local named export.
    fn can_merge_import_export(
        import_decl: &ImportDeclaration<'a>,
        export_decl: &ExportNamedDeclaration<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        let Some(import_specifiers) = import_decl.specifiers.as_ref() else { return false };
        let is_namespace_import = matches!(
            import_specifiers.as_slice(),
            [ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)]
        );

        if import_decl.phase.is_some()
            || import_decl.import_kind.is_type()
            || export_decl.export_kind.is_type()
            || import_specifiers.is_empty()
            || import_specifiers.len() > export_decl.specifiers.len()
            || (is_namespace_import && !ctx.supports_feature(ESFeature::ES2020ExportNamespaceFrom))
            || (is_namespace_import && export_decl.specifiers.len() != 1)
            || ctx.scoping().root_scope_flags().contains_direct_eval()
            || (!is_namespace_import
                && !import_specifiers.iter().all(|specifier| match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        !specifier.import_kind.is_type()
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => true,
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => false,
                }))
        {
            return false;
        }

        // Every local export must refer to one of the imported bindings, and
        // every imported binding must be used exactly by this export.
        for import_specifier in import_specifiers {
            let symbol_id = import_specifier.symbol_id();
            let export_count = export_decl
                .specifiers
                .iter()
                .filter(|export_specifier| {
                    let ModuleExportName::IdentifierReference(id) = &export_specifier.local else {
                        return false;
                    };
                    ctx.scoping().get_reference(id.reference_id()).symbol_id() == Some(symbol_id)
                })
                .count();
            if export_count == 0
                || ctx.scoping().get_resolved_reference_ids(symbol_id).len() != export_count
            {
                return false;
            }
        }

        export_decl.specifiers.iter().all(|export_specifier| {
            if export_specifier.export_kind.is_type() {
                return false;
            }
            let ModuleExportName::IdentifierReference(id) = &export_specifier.local else {
                return false;
            };
            let Some(symbol_id) = ctx.scoping().get_reference(id.reference_id()).symbol_id() else {
                return false;
            };
            import_specifiers
                .iter()
                .any(|import_specifier| import_specifier.symbol_id() == symbol_id)
        })
    }

    /// Merge an import and a local named export into a direct
    /// re-export.
    ///
    /// ```js
    /// import { foo as bar } from "module";
    /// export { bar as baz };
    /// // becomes
    /// export { foo as baz } from "module";
    ///
    /// import * as namespace from "module";
    /// export { namespace };
    /// // becomes
    /// export * as namespace from "module";
    ///
    /// import defaultExport from "module";
    /// export { defaultExport };
    /// // becomes
    /// export { default as defaultExport } from "module";
    /// ```
    ///
    /// The import declaration can only be removed when all of its bindings are
    /// represented by the export and have no other references. This also keeps
    /// direct `eval` from observing a binding that is removed here.
    pub fn merge_import_export(stmts: &mut ArenaVec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        let mut import_export_cache: ArenaHashMap<'a, SymbolId, usize> =
            ArenaHashMap::new_in(ctx.allocator());
        let mut merges = ArenaVec::new_in(ctx);

        for (statement_index, stmt) in stmts.iter().enumerate() {
            match stmt {
                Statement::ImportDeclaration(import_decl) => {
                    if let Some(specifiers) = import_decl.specifiers.as_ref() {
                        import_export_cache.extend(
                            specifiers
                                .iter()
                                .map(|specifier| (specifier.symbol_id(), statement_index)),
                        );
                    }
                }
                Statement::ExportNamedDeclaration(export_decl) => {
                    let Some(import_index) = export_decl.specifiers.iter().find_map(|spec| {
                        let ModuleExportName::IdentifierReference(id) = &spec.local else {
                            return None;
                        };
                        let Some(symbol_id) =
                            ctx.scoping().get_reference(id.reference_id()).symbol_id()
                        else {
                            return None;
                        };
                        import_export_cache.get(&symbol_id).copied()
                    }) else {
                        continue;
                    };
                    let Some(Statement::ImportDeclaration(import_decl)) = stmts.get(import_index)
                    else {
                        continue;
                    };
                    if !Self::can_merge_import_export(import_decl, export_decl, ctx) {
                        continue;
                    }

                    merges.push((import_index, statement_index));
                    if let Some(specifiers) = import_decl.specifiers.as_ref() {
                        for specifier in specifiers {
                            let symbol_id = specifier.symbol_id();
                            if import_export_cache.get(&symbol_id) == Some(&import_index) {
                                import_export_cache.remove(&symbol_id);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Apply in reverse order so removing an export does not shift the
        // indices of any merge that is still waiting to be applied.
        for &(import_index, export_index) in merges.iter().rev() {
            // The old export contains references to the local import binding.
            // Mark those references before replacing them with the imported
            // module name.
            let old_export = stmts.remove(export_index);
            ctx.drop_statement(&old_export);
            let Statement::ExportNamedDeclaration(mut export_decl) = old_export else {
                unreachable!();
            };
            ctx.replace_statement_with(stmts.get_mut(import_index).unwrap(), |old_import, ctx| {
                ctx.drop_statement(&old_import);

                let Statement::ImportDeclaration(import_decl) = old_import else {
                    unreachable!();
                };
                let ImportDeclaration {
                    specifiers: Some(mut import_specifiers),
                    source,
                    with_clause,
                    ..
                } = import_decl.unbox()
                else {
                    unreachable!();
                };

                let is_namespace_import = matches!(
                    import_specifiers.as_slice(),
                    [ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)]
                );
                if is_namespace_import {
                    let exported = export_decl.specifiers.pop().unwrap().exported;
                    return Statement::new_export_all_declaration(
                        export_decl.span,
                        Some(exported),
                        source,
                        with_clause,
                        export_decl.export_kind,
                        ctx,
                    );
                }

                for export_specifier in &mut export_decl.specifiers {
                    let ModuleExportName::IdentifierReference(id) = &export_specifier.local else {
                        unreachable!();
                    };
                    let symbol_id =
                        ctx.scoping().get_reference(id.reference_id()).symbol_id().unwrap();
                    let import_specifier = import_specifiers
                        .iter_mut()
                        .find(|import_specifier| import_specifier.symbol_id() == symbol_id)
                        .unwrap();
                    export_specifier.local = match import_specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(import_specifier) => {
                            import_specifier.imported.clone_in(ctx.allocator())
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(import_default) => {
                            ModuleExportName::IdentifierName(IdentifierName::new(
                                import_default.span,
                                "default",
                                ctx,
                            ))
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => unreachable!(),
                    };
                }

                let export_decl = export_decl.unbox();
                Statement::new_export_from_declaration(
                    export_decl.span,
                    export_decl.specifiers,
                    source,
                    export_decl.export_kind,
                    with_clause,
                    ctx,
                )
            });
        }
    }
}
