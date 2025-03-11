//! Proposal: Explicit Resource Management
//!
//! This plugin transforms explicit resource management syntax into a series of try-catch-finally blocks.
//!
//! ## Example
//!
//! Input:
//! ```js
//! for await (using x of y) {
//!     doSomethingWith(x);
//! }
//! ```
//!
//! Output:
//! ```js
//! for await (const _x of y)
//! try {
//!     var _usingCtx = babelHelpers.usingCtx();
//!     const x = _usingCtx.u(_x);
//!     doSomethingWith(x);
//! } catch (_) {
//!     _usingCtx.e = _;
//! } finally {
//!     _usingCtx.d();
//! }
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-proposal-explicit-resource-management](https://babeljs.io/docs/babel-plugin-proposal-explicit-resource-management).
//!
//! ## References:
//! * Babel plugin implementation: <https://github.com/babel/babel/blob/v7.26.9/packages/babel-plugin-proposal-explicit-resource-management>
//! * Explicit Resource Management TC39 proposal: <https://github.com/tc39/proposal-explicit-resource-management>

use std::mem;

use rustc_hash::FxHashMap;

use oxc_allocator::{Address, GetAddress, Vec as ArenaVec};
use oxc_ast::{NONE, ast::*};
use oxc_ecmascript::BoundNames;
use oxc_semantic::{ScopeFlags, ScopeId, SymbolFlags};
use oxc_span::{Atom, SPAN};
use oxc_traverse::{BoundIdentifier, Traverse, TraverseCtx};

use crate::{Helper, TransformCtx};

pub struct ExplicitResourceManagement<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,

    top_level_using: FxHashMap<Address, /* is await-using */ bool>,
}

impl<'a, 'ctx> ExplicitResourceManagement<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx, top_level_using: FxHashMap::default() }
    }
}

impl<'a> Traverse<'a> for ExplicitResourceManagement<'a, '_> {
    /// Transform `for (using ... of ...)`, ready for `enter_statement` to do the rest.
    ///
    /// * `for (using x of y) {}` -> `for (const _x of y) { using x = _x; }`
    /// * `for await (using x of y) {}` -> `for (const _x of y) { await using x = _x; }`
    fn enter_for_of_statement(&mut self, node: &mut ForOfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        let for_of_stmt_scope_id = node.scope_id();
        let ForStatementLeft::VariableDeclaration(decl) = &mut node.left else { return };
        if !matches!(
            decl.kind,
            VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing
        ) {
            return;
        }
        let variable_decl_kind = decl.kind;

        // `for (using x of y)` -> `for (const _x of y)`
        decl.kind = VariableDeclarationKind::Const;

        let variable_declarator = decl.declarations.first_mut().unwrap();
        variable_declarator.kind = VariableDeclarationKind::Const;

        let temp_id = ctx.generate_uid_based_on_node(
            variable_declarator.id.get_binding_identifier().unwrap(),
            for_of_stmt_scope_id,
            SymbolFlags::ConstVariable | SymbolFlags::BlockScopedVariable,
        );

        let binding_pattern =
            mem::replace(&mut variable_declarator.id, temp_id.create_binding_pattern(ctx));

        // `using x = _x;`
        let stmt = Statement::from(ctx.ast.declaration_variable(
            SPAN,
            variable_decl_kind,
            ctx.ast.vec1(ctx.ast.variable_declarator(
                SPAN,
                variable_decl_kind,
                binding_pattern,
                Some(temp_id.create_read_expression(ctx)),
                false,
            )),
            false,
        ));

        if let Statement::BlockStatement(body) = &mut node.body {
            // `for (const _x of y) { x(); }` -> `for (const _x of y) { using x = _x; x(); }`
            body.body.insert(0, stmt);
        } else {
            // `for (const _x of y) x();` -> `for (const _x of y) { using x = _x; x(); }`
            let old_body = ctx.ast.move_statement(&mut node.body);

            let new_body = ctx.ast.vec_from_array([stmt, old_body]);
            node.body = ctx.ast.statement_block_with_scope_id(
                SPAN,
                new_body,
                ctx.create_child_scope(for_of_stmt_scope_id, ScopeFlags::empty()),
            );
        };
    }

