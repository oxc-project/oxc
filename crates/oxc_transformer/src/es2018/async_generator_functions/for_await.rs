//! This module is responsible for transforming `for await` to `for` statement

use std::cell::Cell;

use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::{ast::*, builder::NONE};
use oxc_ast_visit::Visit;
use oxc_ecmascript::BoundNames;
use oxc_semantic::{ReferenceId, ScopeFlags, ScopeId, Scoping, SymbolFlags, SymbolId};
use oxc_span::{SPAN, Span};
use oxc_str::{Ident, static_ident};
use oxc_traverse::{Ancestor, BoundIdentifier};

use crate::{
    common::helper_loader::{Helper, helper_call_expr},
    context::TraverseCtx,
};

use super::AsyncGeneratorFunctions;

impl<'a> AsyncGeneratorFunctions<'a> {
    /// Check the parent node to see if multiple statements are allowed.
    fn is_multiple_statements_allowed(ctx: &TraverseCtx<'a>) -> bool {
        matches!(
            ctx.parent(),
            Ancestor::ProgramBody(_)
                | Ancestor::FunctionBodyStatements(_)
                | Ancestor::BlockStatementBody(_)
                | Ancestor::SwitchCaseConsequent(_)
                | Ancestor::StaticBlockBody(_)
                | Ancestor::TSModuleBlockBody(_)
        )
    }

    pub(crate) fn transform_statement(&self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let (for_of, label) = match stmt {
            Statement::LabeledStatement(labeled) => {
                let LabeledStatement { label, body, .. } = labeled.as_mut();
                if let Statement::ForOfStatement(for_of) = body {
                    (for_of, Some(label))
                } else {
                    return;
                }
            }
            Statement::ForOfStatement(for_of) => (for_of, None),
            _ => return,
        };

        if !for_of.r#await {
            return;
        }

        let for_of_span = for_of.span;
        let allow_multiple_statements = Self::is_multiple_statements_allowed(ctx);
        let parent_scope_id = if allow_multiple_statements {
            ctx.current_scope_id()
        } else {
            ctx.create_child_scope_of_current(ScopeFlags::empty())
        };

        // We need to replace the current statement with new statements,
        // but we don't have a such method to do it, so we leverage the statement injector.
        //
        // Now, we use below steps to workaround it:
        // 1. Use the last statement as the new statement.
        // 2. insert the rest of the statements before the current statement.
        // TODO: Once we have a method to replace the current statement, we can simplify this logic.
        let mut statements =
            self.transform_for_of_statement(for_of, parent_scope_id, for_of_span, ctx);
        let mut new_stmt = statements.pop().unwrap();

        // If it's a labeled statement, we need to wrap the ForStatement with a labeled statement.
        if let Some(label) = label {
            let Statement::TryStatement(try_statement) = &mut new_stmt else {
                unreachable!(
                    "The last statement should be a try statement, please see the `build_for_await` function"
                );
            };
            let try_statement_block_body = &mut try_statement.block.body;
            let for_statement = try_statement_block_body.pop().unwrap();
            try_statement_block_body.push(Statement::new_labeled_statement(
                for_of_span,
                label.clone(),
                for_statement,
                ctx,
            ));
        }
        ctx.state.statement_injector.insert_many_before(&new_stmt, statements);

