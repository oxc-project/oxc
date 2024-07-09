use oxc_allocator::{Box, Vec};
use oxc_ast::{ast::*, syntax_directed_operations::BoundNames};
use oxc_span::{Atom, CompactStr, SPAN};
use oxc_syntax::{
    operator::{AssignmentOperator, LogicalOperator},
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
};
use oxc_traverse::TraverseCtx;
use rustc_hash::FxHashSet;

use super::{
    diagnostics::{ambient_module_nested, namespace_exporting_non_const, namespace_not_supported},
    TypeScript,
};

// TODO:
// 1. register scope for the newly created function: <https://github.com/babel/babel/blob/08b0472069cd207f043dd40a4d157addfdd36011/packages/babel-plugin-transform-typescript/src/namespace.ts#L38>
impl<'a> TypeScript<'a> {
    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    pub(super) fn transform_program_for_namespace(
        &self,
        program: &mut Program<'a>,
        ctx: &mut TraverseCtx,
    ) {
        // namespace declaration is only allowed at the top level

        if !has_namespace(program.body.as_slice()) {
            return;
        }

        // Collect function/class/enum/namespace binding names
        let mut names: FxHashSet<Atom<'a>> = FxHashSet::default();

        // Recreate the statements vec for memory efficiency.
        // Inserting the `let` declaration multiple times will reallocate the whole statements vec
        // every time a namespace declaration is encountered.
        let mut new_stmts = self.ctx.ast.new_vec();

        for stmt in self.ctx.ast.move_statement_vec(&mut program.body) {
            match stmt {
                Statement::TSModuleDeclaration(decl) => {
                    if !decl.declare {
                        if !self.options.allow_namespaces {
                            self.ctx.error(namespace_not_supported(decl.span));
                        }

                        if let Some(transformed_stmt) =
                            self.handle_nested(self.ctx.ast.copy(&decl).unbox(), None, ctx)
                        {
                            let name = decl.id.name();
                            if names.insert(name.clone()) {
                                new_stmts
                                    .push(Statement::from(self.create_variable_declaration(name)));
                            }
                            new_stmts.push(transformed_stmt);
                            continue;
                        }
                    }
                    new_stmts.push(Statement::TSModuleDeclaration(decl));
                    continue;
                }
                Statement::ExportNamedDeclaration(ref export_decl) => {
                    match &export_decl.declaration {
                        Some(Declaration::TSModuleDeclaration(decl)) => {
                            if !decl.declare {
                                if !self.options.allow_namespaces {
                                    self.ctx.error(namespace_not_supported(decl.span));
                                }

                                if let Some(transformed_stmt) =
                                    self.handle_nested(self.ctx.ast.copy(decl), None, ctx)
                                {
                                    let name = decl.id.name();
                                    if names.insert(name.clone()) {
                                        let declaration = self.create_variable_declaration(name);
                                        let export_named_decl = self
                                            .ctx
                                            .ast
                                            .plain_export_named_declaration_declaration(
                                                SPAN,
                                                declaration,
                                            );
                                        let stmt =
                                            Statement::ExportNamedDeclaration(export_named_decl);
                                        new_stmts.push(stmt);
                                    }
                                    new_stmts.push(transformed_stmt);
                                    continue;
                                }
                            }

                            if let TSModuleDeclarationName::Identifier(id) = &decl.id {
                                names.insert(id.name.clone());
                            }
                        }
                        Some(decl) => match decl {
                            Declaration::FunctionDeclaration(_)
                            | Declaration::ClassDeclaration(_)
                            | Declaration::TSEnumDeclaration(_) => {
                                names.insert(decl.id().as_ref().unwrap().name.clone());
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                // Collect bindings from class, function and enum declarations
                Statement::FunctionDeclaration(_)
                | Statement::ClassDeclaration(_)
                | Statement::TSEnumDeclaration(_) => {
                    names.insert(stmt.to_declaration().id().as_ref().unwrap().name.clone());
                }
                _ => {}
            }

            new_stmts.push(stmt);
        }

        program.body = new_stmts;
    }

    fn handle_nested(
        &self,
        decl: TSModuleDeclaration<'a>,
        parent_export: Option<Expression<'a>>,
        ctx: &mut TraverseCtx,
    ) -> Option<Statement<'a>> {
        // Skip empty declaration e.g. `namespace x;`
        let body = decl.body?;

        let mut names: FxHashSet<Atom<'a>> = FxHashSet::default();

        let TSModuleDeclarationName::Identifier(IdentifierName { name: real_name, .. }) = decl.id
        else {
            return None;
        };

        // Reuse `TSModuleDeclaration`'s scope in transformed function
        let scope_id = decl.scope_id.get().unwrap();
        let symbol_id = ctx.generate_uid(&real_name, scope_id, SymbolFlags::FunctionScopedVariable);
        let name = self.ctx.ast.new_atom(ctx.symbols().get_name(symbol_id));

        let directives;
        let namespace_top_level;

        match body {
            TSModuleDeclarationBody::TSModuleBlock(block) => {
                let block = block.unbox();
                directives = block.directives;
                namespace_top_level = block.body;
            }
            // We handle `namespace X.Y {}` as if it was
            //   namespace X {
            //     export namespace Y {}
            //   }
            TSModuleDeclarationBody::TSModuleDeclaration(declaration) => {
                let declaration = Declaration::TSModuleDeclaration(declaration);
                let export_named_decl =
                    self.ctx.ast.plain_export_named_declaration_declaration(SPAN, declaration);
                let stmt = Statement::ExportNamedDeclaration(export_named_decl);
                directives = self.ctx.ast.new_vec();
                namespace_top_level = self.ctx.ast.new_vec_single(stmt);
            }
        }

        let mut new_stmts = self.ctx.ast.new_vec();

        for stmt in namespace_top_level {
            match stmt {
                Statement::TSModuleDeclaration(decl) => {
                    if decl.id.is_string_literal() {
                        self.ctx.error(ambient_module_nested(decl.span));
                        continue;
                    }

                    let module_name = decl.id.name().clone();
                    if let Some(transformed) = self.handle_nested(decl.unbox(), None, ctx) {
                        if names.insert(module_name.clone()) {
                            new_stmts.push(Statement::from(
                                self.create_variable_declaration(module_name.clone()),
                            ));
                        }
                        new_stmts.push(transformed);
                    }
                    continue;
                }
                Statement::ExportNamedDeclaration(export_decl) => {
                    // NB: `ExportNamedDeclaration` with no declaration (e.g. `export {x}`) is not
                    // legal syntax in TS namespaces
                    let export_decl = export_decl.unbox();
                    if let Some(decl) = export_decl.declaration {
                        if decl.declare() {
                            continue;
                        }
                        match decl {
                            Declaration::TSEnumDeclaration(_)
                            | Declaration::FunctionDeclaration(_)
                            | Declaration::ClassDeclaration(_) => {
                                self.add_declaration(
                                    decl,
                                    name.clone(),
                                    &mut names,
                                    &mut new_stmts,
                                );
                            }
                            Declaration::VariableDeclaration(var_decl) => {
                                var_decl.declarations.iter().for_each(|decl| {
                                    if !decl.kind.is_const() {
                                        self.ctx.error(namespace_exporting_non_const(decl.span));
                                    }
                                });
                                let stmts =
                                    self.handle_variable_declaration(var_decl, name.clone());
                                new_stmts.extend(stmts);
                            }
                            Declaration::TSModuleDeclaration(module_decl) => {
                                if module_decl.id.is_string_literal() {
                                    self.ctx.error(ambient_module_nested(module_decl.span));
                                    continue;
                                }

                                let module_name = module_decl.id.name().clone();
                                if let Some(transformed) = self.handle_nested(
                                    module_decl.unbox(),
                                    Some(self.ctx.ast.expression_identifier_reference(SPAN, &name)),
                                    ctx,
                                ) {
                                    if names.insert(module_name.clone()) {
                                        new_stmts.push(Statement::from(
                                            self.create_variable_declaration(module_name.clone()),
                                        ));
                                    }
                                    new_stmts.push(transformed);
                                }
                            }
                            _ => {}
                        }
                    }
                    continue;
                }
                // Collect bindings from class, function and enum declarations
                Statement::ClassDeclaration(_)
                | Statement::FunctionDeclaration(_)
                | Statement::TSEnumDeclaration(_) => {
                    names.insert(stmt.to_declaration().id().as_ref().unwrap().name.clone());
                }
                Statement::TSTypeAliasDeclaration(_)
                | Statement::TSInterfaceDeclaration(_)
                | Statement::TSImportEqualsDeclaration(_) => continue,
                _ => {}
            }
            new_stmts.push(stmt);
        }

        if new_stmts.is_empty() {
            // Delete the scope binding that `ctx.generate_uid_in_current_scope` created above,
            // as no binding is actually being created
            let current_scope_id = ctx.current_scope_id();
            ctx.scopes_mut().remove_binding(current_scope_id, &CompactStr::from(name.as_str()));

            return None;
        }

        Some(self.transform_namespace(
            name,
            real_name,
            new_stmts,
            directives,
            parent_export,
            scope_id,
            ctx,
        ))
    }

    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    //                         ^^^^^^^
    fn create_variable_declaration(&self, name: Atom<'a>) -> Declaration<'a> {
        let kind = VariableDeclarationKind::Let;
        let declarations = {
            let pattern_kind = self.ctx.ast.binding_pattern_kind_binding_identifier(SPAN, name);
            let binding =
                self.ctx.ast.binding_pattern(pattern_kind, Option::<TSTypeAnnotation>::None, false);
            let decl = self.ctx.ast.variable_declarator(SPAN, kind, binding, None, false);
            self.ctx.ast.new_vec_single(decl)
        };
        self.ctx.ast.declaration_variable(SPAN, kind, declarations, false)
    }

    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    //                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    #[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
    fn transform_namespace(
        &self,
        arg_name: Atom<'a>,
        real_name: Atom<'a>,
        stmts: Vec<'a, Statement<'a>>,
        directives: Vec<'a, Directive<'a>>,
        parent_export: Option<Expression<'a>>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx,
    ) -> Statement<'a> {
        // `(function (_N) { var x; })(N || (N = {}))`;
        //  ^^^^^^^^^^^^^^^^^^^^^^^^^^
        let callee = {
            let body = self.ctx.ast.function_body(SPAN, directives, stmts);
            let params = {
                let ident = self.ctx.ast.binding_pattern_kind_binding_identifier(SPAN, arg_name);
                let pattern =
                    self.ctx.ast.binding_pattern(ident, Option::<TSTypeAnnotation>::None, false);
                let items =
                    self.ctx.ast.new_vec_single(self.ctx.ast.plain_formal_parameter(SPAN, pattern));
                self.ctx.ast.formal_parameters(
                    SPAN,
                    FormalParameterKind::FormalParameter,
                    items,
                    Option::<BindingRestElement>::None,
                )
            };
            let function = self.ctx.ast.plain_function(
                FunctionType::FunctionExpression,
                SPAN,
                None,
                params,
                Some(body),
            );
            function.scope_id.set(Some(scope_id));
            *ctx.scopes_mut().get_flags_mut(scope_id) =
                ScopeFlags::Function | ScopeFlags::StrictMode;
            let function_expr = self.ctx.ast.expression_from_function(function);
            self.ctx.ast.expression_parenthesized(SPAN, function_expr)
        };

        // (function (_N) { var M; (function (_M) { var x; })(M || (M = _N.M || (_N.M = {})));})(N || (N = {}));
        //                                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^    ^^^^^^^^^^^^^^
        //                                                   Nested namespace arguments         Normal namespace arguments
        let arguments = {
            // M
            let logical_left = self.ctx.ast.expression_identifier_reference(SPAN, &real_name);

            // (_N.M = {}) or (N = {})
            let mut logical_right = {
                // _N.M
                let assign_left = if let Some(parent_export) = self.ctx.ast.copy(&parent_export) {
                    self.ctx.ast.simple_assignment_target_member_expression(
                        self.ctx.ast.member_expression_static(
                            SPAN,
                            parent_export,
                            IdentifierName::new(SPAN, real_name.clone()),
                            false,
                        ),
                    )
                } else {
                    // _N
                    self.ctx
                        .ast
                        .simple_assignment_target_identifier_reference(SPAN, real_name.clone())
                };

                let assign_right =
                    self.ctx.ast.expression_object(SPAN, self.ctx.ast.new_vec(), None);
                let op = AssignmentOperator::Assign;
                let assign_expr = self.ctx.ast.expression_assignment(
                    SPAN,
                    op,
                    self.ctx.ast.assignment_target_simple(assign_left),
                    assign_right,
                );
                self.ctx.ast.expression_parenthesized(SPAN, assign_expr)
            };

            // (M = _N.M || (_N.M = {}))
            if let Some(parent_export) = parent_export {
                let assign_left =
                    self.ctx.ast.simple_assignment_target_identifier_reference(SPAN, &real_name);
                let assign_right = {
                    let property = IdentifierName::new(SPAN, real_name.clone());
                    let logical_left =
                        self.ctx.ast.member_expression_static(SPAN, parent_export, property, false);
                    let op = LogicalOperator::Or;
                    self.ctx.ast.expression_logical(SPAN, logical_left.into(), op, logical_right)
                };
                let op = AssignmentOperator::Assign;
                logical_right =
                    self.ctx.ast.expression_assignment(SPAN, op, assign_left.into(), assign_right);
                logical_right = self.ctx.ast.expression_parenthesized(SPAN, logical_right);
            }

            let op = LogicalOperator::Or;
            let expr = self.ctx.ast.expression_logical(SPAN, logical_left, op, logical_right);
            self.ctx.ast.new_vec_single(self.ctx.ast.argument_expression(expr))
        };

        let expr = self.ctx.ast.expression_call(
            SPAN,
            arguments,
            callee,
            Option::<TSTypeParameterInstantiation>::None,
            false,
        );
        self.ctx.ast.statement_expression(SPAN, expr)
    }

    /// Add assignment statement for decl id
    /// function id() {} -> function id() {}; Name.id = id;
    fn add_declaration(
        &self,
        decl: Declaration<'a>,
        name: Atom<'a>,
        names: &mut FxHashSet<Atom<'a>>,
        new_stmts: &mut Vec<'a, Statement<'a>>,
    ) {
        // This function is only called with a function, class, or enum declaration,
        // all of which are guaranteed to have an `id`
        let ident = decl.id().unwrap();
        let item_name = ident.name.clone();
        new_stmts.push(Statement::from(decl));
        let assignment_statement = self.create_assignment_statement(name, item_name.clone());
        let assignment_statement = self.ctx.ast.statement_expression(SPAN, assignment_statement);
        new_stmts.push(assignment_statement);
        names.insert(item_name);
    }

    // name.item_name = item_name
    fn create_assignment_statement(&self, name: Atom<'a>, item_name: Atom<'a>) -> Expression<'a> {
        let object = self.ctx.ast.expression_identifier_reference(SPAN, name);
        let property = self.ctx.ast.identifier_name(SPAN, &item_name);
        let left = self.ctx.ast.member_expression_static(SPAN, object, property, false);
        let left = AssignmentTarget::from(left);
        let right = self.ctx.ast.expression_identifier_reference(SPAN, item_name);
        let op = AssignmentOperator::Assign;
        self.ctx.ast.expression_assignment(SPAN, op, left, right)
    }

    /// Convert `export const foo = 1` to `Namespace.foo = 1`;
    #[allow(clippy::needless_pass_by_value)]
    fn handle_variable_declaration(
        &self,
        mut var_decl: Box<'a, VariableDeclaration<'a>>,
        name: Atom<'a>,
    ) -> Vec<'a, Statement<'a>> {
        let is_all_binding_identifier = var_decl
            .declarations
            .iter()
            .all(|declaration| declaration.id.kind.is_binding_identifier());

        // `export const a = 1` transforms to `const a = N.a = 1`, the output
        // is smaller than `const a = 1; N.a = a`;
        if is_all_binding_identifier {
            var_decl.declarations.iter_mut().for_each(|declarator| {
                let Some(property_name) = declarator.id.get_identifier() else {
                    return;
                };
                if let Some(init) = &declarator.init {
                    declarator.init = Some(
                        self.ctx.ast.expression_assignment(
                            SPAN,
                            AssignmentOperator::Assign,
                            self.ctx
                                .ast
                                .simple_assignment_target_member_expression(
                                    self.ctx.ast.member_expression_static(
                                        SPAN,
                                        self.ctx.ast.expression_identifier_reference(SPAN, &name),
                                        self.ctx.ast.identifier_name(SPAN, property_name),
                                        false,
                                    ),
                                )
                                .into(),
                            self.ctx.ast.copy(init),
                        ),
                    );
                }
            });
            return self.ctx.ast.new_vec_single(Statement::VariableDeclaration(var_decl));
        }

        // Now we have pattern in declarators
        // `export const [a] = 1` transforms to `const [a] = 1; N.a = a`
        let mut assignments = self.ctx.ast.new_vec();
        var_decl.bound_names(&mut |id| {
            assignments.push(self.create_assignment_statement(name.clone(), id.name.clone()));
        });

        let mut stmts = self.ctx.ast.new_vec_with_capacity(2);
        stmts.push(Statement::VariableDeclaration(var_decl));
        stmts.push(
            self.ctx
                .ast
                .statement_expression(SPAN, self.ctx.ast.expression_sequence(SPAN, assignments)),
        );
        stmts
    }
}

/// Check if the statements contain a namespace declaration
fn has_namespace(stmts: &[Statement]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Statement::TSModuleDeclaration(_) => true,
        Statement::ExportNamedDeclaration(decl) => {
            matches!(decl.declaration, Some(Declaration::TSModuleDeclaration(_)))
        }
        _ => false,
    })
}
