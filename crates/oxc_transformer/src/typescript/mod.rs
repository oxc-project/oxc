use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, SPAN};

use std::{cell::RefCell, rc::Rc};

use crate::context::TransformerCtx;

/// Transform TypeScript
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-typescript>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-typescript>
/// * <https://www.typescriptlang.org/tsconfig#verbatimModuleSyntax>
pub struct TypeScript<'a> {
    ast: Rc<AstBuilder<'a>>,
    ctx: Rc<RefCell<TransformerCtx<'a>>>,
    verbatim_module_syntax: bool,
}

impl<'a> TypeScript<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        ctx: Rc<RefCell<TransformerCtx<'a>>>,
        verbatim_module_syntax: bool,
    ) -> Self {
        Self { ast, ctx, verbatim_module_syntax }
    }

    #[allow(clippy::unused_self)]
    pub fn transform_formal_parameters(&self, params: &mut FormalParameters<'a>) {
        if params.items.get(0).is_some_and(|param| matches!(&param.pattern.kind, BindingPatternKind::BindingIdentifier(ident) if ident.name =="this")) {
            params.items.remove(0);
        }
    }

    /// * Remove the top level import / export statements that are types
    /// * Adds `export {}` if all import / export statements are removed, this is used to tell
    /// downstream tools that this file is in ESM.
    pub fn transform_program(&self, program: &mut Program<'a>) {
        let mut needs_explicit_esm = false;

        for stmt in program.body.iter_mut() {
            if let Statement::ModuleDeclaration(module_decl) = stmt {
                needs_explicit_esm = true;
                match &mut **module_decl {
                    ModuleDeclaration::ExportNamedDeclaration(decl) => {
                        decl.specifiers.retain(|specifier| specifier.export_kind.is_value());
                    }
                    ModuleDeclaration::ImportDeclaration(decl) if decl.import_kind.is_value() => {
                        if let Some(specifiers) = &mut decl.specifiers {
                            specifiers.retain(|specifier| match specifier {
                                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                    if s.import_kind.is_type() {
                                        return false;
                                    }

                                    if self.verbatim_module_syntax {
                                        return true;
                                    }

                                    self.has_value_references(&s.local.name)
                                }
                                ImportDeclarationSpecifier::ImportDefaultSpecifier(s)
                                    if !self.verbatim_module_syntax =>
                                {
                                    self.has_value_references(&s.local.name)
                                }
                                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s)
                                    if !self.verbatim_module_syntax =>
                                {
                                    self.has_value_references(&s.local.name)
                                }
                                _ => true,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        program.body.retain(|stmt| match stmt {
            Statement::ModuleDeclaration(module_decl) => match &**module_decl {
                ModuleDeclaration::ImportDeclaration(decl) => {
                    if decl.import_kind.is_type() {
                        return false;
                    }

                    if self.verbatim_module_syntax {
                        return true;
                    }

                    !decl.specifiers.as_ref().is_some_and(|specifiers| specifiers.is_empty())
                }
                ModuleDeclaration::ExportNamedDeclaration(decl) => {
                    if decl.export_kind.is_type() {
                        return false;
                    }

                    if self.verbatim_module_syntax {
                        return true;
                    }

                    if decl.declaration.is_none() && decl.specifiers.is_empty() {
                        return false;
                    }

                    true
                }
                _ => true,
            },
            _ => true,
        });

        if needs_explicit_esm
            && !program.body.iter().any(|s| matches!(s, Statement::ModuleDeclaration(_)))
        {
            let empty_export = self.ast.export_named_declaration(
                SPAN,
                None,
                self.ast.new_vec(),
                None,
                ImportOrExportKind::Value,
            );
            let export_decl = ModuleDeclaration::ExportNamedDeclaration(empty_export);
            program.body.push(self.ast.module_declaration(export_decl));
        }
    }

    fn has_value_references(&self, name: &Atom) -> bool {
        let root_scope_id = self.ctx.borrow().scopes().root_scope_id();

        self.ctx
            .borrow()
            .scopes()
            .get_binding(root_scope_id, name)
            .map(|symbol_id| {
                self.ctx
                    .borrow()
                    .symbols()
                    .get_resolved_references(symbol_id)
                    .any(|x| x.is_read() || x.is_write())
            })
            .unwrap_or_default()
    }
}
