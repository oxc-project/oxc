//! DTS Transformer / Transpiler
//!
//! References:
//! * <https://devblogs.microsoft.com/typescript/announcing-typescript-5-5-rc/#isolated-declarations>
//! * <https://www.typescriptlang.org/tsconfig#isolatedDeclarations>
//! * <https://github.com/microsoft/TypeScript/blob/main/src/compiler/transformers/declarations.ts>

mod class;
mod declaration;
mod diagnostics;
mod r#enum;
mod function;
mod inferrer;
mod module;
mod return_type;
mod scope;
mod types;

use std::{cell::RefCell, collections::VecDeque, mem};

use oxc_allocator::Allocator;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstBuilder, Visit};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{Atom, SourceType, SPAN};

use crate::scope::ScopeTree;

pub struct IsolatedDeclarationsReturn<'a> {
    pub program: Program<'a>,
    pub errors: Vec<OxcDiagnostic>,
}

pub struct IsolatedDeclarations<'a> {
    ast: AstBuilder<'a>,
    // state
    scope: ScopeTree<'a>,
    errors: RefCell<Vec<OxcDiagnostic>>,
}

impl<'a> IsolatedDeclarations<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            ast: AstBuilder::new(allocator),
            scope: ScopeTree::new(allocator),
            errors: RefCell::new(vec![]),
        }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(mut self, program: &Program<'a>) -> IsolatedDeclarationsReturn<'a> {
        let source_type = SourceType::default().with_module(true).with_typescript_definition(true);
        let directives = self.ast.new_vec();
        let stmts = self.transform_program(program);
        let program = self.ast.program(SPAN, source_type, directives, None, stmts);
        IsolatedDeclarationsReturn { program, errors: self.take_errors() }
    }

    fn take_errors(&self) -> Vec<OxcDiagnostic> {
        mem::take(&mut self.errors.borrow_mut())
    }

    /// Add an Error
    fn error(&self, error: OxcDiagnostic) {
        self.errors.borrow_mut().push(error);
    }
}