    /// Transform class static block.
    ///
    /// ```js
    /// class C { static { using x = y(); } }
    /// ```
    /// ->
    /// ```js
    /// class C {
    ///   static {
    ///     try {
    ///       var _usingCtx = babelHelpers.usingCtx();
    ///       const x = _usingCtx.u(y());
    ///     } catch (_) {
    ///       _usingCtx.e = _;
    ///     } finally {
    ///       _usingCtx.d();
    ///     }
    ///   }
    /// }
    /// ```
    fn enter_static_block(&mut self, node: &mut StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        let scope_id = node.scope_id();
        if let Some(replacement) =
            self.transform_statements(&mut node.body, None, scope_id, scope_id, ctx)
        {
            node.body = ctx.ast.vec1(replacement);
        }
    }

    /// Transform function body.
    ///
    /// ```js
    /// function f() {
    ///   using x = y();
    /// }
    /// ```
    /// ->
    /// ```js
    /// function f() {
    ///   try {
    ///     var _usingCtx = babelHelpers.usingCtx();
    ///     const x = _usingCtx.u(y());
    ///   } catch (_) {
    ///     _usingCtx.e = _;
    ///   } finally {
    ///     _usingCtx.d();
    ///   }
    /// }
    /// ```
    fn enter_function_body(&mut self, node: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(replacement) = self.transform_statements(
            &mut node.statements,
            None,
            ctx.current_hoist_scope_id(),
            ctx.current_hoist_scope_id(),
            ctx,
        ) {
            node.statements = ctx.ast.vec1(replacement);
        }
    }

    /// Transform a block statement or switch statement.
    ///
    /// Block statement:
    /// ```js
    /// {
    ///   using x = y();
    /// }
    /// ```
    /// ->
    /// ```js
    /// try {
    ///   var _usingCtx = babelHelpers.usingCtx();
    ///   const x = _usingCtx.u(y());
    /// } catch (_) {
    ///   _usingCtx.e = _;
    /// } finally {
    ///   _usingCtx.d();
    /// }
    /// ```
    ///
    /// Switch statement:
    /// ```js
    /// switch (s) {
    ///   case 1: using x = y();
    /// }
    /// ```
    /// ->
    /// ```js
    /// try {
    ///   var _usingCtx = babelHelpers.usingCtx();
    ///   switch (s) {
    ///     case 1: const x = _usingCtx.u(y());
    ///   }
    /// } catch (_) {
    ///   _usingCtx.e = _;
    /// } finally {
    ///   _usingCtx.d();
    /// }
    /// ```
    fn enter_statement(&mut self, node: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match node {
            Statement::BlockStatement(block_stmt) => {
                let scope_id = block_stmt.scope_id();
                if let Some(replacement) = self.transform_statements(
                    &mut block_stmt.body,
                    Some(scope_id),
                    ctx.current_scope_id(),
                    ctx.current_hoist_scope_id(),
                    ctx,
                ) {
                    *node = replacement;
                }
            }
            Statement::SwitchStatement(_) => {
                if let Some(replacement) = self.transform_switch_statement(node, ctx) {
                    *node = replacement;
                }
            }
            _ => {}
        }
    }

    /// Move any top level `using` declarations within a block statement,
    /// allowing `enter_statement` to transform them.
    fn enter_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.top_level_using.clear();
        if !node.body.iter().any(|stmt| match stmt {
            Statement::VariableDeclaration(var_decl) => matches!(
                var_decl.kind,
                VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing
            ),
            _ => false,
        }) {
            return;
        }

        let program_body = ctx.ast.move_vec(&mut node.body);

        let mut scopes_to_skip_move: Vec<ScopeId> = Vec::new();

