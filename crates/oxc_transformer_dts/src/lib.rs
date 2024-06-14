//! DTS Transformer / Transpiler
//!
//! References:
//! * <https://devblogs.microsoft.com/typescript/announcing-typescript-5-5-rc/#isolated-declarations>
//! * <https://www.typescriptlang.org/tsconfig#isolatedDeclarations>
//! * <https://github.com/microsoft/TypeScript/blob/main/src/compiler/transformers/declarations.ts>

mod class;
mod context;
mod declaration;
mod function;
mod inferrer;
mod module;
mod return_type;
mod scope;
mod types;

use std::{collections::VecDeque, path::Path, rc::Rc};

use context::{Ctx, TransformDtsCtx};
use oxc_allocator::Allocator;
use oxc_ast::Trivias;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, Visit};
use oxc_codegen::{Codegen, CodegenOptions, Context, Gen};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::SPAN;
use scope::ScopeTree;

pub struct TransformerDtsReturn {
    pub source_text: String,
    pub errors: Vec<OxcDiagnostic>,
}

pub struct TransformerDts<'a> {
    ctx: Ctx<'a>,
    codegen: Codegen<'a, false>,
    scope: ScopeTree<'a>,
}

impl<'a> TransformerDts<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_path: &Path,
        source_text: &'a str,
        trivias: Trivias,
    ) -> Self {
        let codegen = Codegen::new(
            &source_path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default(),
            source_text,
            trivias,
            CodegenOptions::default(),
        );

        let ctx = Rc::new(TransformDtsCtx::new(allocator));

        Self { ctx, codegen, scope: ScopeTree::new() }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(mut self, program: &Program<'a>) -> TransformerDtsReturn {
        let has_import_or_export = program.body.iter().any(|stmt| {
            matches!(
                stmt,
                Statement::ImportDeclaration(_)
                    | Statement::ExportAllDeclaration(_)
                    | Statement::ExportDefaultDeclaration(_)
                    | Statement::ExportNamedDeclaration(_)
            )
        });

        if has_import_or_export {
            self.transform_program(program);
        } else {
            self.transform_program_without_module_declaration(program);
        }

        TransformerDtsReturn {
            source_text: self.codegen.into_source_text(),
            errors: self.ctx.take_errors(),
        }
    }

    pub fn modifiers_declare(&self) -> Modifiers<'a> {
        Modifiers::new(
            self.ctx.ast.new_vec_single(Modifier { span: SPAN, kind: ModifierKind::Declare }),
        )
    }
}

impl<'a> TransformerDts<'a> {
    pub fn transform_program_without_module_declaration(&mut self, program: &Program<'a>) {
        program.body.iter().for_each(|stmt| {
            if let Some(decl) = stmt.as_declaration() {
                if let Some(decl) = self.transform_declaration(decl, false) {
                    decl.gen(&mut self.codegen, Context::empty());
                } else {
                    decl.gen(&mut self.codegen, Context::empty());
                }
            }
        });
    }