impl<'a> IsolatedDeclarations<'a> {
    pub fn transform_program(
        &mut self,
        program: &Program<'a>,
    ) -> oxc_allocator::Vec<'a, Statement<'a>> {
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
            self.transform_statements_on_demand(&program.body)
        } else {
            self.transform_program_without_module_declaration(&program.body)
        }
    }

    pub fn transform_program_without_module_declaration(
        &mut self,
        stmts: &oxc_allocator::Vec<'a, Statement<'a>>,
    ) -> oxc_allocator::Vec<'a, Statement<'a>> {
        let mut new_ast_stmts = self.ast.new_vec::<Statement<'a>>();
        for stmt in Self::remove_function_overloads_implementation(self.ast.copy(stmts)) {
            if let Some(decl) = stmt.as_declaration() {
                if let Some(decl) = self.transform_declaration(decl, false) {
                    new_ast_stmts.push(Statement::from(decl));
                } else {
                    new_ast_stmts.push(Statement::from(self.ast.copy(decl)));
                }
            }
        }
        new_ast_stmts
    }

    pub fn transform_statements_on_demand(
        &mut self,
        stmts: &oxc_allocator::Vec<'a, Statement<'a>>,
    ) -> oxc_allocator::Vec<'a, Statement<'a>> {
        let mut new_stmts = Vec::new();
        let mut variables_declarations = VecDeque::new();
        let mut variable_transformed_indexes = VecDeque::new();
        let mut transformed_indexes = Vec::new();
        // 1. Collect all declarations, module declarations
        // 2. Transform export declarations
        // 3. Collect all bindings / reference from module declarations
        // 4. Collect transformed indexes
        for stmt in Self::remove_function_overloads_implementation(self.ast.copy(stmts)) {
            match stmt {
                match_declaration!(Statement) => {
                    match stmt.to_declaration() {
                        Declaration::VariableDeclaration(decl) => {
                            variables_declarations.push_back(
                                self.ast.copy(&decl.declarations).into_iter().collect::<Vec<_>>(),
                            );
                            variable_transformed_indexes.push_back(Vec::default());
                        }
                        Declaration::UsingDeclaration(decl) => {
                            variables_declarations.push_back(
                                self.ast.copy(&decl.declarations).into_iter().collect::<Vec<_>>(),
                            );
                            variable_transformed_indexes.push_back(Vec::default());
                        }
                        _ => {}
                    }
                    new_stmts.push(stmt);
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
                                        self.ast.alloc(var_decl),
                                    ));
                                    transformed_indexes.push(new_stmts.len());
                                }

                                self.scope.visit_export_default_declaration(&new_decl);
                                new_stmts.push(Statement::ExportDefaultDeclaration(
                                    self.ast.alloc(new_decl),
                                ));
                                continue;
                            }

                            self.scope.visit_export_default_declaration(decl);
                        }

                        ModuleDeclaration::ExportNamedDeclaration(decl) => {
                            if let Some(new_decl) = self.transform_export_named_declaration(decl) {
                                self.scope.visit_declaration(
                                    new_decl.declaration.as_ref().unwrap_or_else(|| unreachable!()),
                                );

                                new_stmts.push(Statement::ExportNamedDeclaration(
                                    self.ast.alloc(new_decl),
                                ));
                                continue;
                            }

                            self.scope.visit_export_named_declaration(decl);
                        }
                        module_declaration => {
                            self.scope.visit_module_declaration(module_declaration);
                        }
                    }

                    new_stmts.push(stmt);
                }
                _ => {}
            }
        }

        // 5. Transform statements until no more transformation can be done
        let mut last_reference_len = 0;
        while last_reference_len != self.scope.references_len() {
            last_reference_len = self.scope.references_len();

            let mut variables_declarations_iter = variables_declarations.iter_mut();
            let mut variable_transformed_indexes_iter = variable_transformed_indexes.iter_mut();

            for (i, stmt) in new_stmts.iter_mut().enumerate() {
                if transformed_indexes.contains(&i) {
                    continue;
                }
                let Some(decl) = stmt.as_declaration() else { continue };

                if let Declaration::VariableDeclaration(_) | Declaration::UsingDeclaration(_) = decl
                {
                    let Some(cur_variable_declarations) = variables_declarations_iter.next() else {
                        unreachable!()
                    };
                    let Some(cur_transformed_indexes) = variable_transformed_indexes_iter.next()
                    else {
                        unreachable!()
                    };

                    for (ii, declarator) in cur_variable_declarations.iter_mut().enumerate() {
                        if cur_transformed_indexes.contains(&ii) {
                            continue;
                        }

                        if let Some(decl) = self.transform_variable_declarator(declarator, true) {
                            self.scope.visit_variable_declarator(&decl);
                            cur_transformed_indexes.push(ii);
                            *declarator = decl;
                        }
                    }
                } else if let Some(decl) = self.transform_declaration(decl, true) {
                    self.scope.visit_declaration(&decl);
                    transformed_indexes.push(i);
                    *stmt = Statement::from(decl);
                }
            }
        }

        // 6. Transform variable/using declarations, import statements, remove unused imports
        // 7. Return transformed statements
        let mut new_ast_stmts = self.ast.new_vec_with_capacity(transformed_indexes.len());
        for (index, stmt) in new_stmts.into_iter().enumerate() {
            match stmt {
                _ if transformed_indexes.contains(&index) => {
                    new_ast_stmts.push(stmt);
                }
                Statement::VariableDeclaration(decl) => {
                    let indexes =
                        variable_transformed_indexes.pop_front().unwrap_or_else(|| unreachable!());
                    let declarations =
                        variables_declarations.pop_front().unwrap_or_else(|| unreachable!());

                    if !indexes.is_empty() {
                        let variables_declaration = self
                            .transform_variable_declaration_with_new_declarations(
                                &decl,
                                self.ast.new_vec_from_iter(
                                    declarations
                                        .into_iter()
                                        .enumerate()
                                        .filter(|(i, _)| indexes.contains(i))
                                        .map(|(_, decl)| decl),
                                ),
                            );
                        new_ast_stmts.push(Statement::VariableDeclaration(variables_declaration));
                    }
                }
                Statement::UsingDeclaration(decl) => {
                    let indexes =
                        variable_transformed_indexes.pop_front().unwrap_or_else(|| unreachable!());
                    let declarations =
                        variables_declarations.pop_front().unwrap_or_else(|| unreachable!());

                    if !indexes.is_empty() {
                        let variable_declaration = self
                            .transform_using_declaration_with_new_declarations(
                                &decl,
                                self.ast.new_vec_from_iter(
                                    declarations
                                        .into_iter()
                                        .enumerate()
                                        .filter(|(i, _)| indexes.contains(i))
                                        .map(|(_, decl)| decl),
                                ),
                            );
                        new_ast_stmts.push(Statement::VariableDeclaration(variable_declaration));
                    }
                }
                Statement::ImportDeclaration(decl) => {
                    // We must transform this in the end, because we need to know all references
                    if decl.specifiers.is_none() {
                        new_ast_stmts.push(Statement::ImportDeclaration(decl));
                    } else if let Some(decl) = self.transform_import_declaration(&decl) {
                        new_ast_stmts.push(Statement::ImportDeclaration(decl));
                    }
                }
                _ => {}
            }
        }

        new_ast_stmts
    }

    pub fn remove_function_overloads_implementation(
        stmts: oxc_allocator::Vec<'a, Statement<'a>>,
    ) -> impl Iterator<Item = Statement<'a>> + '_ {
        let mut last_function_name: Option<Atom<'a>> = None;

        stmts.into_iter().filter_map(move |stmt| match stmt {
            Statement::FunctionDeclaration(ref func) => {
                let name = &func
                    .id
                    .as_ref()
                    .unwrap_or_else(|| {
                        unreachable!(
                            "Only export default function declaration is allowed to have no name"
                        )
                    })
                    .name;

                if func.body.is_some() {
                    if last_function_name.as_ref().is_some_and(|last_name| last_name == name) {
                        return None;
                    }
                } else {
                    last_function_name = Some(name.clone());
                }
                Some(stmt)
            }
            Statement::ExportNamedDeclaration(ref decl) => {
                if let Some(Declaration::FunctionDeclaration(ref func)) = decl.declaration {
                    let name = &func
                        .id
                        .as_ref()
                        .unwrap_or_else(|| {
                            unreachable!(
                            "Only export default function declaration is allowed to have no name"
                        )
                        })
                        .name;
                    if func.body.is_some() {
                        if last_function_name.as_ref().is_some_and(|last_name| last_name == name) {
                            return None;
                        }
                    } else {
                        last_function_name = Some(name.clone());
                    }
                    Some(stmt)
                } else {
                    Some(stmt)
                }
            }
            _ => Some(stmt),
        })
    }

    pub fn modifiers_declare(&self) -> Modifiers<'a> {
        if self.scope.is_ts_module_block_flag() {
            // If we are in a module block, we don't need to add declare
            Modifiers::empty()
        } else {
            Modifiers::new(
                self.ast.new_vec_single(Modifier { span: SPAN, kind: ModifierKind::Declare }),
            )
        }
    }
}