        let (mut program_body, inner_block): (
            ArenaVec<'a, Statement<'a>>,
            ArenaVec<'a, Statement<'a>>,
        ) = program_body.into_iter().fold(
            (ctx.ast.vec(), ctx.ast.vec()),
            |(mut program_body, mut inner_block), mut stmt| {
                let address = stmt.address();
                match stmt {
                    Statement::FunctionDeclaration(ref fun) => {
                        scopes_to_skip_move.push(fun.scope_id());
                        program_body.push(stmt);
                    }
                    Statement::ImportDeclaration(_) | Statement::ExportAllDeclaration(_) => {
                        program_body.push(stmt);
                    }
                    Statement::ExportDefaultDeclaration(ref mut export_default_decl) => {
                        let (var_id, span) = match &mut export_default_decl.declaration {
                            ExportDefaultDeclarationKind::ClassDeclaration(class_decl)
                                if class_decl.id.is_some() =>
                            {
                                let id = class_decl.id.take().unwrap();

                                *ctx.scoping_mut().symbol_flags_mut(id.symbol_id()) =
                                    SymbolFlags::FunctionScopedVariable;

                                (BoundIdentifier::from_binding_ident(&id), id.span)
                            }
                            ExportDefaultDeclarationKind::FunctionDeclaration(fun) => {
                                scopes_to_skip_move.push(fun.scope_id());
                                program_body.push(stmt);
                                return (program_body, inner_block);
                            }
                            _ => (
                                ctx.generate_binding_in_current_scope(
                                    Atom::from("_default"),
                                    SymbolFlags::FunctionScopedVariable,
                                ),
                                SPAN,
                            ),
                        };

                        let decl = mem::replace(
                            &mut export_default_decl.declaration,
                            ExportDefaultDeclarationKind::NullLiteral(
                                ctx.ast.alloc_null_literal(SPAN),
                            ),
                        );

                        let expr = match decl {
                            ExportDefaultDeclarationKind::FunctionDeclaration(decl) => {
                                Expression::FunctionExpression(decl)
                            }
                            ExportDefaultDeclarationKind::ClassDeclaration(mut decl) => {
                                decl.r#type = ClassType::ClassExpression;
                                Expression::ClassExpression(decl)
                            }
                            _ => decl.into_expression(),
                        };

                        inner_block.push(Statement::VariableDeclaration(ctx.ast.alloc(
                            ctx.ast.variable_declaration(
                                span,
                                VariableDeclarationKind::Var,
                                ctx.ast.vec1(ctx.ast.variable_declarator(
                                    span,
                                    VariableDeclarationKind::Var,
                                    ctx.ast.binding_pattern(
                                        BindingPatternKind::BindingIdentifier(
                                            ctx.ast.alloc(var_id.create_binding_identifier(ctx)),
                                        ),
                                        NONE,
                                        false,
                                    ),
                                    Some(expr),
                                    false,
                                )),
                                false,
                            ),
                        )));

                        program_body.push(Statement::ExportNamedDeclaration(ctx.ast.alloc(
                            ctx.ast.export_named_declaration(
                                SPAN,
                                None,
                                ctx.ast.vec1(ctx.ast.export_specifier(
                                    SPAN,
                                    ModuleExportName::IdentifierReference(
                                        var_id.create_read_reference(ctx),
                                    ),
                                    ctx.ast.module_export_name_identifier_name(SPAN, "default"),
                                    ImportOrExportKind::Value,
                                )),
                                None,
                                ImportOrExportKind::Value,
                                NONE,
                            ),
                        )));
                    }
                    Statement::ExportNamedDeclaration(ref mut export_named_declaration) => {
                        let Some(ref mut decl) = export_named_declaration.declaration else {
                            program_body.push(stmt);
                            return (program_body, inner_block);
                        };
                        if matches!(
                            decl,
                            Declaration::FunctionDeclaration(_)
                                | Declaration::TSTypeAliasDeclaration(_)
                                | Declaration::TSInterfaceDeclaration(_)
                                | Declaration::TSEnumDeclaration(_)
                                | Declaration::TSModuleDeclaration(_)
                                | Declaration::TSImportEqualsDeclaration(_)
                        ) {
                            if let Some(stmt_scope) = match decl {
                                Declaration::FunctionDeclaration(fun) => Some(fun.scope_id()),
                                Declaration::TSTypeAliasDeclaration(decl) => Some(decl.scope_id()),
                                Declaration::TSInterfaceDeclaration(decl) => Some(decl.scope_id()),
                                Declaration::TSEnumDeclaration(decl) => Some(decl.scope_id()),
                                Declaration::TSModuleDeclaration(decl) => Some(decl.scope_id()),
                                Declaration::TSImportEqualsDeclaration(_) => None,
                                _ => unreachable!(),
                            } {
                                scopes_to_skip_move.push(stmt_scope);
                            }
                            program_body.push(stmt);

                            return (program_body, inner_block);
                        }
                        let decl = ctx.ast.move_declaration(decl);

                        let export_specifiers = match decl {
                            Declaration::ClassDeclaration(class_decl) => {
                                let class_binding = class_decl.id.as_ref().unwrap();
                                let class_binding_name = class_binding.name;

                                let xx = BoundIdentifier::from_binding_ident(class_binding)
                                    .create_read_reference(ctx);

                                inner_block
                                    .push(Self::transform_class_decl(class_decl.unbox(), ctx));

                                let local = ModuleExportName::IdentifierReference(xx);
                                let exported = ctx
                                    .ast
                                    .module_export_name_identifier_name(SPAN, class_binding_name);
                                ctx.ast.vec1(ctx.ast.export_specifier(
                                    SPAN,
                                    local,
                                    exported,
                                    ImportOrExportKind::Value,
                                ))
                            }
                            Declaration::VariableDeclaration(mut var_decl) => {
                                var_decl.kind = VariableDeclarationKind::Var;
                                let mut export_specifiers = ctx.ast.vec();

                                for decl in &mut var_decl.declarations {
                                    decl.kind = VariableDeclarationKind::Var;
                                }

                                var_decl.bound_names(&mut |ident| {
                                    *ctx.scoping_mut().symbol_flags_mut(ident.symbol_id()) =
                                        SymbolFlags::FunctionScopedVariable;

                                    export_specifiers.push(
                                        ctx.ast.export_specifier(
                                            SPAN,
                                            ModuleExportName::IdentifierReference(
                                                BoundIdentifier::from_binding_ident(ident)
                                                    .create_read_reference(ctx),
                                            ),
                                            ctx.ast.module_export_name_identifier_name(
                                                SPAN, ident.name,
                                            ),
                                            ImportOrExportKind::Value,
                                        ),
                                    );
                                });
                                inner_block.push(Statement::VariableDeclaration(var_decl));
                                export_specifiers
                            }
                            _ => unreachable!(),
                        };

                        program_body.push(Statement::ExportNamedDeclaration(ctx.ast.alloc(
                            ctx.ast.export_named_declaration(
                                SPAN,
                                None,
                                export_specifiers,
                                None,
                                export_named_declaration.export_kind,
                                NONE,
                            ),
                        )));
                    }
                    Statement::ClassDeclaration(class_decl) => {
                        inner_block.push(Self::transform_class_decl(class_decl.unbox(), ctx));
                    }
                    Statement::VariableDeclaration(ref mut var_declaration) => {
                        if var_declaration.kind == VariableDeclarationKind::Using {
                            self.top_level_using.insert(address, false);
                        } else if var_declaration.kind == VariableDeclarationKind::AwaitUsing {
                            self.top_level_using.insert(address, true);
                        }
                        var_declaration.kind = VariableDeclarationKind::Var;

                        for decl in &mut var_declaration.declarations {
                            decl.kind = VariableDeclarationKind::Var;
                            decl.id.bound_names(&mut |c| {
                                *ctx.scoping_mut().symbol_flags_mut(c.symbol_id()) =
                                    SymbolFlags::FunctionScopedVariable;
                            });
                        }

                        inner_block.push(stmt);
                    }
                    _ => inner_block.push(stmt),
                }

                (program_body, inner_block)
            },
        );

