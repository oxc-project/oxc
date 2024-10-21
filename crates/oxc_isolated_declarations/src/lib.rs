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
mod signatures;
mod types;

use std::{cell::RefCell, mem};

use diagnostics::function_with_assigning_properties;
use oxc_allocator::{Allocator, CloneIn};
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstBuilder, Visit, NONE};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{Atom, GetSpan, SourceType, SPAN};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::scope::ScopeTree;

#[derive(Debug, Default, Clone, Copy)]
pub struct IsolatedDeclarationsOptions {
    /// Do not emit declarations for code that has an `@internal` annotation in its JSDoc comment.
    /// This is an internal compiler option; use at your own risk, because the compiler does not
    /// check that the result is valid.
    ///
    /// Default: `false`
    ///
    /// ## References
    /// [TSConfig - `stripInternal`](https://www.typescriptlang.org/tsconfig/#stripInternal)
    pub strip_internal: bool,
}

#[non_exhaustive]
pub struct IsolatedDeclarationsReturn<'a> {
    pub program: Program<'a>,
    pub errors: Vec<OxcDiagnostic>,
}

pub struct IsolatedDeclarations<'a> {
    ast: AstBuilder<'a>,

    // state
    scope: ScopeTree<'a>,
    errors: RefCell<Vec<OxcDiagnostic>>,

    // options
    strip_internal: bool,

    /// Start position of `@internal` jsdoc annotations.
    internal_annotations: FxHashSet<u32>,
}