    pub fn transform_program(&mut self, program: &Program<'a>) {
        let mut new_stmts = Vec::new();
        let mut variables_declarations = VecDeque::new();
        let mut variable_transformed_indexes = VecDeque::new();
        let mut transformed_indexes = Vec::new();
        // 1. Collect all declarations, module declarations
        // 2. Transform export declarations
        // 3. Collect all bindings / reference from module declarations
        // 4. Collect transformed indexes
        program.body.iter().for_each(|stmt| match stmt {
            match_declaration!(Statement) => {
                match stmt.to_declaration() {
                    Declaration::VariableDeclaration(decl) => {
                        variables_declarations.push_back(
                            self.ctx.ast.copy(&decl.declarations).into_iter().collect::<Vec<_>>(),
                        );
                        variable_transformed_indexes.push_back(Vec::default());
                    }
                    Declaration::UsingDeclaration(decl) => {
                        variables_declarations.push_back(
                            self.ctx.ast.copy(&decl.declarations).into_iter().collect::<Vec<_>>(),
                        );
                        variable_transformed_indexes.push_back(Vec::default());
                    }
                    _ => {}
                }
                new_stmts.push(self.ctx.ast.copy(stmt));
            }
            match_module_declaration!(Statement) => {
                transformed_indexes.push(new_stmts.len());
                match stmt.to_module_declaration() {
                    ModuleDeclaration::ExportDefaultDeclaration(decl) => {
                        if let Some((var_decl, new_decl)) =
                            self.transform_export_default_declaration(decl)
                        {
                            if let Some(var_decl) = var_decl {
                                self.scope.visit_variable_declaration(&var_decl);
                                new_stmts.push(Statement::VariableDeclaration(
                                    self.ctx.ast.alloc(var_decl),
                                ));
                                transformed_indexes.push(new_stmts.len());
                            }

                            self.scope.visit_export_default_declaration(&new_decl);
                            new_stmts.push(Statement::ExportDefaultDeclaration(
                                self.ctx.ast.alloc(new_decl),
                            ));
                            return;
                        }

                        self.scope.visit_export_default_declaration(decl);
                    }
                    ModuleDeclaration::ExportNamedDeclaration(decl) => {
                        if let Some(new_decl) = self.transform_export_named_declaration(decl) {
                            self.scope.visit_declaration(
                                new_decl.declaration.as_ref().unwrap_or_else(|| unreachable!()),
                            );

                            new_stmts.push(Statement::ExportNamedDeclaration(
                                self.ctx.ast.alloc(new_decl),
                            ));
                            return;
                        }

                        self.scope.visit_export_named_declaration(decl);
                    }
                    module_declaration => self.scope.visit_module_declaration(module_declaration),
                }

                new_stmts.push(self.ctx.ast.copy(stmt));
            }
            _ => {}
        });

        // 5. Transform statements until no more transformation can be done
        let mut last_reference_len = 0;
        while last_reference_len != self.scope.references_len() {
            last_reference_len = self.scope.references_len();

            let mut variables_declarations_iter = variables_declarations.iter_mut();
            let mut variable_transformed_indexes_iter = variable_transformed_indexes.iter_mut();

            (0..new_stmts.len()).for_each(|i| {
                if transformed_indexes.contains(&i) {
                    return;
                }
                let Some(decl) = new_stmts[i].as_declaration() else {
                    return;
                };

                if let Declaration::VariableDeclaration(_) | Declaration::UsingDeclaration(_) = decl
                {
                    let Some(cur_variable_declarations) = variables_declarations_iter.next() else {
                        unreachable!()
                    };
                    let Some(cur_transformed_indexes) = variable_transformed_indexes_iter.next()
                    else {
                        unreachable!()
                    };

                    (0..cur_variable_declarations.len()).for_each(|ii| {
                        if cur_transformed_indexes.contains(&ii) {
                            return;
                        }

                        if let Some(decl) =
                            self.transform_variable_declarator(&cur_variable_declarations[ii], true)
                        {
                            self.scope.visit_variable_declarator(&decl);
                            cur_transformed_indexes.push(ii);
                            cur_variable_declarations[ii] = decl;
                        }
                    });
                } else if let Some(decl) = self.transform_declaration(decl, true) {
                    self.scope.visit_declaration(&decl);
                    transformed_indexes.push(i);
                    new_stmts[i] = Statement::from(decl);
                }
            });
        }

        // 6. Transform variable/using declarations, import statements, remove unused imports
        // 7. Generate code
        for (index, stmt) in new_stmts.iter().enumerate() {
            match stmt {
                _ if transformed_indexes.contains(&index) => {
                    stmt.gen(&mut self.codegen, Context::empty());
                }
                Statement::VariableDeclaration(decl) => {
                    let indexes =
                        variable_transformed_indexes.pop_front().unwrap_or_else(|| unreachable!());
                    let declarations =
                        variables_declarations.pop_front().unwrap_or_else(|| unreachable!());

                    if !indexes.is_empty() {
                        self.transform_variable_declaration_with_new_declarations(
                            decl,
                            self.ctx.ast.new_vec_from_iter(
                                declarations
                                    .into_iter()
                                    .enumerate()
                                    .filter(|(i, _)| indexes.contains(i))
                                    .map(|(_, decl)| decl),
                            ),
                        )
                        .gen(&mut self.codegen, Context::empty());
                    }
                }
                Statement::UsingDeclaration(decl) => {
                    let indexes =
                        variable_transformed_indexes.pop_front().unwrap_or_else(|| unreachable!());
                    let declarations =
                        variables_declarations.pop_front().unwrap_or_else(|| unreachable!());

                    if !indexes.is_empty() {
                        self.transform_using_declaration_with_new_declarations(
                            decl,
                            self.ctx.ast.new_vec_from_iter(
                                declarations
                                    .into_iter()
                                    .enumerate()
                                    .filter(|(i, _)| indexes.contains(i))
                                    .map(|(_, decl)| decl),
                            ),
                        )
                        .gen(&mut self.codegen, Context::empty());
                    }
                }
                Statement::ImportDeclaration(decl) => {
                    // We must transform this in the end, because we need to know all references
                    if decl.specifiers.is_none() {
                        decl.gen(&mut self.codegen, Context::empty());
                    } else if let Some(decl) = self.transform_import_declaration(decl) {
                        decl.gen(&mut self.codegen, Context::empty());
                    }
                }
                _ => {}
            }
        }
    }
}
