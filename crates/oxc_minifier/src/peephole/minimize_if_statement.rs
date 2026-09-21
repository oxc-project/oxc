use oxc_allocator::TakeIn;
use oxc_ast::ast::*;

use oxc_semantic::ScopeFlags;
use oxc_span::GetSpan;

use crate::TraverseCtx;
use crate::generated::ancestor::Ancestor;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// `MangleIf`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_parser/js_parser.go#L9860>
    pub fn try_minimize_if(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::IfStatement(if_stmt) = stmt else { return };

        // Flip empty consequent so the rest of the function can assume consequent is non-empty.
        if Self::is_statement_empty(&if_stmt.consequent) {
            if if_stmt.alternate.is_none() {
                // `if (a) {}` => `a;`
                ctx.replace_statement_with(stmt, |stmt, ctx| {
                    let Statement::IfStatement(if_stmt) = stmt else { unreachable!() };
                    let IfStatement { mut test, span, .. } = if_stmt.unbox();
                    Self::remove_unused_expression(&mut test, ctx);
                    Statement::new_expression_statement(span, test, ctx)
                });
                return;
            }
            let new_consequent = if_stmt.alternate.take().unwrap();

            if let Statement::ExpressionStatement(expr_stmt) = new_consequent {
                ctx.replace_statement_with(stmt, |stmt, ctx| {
                    let Statement::IfStatement(if_stmt) = stmt else { unreachable!() };
                    let IfStatement { test, span, .. } = if_stmt.unbox();
                    let (op, a) = match test {
                        // `if (!a); else b();` => `a && b();`
                        Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                            (LogicalOperator::And, unary_expr.unbox().argument)
                        }
                        // `if (a); else b();` => `a || b();`
                        e => (LogicalOperator::Or, e),
                    };
                    let b = expr_stmt.unbox().expression;
                    let expr = Self::join_with_left_associative_op(span, op, a, b, ctx);
                    Statement::new_expression_statement(span, expr, ctx)
                });
                return;
            }

            // `if (!a) {} else x;` => `if (a) x;`
            // `if (a)  {} else x;` => `if (!a) x;`
            ctx.replace_expression_with(&mut if_stmt.test, |old, ctx| {
                Self::minimize_not(old.span(), old, ctx, true)
            });
            ctx.replace_statement(&mut if_stmt.consequent, new_consequent);
        }

        // Consequent is non-empty from here on.

        if let Some(alternate) = &if_stmt.alternate {
            if matches!(&if_stmt.consequent, Statement::ExpressionStatement(_)) && matches!(alternate, Statement::ExpressionStatement(_)) {
                // `if (a) b(); else c();` => `a ? b() : c();`
                ctx.replace_statement_with(stmt, |stmt, ctx| {
                    let Statement::IfStatement(if_stmt) = stmt else { unreachable!() };
                    let IfStatement { test, consequent, alternate, span, .. } = if_stmt.unbox();
                    let Statement::ExpressionStatement(a) = consequent else { unreachable!() };
                    let Statement::ExpressionStatement(b) = alternate.unwrap() else {
                        unreachable!()
                    };
                    let a = a.unbox().expression;
                    let b = b.unbox().expression;
                    let expr = Self::minimize_conditional(span, test, a, b, ctx);
                    Statement::new_expression_statement(span, expr, ctx)
                });
                return;
            }

            if Self::should_invert_if(&if_stmt.consequent, alternate, &if_stmt.test, ctx) {
                ctx.replace_expression_with(&mut if_stmt.test, |old, ctx| {
                    Self::minimize_not(old.span(), old, ctx, true)
                });
                let if_mut = if_stmt.as_mut();
                let Some(alternate) = &mut if_mut.alternate else { unreachable!() };
                std::mem::swap(&mut if_mut.consequent, alternate);
            }
        } else if matches!(&if_stmt.consequent, Statement::ExpressionStatement(_)) {
            ctx.replace_statement_with(stmt, |stmt, ctx| {
                let Statement::IfStatement(if_stmt) = stmt else { unreachable!() };
                let IfStatement { test, consequent, span, .. } = if_stmt.unbox();
                let Statement::ExpressionStatement(b) = consequent else { unreachable!() };
                let (op, a) = match test {
                    // `if (!a) b();` => `a || b();`
                    Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                        (LogicalOperator::Or, unary_expr.unbox().argument)
                    }
                    // `if (a)  b();` => `a && b();`
                    e => (LogicalOperator::And, e),
                };
                let b = b.unbox().expression;
                let expr = Self::join_with_left_associative_op(span, op, a, b, ctx);
                Statement::new_expression_statement(span, expr, ctx)
            });
            return;
        } else if let Statement::IfStatement(if2_stmt) = &mut if_stmt.consequent
            && if2_stmt.alternate.is_none()
        {
            // `if (a) if (b) x;` => `if (a && b) x;`
            let IfStatement { test, consequent, .. } = if2_stmt.take_in(ctx);
            ctx.replace_expression_with(&mut if_stmt.test, |a, ctx| {
                Self::join_with_left_associative_op(test.span(), LogicalOperator::And, a, test, ctx)
            });
            ctx.replace_statement(&mut if_stmt.consequent, consequent);
        }

        Self::wrap_to_avoid_ambiguous_else(if_stmt, ctx);
    }

    /// Returns true when the current statement position accepts only a single
    /// statement, so rewriting to multiple statements requires a block wrapper.
    fn parent_requires_single_statement(ctx: &TraverseCtx<'a>) -> bool {
        matches!(
            ctx.parent(),
            Ancestor::ForStatementBody(_)
                | Ancestor::ForInStatementBody(_)
                | Ancestor::ForOfStatementBody(_)
                | Ancestor::WhileStatementBody(_)
                | Ancestor::DoWhileStatementBody(_)
                | Ancestor::IfStatementConsequent(_)
                | Ancestor::IfStatementAlternate(_)
                | Ancestor::LabeledStatementBody(_)
        )
    }

    fn should_invert_if(
        consequent: &Statement<'a>,
        alternate: &Statement<'a>,
        test: &Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        let is_alternate_terminated = alternate.is_jump_statement();
        let is_consequent_terminated = consequent.is_jump_statement();

        // When exactly one branch terminates (and parent is not `if`),
        // place the terminating branch in the consequent.
        // `if (a) c(); else return;` => `if (!a) return; else c();`
        // `if (!a) c(); else return;` => `if (a) return; else c();`
        if is_alternate_terminated != is_consequent_terminated
            && !Self::parent_requires_single_statement(ctx)
            && !Self::statement_cares_about_scope(consequent)
            && !Self::statement_cares_about_scope(alternate)
        {
            return is_alternate_terminated;
        }

        // Normalize: move the `!` out of the test by swapping branches.
        // `if (!a) b; else c;` => `if (a) c; else b;`
        // `if (!a) return b; else return c;` => `if (a) return c; else return b;`
        // Avoid swapping when alternate is an `if` — that risks a worse chain.
        if !matches!(alternate, Statement::IfStatement(_))
            && let Expression::UnaryExpression(unary_expr) = test
            && unary_expr.operator.is_not()
        {
            return true;
        }

        false
    }

    /// Wrap to avoid ambiguous else.
    /// `if (foo) if (bar) baz else quaz` ->  `if (foo) { if (bar) baz else quaz }`
    fn wrap_to_avoid_ambiguous_else(if_stmt: &mut IfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Statement::IfStatement(if2) = &if_stmt.consequent
            && if2.alternate.is_some()
        {
            let scope_id = ctx.create_child_scope_of_current(ScopeFlags::empty());
            let new_consequent = Statement::new_block_statement_with_scope_id(
                if_stmt.consequent.span(),
                [if_stmt.consequent.take_in(ctx)],
                scope_id,
                ctx,
            );
            ctx.replace_statement(&mut if_stmt.consequent, new_consequent);
        }
    }

    fn is_statement_empty(stmt: &Statement<'a>) -> bool {
        match stmt {
            Statement::BlockStatement(block_stmt) if block_stmt.body.is_empty() => true,
            Statement::EmptyStatement(_) => true,
            _ => false,
        }
    }
}