        let current_scope_id = ctx.current_scope_id();
        let block_scope_id = ctx.create_child_scope(current_scope_id, ScopeFlags::empty());

        let child_ids = ctx.scoping_mut().get_scope_child_ids(current_scope_id).to_vec();
        for id in child_ids
            .iter()
            .filter(|id| !scopes_to_skip_move.contains(id) && *id != &block_scope_id)
        {
            ctx.scoping_mut().change_scope_parent_id(*id, Some(block_scope_id));
        }

        program_body.push(Statement::BlockStatement(
            ctx.ast.alloc(ctx.ast.block_statement_with_scope_id(SPAN, inner_block, block_scope_id)),
        ));

        std::mem::swap(&mut node.body, &mut program_body);
    }
}

impl<'a> ExplicitResourceManagement<'a, '_> {
    /// This function returns `None` if the switch statement was not transformed.
    ///
    /// Input:
    /// ```js
    /// switch (0) {
    ///   case 1:
    ///     using foo = bar;
    ///     doSomethingWithFoo(foo)
    ///   case 2:
    ///     throw new Error('oops')
    /// }
    /// ```
    /// Output:
    /// ```js
    /// try {
    ///   var _usingCtx = babelHelpers.usingCtx();
    ///   switch (0) {
    ///     case 1:
    ///       const foo = _usingCtx.u(bar);
    ///       doSomethingWithFoo(foo);
    ///     case 2:
    ///       throw new Error('oops');
    ///   }
    /// } catch (_) {
    ///   _usingCtx.e = _;
    /// } finally {
    ///   _usingCtx.d();
    /// }
    /// ```
    fn transform_switch_statement(
        &self,
        node: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        let mut using_ctx = None;
        let mut needs_await = false;
        let current_scope_id = ctx.current_scope_id();

        let Statement::SwitchStatement(switch_stmt) = node else { unreachable!() };

        let switch_stmt_scope_id = switch_stmt.scope_id();

        for case in &mut switch_stmt.cases {
            for stmt in &mut case.consequent {
                let Statement::VariableDeclaration(stmt) = stmt else { continue };
                if !matches!(
                    stmt.kind,
                    VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing
                ) {
                    continue;
                };
                needs_await = needs_await || stmt.kind == VariableDeclarationKind::AwaitUsing;

                stmt.kind = VariableDeclarationKind::Const;

                using_ctx = using_ctx.or_else(|| {
                    Some(ctx.generate_uid(
                        "usingCtx",
                        current_scope_id,
                        SymbolFlags::FunctionScopedVariable,
                    ))
                });

                for decl in &mut stmt.declarations {
                    if let Some(old_init) = decl.init.take() {
                        decl.init = Some(
                            ctx.ast.expression_call(
                                SPAN,
                                ctx.ast
                                    .member_expression_static(
                                        SPAN,
                                        using_ctx
                                            .as_ref()
                                            .expect("`using_ctx` should have been set")
                                            .create_read_expression(ctx),
                                        ctx.ast.identifier_name(
                                            SPAN,
                                            if needs_await { "a" } else { "u" },
                                        ),
                                        false,
                                    )
                                    .into(),
                                NONE,
                                ctx.ast.vec1(old_init.into()),
                                false,
                            ),
                        );
                    }
                }
            }
        }

        let using_ctx = using_ctx?;

        let block_stmt_sid = ctx.create_child_scope(ctx.current_scope_id(), ScopeFlags::empty());

        ctx.scoping_mut().change_scope_parent_id(switch_stmt_scope_id, Some(block_stmt_sid));

        let callee = self.ctx.helper_load(Helper::UsingCtx, ctx);

        let block = {
            let vec = ctx.ast.vec_from_array([
                Statement::from(ctx.ast.declaration_variable(
                    SPAN,
                    VariableDeclarationKind::Var,
                    ctx.ast.vec1(ctx.ast.variable_declarator(
                        SPAN,
                        VariableDeclarationKind::Var,
                        using_ctx.create_binding_pattern(ctx),
                        Some(ctx.ast.expression_call(SPAN, callee, NONE, ctx.ast.vec(), false)),
                        false,
                    )),
                    false,
                )),
                ctx.ast.move_statement(node),
            ]);

            ctx.ast.block_statement_with_scope_id(SPAN, vec, block_stmt_sid)
        };

        let catch = Self::create_catch_clause(&using_ctx, ctx.current_scope_id(), ctx);

        let finally = Self::create_finally_block(&using_ctx, current_scope_id, needs_await, ctx);

        Some(ctx.ast.statement_try(SPAN, block, Some(catch), Some(finally)))
    }

    /// Transforms:
    ///  - `node` - the statements to transform
    ///  - `scope_id` - if provided, it will be used as the scope_id for the new block.
    ///  - `parent_scope_id` - the parent scope
    ///  - `hoist_scope_id` - the hoist scope, used for generating new var bindings
    ///  - `ctx` - the traverse context
    ///
    /// Input:
    /// ```js
    /// {
    ///     using foo = bar;
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// try {
    ///   var _usingCtx = babelHelpers.usingCtx();
    ///   const foo = _usingCtx.u(bar);
    /// } catch (_) {
    ///   _usingCtx.e = _;
    /// } finally {
    ///   _usingCtx.d();
    /// }
    /// ```
    ///
    /// Returns `Some` if the statements were transformed, `None` otherwise.
    fn transform_statements(
        &mut self,
        node: &mut ArenaVec<'a, Statement<'a>>,
        scope_id: Option<ScopeId>,
        parent_scope_id: ScopeId,
        hoist_scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        let mut needs_await = false;

        let mut using_ctx = None;

        for stmt in node.iter_mut() {
            let address = stmt.address();
            let Statement::VariableDeclaration(variable_declaration) = stmt else { continue };
            if !matches!(
                variable_declaration.kind,
                VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing
            ) && !self.top_level_using.contains_key(&address)
            {
                continue;
            };
            let is_await_using = variable_declaration.kind == VariableDeclarationKind::AwaitUsing
                || self.top_level_using.get(&address).copied().unwrap_or(false);
            needs_await = needs_await || is_await_using;

            if self.top_level_using.remove(&address).is_none() {
                variable_declaration.kind = VariableDeclarationKind::Const;
            }

            using_ctx = using_ctx.or_else(|| {
                let binding = ctx.generate_uid(
                    "usingCtx",
                    hoist_scope_id,
                    SymbolFlags::FunctionScopedVariable,
                );
                Some(binding)
            });

            // `using foo = bar;` -> `const foo = _usingCtx.u(bar);`
            // `await using foo = bar;` -> `const foo = _usingCtx.a(bar);`
            for decl in &mut variable_declaration.declarations {
                if let Some(old_init) = decl.init.take() {
                    decl.init = Some(
                        ctx.ast.expression_call(
                            SPAN,
                            ctx.ast
                                .member_expression_static(
                                    SPAN,
                                    using_ctx.as_ref().unwrap().create_read_expression(ctx),
                                    ctx.ast.identifier_name(
                                        SPAN,
                                        if is_await_using { "a" } else { "u" },
                                    ),
                                    false,
                                )
                                .into(),
                            NONE,
                            ctx.ast.vec1(old_init.into()),
                            false,
                        ),
                    );
                }
            }
        }

        let using_ctx = using_ctx?;

        let mut stmts = ctx.ast.move_vec(&mut *node);

        // `var _usingCtx = babelHelpers.usingCtx();`
        let callee = self.ctx.helper_load(Helper::UsingCtx, ctx);
        let helper = ctx.ast.declaration_variable(
            SPAN,
            VariableDeclarationKind::Var,
            ctx.ast.vec1(ctx.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Var,
                using_ctx.create_binding_pattern(ctx),
                Some(ctx.ast.expression_call(SPAN, callee, NONE, ctx.ast.vec(), false)),
                false,
            )),
            false,
        );
        stmts.insert(0, Statement::from(helper));

        let scope_id_children_to_move = scope_id.unwrap_or(parent_scope_id);

        let scope_id = scope_id
            .unwrap_or_else(|| ctx.create_child_scope(parent_scope_id, ScopeFlags::empty()));
        let block = ctx.ast.block_statement_with_scope_id(SPAN, stmts, scope_id);

        let child_ids = ctx.scoping_mut().get_scope_child_ids(scope_id_children_to_move).to_vec();
        for id in child_ids.iter().filter(|id| *id != &scope_id) {
            ctx.scoping_mut().change_scope_parent_id(*id, Some(scope_id));
        }

        let catch = Self::create_catch_clause(&using_ctx, parent_scope_id, ctx);
        let finally = Self::create_finally_block(&using_ctx, parent_scope_id, needs_await, ctx);

        Some(ctx.ast.statement_try(SPAN, block, Some(catch), Some(finally)))
    }

    /// `catch (_) { _usingCtx.e = _; }`
    fn create_catch_clause(
        using_ctx: &BoundIdentifier<'a>,
        parent_scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> CatchClause<'a> {
        // catch (_) { _usingCtx.e = _; }
        //        ^                       catch_parameter
        //       ^^^^^^^^^^^^^^^^^^^^^^^^ catch_scope_id
        //           ^^^^^^^^^^^^^^^^^^^^ block_scope_id
        let catch_scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::CatchClause);
        let block_scope_id = ctx.create_child_scope(catch_scope_id, ScopeFlags::empty());
        // We can skip using `generate_uid` here as no code within the `catch` block which can use a
        // binding called `_`. `using_ctx` is a UID with prefix `_usingCtx`.
        let ident = ctx.generate_binding(
            Atom::from("_"),
            block_scope_id,
            SymbolFlags::CatchVariable | SymbolFlags::FunctionScopedVariable,
        );

        let catch_parameter = ctx.ast.catch_parameter(SPAN, ident.create_binding_pattern(ctx));

        // `_usingCtx.e = _;`
        let stmt = ctx.ast.statement_expression(
            SPAN,
            ctx.ast.expression_assignment(
                SPAN,
                AssignmentOperator::Assign,
                SimpleAssignmentTarget::from(ctx.ast.member_expression_static(
                    SPAN,
                    using_ctx.create_read_expression(ctx),
                    ctx.ast.identifier_name(SPAN, "e"),
                    false,
                ))
                .into(),
                ident.create_read_expression(ctx),
            ),
        );

        // `catch (_) { _usingCtx.e = _; }`
        ctx.ast.catch_clause_with_scope_id(
            SPAN,
            Some(catch_parameter),
            ctx.ast.block_statement_with_scope_id(SPAN, ctx.ast.vec1(stmt), block_scope_id),
            catch_scope_id,
        )
    }

    /// `{ _usingCtx.d(); }`
    fn create_finally_block(
        using_ctx: &BoundIdentifier<'a>,
        parent_scope_id: ScopeId,
        needs_await: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> BlockStatement<'a> {
        let finally_scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::empty());

        // `_usingCtx.d()`
        let expr = ctx.ast.expression_call(
            SPAN,
            ctx.ast
                .member_expression_static(
                    SPAN,
                    using_ctx.create_read_expression(ctx),
                    ctx.ast.identifier_name(SPAN, "d"),
                    false,
                )
                .into(),
            NONE,
            ctx.ast.vec(),
            false,
        );

        let stmt = if needs_await { ctx.ast.expression_await(SPAN, expr) } else { expr };

        ctx.ast.block_statement_with_scope_id(
            SPAN,
            ctx.ast.vec1(ctx.ast.statement_expression(SPAN, stmt)),
            finally_scope_id,
        )
    }

    /// `class C {}` -> `var C = class {};`
    fn transform_class_decl(mut class_decl: Class<'a>, ctx: &mut TraverseCtx<'a>) -> Statement<'a> {
        let id = class_decl.id.take().expect("ClassDeclaration should have an id");

        class_decl.r#type = ClassType::ClassExpression;
        let class_expr = Expression::ClassExpression(ctx.ast.alloc(class_decl));

        *ctx.scoping_mut().symbol_flags_mut(id.symbol_id()) = SymbolFlags::FunctionScopedVariable;

        Statement::VariableDeclaration(ctx.ast.alloc(ctx.ast.variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            ctx.ast.vec1(ctx.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Var,
                ctx.ast.binding_pattern(
                    BindingPatternKind::BindingIdentifier(ctx.ast.alloc(id)),
                    NONE,
                    false,
                ),
                Some(class_expr),
                false,
            )),
            false,
        )))
    }
}
