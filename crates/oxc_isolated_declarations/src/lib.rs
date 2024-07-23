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
mod formal_parameter_binding_pattern;
mod function;
mod inferrer;
mod literal;
mod module;
mod return_type;
mod scope;
mod types;

use std::{cell::RefCell, collections::VecDeque, mem};

use diagnostics::function_with_assigning_properties;
use oxc_allocator::Allocator;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstBuilder, Visit};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{Atom, SourceType, SPAN};
use rustc_hash::FxHashSet;

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
        let directives = self.ast.vec();
        let stmts = self.transform_program(program);
        let program = self.ast.program(SPAN, source_type, None, directives, stmts);
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
        let mut new_ast_stmts = self.ast.vec::<Statement<'a>>();
        for stmt in Self::remove_function_overloads_implementation(self.ast.copy(stmts)) {
            if let Some(decl) = stmt.as_declaration() {
                if let Some(decl) = self.transform_declaration(decl, false) {
                    new_ast_stmts.push(Statement::from(decl));
                } else {
                    new_ast_stmts.push(Statement::from(self.ast.copy(decl)));
                }
            }
        }
        self.report_error_for_expando_function(stmts);
        new_ast_stmts
    }

    pub fn transform_statements_on_demand(
        &mut self,
        stmts: &oxc_allocator::Vec<'a, Statement<'a>>,
    ) -> oxc_allocator::Vec<'a, Statement<'a>> {
        // https://github.com/microsoft/TypeScript/pull/58912
        let mut need_empty_export_marker = true;

        let mut new_stmts = Vec::new();
        let mut variables_declarations = VecDeque::new();
        let mut variable_transformed_indexes = VecDeque::new();
        let mut transformed_indexes = FxHashSet::default();
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
                            variable_transformed_indexes.push_back(FxHashSet::default());
                        }
                        Declaration::UsingDeclaration(decl) => {
                            variables_declarations.push_back(
                                self.ast.copy(&decl.declarations).into_iter().collect::<Vec<_>>(),
                            );
                            variable_transformed_indexes.push_back(FxHashSet::default());
                        }
                        Declaration::TSModuleDeclaration(decl) => {
                            if decl.kind.is_global() {
                                self.scope.visit_ts_module_declaration(decl);
                                transformed_indexes.insert(new_stmts.len());
                            }
                        }
                        _ => {}
                    }
                    new_stmts.push(stmt);
                }
                match_module_declaration!(Statement) => {
                    match stmt.to_module_declaration() {
                        ModuleDeclaration::ExportDefaultDeclaration(decl) => {
                            transformed_indexes.insert(new_stmts.len());
                            if let Some((var_decl, new_decl)) =
                                self.transform_export_default_declaration(decl)
                            {
                                if let Some(var_decl) = var_decl {
                                    need_empty_export_marker = false;
                                    self.scope.visit_variable_declaration(&var_decl);
                                    new_stmts.push(Statement::VariableDeclaration(
                                        self.ast.alloc(var_decl),
                                    ));
                                    transformed_indexes.insert(new_stmts.len());
                                }

                                self.scope.visit_export_default_declaration(&new_decl);
                                new_stmts.push(Statement::ExportDefaultDeclaration(
                                    self.ast.alloc(new_decl),
                                ));
                                continue;
                            }

                            need_empty_export_marker = false;
                            self.scope.visit_export_default_declaration(decl);
                        }

                        ModuleDeclaration::ExportNamedDeclaration(decl) => {
                            transformed_indexes.insert(new_stmts.len());
                            if let Some(new_decl) = self.transform_export_named_declaration(decl) {
                                self.scope.visit_declaration(
                                    new_decl.declaration.as_ref().unwrap_or_else(|| unreachable!()),
                                );

                                new_stmts.push(Statement::ExportNamedDeclaration(
                                    self.ast.alloc(new_decl),
                                ));
                                continue;
                            }
                            need_empty_export_marker = false;
                            self.scope.visit_export_named_declaration(decl);
                        }
                        ModuleDeclaration::ImportDeclaration(_) => {
                            // We must transform this in the end, because we need to know all references
                        }
                        module_declaration => {
                            transformed_indexes.insert(new_stmts.len());
                            self.scope.visit_module_declaration(module_declaration);
                        }
                    }

                    new_stmts.push(stmt);
                }
                _ => {}
            }
        }

        // 5. Transform statements until no more transformation can be done
        let last_transformed_len = transformed_indexes.len();
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
                            cur_transformed_indexes.insert(ii);
                            *declarator = decl;
                        }
                    }
                } else if let Some(decl) = self.transform_declaration(decl, true) {
                    self.scope.visit_declaration(&decl);
                    transformed_indexes.insert(i);
                    *stmt = Statement::from(decl);
                }
            }
        }

        // 6. Transform variable/using declarations, import statements, remove unused imports
        // 7. Return transformed statements
        let mut new_ast_stmts = self.ast.vec_with_capacity(transformed_indexes.len());
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
                                self.ast.vec_from_iter(
                                    declarations
                                        .into_iter()
                                        .enumerate()
                                        .filter(|(i, _)| indexes.contains(i))
                                        .map(|(_, decl)| decl),
                                ),
                            );
                        new_ast_stmts.push(Statement::VariableDeclaration(variables_declaration));
                        transformed_indexes.insert(index);
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
                                self.ast.vec_from_iter(
                                    declarations
                                        .into_iter()
                                        .enumerate()
                                        .filter(|(i, _)| indexes.contains(i))
                                        .map(|(_, decl)| decl),
                                ),
                            );
                        new_ast_stmts.push(Statement::VariableDeclaration(variable_declaration));
                        transformed_indexes.insert(index);
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

        if !transformed_indexes.is_empty() && last_transformed_len == transformed_indexes.len() {
            need_empty_export_marker = false;
        }

        if need_empty_export_marker {
            let specifiers = self.ast.vec();
            let kind = ImportOrExportKind::Value;
            let empty_export =
                self.ast.alloc_export_named_declaration(SPAN, None, specifiers, None, kind, None);
            new_ast_stmts
                .push(Statement::from(ModuleDeclaration::ExportNamedDeclaration(empty_export)));
        }

        self.report_error_for_expando_function(stmts);
        new_ast_stmts
    }

    pub fn remove_function_overloads_implementation(
        stmts: oxc_allocator::Vec<'a, Statement<'a>>,
    ) -> impl Iterator<Item = Statement<'a>> + '_ {
        let mut last_function_name: Option<Atom<'a>> = None;
        let mut is_export_default_function_overloads = false;

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
            Statement::ExportDefaultDeclaration(ref decl) => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(ref func) =
                    decl.declaration
                {
                    if is_export_default_function_overloads && func.body.is_some() {
                        is_export_default_function_overloads = false;
                        return None;
                    }
                    is_export_default_function_overloads = true;
                    Some(stmt)
                } else {
                    is_export_default_function_overloads = false;
                    Some(stmt)
                }
            }
            _ => Some(stmt),
        })
    }

    pub fn report_error_for_expando_function(&self, stmts: &oxc_allocator::Vec<'a, Statement<'a>>) {
        let mut can_expando_function_names = FxHashSet::default();
        for stmt in stmts {
            match stmt {
                Statement::ExportNamedDeclaration(decl) => match decl.declaration.as_ref() {
                    Some(Declaration::FunctionDeclaration(func)) => {
                        if func.body.is_some() {
                            if let Some(id) = func.id.as_ref() {
                                can_expando_function_names.insert(id.name.clone());
                            }
                        }
                    }
                    Some(Declaration::VariableDeclaration(decl)) => {
                        for declarator in &decl.declarations {
                            if declarator.id.type_annotation.is_none()
                                && declarator.init.as_ref().is_some_and(Expression::is_function)
                            {
                                if let Some(name) = declarator.id.get_identifier() {
                                    can_expando_function_names.insert(name.clone());
                                }
                            }
                        }
                    }
                    _ => (),
                },
                Statement::ExportDefaultDeclaration(decl) => {
                    if let ExportDefaultDeclarationKind::FunctionDeclaration(func) =
                        &decl.declaration
                    {
                        if func.body.is_some() {
                            if let Some(id) = func.id.as_ref() {
                                can_expando_function_names.insert(id.name.clone());
                            }
                        }
                    }
                }
                Statement::FunctionDeclaration(func) => {
                    if func.body.is_some() {
                        if let Some(id) = func.id.as_ref() {
                            if self.scope.has_reference(&id.name) {
                                can_expando_function_names.insert(id.name.clone());
                            }
                        }
                    }
                }
                Statement::VariableDeclaration(decl) => {
                    for declarator in &decl.declarations {
                        if declarator.id.type_annotation.is_none()
                            && declarator.init.as_ref().is_some_and(Expression::is_function)
                        {
                            if let Some(name) = declarator.id.get_identifier() {
                                if self.scope.has_reference(&name) {
                                    can_expando_function_names.insert(name.clone());
                                }
                            }
                        }
                    }
                }
                Statement::ExpressionStatement(stmt) => {
                    if let Expression::AssignmentExpression(assignment) = &stmt.expression {
                        if let AssignmentTarget::StaticMemberExpression(static_member_expr) =
                            &assignment.left
                        {
                            if let Expression::Identifier(ident) = &static_member_expr.object {
                                if can_expando_function_names.contains(&ident.name) {
                                    self.error(function_with_assigning_properties(
                                        static_member_expr.span,
                                    ));
                                }
                            }
                        }
                    }
                }

                _ => {}
            }
        }
    }

    pub fn is_declare(&self) -> bool {
        // If we are in a module block, we don't need to add declare
        !self.scope.is_ts_module_block_flag()
    }
}