        // If the parent node doesn't allow multiple statements, we need to wrap the new statement
        // with a block statement, this way we can ensure can insert statement correctly.
        // e.g. `if (true) statement` to `if (true) { statement }`
        if !allow_multiple_statements {
            new_stmt = Statement::new_block_statement_with_scope_id(
                SPAN,
                ArenaVec::from_value_in(new_stmt, ctx),
                parent_scope_id,
                ctx,
            );
        }
        *stmt = new_stmt;
    }

    #[expect(clippy::unused_self)]
    pub(self) fn transform_for_of_statement(
        &self,
        stmt: &mut ForOfStatement<'a>,
        parent_scope_id: ScopeId,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaVec<'a, Statement<'a>> {
        let step_key = ctx.generate_uid(
            "step",
            ctx.current_hoist_scope_id(),
            SymbolFlags::FunctionScopedVariable,
        );
        // step.value
        let step_value = Expression::new_static_member_expression(
            SPAN,
            step_key.create_read_expression(ctx),
            IdentifierName::new(SPAN, "value", ctx),
            false,
            ctx,
        );

        Self::retarget_for_of_right_references(stmt, ctx);

        let assignment_statement = match &mut stmt.left {
            ForStatementLeft::VariableDeclaration(variable) => {
                // for await (let i of test)
                let mut declarator = variable.declarations.pop().unwrap();
                declarator.init = Some(step_value);
                Statement::new_variable_declaration(
                    SPAN,
                    declarator.kind,
                    ArenaVec::from_value_in(declarator, ctx),
                    false,
                    ctx,
                )
            }
            left @ match_assignment_target!(ForStatementLeft) => {
                // for await (i of test), for await ({ i } of test)
                let target = left.to_assignment_target_mut().take_in(ctx);
                let expression = Expression::new_assignment_expression(
                    SPAN,
                    AssignmentOperator::Assign,
                    target,
                    step_value,
                    ctx,
                );
                Statement::new_expression_statement(SPAN, expression, ctx)
            }
        };

        let body = {
            let mut statements = ArenaVec::with_capacity_in(2, ctx);
            statements.push(assignment_statement);
            let stmt_body = &mut stmt.body;
            match stmt_body {
                Statement::BlockStatement(block) if block.body.is_empty() => {}
                _ => statements.push(stmt_body.take_in(ctx)),
            }
            statements
        };

        let iterator = stmt.right.take_in(ctx);
        let iterator = helper_call_expr(
            Helper::AsyncIterator,
            ArenaVec::from_value_in(Argument::from(iterator), ctx),
            ctx,
        );
        Self::build_for_await(
            iterator,
            &step_key,
            body,
            stmt.scope_id(),
            parent_scope_id,
            span,
            ctx,
        )
    }

    fn retarget_for_of_right_references(stmt: &ForOfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        let ForStatementLeft::VariableDeclaration(decl) = &stmt.left else { return };

        let outer_scope_id = ctx.scoping().scope_parent_id(stmt.scope_id());
        let mut bindings = Vec::new();
        decl.bound_names(&mut |ident| {
            let outer_symbol_id = outer_scope_id
                .and_then(|scope_id| ctx.scoping().find_binding(scope_id, ident.name));
            bindings.push((ident.name, ident.symbol_id(), outer_symbol_id));
        });

        let mut collector = ForOfRightReferenceCollector {
            bindings,
            scoping: ctx.scoping(),
            reference_retargets: Vec::new(),
            _marker: std::marker::PhantomData,
        };
        collector.visit_expression(&stmt.right);

        for (reference_id, name, old_symbol_id, new_symbol_id) in collector.reference_retargets {
            ctx.scoping_mut().delete_resolved_reference(old_symbol_id, reference_id);
            if let Some(new_symbol_id) = new_symbol_id {
                ctx.scoping_mut().get_reference_mut(reference_id).set_symbol_id(new_symbol_id);
                ctx.scoping_mut().add_resolved_reference(new_symbol_id, reference_id);
            } else {
                ctx.scoping_mut().get_reference_mut(reference_id).clear_symbol_id();
                ctx.scoping_mut().add_root_unresolved_reference(name, reference_id);
            }
        }
    }

    /// Build a `for` statement used to replace the `for await` statement.
    ///
    /// This function builds the following code:
    ///
    /// ```js
    // var ITERATOR_ABRUPT_COMPLETION = false;
    // var ITERATOR_HAD_ERROR_KEY = false;
    // var ITERATOR_ERROR_KEY;
    // try {
    //   for (
    //     var ITERATOR_KEY = GET_ITERATOR(OBJECT), STEP_KEY;
    //     ITERATOR_ABRUPT_COMPLETION = !(STEP_KEY = await ITERATOR_KEY.next()).done;
    //     ITERATOR_ABRUPT_COMPLETION = false
    //   ) {
    //   }
    // } catch (err) {
    //   ITERATOR_HAD_ERROR_KEY = true;
    //   ITERATOR_ERROR_KEY = err;
    // } finally {
    //   try {
    //     if (ITERATOR_ABRUPT_COMPLETION && ITERATOR_KEY.return != null) {
    //       await ITERATOR_KEY.return();
    //     }
    //   } finally {
    //     if (ITERATOR_HAD_ERROR_KEY) {
    //       throw ITERATOR_ERROR_KEY;
    //     }
    //   }
    // }
    /// ```
    ///
    /// Based on Babel's implementation:
    /// <https://github.com/babel/babel/blob/d20b314c14533ab86351ecf6ca6b7296b66a57b3/packages/babel-plugin-transform-async-generator-functions/src/for-await.ts#L3-L30>
    fn build_for_await(
        iterator: Expression<'a>,
        step_key: &BoundIdentifier<'a>,
        body: ArenaVec<'a, Statement<'a>>,
        for_of_scope_id: ScopeId,
        parent_scope_id: ScopeId,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaVec<'a, Statement<'a>> {
        let var_scope_id = ctx.current_hoist_scope_id();

        let iterator_had_error_key =
            ctx.generate_uid("didIteratorError", var_scope_id, SymbolFlags::FunctionScopedVariable);
        let iterator_abrupt_completion = ctx.generate_uid(
            "iteratorAbruptCompletion",
            var_scope_id,
            SymbolFlags::FunctionScopedVariable,
        );
        let iterator_error_key =
            ctx.generate_uid("iteratorError", var_scope_id, SymbolFlags::FunctionScopedVariable);

        let mut items = ArenaVec::with_capacity_in(4, ctx);
        items.push(Statement::new_variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            ArenaVec::from_value_in(
                VariableDeclarator::new(
                    SPAN,
                    VariableDeclarationKind::Var,
                    iterator_abrupt_completion.create_binding_pattern(ctx),
                    NONE,
                    Some(Expression::new_boolean_literal(SPAN, false, ctx)),
                    false,
                    ctx,
                ),
                ctx,
            ),
            false,
            ctx,
        ));
        items.push(Statement::new_variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            ArenaVec::from_value_in(
                VariableDeclarator::new(
                    SPAN,
                    VariableDeclarationKind::Var,
                    iterator_had_error_key.create_binding_pattern(ctx),
                    NONE,
                    Some(Expression::new_boolean_literal(SPAN, false, ctx)),
                    false,
                    ctx,
                ),
                ctx,
            ),
            false,
            ctx,
        ));
        items.push(Statement::new_variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            ArenaVec::from_value_in(
                VariableDeclarator::new(
                    SPAN,
                    VariableDeclarationKind::Var,
                    iterator_error_key.create_binding_pattern(ctx),
                    NONE,
                    None,
                    false,
                    ctx,
                ),
                ctx,
            ),
            false,
            ctx,
        ));

        let iterator_key =
            ctx.generate_uid("iterator", var_scope_id, SymbolFlags::FunctionScopedVariable);
        let block = {
            let block_scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::empty());
            let for_statement_scope_id =
                ctx.create_child_scope(block_scope_id, ScopeFlags::empty());
            reparent_first_level_expression_scopes(&iterator, for_statement_scope_id, ctx);

            let for_statement = Statement::new_for_statement_with_scope_id(
                span,
                Some(ForStatementInit::new_variable_declaration(
                    SPAN,
                    VariableDeclarationKind::Var,
                    ArenaVec::from_array_in(
                        [
                            VariableDeclarator::new(
                                SPAN,
                                VariableDeclarationKind::Var,
                                iterator_key.create_binding_pattern(ctx),
                                NONE,
                                Some(iterator),
                                false,
                                ctx,
                            ),
                            VariableDeclarator::new(
                                SPAN,
                                VariableDeclarationKind::Var,
                                step_key.create_binding_pattern(ctx),
                                NONE,
                                None,
                                false,
                                ctx,
                            ),
                        ],
                        ctx,
                    ),
                    false,
                    ctx,
                )),
                Some(Expression::new_assignment_expression(
                    SPAN,
                    AssignmentOperator::Assign,
                    iterator_abrupt_completion.create_write_target(ctx),
                    Expression::new_unary_expression(
                        SPAN,
                        UnaryOperator::LogicalNot,
                        Expression::new_static_member_expression(
                            SPAN,
                            Expression::new_parenthesized_expression(
                                SPAN,
                                Expression::new_assignment_expression(
                                    SPAN,
                                    AssignmentOperator::Assign,
                                    step_key.create_write_target(ctx),
                                    Expression::new_await_expression(
                                        SPAN,
                                        Expression::new_call_expression(
                                            SPAN,
                                            Expression::new_static_member_expression(
                                                SPAN,
                                                iterator_key.create_read_expression(ctx),
                                                IdentifierName::new(SPAN, "next", ctx),
                                                false,
                                                ctx,
                                            ),
                                            NONE,
                                            ArenaVec::new_in(ctx),
                                            false,
                                            ctx,
                                        ),
                                        ctx,
                                    ),
                                    ctx,
                                ),
                                ctx,
                            ),
                            IdentifierName::new(SPAN, "done", ctx),
                            false,
                            ctx,
                        ),
                        ctx,
                    ),
                    ctx,
                )),
                Some(Expression::new_assignment_expression(
                    SPAN,
                    AssignmentOperator::Assign,
                    iterator_abrupt_completion.create_write_target(ctx),
                    Expression::new_boolean_literal(SPAN, false, ctx),
                    ctx,
                )),
                {
                    // Handle the for-of statement move to the body of new for-statement
                    let for_statement_body_scope_id = for_of_scope_id;
                    {
                        ctx.scoping_mut().change_scope_parent_id(
                            for_statement_body_scope_id,
                            Some(for_statement_scope_id),
                        );
                    }

                    Statement::new_block_statement_with_scope_id(SPAN, body, for_of_scope_id, ctx)
                },
                for_statement_scope_id,
                ctx,
            );

            BlockStatement::new_with_scope_id(
                SPAN,
                ArenaVec::from_value_in(for_statement, ctx),
                block_scope_id,
                ctx,
            )
        };

        let catch_clause = {
            let catch_scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::CatchClause);
            let block_scope_id = ctx.create_child_scope(catch_scope_id, ScopeFlags::empty());
            let err_ident = ctx.generate_binding(
                static_ident!("err"),
                block_scope_id,
                SymbolFlags::CatchVariable | SymbolFlags::FunctionScopedVariable,
            );
            Some(CatchClause::new_with_scope_id(
                SPAN,
                Some(CatchParameter::new(SPAN, err_ident.create_binding_pattern(ctx), NONE, ctx)),
                {
                    BlockStatement::new_with_scope_id(
                        SPAN,
                        ArenaVec::from_array_in(
                            [
                                Statement::new_expression_statement(
                                    SPAN,
                                    Expression::new_assignment_expression(
                                        SPAN,
                                        AssignmentOperator::Assign,
                                        iterator_had_error_key.create_write_target(ctx),
                                        Expression::new_boolean_literal(SPAN, true, ctx),
                                        ctx,
                                    ),
                                    ctx,
                                ),
                                Statement::new_expression_statement(
                                    SPAN,
                                    Expression::new_assignment_expression(
                                        SPAN,
                                        AssignmentOperator::Assign,
                                        iterator_error_key.create_write_target(ctx),
                                        err_ident.create_read_expression(ctx),
                                        ctx,
                                    ),
                                    ctx,
                                ),
                            ],
                            ctx,
                        ),
                        block_scope_id,
                        ctx,
                    )
                },
                catch_scope_id,
                ctx,
            ))
        };

        let finally = {
            let finally_scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::empty());
            let try_statement = {
                let try_block_scope_id =
                    ctx.create_child_scope(finally_scope_id, ScopeFlags::empty());
                let if_statement = {
                    let if_block_scope_id =
                        ctx.create_child_scope(try_block_scope_id, ScopeFlags::empty());
                    Statement::new_if_statement(
                        SPAN,
                        Expression::new_logical_expression(
                            SPAN,
                            iterator_abrupt_completion.create_read_expression(ctx),
                            LogicalOperator::And,
                            Expression::new_binary_expression(
                                SPAN,
                                Expression::new_static_member_expression(
                                    SPAN,
                                    iterator_key.create_read_expression(ctx),
                                    IdentifierName::new(SPAN, "return", ctx),
                                    false,
                                    ctx,
                                ),
                                BinaryOperator::Inequality,
                                Expression::new_null_literal(SPAN, ctx),
                                ctx,
                            ),
                            ctx,
                        ),
                        Statement::new_block_statement_with_scope_id(
                            SPAN,
                            ArenaVec::from_value_in(
                                Statement::new_expression_statement(
                                    SPAN,
                                    Expression::new_await_expression(
                                        SPAN,
                                        Expression::new_call_expression(
                                            SPAN,
                                            Expression::new_static_member_expression(
                                                SPAN,
                                                iterator_key.create_read_expression(ctx),
                                                IdentifierName::new(SPAN, "return", ctx),
                                                false,
                                                ctx,
                                            ),
                                            NONE,
                                            ArenaVec::new_in(ctx),
                                            false,
                                            ctx,
                                        ),
                                        ctx,
                                    ),
                                    ctx,
                                ),
                                ctx,
                            ),
                            if_block_scope_id,
                            ctx,
                        ),
                        None,
                        ctx,
                    )
                };
                let block = BlockStatement::new_with_scope_id(
                    SPAN,
                    ArenaVec::from_value_in(if_statement, ctx),
                    try_block_scope_id,
                    ctx,
                );
                let finally = {
                    let finally_scope_id =
                        ctx.create_child_scope(finally_scope_id, ScopeFlags::empty());
                    let if_statement = {
                        let if_block_scope_id =
                            ctx.create_child_scope(finally_scope_id, ScopeFlags::empty());
                        Statement::new_if_statement(
                            SPAN,
                            iterator_had_error_key.create_read_expression(ctx),
                            Statement::new_block_statement_with_scope_id(
                                SPAN,
                                ArenaVec::from_value_in(
                                    Statement::new_throw_statement(
                                        SPAN,
                                        iterator_error_key.create_read_expression(ctx),
                                        ctx,
                                    ),
                                    ctx,
                                ),
                                if_block_scope_id,
                                ctx,
                            ),
                            None,
                            ctx,
                        )
                    };
                    BlockStatement::new_with_scope_id(
                        SPAN,
                        ArenaVec::from_value_in(if_statement, ctx),
                        finally_scope_id,
                        ctx,
                    )
                };
                Statement::new_try_statement(SPAN, block, NONE, Some(finally), ctx)
            };

            let block_statement = BlockStatement::new_with_scope_id(
                SPAN,
                ArenaVec::from_value_in(try_statement, ctx),
                finally_scope_id,
                ctx,
            );
            Some(block_statement)
        };

        let try_statement = Statement::new_try_statement(span, block, catch_clause, finally, ctx);

        items.push(try_statement);
        items
    }
}