impl<'a> IsolatedDeclarations<'a> {
    pub fn new(allocator: &'a Allocator, options: IsolatedDeclarationsOptions) -> Self {
        let strip_internal = options.strip_internal;
        Self {
            ast: AstBuilder::new(allocator),
            strip_internal,
            internal_annotations: FxHashSet::default(),
            scope: ScopeTree::new(),
            errors: RefCell::new(vec![]),
        }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(mut self, program: &Program<'a>) -> IsolatedDeclarationsReturn<'a> {
        self.internal_annotations = self
            .strip_internal
            .then(|| Self::build_internal_annotations(program))
            .unwrap_or_default();
        let source_type = SourceType::d_ts();
        let directives = self.ast.vec();
        let stmts = self.transform_program(program);
        let program = self.ast.program(
            SPAN,
            source_type,
            program.source_text,
            self.ast.vec_from_iter(program.comments.iter().copied()),
            None,
            directives,
            stmts,
        );
        IsolatedDeclarationsReturn { program, errors: self.take_errors() }
    }

    fn take_errors(&self) -> Vec<OxcDiagnostic> {
        mem::take(&mut self.errors.borrow_mut())
    }

    /// Add an Error
    fn error(&self, error: OxcDiagnostic) {
        self.errors.borrow_mut().push(error);
    }

    /// Build the lookup table for jsdoc `@internal`.
    fn build_internal_annotations(program: &Program<'a>) -> FxHashSet<u32> {
        let mut set = FxHashSet::default();
        for comment in &program.comments {
            let has_internal = comment.span.source_text(program.source_text).contains("@internal");
            // Use the first jsdoc comment if there are multiple jsdoc comments for the same node.
            if has_internal && !set.contains(&comment.attached_to) {
                set.insert(comment.attached_to);
            }
        }
        set
    }

    /// Check if the node has an `@internal` annotation.
    fn has_internal_annotation(&self, span: Span) -> bool {
        if !self.strip_internal {
            return false;
        }
        self.internal_annotations.contains(&span.start)
    }
}

impl<'a> IsolatedDeclarations<'a> {
    fn transform_program(
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

    fn transform_program_without_module_declaration(
        &mut self,
        stmts: &oxc_allocator::Vec<'a, Statement<'a>>,
    ) -> oxc_allocator::Vec<'a, Statement<'a>> {
        self.report_error_for_expando_function(stmts);

        let mut stmts =
            self.ast.vec_from_iter(stmts.iter().filter(|stmt| {
                stmt.is_declaration() && !self.has_internal_annotation(stmt.span())
            }));

        Self::remove_function_overloads_implementation(&mut stmts);

        self.ast.vec_from_iter(stmts.iter().map(|stmt| {
            if let Some(new_decl) = self.transform_declaration(stmt.to_declaration(), false) {
                Statement::from(new_decl)
            } else {
                stmt.clone_in(self.ast.allocator)
            }
        }))
    }

    #[allow(clippy::missing_panics_doc)]
    fn transform_statements_on_demand(
        &mut self,
        stmts: &oxc_allocator::Vec<'a, Statement<'a>>,
    ) -> oxc_allocator::Vec<'a, Statement<'a>> {
        self.report_error_for_expando_function(stmts);

        let mut stmts = self.ast.vec_from_iter(stmts.iter().filter(|stmt| {
            (stmt.is_declaration() || stmt.is_module_declaration())
                && !self.has_internal_annotation(stmt.span())
        }));
        Self::remove_function_overloads_implementation(&mut stmts);

        // https://github.com/microsoft/TypeScript/pull/58912
        let mut need_empty_export_marker = true;

        // Use span to identify transformed nodes
        let mut transformed_spans: FxHashSet<Span> = FxHashSet::default();
        let mut transformed_stmts: FxHashMap<Span, Statement<'a>> = FxHashMap::default();
        let mut transformed_variable_declarator: FxHashMap<Span, VariableDeclarator<'a>> =
            FxHashMap::default();
        let mut export_default_var = None;

        // 1. Collect all declarations, module declarations
        // 2. Transform export declarations
        // 3. Collect all bindings / reference from module declarations
        // 4. Collect transformed indexes
        for stmt in &stmts {
            match stmt {
                match_declaration!(Statement) => {
                    if let Statement::TSModuleDeclaration(decl) = stmt {
                        // `declare global { ... }` or `declare module "foo" { ... }`
                        // We need to emit it anyway
                        let is_global = decl.kind.is_global();
                        if is_global || decl.id.is_string_literal() {
                            transformed_spans.insert(decl.span);

                            let mut decl = decl.clone_in(self.ast.allocator);
                            // Remove export keyword from all statements in `declare module "xxx" { ... }`
                            if !is_global {
                                if let Some(body) =
                                    decl.body.as_mut().and_then(|body| body.as_module_block_mut())
                                {
                                    self.strip_export_keyword(&mut body.body);
                                }
                            }

                            // We need to visit the module declaration to collect all references
                            self.scope.visit_ts_module_declaration(decl.as_ref());

                            transformed_stmts.insert(
                                decl.span,
                                Statement::from(Declaration::TSModuleDeclaration(decl)),
                            );
                        }
                    }
                }
                match_module_declaration!(Statement) => {
                    match stmt.to_module_declaration() {
                        ModuleDeclaration::ExportDefaultDeclaration(decl) => {
                            transformed_spans.insert(decl.span);
                            if let Some((var_decl, new_decl)) =
                                self.transform_export_default_declaration(decl)
                            {
                                if let Some(var_decl) = var_decl {
                                    self.scope.visit_variable_declaration(&var_decl);
                                    export_default_var = Some(Statement::VariableDeclaration(
                                        self.ast.alloc(var_decl),
                                    ));
                                }

                                self.scope.visit_export_default_declaration(&new_decl);
                                transformed_stmts.insert(
                                    decl.span,
                                    Statement::from(ModuleDeclaration::ExportDefaultDeclaration(
                                        self.ast.alloc(new_decl),
                                    )),
                                );
                            } else {
                                self.scope.visit_export_default_declaration(decl);
                            }

                            need_empty_export_marker = false;
                        }

                        ModuleDeclaration::ExportNamedDeclaration(decl) => {
                            transformed_spans.insert(decl.span);
                            if let Some(new_decl) = self.transform_export_named_declaration(decl) {
                                self.scope.visit_export_named_declaration(&new_decl);
                                transformed_stmts.insert(
                                    decl.span,
                                    Statement::from(ModuleDeclaration::ExportNamedDeclaration(
                                        self.ast.alloc(new_decl),
                                    )),
                                );
                            } else if decl.declaration.is_none() {
                                need_empty_export_marker = false;
                                self.scope.visit_export_named_declaration(decl);
                            }
                        }
                        ModuleDeclaration::ImportDeclaration(_) => {
                            // We must transform this in the end, because we need to know all references
                        }
                        module_declaration => {
                            transformed_spans.insert(module_declaration.span());
                            self.scope.visit_module_declaration(module_declaration);
                        }
                    }
                }
                _ => {}
            }
        }

        let last_transformed_len = transformed_spans.len() + transformed_variable_declarator.len();
        // 5. Transform statements until no more transformation can be done
        loop {
            let cur_transformed_len =
                transformed_spans.len() + transformed_variable_declarator.len();
            for stmt in &stmts {
                if transformed_spans.contains(&stmt.span()) {
                    continue;
                }
                let Some(decl) = stmt.as_declaration() else { continue };

                // Skip transformed declarations
                if transformed_spans.contains(&decl.span()) {
                    continue;
                }

                if let Declaration::VariableDeclaration(declaration) = decl {
                    let mut all_declarator_has_transformed = true;
                    for declarator in &declaration.declarations {
                        if transformed_spans.contains(&declarator.span) {
                            continue;
                        }

                        if let Some(new_declarator) =
                            self.transform_variable_declarator(declarator, true)
                        {
                            self.scope.visit_variable_declarator(&new_declarator);
                            transformed_variable_declarator.insert(declarator.span, new_declarator);
                        } else {
                            all_declarator_has_transformed = false;
                        }
                    }
                    if all_declarator_has_transformed {
                        let declarations = self.ast.vec_from_iter(
                            declaration.declarations.iter().map(|declarator| {
                                transformed_variable_declarator.remove(&declarator.span).unwrap()
                            }),
                        );
                        let decl = self.ast.variable_declaration(
                            declaration.span,
                            declaration.kind,
                            declarations,
                            self.is_declare(),
                        );
                        transformed_stmts.insert(
                            declaration.span,
                            Statement::from(self.ast.declaration_from_variable(decl)),
                        );
                        transformed_spans.insert(declaration.span);
                    }
                } else if let Some(new_decl) = self.transform_declaration(decl, true) {
                    self.scope.visit_declaration(&new_decl);
                    transformed_spans.insert(new_decl.span());
                    transformed_stmts.insert(new_decl.span(), Statement::from(new_decl));
                }
            }

            // Use transformed_spans to weather we need to continue
            if cur_transformed_len
                == transformed_spans.len() + transformed_variable_declarator.len()
            {
                break;
            }
        }

        // If any declaration is transformed in previous step, we don't need to add empty export marker
        if last_transformed_len != 0 && last_transformed_len == transformed_spans.len() {
            need_empty_export_marker = false;
        }

        // 6. Transform variable/using declarations, import statements, remove unused imports
        let mut new_stm =
            self.ast.vec_with_capacity(stmts.len() + usize::from(export_default_var.is_some()));
        stmts.iter().for_each(|stmt| {
            if transformed_spans.contains(&stmt.span()) {
                let new_stmt = transformed_stmts
                    .remove(&stmt.span())
                    .unwrap_or_else(|| stmt.clone_in(self.ast.allocator));
                if matches!(new_stmt, Statement::ExportDefaultDeclaration(_)) {
                    if let Some(export_default_var) = export_default_var.take() {
                        new_stm.push(export_default_var);
                    }
                }
                new_stm.push(new_stmt);
                return;
            }
            match stmt {
                Statement::ImportDeclaration(decl) => {
                    // We must transform this in the end, because we need to know all references
                    if decl.specifiers.is_none() {
                        new_stm.push(stmt.clone_in(self.ast.allocator));
                    } else if let Some(new_decl) = self.transform_import_declaration(decl) {
                        new_stm.push(Statement::from(
                            self.ast.module_declaration_from_import_declaration(new_decl),
                        ));
                    }
                }
                Statement::VariableDeclaration(decl) => {
                    if decl.declarations.len() > 1 {
                        // Remove unreferenced declarations
                        let declarations = self.ast.vec_from_iter(
                            decl.declarations.iter().filter_map(|declarator| {
                                transformed_variable_declarator.remove(&declarator.span)
                            }),
                        );
                        new_stm.push(Statement::from(self.ast.declaration_from_variable(
                            self.ast.variable_declaration(
                                decl.span,
                                decl.kind,
                                declarations,
                                self.is_declare(),
                            ),
                        )));
                    }
                }
                _ => {}
            }
        });

        if need_empty_export_marker {
            let specifiers = self.ast.vec();
            let kind = ImportOrExportKind::Value;
            let empty_export =
                self.ast.alloc_export_named_declaration(SPAN, None, specifiers, None, kind, NONE);
            new_stm.push(Statement::from(ModuleDeclaration::ExportNamedDeclaration(empty_export)));
        } else if self.scope.is_ts_module_block() {
            // If we are in a module block and we don't need to add `export {}`, in that case we need to remove `export` keyword from all ExportNamedDeclaration
            // <https://github.com/microsoft/TypeScript/blob/a709f9899c2a544b6de65a0f2623ecbbe1394eab/src/compiler/transformers/declarations.ts#L1556-L1563>
            self.strip_export_keyword(&mut new_stm);
        }

        new_stm
    }

    fn remove_function_overloads_implementation(
        stmts: &mut oxc_allocator::Vec<'a, &Statement<'a>>,
    ) {
        let mut last_function_name: Option<Atom<'a>> = None;
        let mut is_export_default_function_overloads = false;

        stmts.retain(move |stmt| match stmt {
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
                        return false;
                    }
                } else {
                    last_function_name = Some(name.clone());
                }
                true
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
                            return false;
                        }
                    } else {
                        last_function_name = Some(name.clone());
                    }
                    true
                } else {
                    true
                }
            }
            Statement::ExportDefaultDeclaration(ref decl) => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(ref func) =
                    decl.declaration
                {
                    if is_export_default_function_overloads && func.body.is_some() {
                        is_export_default_function_overloads = false;
                        return false;
                    }
                    is_export_default_function_overloads = true;
                    true
                } else {
                    is_export_default_function_overloads = false;
                    true
                }
            }
            _ => true,
        });
    }

    fn get_assignable_properties_for_namespaces(
        stmts: &'a oxc_allocator::Vec<'a, Statement<'a>>,
    ) -> FxHashMap<&'a str, FxHashSet<Atom>> {
        let mut assignable_properties_for_namespace = FxHashMap::<&str, FxHashSet<Atom>>::default();
        for stmt in stmts {
            let decl = match stmt {
                Statement::ExportNamedDeclaration(decl) => {
                    if let Some(Declaration::TSModuleDeclaration(decl)) = &decl.declaration {
                        decl
                    } else {
                        continue;
                    }
                }
                Statement::TSModuleDeclaration(decl) => decl,
                _ => continue,
            };

            if decl.kind != TSModuleDeclarationKind::Namespace {
                continue;
            }
            let TSModuleDeclarationName::Identifier(ident) = &decl.id else { continue };
            let Some(TSModuleDeclarationBody::TSModuleBlock(block)) = &decl.body else {
                continue;
            };
            for stmt in &block.body {
                let Statement::ExportNamedDeclaration(decl) = stmt else { continue };
                match &decl.declaration {
                    Some(Declaration::VariableDeclaration(var)) => {
                        for declarator in &var.declarations {
                            if let Some(name) = declarator.id.get_identifier() {
                                assignable_properties_for_namespace
                                    .entry(&ident.name)
                                    .or_default()
                                    .insert(name);
                            }
                        }
                    }
                    Some(Declaration::FunctionDeclaration(func)) => {
                        if let Some(name) = func.name() {
                            assignable_properties_for_namespace
                                .entry(&ident.name)
                                .or_default()
                                .insert(name);
                        }
                    }
                    Some(Declaration::ClassDeclaration(cls)) => {
                        if let Some(id) = cls.id.as_ref() {
                            assignable_properties_for_namespace
                                .entry(&ident.name)
                                .or_default()
                                .insert(id.name.clone());
                        }
                    }
                    Some(Declaration::TSEnumDeclaration(decl)) => {
                        assignable_properties_for_namespace
                            .entry(&ident.name)
                            .or_default()
                            .insert(decl.id.name.clone());
                    }
                    _ => {}
                }
            }
        }
        assignable_properties_for_namespace
    }

    fn report_error_for_expando_function(&self, stmts: &oxc_allocator::Vec<'a, Statement<'a>>) {
        let assignable_properties_for_namespace =
            IsolatedDeclarations::get_assignable_properties_for_namespaces(stmts);

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
                            if let Some(name) = func.name() {
                                can_expando_function_names.insert(name);
                            }
                        }
                    }
                }
                Statement::FunctionDeclaration(func) => {
                    if func.body.is_some() {
                        if let Some(name) = func.name() {
                            if self.scope.has_reference(&name) {
                                can_expando_function_names.insert(name);
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
                                if can_expando_function_names.contains(&ident.name)
                                    && !assignable_properties_for_namespace
                                        .get(&ident.name.as_str())
                                        .map_or(false, |properties| {
                                            properties.contains(&static_member_expr.property.name)
                                        })
                                {
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

    fn is_declare(&self) -> bool {
        // If we are in a module block, we don't need to add declare
        !self.scope.is_ts_module_block()
    }
}
