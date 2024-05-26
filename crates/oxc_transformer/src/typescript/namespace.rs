use rustc_hash::FxHashSet;

use super::{diagnostics::ambient_module_nested, TypeScript};

use oxc_allocator::{Box, Vec};
use oxc_ast::{ast::*, syntax_directed_operations::BoundNames};
use oxc_span::{Atom, CompactStr, SPAN};
use oxc_syntax::{
    operator::{AssignmentOperator, LogicalOperator},
    symbol::SymbolFlags,
};
use oxc_traverse::TraverseCtx;

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

        // Collect all binding names. Such as function name and class name.
        let mut names: FxHashSet<Atom<'a>> = FxHashSet::default();

        // Recreate the statements vec for memory efficiency.
        // Inserting the `let` declaration multiple times will reallocate the whole statements vec
        // every time a namespace declaration is encountered.
        let mut new_stmts = self.ctx.ast.new_vec();

        for stmt in self.ctx.ast.move_statement_vec(&mut program.body) {
            match stmt {
                Statement::TSModuleDeclaration(decl) => {
                    if !decl.modifiers.is_contains_declare() {
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
                }
                match_module_declaration!(Statement) => {
                    if let Statement::ExportNamedDeclaration(export_decl) = &stmt {
                        if let Some(Declaration::TSModuleDeclaration(decl)) =
                            &export_decl.declaration
                        {
                            if !decl.modifiers.is_contains_declare() {
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
                                        let export_named_decl =
                                            ModuleDeclaration::ExportNamedDeclaration(
                                                export_named_decl,
                                            );
                                        let stmt =
                                            self.ctx.ast.module_declaration(export_named_decl);
                                        new_stmts.push(stmt);
                                    }
                                    new_stmts.push(transformed_stmt);
                                    continue;
                                }
                            }
                        }
                    }

                    stmt.to_module_declaration().bound_names(&mut |id| {
                        names.insert(id.name.clone());
                    });
                    new_stmts.push(stmt);
                }
                // Collect bindings from class, function, variable and enum declarations
                Statement::FunctionDeclaration(ref decl) => {
                    if let Some(ident) = &decl.id {
                        names.insert(ident.name.clone());
                    }
                    new_stmts.push(stmt);
                }
                Statement::ClassDeclaration(ref decl) => {
                    if let Some(ident) = &decl.id {
                        names.insert(ident.name.clone());
                    }
                    new_stmts.push(stmt);
                }
                Statement::TSEnumDeclaration(ref decl) => {
                    names.insert(decl.id.name.clone());
                    new_stmts.push(stmt);
                }
                Statement::VariableDeclaration(ref decl) => {
                    decl.bound_names(&mut |id| {
                        names.insert(id.name.clone());
                    });
                    new_stmts.push(stmt);
                }
                _ => {
                    new_stmts.push(stmt);
                }
            }
        }

        program.body = new_stmts;
    }

    fn handle_nested(
        &self,
        decl: TSModuleDeclaration<'a>,
        parent_export: Option<Expression<'a>>,
        ctx: &mut TraverseCtx,
    ) -> Option<Statement<'a>> {
        let mut names: FxHashSet<Atom<'a>> = FxHashSet::default();

        let real_name = decl.id.name();

        // TODO: This binding is created in wrong scope.
        // Needs to be created in scope of function which `transform_namespace` creates below.
        let name = self.ctx.ast.new_atom(
            &ctx.generate_uid_in_current_scope(real_name, SymbolFlags::FunctionScopedVariable),
        );

        let namespace_top_level = if let Some(body) = decl.body {
            match body {
                TSModuleDeclarationBody::TSModuleBlock(mut block) => {
                    self.ctx.ast.move_statement_vec(&mut block.body)
                }
                // We handle `namespace X.Y {}` as if it was
                //   namespace X {
                //     export namespace Y {}
                //   }
                TSModuleDeclarationBody::TSModuleDeclaration(declaration) => {
                    let declaration =
                        Declaration::TSModuleDeclaration(self.ctx.ast.copy(&declaration));
                    let export_named_decl =
                        self.ctx.ast.plain_export_named_declaration_declaration(SPAN, declaration);
                    let stmt = self.ctx.ast.module_declaration(
                        ModuleDeclaration::ExportNamedDeclaration(export_named_decl),
                    );
                    self.ctx.ast.new_vec_single(stmt)
                }
            }
        } else {
            self.ctx.ast.new_vec()
        };

        let mut is_empty = true;
        let mut new_stmts = self.ctx.ast.new_vec();

        for stmt in namespace_top_level {
            match stmt {
                Statement::TSModuleDeclaration(decl) => {
                    if decl.id.is_string_literal() {
                        self.ctx.error(ambient_module_nested(decl.span));
                    }

                    let module_name = decl.id.name().clone();
                    if let Some(transformed) = self.handle_nested(decl.unbox(), None, ctx) {
                        is_empty = false;
                        if names.insert(module_name.clone()) {
                            new_stmts.push(Statement::from(
                                self.create_variable_declaration(&module_name),
                            ));
                        }
                        new_stmts.push(transformed);
                    }
                }
                Statement::ClassDeclaration(decl) => {
                    is_empty = false;
                    decl.bound_names(&mut |id| {
                        names.insert(id.name.clone());
                    });
                    new_stmts.push(Statement::ClassDeclaration(decl));
                }
                Statement::TSEnumDeclaration(enum_decl) => {
                    is_empty = false;
                    names.insert(enum_decl.id.name.clone());
                    new_stmts.push(Statement::TSEnumDeclaration(enum_decl));
                }
                Statement::ExportNamedDeclaration(export_decl) => {
                    let export_decl = export_decl.unbox();
                    if let Some(decl) = export_decl.declaration {
                        if decl.modifiers().is_some_and(Modifiers::is_contains_declare) {
                            continue;
                        }
                        match decl {
                            Declaration::TSEnumDeclaration(enum_decl) => {
                                is_empty = false;
                                self.add_declaration(
                                    Declaration::TSEnumDeclaration(enum_decl),
                                    &name,
                                    &mut names,
                                    &mut new_stmts,
                                );
                            }
                            Declaration::FunctionDeclaration(func_decl) => {
                                is_empty = false;
                                self.add_declaration(
                                    Declaration::FunctionDeclaration(func_decl),
                                    &name,
                                    &mut names,
                                    &mut new_stmts,
                                );
                            }
                            Declaration::ClassDeclaration(class_decl) => {
                                is_empty = false;
                                self.add_declaration(
                                    Declaration::ClassDeclaration(class_decl),
                                    &name,
                                    &mut names,
                                    &mut new_stmts,
                                );
                            }
                            Declaration::VariableDeclaration(var_decl) => {
                                is_empty = false;
                                let stmts = self.handle_variable_declaration(var_decl, &name);
                                new_stmts.extend(stmts);
                            }
                            Declaration::TSModuleDeclaration(module_decl) => {
                                if module_decl.id.is_string_literal() {
                                    self.ctx.error(ambient_module_nested(module_decl.span));
                                }

                                let module_name = module_decl.id.name().clone();
                                if let Some(transformed) = self.handle_nested(
                                    module_decl.unbox(),
                                    Some(self.ctx.ast.identifier_reference_expression(
                                        IdentifierReference::new(SPAN, name.clone()),
                                    )),
                                    ctx,
                                ) {
                                    is_empty = false;
                                    if names.insert(module_name.clone()) {
                                        new_stmts.push(Statement::from(
                                            self.create_variable_declaration(&module_name),
                                        ));
                                    }
                                    new_stmts.push(transformed);
                                }
                            }
                            _ => {}
                        }
                    } else {
                        let stmt = self.ctx.ast.module_declaration(
                            ModuleDeclaration::ExportNamedDeclaration(
                                self.ctx.ast.alloc(export_decl),
                            ),
                        );
                        new_stmts.push(stmt);
                    }
                }
                stmt => {
                    if let Some(decl) = stmt.as_declaration() {
                        if decl.is_typescript_syntax() {
                            continue;
                        }
                    }
                    is_empty = false;
                    new_stmts.push(stmt);
                }
            }
        }

        if is_empty {
            // Delete the scope binding that `ctx.generate_uid_in_current_scope` created above,
            // as no binding is actually being created
            let current_scope_id = ctx.current_scope_id();
            ctx.scopes_mut().remove_binding(current_scope_id, &CompactStr::from(name.as_str()));

            return None;
        }

        Some(self.transform_namespace(&name, real_name, new_stmts, parent_export))
    }

    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    //                         ^^^^^^^
    fn create_variable_declaration(&self, name: &Atom<'a>) -> Declaration<'a> {
        let kind = VariableDeclarationKind::Let;
        let declarator = {
            let ident = BindingIdentifier::new(SPAN, name.clone());
            let pattern_kind = self.ctx.ast.binding_pattern_identifier(ident);
            let binding = self.ctx.ast.binding_pattern(pattern_kind, None, false);
            let decl = self.ctx.ast.variable_declarator(SPAN, kind, binding, None, false);
            self.ctx.ast.new_vec_single(decl)
        };
        Declaration::VariableDeclaration(self.ctx.ast.variable_declaration(
            SPAN,
            kind,
            declarator,
            Modifiers::empty(),
        ))
    }

    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    //                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    fn transform_namespace(
        &self,
        arg_name: &Atom<'a>,
        real_name: &Atom<'a>,
        stmts: Vec<'a, Statement<'a>>,
        parent_export: Option<Expression<'a>>,
    ) -> Statement<'a> {
        // `(function (_N) { var x; })(N || (N = {}))`;
        //  ^^^^^^^^^^^^^^^^^^^^^^^^^^
        let callee = {
            let body = self.ctx.ast.function_body(SPAN, self.ctx.ast.new_vec(), stmts);
            let params = {
                let ident = self.ctx.ast.binding_pattern_identifier(BindingIdentifier::new(
                    SPAN,
                    self.ctx.ast.new_atom(arg_name),
                ));
                let pattern = self.ctx.ast.binding_pattern(ident, None, false);
                let items =
                    self.ctx.ast.new_vec_single(self.ctx.ast.plain_formal_parameter(SPAN, pattern));
                self.ctx.ast.formal_parameters(
                    SPAN,
                    FormalParameterKind::FormalParameter,
                    items,
                    None,
                )
            };
            let function = self.ctx.ast.plain_function(
                FunctionType::FunctionExpression,
                SPAN,
                None,
                params,
                Some(body),
            );
            let function_expr = self.ctx.ast.function_expression(function);
            self.ctx.ast.parenthesized_expression(SPAN, function_expr)
        };

        // (function (_N) { var M; (function (_M) { var x; })(M || (M = _N.M || (_N.M = {})));})(N || (N = {}));
        //                                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^    ^^^^^^^^^^^^^^
        //                                                   Nested namespace arguments         Normal namespace arguments
        let arguments = {
            // M
            let logical_left = {
                let ident = IdentifierReference::new(SPAN, real_name.clone());
                self.ctx.ast.identifier_reference_expression(ident)
            };

            // (_N.M = {}) or (N = {})
            let mut logical_right = {
                // _N.M
                let assign_left = if let Some(parent_export) = self.ctx.ast.copy(&parent_export) {
                    self.ctx.ast.simple_assignment_target_member_expression(
                        self.ctx.ast.static_member(
                            SPAN,
                            parent_export,
                            self.ctx.ast.identifier_name(SPAN, real_name),
                            false,
                        ),
                    )
                } else {
                    // _N
                    self.ctx.ast.simple_assignment_target_identifier(IdentifierReference::new(
                        SPAN,
                        real_name.clone(),
                    ))
                };

                let assign_right =
                    self.ctx.ast.object_expression(SPAN, self.ctx.ast.new_vec(), None);
                let op = AssignmentOperator::Assign;
                let assign_expr =
                    self.ctx.ast.assignment_expression(SPAN, op, assign_left, assign_right);
                self.ctx.ast.parenthesized_expression(SPAN, assign_expr)
            };

            // (M = _N.M || (_N.M = {}))
            if let Some(parent_export) = parent_export {
                let assign_left = self.ctx.ast.simple_assignment_target_identifier(
                    IdentifierReference::new(SPAN, real_name.clone()),
                );
                let assign_right = {
                    let property = self.ctx.ast.identifier_name(SPAN, real_name);
                    let logical_left =
                        self.ctx.ast.static_member_expression(SPAN, parent_export, property, false);
                    let op = LogicalOperator::Or;
                    self.ctx.ast.logical_expression(SPAN, logical_left, op, logical_right)
                };
                let op = AssignmentOperator::Assign;
                logical_right =
                    self.ctx.ast.assignment_expression(SPAN, op, assign_left, assign_right);
                logical_right = self.ctx.ast.parenthesized_expression(SPAN, logical_right);
            }

            let op = LogicalOperator::Or;
            let expr = self.ctx.ast.logical_expression(SPAN, logical_left, op, logical_right);
            self.ctx.ast.new_vec_single(Argument::from(expr))
        };

        let expr = self.ctx.ast.call_expression(SPAN, callee, arguments, false, None);
        self.ctx.ast.expression_statement(SPAN, expr)
    }

    /// Add assignment statement for decl id
    /// function id() {} -> function id() {}; Name.id = id;
    fn add_declaration(
        &self,
        decl: Declaration<'a>,
        name: &Atom<'a>,
        names: &mut FxHashSet<Atom<'a>>,
        new_stmts: &mut Vec<'a, Statement<'a>>,
    ) {
        if let Some(ident) = decl.id() {
            let item_name = ident.name.clone();
            let assignment_statement = self.create_assignment_statement(name, &item_name);
            new_stmts.push(Statement::from(decl));
            let assignment_statement =
                self.ctx.ast.expression_statement(SPAN, assignment_statement);
            new_stmts.push(assignment_statement);
            names.insert(item_name);
        }
    }

    // name.item_name = item_name
    fn create_assignment_statement(&self, name: &Atom<'a>, item_name: &Atom<'a>) -> Expression<'a> {
        let ident = self.ctx.ast.identifier_reference(SPAN, name.as_str());
        let object = self.ctx.ast.identifier_reference_expression(ident);
        let property = IdentifierName::new(SPAN, item_name.clone());
        let left = self.ctx.ast.static_member(SPAN, object, property, false);
        let left = AssignmentTarget::from(left);
        let ident = self.ctx.ast.identifier_reference(SPAN, item_name.as_str());
        let right = self.ctx.ast.identifier_reference_expression(ident);
        let op = AssignmentOperator::Assign;
        self.ctx.ast.assignment_expression(SPAN, op, left, right)
    }

    /// Convert `export const foo = 1` to `Namespace.foo = 1`;
    fn handle_variable_declaration(
        &self,
        mut var_decl: Box<'a, VariableDeclaration<'a>>,
        name: &Atom<'a>,
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
                    declarator.init = Some(self.ctx.ast.assignment_expression(
                        SPAN,
                        AssignmentOperator::Assign,
                        self.ctx.ast.simple_assignment_target_member_expression(
                            self.ctx.ast.static_member(
                                SPAN,
                                self.ctx.ast.identifier_reference_expression(
                                    IdentifierReference::new(SPAN, name.clone()),
                                ),
                                IdentifierName::new(SPAN, property_name.clone()),
                                false,
                            ),
                        ),
                        self.ctx.ast.copy(init),
                    ));
                }
            });
            return self.ctx.ast.new_vec_single(Statement::VariableDeclaration(var_decl));
        }

        // Now we have pattern in declarators
        // `export const [a] = 1` transforms to `const [a] = 1; N.a = a`
        let mut assignments = self.ctx.ast.new_vec();
        var_decl.bound_names(&mut |id| {
            assignments.push(self.create_assignment_statement(name, &id.name));
        });

        let mut stmts = self.ctx.ast.new_vec_with_capacity(2);
        stmts.push(Statement::VariableDeclaration(var_decl));
        stmts.push(
            self.ctx
                .ast
                .expression_statement(SPAN, self.ctx.ast.sequence_expression(SPAN, assignments)),
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