fn reparent_first_level_expression_scopes<'a>(
    expr: &Expression<'a>,
    parent_scope_id: ScopeId,
    ctx: &mut TraverseCtx<'a>,
) {
    let mut reparenter = FirstLevelScopeReparenter { parent_scope_id, scope_depth: 0, ctx };
    reparenter.visit_expression(expr);
}

struct FirstLevelScopeReparenter<'a, 'v> {
    parent_scope_id: ScopeId,
    scope_depth: u32,
    ctx: &'v mut TraverseCtx<'a>,
}

impl<'a> Visit<'a> for FirstLevelScopeReparenter<'a, '_> {
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        let scope_id = scope_id.get().unwrap();
        if self.scope_depth == 0 {
            let has_direct_eval = self.ctx.scoping().scope_flags(scope_id).contains_direct_eval();
            self.ctx.scoping_mut().change_scope_parent_id(scope_id, Some(self.parent_scope_id));
            if has_direct_eval {
                self.ctx
                    .scoping_mut()
                    .scope_flags_mut(self.parent_scope_id)
                    .insert(ScopeFlags::DirectEval);
            }
        }
        self.scope_depth += 1;
    }

    fn leave_scope(&mut self) {
        self.scope_depth -= 1;
    }
}

struct ForOfRightReferenceCollector<'a, 's> {
    bindings: Vec<(Ident<'a>, SymbolId, Option<SymbolId>)>,
    scoping: &'s Scoping,
    reference_retargets: Vec<(ReferenceId, Ident<'a>, SymbolId, Option<SymbolId>)>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Visit<'a> for ForOfRightReferenceCollector<'a, '_> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        let reference_id = ident.reference_id();
        let current_symbol_id = self.scoping.get_reference(reference_id).symbol_id();
        if let Some(&(name, old_symbol_id, new_symbol_id)) =
            self.bindings.iter().find(|&&(name, symbol_id, _)| {
                ident.name == name && current_symbol_id == Some(symbol_id)
            })
        {
            self.reference_retargets.push((reference_id, name, old_symbol_id, new_symbol_id));
        }
    }
}
