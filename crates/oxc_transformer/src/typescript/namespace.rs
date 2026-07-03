use oxc_allocator::{ArenaBox, ArenaVec, TakeIn};
use oxc_ast::{ast::*, builder::NONE};
use oxc_ecmascript::BoundNames;
use oxc_span::{SPAN, Span};
use oxc_syntax::{
    operator::{AssignmentOperator, LogicalOperator},
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
};
use oxc_traverse::{BoundIdentifier, Traverse};

use crate::{context::TraverseCtx, state::TransformState};

use super::{
    TypeScriptOptions,
    diagnostics::{ambient_module_nested, namespace_exporting_non_const, namespace_not_supported},
};

pub struct TypeScriptNamespace {
    // Options
    allow_namespaces: bool,
}

impl TypeScriptNamespace {
    pub fn new(options: &TypeScriptOptions) -> Self {
        Self { allow_namespaces: options.allow_namespaces }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for TypeScriptNamespace {
    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        // namespace declaration is only allowed at the top level
        if !has_namespace(program.body.as_slice()) {
            return;
        }

        // Recreate the statements vec for memory efficiency.
        // Inserting the `let` declaration multiple times will reallocate the whole statements vec
        // every time a namespace declaration is encountered. Pre-size to the current body length
        // (a lower bound — namespaces expand) to avoid the growth reallocations for the common
        // pass-through statements.
        let mut new_stmts = ArenaVec::with_capacity_in(program.body.len(), ctx);

        for stmt in program.body.take_in(ctx) {
            match stmt {
                Statement::TSModuleDeclaration(decl) => {
                    if !self.allow_namespaces {
                        ctx.state.error(namespace_not_supported(decl.span));
                    }

                    self.handle_nested(decl, /* is_export */ false, &mut new_stmts, None, ctx);
                    continue;
                }
                Statement::TSGlobalDeclaration(decl) => {
                    if !self.allow_namespaces {
                        ctx.state.error(namespace_not_supported(decl.span));
                    }
                    continue;
                }
                Statement::ExportNamedDeclaration(export_decl)
                    if export_decl.declaration.as_ref().is_some_and(|declaration| {
                        // Note: No need to check for `TSGlobalDeclaration` here, as it can't be exported
                        debug_assert!(!matches!(declaration, Declaration::TSGlobalDeclaration(_)));
                        matches!(declaration, Declaration::TSModuleDeclaration(module_decl) if !module_decl.declare)
                    }) =>
                {
                    let Some(Declaration::TSModuleDeclaration(decl)) =
                        export_decl.unbox().declaration
                    else {
                        unreachable!()
                    };

                    if !self.allow_namespaces {
                        ctx.state.error(namespace_not_supported(decl.span));
                    }

                    self.handle_nested(decl, /* is_export */ true, &mut new_stmts, None, ctx);
                    continue;
                }
                _ => {}
            }

            new_stmts.push(stmt);
        }

        program.body = new_stmts;
    }
}

impl<'a> TypeScriptNamespace {
    #[expect(clippy::self_only_used_in_recursion)]
    fn handle_nested(
        &self,
        decl: ArenaBox<'a, TSModuleDeclaration<'a>>,
        is_export: bool,
        parent_stmts: &mut ArenaVec<'a, Statement<'a>>,
        parent_binding: Option<&BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if decl.declare {
            if let TSModuleDeclarationName::Identifier(ident) = &decl.id {
                Self::remove_binding(ident, ctx);
            }
            return;
        }

        // Skip empty declaration e.g. `namespace x;`
        let TSModuleDeclaration { span, id, body, scope_id, .. } = decl.unbox();

        let TSModuleDeclarationName::Identifier(ident) = id else {
            ctx.state.error(ambient_module_nested(span));
            return;
        };

        // Check if this is an empty namespace or only contains type declarations
        let symbol_id = ident.symbol_id();
        let flags = ctx.scoping().symbol_flags(symbol_id);

        // If it's a namespace, we need additional checks to determine if it can return early.
        if flags.is_namespace_module() {
            // Don't need further check because NO `ValueModule` namespace redeclaration
            if !flags.is_value_module() {
                Self::remove_binding(&ident, ctx);
                return;
            }

            // Input:
            // ```ts
            // // SymbolFlags: NameSpaceModule
            // export namespace Foo {
            // 	 export type T = 0;
            // }
            // // SymbolFlags: ValueModule
            // export namespace Foo {
            // 	 export const Bar = 1;
            // }
            // ```
            //
            // Output:
            // ```js
            // // SymbolFlags: ValueModule
            // export let Foo;
            // (function(_Foo) {
            //   const Bar = _Foo.Bar = 1;
            // })(Foo || (Foo = {}));
            // ```
            //
            // When both `NameSpaceModule` and `ValueModule` are present, we need to check the current
            // declaration flags. If the current declaration is `NameSpaceModule`, we can return early
            // because it's a type-only namespace and doesn't emit any JS code, otherwise we need to
            // continue transforming it.

            // Find the current declaration flag
            let current_declaration_flags = ctx
                .scoping()
                .symbol_redeclarations(symbol_id)
                .iter()
                .find(|rd| rd.span == ident.span)
                .unwrap()
                .flags;

            // Return if the current declaration is a namespace
            if current_declaration_flags.is_namespace_module() {
                Self::remove_binding(&ident, ctx);
                return;
            }
        }

        let Some(body) = body else {
            return;
        };

        let binding = BoundIdentifier::from_binding_ident(&ident);
        let is_redeclaration_namespace = Self::is_redeclaration_namespace(&ident, ctx);
        Self::sync_namespace_binding(&ident, is_redeclaration_namespace, ctx);

        // Reuse `TSModuleDeclaration`'s scope in transformed function
        let scope_id = scope_id.get().unwrap();
        let uid_binding =
            ctx.generate_uid(&binding.name, scope_id, SymbolFlags::FunctionScopedVariable);

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
                    ExportNamedDeclaration::boxed_plain_declaration(SPAN, declaration, ctx);
                let stmt = Statement::ExportNamedDeclaration(export_named_decl);
                directives = ArenaVec::new_in(ctx);
                namespace_top_level = ArenaVec::from_value_in(stmt, ctx);
            }
        }

        let mut new_stmts = ArenaVec::new_in(ctx);

        for stmt in namespace_top_level {
            match stmt {
                Statement::TSModuleDeclaration(decl) => {
                    self.handle_nested(decl, /* is_export */ false, &mut new_stmts, None, ctx);
                }
                Statement::TSGlobalDeclaration(_) => {
                    // Remove it.
                    // Note: It is legal to have a `TSGlobalDeclaration` nested within a `TSModuleDeclaration`,
                    // where identifier is a string literal: `declare module 'foo' { global {} }`
                }
                Statement::ExportNamedDeclaration(export_decl) => {
                    // NB: `ExportNamedDeclaration` with no declaration (e.g. `export {x}`) is not
                    // legal syntax in TS namespaces
                    let export_decl = export_decl.unbox();
                    if let Some(decl) = export_decl.declaration {
                        if decl.declare() {
                            Self::remove_declaration_bindings(&decl, ctx);
                            continue;
                        }
                        match decl {
                            Declaration::TSImportEqualsDeclaration(ref import_equals) => {
                                let binding =
                                    BoundIdentifier::from_binding_ident(&import_equals.id);
                                new_stmts.push(Statement::from(decl));
                                Self::add_declaration(&uid_binding, &binding, &mut new_stmts, ctx);
                            }
                            Declaration::TSEnumDeclaration(ref enum_decl) => {
                                let binding = BoundIdentifier::from_binding_ident(&enum_decl.id);
                                new_stmts.push(Statement::from(decl));
                                Self::add_declaration(&uid_binding, &binding, &mut new_stmts, ctx);
                            }
                            Declaration::ClassDeclaration(ref class_decl) => {
                                // Class declaration always has a binding
                                let binding = BoundIdentifier::from_binding_ident(
                                    class_decl.id.as_ref().unwrap(),
                                );
                                new_stmts.push(Statement::from(decl));
                                Self::add_declaration(&uid_binding, &binding, &mut new_stmts, ctx);
                            }
                            Declaration::FunctionDeclaration(ref func_decl) => {
                                if !func_decl.is_typescript_syntax() {
                                    // Function declaration always has a binding
                                    let binding = BoundIdentifier::from_binding_ident(
                                        func_decl.id.as_ref().unwrap(),
                                    );
                                    new_stmts.push(Statement::from(decl));
                                    Self::add_declaration(
                                        &uid_binding,
                                        &binding,
                                        &mut new_stmts,
                                        ctx,
                                    );
                                }
                            }
                            Declaration::VariableDeclaration(var_decl) => {
                                var_decl.declarations.iter().for_each(|decl| {
                                    if !decl.kind.is_const() {
                                        ctx.state.error(namespace_exporting_non_const(decl.span));
                                    }
                                });
                                let stmts =
                                    Self::handle_variable_declaration(var_decl, &uid_binding, ctx);
                                new_stmts.extend(stmts);
                            }
                            Declaration::TSModuleDeclaration(module_decl) => {
                                self.handle_nested(
                                    module_decl,
                                    /* is_export */
                                    false,
                                    &mut new_stmts,
                                    Some(&uid_binding),
                                    ctx,
                                );
                            }
                            Declaration::TSTypeAliasDeclaration(_)
                            | Declaration::TSInterfaceDeclaration(_)
                            | Declaration::TSGlobalDeclaration(_) => {}
                        }
                    }
                }
                _ => new_stmts.push(stmt),
            }
        }

        if !is_redeclaration_namespace {
            ctx.state.emitted_namespace_bindings.push(symbol_id);
            let declaration = Self::create_variable_declaration(&binding, span, ctx);
            if is_export {
                let export_named_decl =
                    ExportNamedDeclaration::boxed_plain_declaration(span, declaration, ctx);
                let stmt = Statement::ExportNamedDeclaration(export_named_decl);
                parent_stmts.push(stmt);
            } else {
                parent_stmts.push(Statement::from(declaration));
            }
        }
        let func_body = FunctionBody::new(SPAN, directives, new_stmts, ctx);

        parent_stmts.push(Self::transform_namespace(
            span,
            &uid_binding,
            &binding,
            parent_binding,
            func_body,
            scope_id,
            ctx,
        ));
        let redeclarations = ctx.scoping().symbol_redeclarations(symbol_id);
        if !redeclarations
            .iter()
            .any(|redeclaration| redeclaration.flags.intersects(SymbolFlags::Enum))
            && redeclarations.last().is_some_and(|redeclaration| redeclaration.span == ident.span)
        {
            ctx.scoping_mut().clear_symbol_redeclarations(symbol_id);
        }
    }

    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    //                         ^^^^^^^
    fn create_variable_declaration(
        binding: &BoundIdentifier<'a>,
        span: Span,
        ctx: &TraverseCtx<'a>,
    ) -> Declaration<'a> {
        let kind = VariableDeclarationKind::Let;
        let declarations = {
            let pattern = binding.create_binding_pattern(ctx);
            let decl = VariableDeclarator::new(span, kind, pattern, NONE, None, false, ctx);
            ArenaVec::from_value_in(decl, ctx)
        };
        Declaration::new_variable_declaration(span, kind, declarations, false, ctx)
    }

    // `namespace Foo { }` -> `let Foo; (function (_Foo) { })(Foo || (Foo = {}));`
    fn transform_namespace(
        span: Span,
        param_binding: &BoundIdentifier<'a>,
        binding: &BoundIdentifier<'a>,
        parent_binding: Option<&BoundIdentifier<'a>>,
        func_body: FunctionBody<'a>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        // `(function (_N) { var x; })(N || (N = {}))`;
        //  ^^^^^^^^^^^^^^^^^^^^^^^^^^
        let callee = {
            let mut scope_flags = ScopeFlags::Function;
            if ctx.current_scope_flags().is_strict_mode() || func_body.has_use_strict_directive() {
                scope_flags |= ScopeFlags::StrictMode;
            }
            let params = {
                let pattern = param_binding.create_binding_pattern(ctx);
                let items =
                    ArenaVec::from_value_in(FormalParameter::new_plain(SPAN, pattern, ctx), ctx);
                FormalParameters::new(SPAN, FormalParameterKind::FormalParameter, items, NONE, ctx)
            };
            let function_expr =
                Expression::FunctionExpression(Function::boxed_plain_with_scope_id(
                    FunctionType::FunctionExpression,
                    span,
                    None,
                    params,
                    func_body,
                    scope_id,
                    ctx,
                ));
            *ctx.scoping_mut().scope_flags_mut(scope_id) = scope_flags;
            Expression::new_parenthesized_expression(span, function_expr, ctx)
        };

        // (function (_N) { var M; (function (_M) { var x; })(M || (M = _N.M || (_N.M = {})));})(N || (N = {}));
        //                                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^    ^^^^^^^^^^^^^^
        //                                                   Nested namespace arguments         Normal namespace arguments
        let arguments = {
            // M
            let logical_left = binding.create_read_expression(ctx);

            // (_N.M = {}) or (N = {})
            let mut logical_right = {
                // _N.M
                let assign_left = if let Some(parent_binding) = parent_binding {
                    AssignmentTarget::new_static_member_expression(
                        SPAN,
                        parent_binding.create_read_expression(ctx),
                        IdentifierName::new(SPAN, binding.name, ctx),
                        false,
                        ctx,
                    )
                } else {
                    // _N
                    binding.create_write_target(ctx)
                };

                let assign_right =
                    Expression::new_object_expression(SPAN, ArenaVec::new_in(ctx), ctx);
                let op = AssignmentOperator::Assign;
                let assign_expr =
                    Expression::new_assignment_expression(SPAN, op, assign_left, assign_right, ctx);
                Expression::new_parenthesized_expression(SPAN, assign_expr, ctx)
            };

            // (M = _N.M || (_N.M = {}))
            if let Some(parent_binding) = parent_binding {
                let assign_left = binding.create_write_target(ctx);
                let assign_right = {
                    let property = IdentifierName::new(SPAN, binding.name, ctx);
                    let logical_left = MemberExpression::new_static_member_expression(
                        SPAN,
                        parent_binding.create_read_expression(ctx),
                        property,
                        false,
                        ctx,
                    );
                    let op = LogicalOperator::Or;
                    Expression::new_logical_expression(
                        SPAN,
                        logical_left.into(),
                        op,
                        logical_right,
                        ctx,
                    )
                };
                let op = AssignmentOperator::Assign;
                logical_right =
                    Expression::new_assignment_expression(SPAN, op, assign_left, assign_right, ctx);
                logical_right = Expression::new_parenthesized_expression(SPAN, logical_right, ctx);
            }

            let expr = Expression::new_logical_expression(
                SPAN,
                logical_left,
                LogicalOperator::Or,
                logical_right,
                ctx,
            );
            ArenaVec::from_value_in(Argument::from(expr), ctx)
        };

        let expr = Expression::new_call_expression(span, callee, NONE, arguments, false, ctx);
        Statement::new_expression_statement(span, expr, ctx)
    }

    fn remove_declaration_bindings(decl: &Declaration<'a>, ctx: &mut TraverseCtx<'a>) {
        match decl {
            Declaration::TSEnumDeclaration(enum_decl) => Self::remove_binding(&enum_decl.id, ctx),
            Declaration::TSModuleDeclaration(module_decl) => {
                if let TSModuleDeclarationName::Identifier(ident) = &module_decl.id {
                    Self::remove_binding(ident, ctx);
                }
            }
            _ => decl.bound_names(&mut |id| Self::remove_binding(id, ctx)),
        }
    }

    /// Add assignment statement for decl id
    /// function id() {} -> function id() {}; Name.id = id;
    fn add_declaration(
        namespace_binding: &BoundIdentifier<'a>,
        value_binding: &BoundIdentifier<'a>,
        new_stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let assignment_statement =
            Self::create_assignment_statement(namespace_binding, value_binding, ctx);
        let assignment_statement =
            Statement::new_expression_statement(SPAN, assignment_statement, ctx);
        new_stmts.push(assignment_statement);
    }

    // parent_binding.binding = binding
    fn create_assignment_statement(
        object_binding: &BoundIdentifier<'a>,
        value_binding: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let object = object_binding.create_read_expression(ctx);
        let property = IdentifierName::new(SPAN, value_binding.name, ctx);
        let left =
            MemberExpression::new_static_member_expression(SPAN, object, property, false, ctx);
        let left = AssignmentTarget::from(left);
        let right = value_binding.create_read_expression(ctx);
        let op = AssignmentOperator::Assign;
        Expression::new_assignment_expression(SPAN, op, left, right, ctx)
    }

    /// Convert `export const foo = 1` to `Namespace.foo = 1`;
    fn handle_variable_declaration(
        mut var_decl: ArenaBox<'a, VariableDeclaration<'a>>,
        binding: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaVec<'a, Statement<'a>> {
        let is_all_binding_identifier =
            var_decl.declarations.iter().all(|declaration| declaration.id.is_binding_identifier());

        // `export const a = 1` transforms to `const a = N.a = 1`, the output
        // is smaller than `const a = 1; N.a = a`;
        if is_all_binding_identifier {
            Self::sync_variable_declaration_bindings(&var_decl, ctx);
            var_decl.declarations.iter_mut().for_each(|declarator| {
                let Some(property_name) = declarator.id.get_identifier_name() else {
                    return;
                };
                if let Some(init) = &mut declarator.init {
                    declarator.init = Some(Expression::new_assignment_expression(
                        SPAN,
                        AssignmentOperator::Assign,
                        SimpleAssignmentTarget::new_static_member_expression(
                            SPAN,
                            binding.create_read_expression(ctx),
                            IdentifierName::new(SPAN, property_name, ctx),
                            false,
                            ctx,
                        )
                        .into(),
                        init.take_in(ctx),
                        ctx,
                    ));
                }
            });
            return ArenaVec::from_value_in(Statement::VariableDeclaration(var_decl), ctx);
        }

        // Now we have pattern in declarators
        // `export const [a] = 1` transforms to `const [a] = 1; N.a = a`
        let mut assignments = ArenaVec::new_in(ctx);
        var_decl.bound_names(&mut |id| {
            assignments.push(Self::create_assignment_statement(
                binding,
                &BoundIdentifier::from_binding_ident(id),
                ctx,
            ));
        });

        ArenaVec::from_array_in(
            [
                Statement::VariableDeclaration(var_decl),
                Statement::new_expression_statement(
                    SPAN,
                    Expression::new_sequence_expression(SPAN, assignments, ctx),
                    ctx,
                ),
            ],
            ctx,
        )
    }

    fn sync_variable_declaration_bindings(
        var_decl: &VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let flags = match var_decl.kind {
            VariableDeclarationKind::Var => SymbolFlags::FunctionScopedVariable,
            VariableDeclarationKind::Let => SymbolFlags::BlockScopedVariable,
            VariableDeclarationKind::Const => {
                SymbolFlags::BlockScopedVariable | SymbolFlags::ConstVariable
            }
            VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing => return,
        };

        for decl in &var_decl.declarations {
            for ident in decl.id.get_binding_identifiers() {
                let symbol_id = ident.symbol_id();
                let has_type_metadata = {
                    let scoping = ctx.scoping();
                    scoping.symbol_flags(symbol_id).is_type()
                        || scoping
                            .symbol_redeclarations(symbol_id)
                            .iter()
                            .any(|redeclaration| redeclaration.flags.is_type())
                };
                if !has_type_metadata {
                    continue;
                }

                *ctx.scoping_mut().symbol_flags_mut(symbol_id) = flags;
                ctx.scoping_mut().set_symbol_span(symbol_id, ident.span);
                ctx.scoping_mut().clear_symbol_redeclarations(symbol_id);
            }
        }
    }

    /// Check the namespace binding identifier if it is a redeclaration
    fn is_redeclaration_namespace(id: &BindingIdentifier<'a>, ctx: &TraverseCtx<'a>) -> bool {
        let symbol_id = id.symbol_id();
        let redeclarations = ctx.scoping().symbol_redeclarations(symbol_id);
        // Find first value declaration because only value declaration will emit JS code.
        redeclarations.iter().find(|rd| rd.flags.is_value()).is_some_and(|rd| rd.span != id.span)
    }

    fn sync_namespace_binding(
        id: &BindingIdentifier<'a>,
        is_redeclaration_namespace: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let symbol_id = id.symbol_id();
        if is_redeclaration_namespace
            && let Some(redeclaration) =
                ctx.scoping().symbol_redeclarations(symbol_id).iter().find(|redeclaration| {
                    redeclaration
                        .flags
                        .intersects(SymbolFlags::Class | SymbolFlags::Function | SymbolFlags::Enum)
                        && !redeclaration.flags.is_ambient()
                })
        {
            let span = redeclaration.span;
            let flags = redeclaration.flags;
            *ctx.scoping_mut().symbol_flags_mut(symbol_id) = flags;
            ctx.scoping_mut().set_symbol_span(symbol_id, span);
            return;
        }

        *ctx.scoping_mut().symbol_flags_mut(symbol_id) = SymbolFlags::BlockScopedVariable;
        ctx.scoping_mut().set_symbol_span(symbol_id, SPAN);
    }

    fn remove_binding(id: &BindingIdentifier<'a>, ctx: &mut TraverseCtx<'a>) {
        let symbol_id = id.symbol_id();
        let has_non_ambient_value_module_redeclaration =
            ctx.scoping().symbol_redeclarations(symbol_id).iter().any(|redeclaration| {
                redeclaration.flags.is_value_module() && !redeclaration.flags.is_ambient()
            });
        if !has_non_ambient_value_module_redeclaration {
            if let Some(redeclaration) =
                ctx.scoping().symbol_redeclarations(symbol_id).iter().find(|redeclaration| {
                    redeclaration.flags.intersects(SymbolFlags::Class | SymbolFlags::Function)
                        && !redeclaration.flags.is_ambient()
                })
            {
                let span = redeclaration.span;
                let flags = redeclaration.flags;
                *ctx.scoping_mut().symbol_flags_mut(symbol_id) = flags;
                ctx.scoping_mut().set_symbol_span(symbol_id, span);
                ctx.scoping_mut().clear_symbol_redeclarations(symbol_id);
                return;
            }
        }

        let has_non_ambient_value_redeclaration =
            ctx.scoping().symbol_redeclarations(symbol_id).iter().any(|redeclaration| {
                redeclaration.flags.is_value() && !redeclaration.flags.is_ambient()
            });
        let has_non_ambient_enum_redeclaration =
            ctx.scoping().symbol_redeclarations(symbol_id).iter().any(|redeclaration| {
                redeclaration.flags.is_enum() && !redeclaration.flags.is_ambient()
            });
        if let Some(redeclaration) =
            ctx.scoping().symbol_redeclarations(symbol_id).iter().find(|redeclaration| {
                redeclaration.flags.intersects(SymbolFlags::Variable)
                    && !redeclaration.flags.is_ambient()
            })
        {
            let span = redeclaration.span;
            let flags = redeclaration.flags;
            *ctx.scoping_mut().symbol_flags_mut(symbol_id) = flags;
            ctx.scoping_mut().set_symbol_span(symbol_id, span);
            ctx.scoping_mut().clear_symbol_redeclarations(symbol_id);
            return;
        }
        if has_non_ambient_value_redeclaration && !has_non_ambient_enum_redeclaration {
            return;
        }

        let scope_id = ctx.scoping().symbol_scope_id(symbol_id);
        ctx.scoping_mut().remove_binding(scope_id, id.name);
        ctx.state.removed_ambient_bindings.push((id.name, symbol_id));
    }
}

/// Check if the statements contain a namespace declaration
fn has_namespace(stmts: &[Statement]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Statement::TSModuleDeclaration(_) | Statement::TSGlobalDeclaration(_) => true,
        Statement::ExportNamedDeclaration(decl) => {
            matches!(decl.declaration, Some(Declaration::TSModuleDeclaration(_)))
        }
        _ => false,
    })
}
