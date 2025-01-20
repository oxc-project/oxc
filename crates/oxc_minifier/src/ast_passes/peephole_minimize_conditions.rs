use oxc_allocator::Vec;
use oxc_ast::{ast::*, NONE};
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, ValueType};
use oxc_span::{cmp::ContentEq, GetSpan};
use oxc_syntax::es_target::ESTarget;
use oxc_traverse::{traverse_mut_with_ctx, Ancestor, ReusableTraverseCtx, Traverse, TraverseCtx};

use crate::{ctx::Ctx, CompressorPass};

/// Minimize Conditions
///
/// A peephole optimization that minimizes conditional expressions according to De Morgan's laws.
/// Also rewrites conditional statements as expressions by replacing them
/// with `? :` and short-circuit binary operators.
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeMinimizeConditions.java>
pub struct PeepholeMinimizeConditions {
    #[allow(unused)]
    target: ESTarget,
    pub(crate) changed: bool,
}

impl<'a> CompressorPass<'a> for PeepholeMinimizeConditions {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        self.changed = false;
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for PeepholeMinimizeConditions {
    fn exit_statements(
        &mut self,
        stmts: &mut oxc_allocator::Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.try_replace_if(stmts, ctx);
        let changed = self.changed;
        while self.changed {
            self.changed = false;
            self.try_replace_if(stmts, ctx);
            if stmts.iter().any(|stmt| matches!(stmt, Statement::EmptyStatement(_))) {
                stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
            }
        }
        self.changed = self.changed || changed;
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let expr = match stmt {
            Statement::IfStatement(s) => Some(&mut s.test),
            Statement::WhileStatement(s) => Some(&mut s.test),
            Statement::ForStatement(s) => s.test.as_mut(),
            Statement::DoWhileStatement(s) => Some(&mut s.test),
            Statement::ExpressionStatement(s)
                if !matches!(
                    ctx.ancestry.ancestor(1),
                    Ancestor::ArrowFunctionExpressionBody(_)
                ) =>
            {
                Some(&mut s.expression)
            }
            _ => None,
        };

        if let Some(expr) = expr {
            Self::try_fold_expr_in_boolean_context(expr, Ctx(ctx));
        }

        if let Some(folded_stmt) = match stmt {
            // If the condition is a literal, we'll let other optimizations try to remove useless code.
            Statement::IfStatement(_) => Self::try_minimize_if(stmt, ctx),
            _ => None,
        } {
            *stmt = folded_stmt;
            self.changed = true;
        };
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        loop {
            let mut changed = false;
            if let Expression::ConditionalExpression(logical_expr) = expr {
                if let Some(e) = Self::try_minimize_conditional(logical_expr, ctx) {
                    *expr = e;
                    changed = true;
                }
            }
            if let Expression::ConditionalExpression(logical_expr) = expr {
                if Self::try_fold_expr_in_boolean_context(&mut logical_expr.test, Ctx(ctx)) {
                    changed = true;
                }
            }
            if changed {
                self.changed = true;
            } else {
                break;
            }
        }

        if let Some(folded_expr) = match expr {
            Expression::UnaryExpression(e) => Self::try_minimize_not(e, ctx),
            Expression::BinaryExpression(e) => Self::try_minimize_binary(e, ctx),
            _ => None,
        } {
            *expr = folded_expr;
            self.changed = true;
        };
    }
}

impl<'a> PeepholeMinimizeConditions {
    pub fn new(target: ESTarget) -> Self {
        Self { target, changed: false }
    }

    fn try_minimize_not(
        expr: &mut UnaryExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if !expr.operator.is_not() {
            return None;
        }
        match &mut expr.argument {
            Expression::UnaryExpression(e)
                if e.operator.is_not() && ValueType::from(&e.argument).is_boolean() =>
            {
                Some(ctx.ast.move_expression(&mut e.argument))
            }
            Expression::BinaryExpression(e) if e.operator.is_equality() => {
                e.operator = e.operator.equality_inverse_operator().unwrap();
                Some(ctx.ast.move_expression(&mut expr.argument))
            }
            _ => None,
        }
    }

    fn try_minimize_if(
        stmt: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        let Statement::IfStatement(if_stmt) = stmt else { unreachable!() };

        // `if (x) foo()` -> `x && foo()`
        if !if_stmt.test.is_literal() {
            let then_branch = &if_stmt.consequent;
            let else_branch = &if_stmt.alternate;
            match else_branch {
                None => {
                    if Self::is_foldable_express_block(&if_stmt.consequent) {
                        let right = Self::get_block_expression(&mut if_stmt.consequent, ctx);
                        let test = ctx.ast.move_expression(&mut if_stmt.test);
                        // `if(!x) foo()` -> `x || foo()`
                        if let Expression::UnaryExpression(unary_expr) = test {
                            if unary_expr.operator.is_not() {
                                let left = unary_expr.unbox().argument;
                                let logical_expr = ctx.ast.expression_logical(
                                    if_stmt.span,
                                    left,
                                    LogicalOperator::Or,
                                    right,
                                );
                                return Some(
                                    ctx.ast.statement_expression(if_stmt.span, logical_expr),
                                );
                            }
                        } else {
                            // `if(x) foo()` -> `x && foo()`
                            let logical_expr = ctx.ast.expression_logical(
                                if_stmt.span,
                                test,
                                LogicalOperator::And,
                                right,
                            );
                            return Some(ctx.ast.statement_expression(if_stmt.span, logical_expr));
                        }
                    } else {
                        // `if (x) if (y) z` -> `if (x && y) z`
                        if let Some(Statement::IfStatement(then_if_stmt)) =
                            then_branch.get_one_child()
                        {
                            if then_if_stmt.alternate.is_none() {
                                let and_left = ctx.ast.move_expression(&mut if_stmt.test);
                                let Some(then_if_stmt) = if_stmt.consequent.get_one_child_mut()
                                else {
                                    unreachable!()
                                };
                                let Statement::IfStatement(mut then_if_stmt) =
                                    ctx.ast.move_statement(then_if_stmt)
                                else {
                                    unreachable!()
                                };
                                let and_right = ctx.ast.move_expression(&mut then_if_stmt.test);
                                then_if_stmt.test = ctx.ast.expression_logical(
                                    and_left.span(),
                                    and_left,
                                    LogicalOperator::And,
                                    and_right,
                                );
                                return Some(Statement::IfStatement(then_if_stmt));
                            }
                        }
                    }
                }
                Some(else_branch) => {
                    let then_branch_is_expression_block =
                        Self::is_foldable_express_block(then_branch);
                    let else_branch_is_expression_block =
                        Self::is_foldable_express_block(else_branch);
                    // `if(foo) bar else baz` -> `foo ? bar : baz`
                    if then_branch_is_expression_block && else_branch_is_expression_block {
                        let test = ctx.ast.move_expression(&mut if_stmt.test);
                        let consequent = Self::get_block_expression(&mut if_stmt.consequent, ctx);
                        let else_branch = if_stmt.alternate.as_mut().unwrap();
                        let alternate = Self::get_block_expression(else_branch, ctx);
                        let expr = ctx.ast.expression_conditional(
                            if_stmt.span,
                            test,
                            consequent,
                            alternate,
                        );
                        return Some(ctx.ast.statement_expression(if_stmt.span, expr));
                    }
                }
            }
        }

        // `if (x) {} else foo` -> `if (!x) foo`
        if match &if_stmt.consequent {
            Statement::EmptyStatement(_) => true,
            Statement::BlockStatement(block_stmt) => block_stmt.body.is_empty(),
            _ => false,
        } && if_stmt.alternate.is_some()
        {
            return Some(ctx.ast.statement_if(
                if_stmt.span,
                ctx.ast.expression_unary(
                    if_stmt.test.span(),
                    UnaryOperator::LogicalNot,
                    ctx.ast.move_expression(&mut if_stmt.test),
                ),
                ctx.ast.move_statement(if_stmt.alternate.as_mut().unwrap()),
                None,
            ));
        }

        None
    }

    fn try_replace_if(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        for i in 0..stmts.len() {
            let Statement::IfStatement(if_stmt) = &stmts[i] else {
                continue;
            };
            let then_branch = &if_stmt.consequent;
            let else_branch = &if_stmt.alternate;
            let next_node = stmts.get(i + 1);

            if next_node.is_some_and(|s| matches!(s, Statement::IfStatement(_)))
                && else_branch.is_none()
                && Self::is_return_block(then_branch)
            {
                /* TODO */
            } else if next_node.is_some_and(Self::is_return_expression)
                && else_branch.is_none()
                && Self::is_return_block(then_branch)
            {
                // `if (x) return; return 1` -> `return x ? void 0 : 1`
                let Statement::IfStatement(if_stmt) = ctx.ast.move_statement(&mut stmts[i]) else {
                    unreachable!()
                };
                let mut if_stmt = if_stmt.unbox();
                let consequent = Self::get_block_return_expression(&mut if_stmt.consequent, ctx);
                let alternate = Self::take_return_argument(&mut stmts[i + 1], ctx);
                let argument = ctx.ast.expression_conditional(
                    if_stmt.span,
                    if_stmt.test,
                    consequent,
                    alternate,
                );
                stmts[i] = ctx.ast.statement_return(if_stmt.span, Some(argument));
                self.changed = true;
                break;
            } else if else_branch.is_some() && Self::statement_must_exit_parent(then_branch) {
                let Statement::IfStatement(if_stmt) = &mut stmts[i] else {
                    unreachable!();
                };
                let else_branch = if_stmt.alternate.take().unwrap();
                stmts.insert(i + 1, else_branch);
                self.changed = true;
            }
        }
    }

    fn is_foldable_express_block(stmt: &Statement<'a>) -> bool {
        matches!(stmt.get_one_child(), Some(Statement::ExpressionStatement(_)))
    }

    fn get_block_expression(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        let Some(Statement::ExpressionStatement(s)) = stmt.get_one_child_mut() else {
            unreachable!()
        };
        ctx.ast.move_expression(&mut s.expression)
    }

    fn is_return_block(stmt: &Statement<'a>) -> bool {
        matches!(stmt.get_one_child(), Some(Statement::ReturnStatement(_)))
    }

    fn is_return_expression(stmt: &Statement<'a>) -> bool {
        matches!(stmt, Statement::ReturnStatement(return_stmt) if return_stmt.argument.is_some())
    }

    fn statement_must_exit_parent(stmt: &Statement<'a>) -> bool {
        match stmt {
            Statement::ThrowStatement(_) | Statement::ReturnStatement(_) => true,
            Statement::BlockStatement(block_stmt) => {
                block_stmt.body.last().is_some_and(Self::statement_must_exit_parent)
            }
            _ => false,
        }
    }

    fn get_block_return_expression(
        stmt: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let Some(stmt) = stmt.get_one_child_mut() else { unreachable!() };
        Self::take_return_argument(stmt, ctx)
    }

    fn take_return_argument(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        let Statement::ReturnStatement(return_stmt) = ctx.ast.move_statement(stmt) else {
            unreachable!()
        };
        let return_stmt = return_stmt.unbox();
        match return_stmt.argument {
            Some(e) => e,
            None => ctx.ast.void_0(return_stmt.span),
        }
    }

    // https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L2745
    fn try_minimize_conditional(
        expr: &mut ConditionalExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        // `(a, b) ? c : d` -> `a, b ? c : d`
        if let Expression::SequenceExpression(sequence_expr) = &mut expr.test {
            if sequence_expr.expressions.len() > 1 {
                let span = expr.span();
                let mut sequence = ctx.ast.move_expression(&mut expr.test);
                let Expression::SequenceExpression(ref mut sequence_expr) = &mut sequence else {
                    unreachable!()
                };
                let test = sequence_expr.expressions.pop().unwrap();
                sequence_expr.expressions.push(ctx.ast.expression_conditional(
                    span,
                    test,
                    ctx.ast.move_expression(&mut expr.consequent),
                    ctx.ast.move_expression(&mut expr.alternate),
                ));
                return Some(sequence);
            }
        }

        // `!a ? b : c` -> `a ? c : b`
        if let Expression::UnaryExpression(test_expr) = &mut expr.test {
            if test_expr.operator.is_not()
                // Skip `!!!a`
                && !matches!(test_expr.argument, Expression::UnaryExpression(_))
            {
                let test = ctx.ast.move_expression(&mut test_expr.argument);
                let consequent = ctx.ast.move_expression(&mut expr.consequent);
                let alternate = ctx.ast.move_expression(&mut expr.alternate);
                return Some(
                    ctx.ast.expression_conditional(expr.span, test, alternate, consequent),
                );
            }
        }

        // `x != y ? b : c` -> `x == y ? c : b`
        if let Expression::BinaryExpression(test_expr) = &mut expr.test {
            if matches!(
                test_expr.operator,
                BinaryOperator::Inequality | BinaryOperator::StrictInequality
            ) {
                test_expr.operator = test_expr.operator.equality_inverse_operator().unwrap();
                let test = ctx.ast.move_expression(&mut expr.test);
                let consequent = ctx.ast.move_expression(&mut expr.consequent);
                let alternate = ctx.ast.move_expression(&mut expr.alternate);
                return Some(
                    ctx.ast.expression_conditional(expr.span, test, alternate, consequent),
                );
            }
        }

        // TODO: `/* @__PURE__ */ a() ? b : b` -> `b`

        // `a ? b : b` -> `a, b`
        if expr.alternate.content_eq(&expr.consequent) {
            let expressions = ctx.ast.vec_from_array([
                ctx.ast.move_expression(&mut expr.test),
                ctx.ast.move_expression(&mut expr.consequent),
            ]);
            return Some(ctx.ast.expression_sequence(expr.span, expressions));
        }

        // `a ? true : false` -> `!!a`
        // `a ? false : true` -> `!a`
        if let (
            Expression::Identifier(_),
            Expression::BooleanLiteral(consequent_lit),
            Expression::BooleanLiteral(alternate_lit),
        ) = (&expr.test, &expr.consequent, &expr.alternate)
        {
            match (consequent_lit.value, alternate_lit.value) {
                (false, true) => {
                    let ident = ctx.ast.move_expression(&mut expr.test);
                    return Some(ctx.ast.expression_unary(
                        expr.span,
                        UnaryOperator::LogicalNot,
                        ident,
                    ));
                }
                (true, false) => {
                    let ident = ctx.ast.move_expression(&mut expr.test);
                    return Some(ctx.ast.expression_unary(
                        expr.span,
                        UnaryOperator::LogicalNot,
                        ctx.ast.expression_unary(expr.span, UnaryOperator::LogicalNot, ident),
                    ));
                }
                _ => {}
            }
        }

        // `a ? a : b` -> `a || b`
        if let (Expression::Identifier(test_ident), Expression::Identifier(consequent_ident)) =
            (&expr.test, &expr.consequent)
        {
            if test_ident.name == consequent_ident.name {
                return Some(ctx.ast.expression_logical(
                    expr.span,
                    ctx.ast.move_expression(&mut expr.test),
                    LogicalOperator::Or,
                    ctx.ast.move_expression(&mut expr.alternate),
                ));
            }
        }

        // `a ? b : a` -> `a && b`
        if let (Expression::Identifier(test_ident), Expression::Identifier(alternate_ident)) =
            (&expr.test, &expr.alternate)
        {
            if test_ident.name == alternate_ident.name {
                return Some(ctx.ast.expression_logical(
                    expr.span,
                    ctx.ast.move_expression(&mut expr.test),
                    LogicalOperator::And,
                    ctx.ast.move_expression(&mut expr.consequent),
                ));
            }
        }

        // `a ? b ? c : d : d` -> `a && b ? c : d`
        if let Expression::ConditionalExpression(consequent) = &mut expr.consequent {
            if consequent.alternate.content_eq(&expr.alternate) {
                return Some(ctx.ast.expression_conditional(
                    expr.span,
                    ctx.ast.expression_logical(
                        expr.test.span(),
                        ctx.ast.move_expression(&mut expr.test),
                        LogicalOperator::And,
                        ctx.ast.move_expression(&mut consequent.test),
                    ),
                    ctx.ast.move_expression(&mut consequent.consequent),
                    ctx.ast.move_expression(&mut consequent.alternate),
                ));
            }
        }

        // `a ? b : c ? b : d` -> `a || c ? b : d`
        if let Expression::ConditionalExpression(alternate) = &mut expr.alternate {
            if alternate.consequent.content_eq(&expr.consequent) {
                return Some(ctx.ast.expression_conditional(
                    expr.span,
                    ctx.ast.expression_logical(
                        expr.test.span(),
                        ctx.ast.move_expression(&mut expr.test),
                        LogicalOperator::Or,
                        ctx.ast.move_expression(&mut alternate.test),
                    ),
                    ctx.ast.move_expression(&mut expr.consequent),
                    ctx.ast.move_expression(&mut alternate.alternate),
                ));
            }
        }

        // `a ? c : (b, c)` -> `(a || b), c`
        if let Expression::SequenceExpression(alternate) = &mut expr.alternate {
            if alternate.expressions.len() == 2
                && alternate.expressions[1].content_eq(&expr.consequent)
            {
                return Some(ctx.ast.expression_sequence(
                    expr.span,
                    ctx.ast.vec_from_array([
                        ctx.ast.expression_logical(
                            expr.test.span(),
                            ctx.ast.move_expression(&mut expr.test),
                            LogicalOperator::Or,
                            ctx.ast.move_expression(&mut alternate.expressions[0]),
                        ),
                        ctx.ast.move_expression(&mut expr.consequent),
                    ]),
                ));
            }
        }

        // `a ? (b, c) : c` -> `(a && b), c`
        if let Expression::SequenceExpression(consequent) = &mut expr.consequent {
            if consequent.expressions.len() == 2
                && consequent.expressions[1].content_eq(&expr.alternate)
            {
                return Some(ctx.ast.expression_sequence(
                    expr.span,
                    ctx.ast.vec_from_array([
                        ctx.ast.expression_logical(
                            expr.test.span(),
                            ctx.ast.move_expression(&mut expr.test),
                            LogicalOperator::And,
                            ctx.ast.move_expression(&mut consequent.expressions[0]),
                        ),
                        ctx.ast.move_expression(&mut expr.alternate),
                    ]),
                ));
            }
        }

        // `a ? b || c : c` => "(a && b) || c"
        if let Expression::LogicalExpression(logical_expr) = &mut expr.consequent {
            if logical_expr.operator == LogicalOperator::Or
                && logical_expr.right.content_eq(&expr.alternate)
            {
                return Some(ctx.ast.expression_logical(
                    expr.span,
                    ctx.ast.expression_logical(
                        expr.test.span(),
                        ctx.ast.move_expression(&mut expr.test),
                        LogicalOperator::And,
                        ctx.ast.move_expression(&mut logical_expr.left),
                    ),
                    LogicalOperator::Or,
                    ctx.ast.move_expression(&mut expr.alternate),
                ));
            }
        }

        // `a ? c : b && c` -> `(a || b) && c`
        if let Expression::LogicalExpression(logical_expr) = &mut expr.alternate {
            if logical_expr.operator == LogicalOperator::And
                && logical_expr.right.content_eq(&expr.consequent)
            {
                return Some(ctx.ast.expression_logical(
                    expr.span,
                    ctx.ast.expression_logical(
                        expr.test.span(),
                        ctx.ast.move_expression(&mut expr.test),
                        LogicalOperator::Or,
                        ctx.ast.move_expression(&mut logical_expr.left),
                    ),
                    LogicalOperator::And,
                    ctx.ast.move_expression(&mut expr.consequent),
                ));
            }
        }

        // `a ? b(c, d) : b(e, d)` -> `b(a ? c : e, d)``
        if let (
            Expression::Identifier(test),
            Expression::CallExpression(consequent),
            Expression::CallExpression(alternate),
        ) = (&expr.test, &mut expr.consequent, &mut expr.alternate)
        {
            if consequent.callee.content_eq(&alternate.callee)
                && consequent.arguments.len() == alternate.arguments.len()
                && ctx.scopes().find_binding(ctx.current_scope_id(), &test.name).is_some()
                && consequent
                    .arguments
                    .iter()
                    .zip(&alternate.arguments)
                    .skip(1)
                    .all(|(a, b)| a.content_eq(b))
            {
                // `a ? b(...c) : b(...e)` -> `b(...a ? c : e)``
                if matches!(consequent.arguments[0], Argument::SpreadElement(_))
                    && matches!(alternate.arguments[0], Argument::SpreadElement(_))
                {
                    let callee = ctx.ast.move_expression(&mut consequent.callee);
                    let consequent_first_arg = {
                        let Argument::SpreadElement(ref mut el) = &mut consequent.arguments[0]
                        else {
                            unreachable!()
                        };
                        ctx.ast.move_expression(&mut el.argument)
                    };
                    let alternate_first_arg = {
                        let Argument::SpreadElement(ref mut el) = &mut alternate.arguments[0]
                        else {
                            unreachable!()
                        };
                        ctx.ast.move_expression(&mut el.argument)
                    };
                    let mut args = std::mem::replace(&mut consequent.arguments, ctx.ast.vec());
                    args[0] = ctx.ast.argument_spread_element(
                        expr.span,
                        ctx.ast.expression_conditional(
                            expr.test.span(),
                            ctx.ast.move_expression(&mut expr.test),
                            consequent_first_arg,
                            alternate_first_arg,
                        ),
                    );

                    return Some(ctx.ast.expression_call(expr.span, callee, NONE, args, false));
                }
                // `a ? b(c) : b(e)` -> `b(a ? c : e)``
                if !matches!(consequent.arguments[0], Argument::SpreadElement(_))
                    && !matches!(alternate.arguments[0], Argument::SpreadElement(_))
                {
                    let callee = ctx.ast.move_expression(&mut consequent.callee);

                    let consequent_first_arg =
                        ctx.ast.move_expression(consequent.arguments[0].to_expression_mut());
                    let alternate_first_arg =
                        ctx.ast.move_expression(alternate.arguments[0].to_expression_mut());
                    let mut args = std::mem::replace(&mut consequent.arguments, ctx.ast.vec());
                    args[0] = Argument::from(ctx.ast.expression_conditional(
                        expr.test.span(),
                        ctx.ast.move_expression(&mut expr.test),
                        consequent_first_arg,
                        alternate_first_arg,
                    ));
                    return Some(ctx.ast.expression_call(expr.span, callee, NONE, args, false));
                }
            }
        }

        // TODO: Try using the "??" or "?." operators

        // Non esbuild optimizations

        // `x ? true : y` -> `x || y`
        // `x ? false : y` -> `!x && y`
        if let (Expression::Identifier(_), Expression::BooleanLiteral(consequent_lit), _) =
            (&expr.test, &expr.consequent, &expr.alternate)
        {
            if consequent_lit.value {
                let ident = ctx.ast.move_expression(&mut expr.test);
                return Some(ctx.ast.expression_logical(
                    expr.span,
                    ctx.ast.expression_unary(
                        ident.span(),
                        UnaryOperator::LogicalNot,
                        ctx.ast.expression_unary(ident.span(), UnaryOperator::LogicalNot, ident),
                    ),
                    LogicalOperator::Or,
                    ctx.ast.move_expression(&mut expr.alternate),
                ));
            }
            let ident = ctx.ast.move_expression(&mut expr.test);
            return Some(ctx.ast.expression_logical(
                expr.span,
                ctx.ast.expression_unary(expr.span, UnaryOperator::LogicalNot, ident),
                LogicalOperator::And,
                ctx.ast.move_expression(&mut expr.alternate),
            ));
        }

        // `x ? y : true` -> `!x || y`
        // `x ? y : false` -> `x && y`
        if let (Expression::Identifier(_), _, Expression::BooleanLiteral(alternate_lit)) =
            (&expr.test, &expr.consequent, &expr.alternate)
        {
            if alternate_lit.value {
                let ident = ctx.ast.move_expression(&mut expr.test);
                return Some(ctx.ast.expression_logical(
                    expr.span,
                    ctx.ast.expression_unary(expr.span, UnaryOperator::LogicalNot, ident),
                    LogicalOperator::Or,
                    ctx.ast.move_expression(&mut expr.consequent),
                ));
            }
            let ident = ctx.ast.move_expression(&mut expr.test);
            return Some(ctx.ast.expression_logical(
                expr.span,
                ctx.ast.expression_unary(
                    expr.span,
                    UnaryOperator::LogicalNot,
                    ctx.ast.expression_unary(ident.span(), UnaryOperator::LogicalNot, ident),
                ),
                LogicalOperator::And,
                ctx.ast.move_expression(&mut expr.consequent),
            ));
        }

        None
    }

    /// Simplify syntax when we know it's used inside a boolean context, e.g. `if (boolean_context) {}`.
    ///
    /// <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L2059>
    fn try_fold_expr_in_boolean_context(expr: &mut Expression<'a>, ctx: Ctx<'a, '_>) -> bool {
        match expr {
            // "!!a" => "a"
            Expression::UnaryExpression(u1) if u1.operator.is_not() => {
                if let Expression::UnaryExpression(u2) = &mut u1.argument {
                    if u2.operator.is_not() {
                        let mut e = ctx.ast.move_expression(&mut u2.argument);
                        Self::try_fold_expr_in_boolean_context(&mut e, ctx);
                        *expr = e;
                        return true;
                    }
                }
            }
            Expression::BinaryExpression(e)
                if e.operator.is_equality()
                    && matches!(&e.right, Expression::NumericLiteral(lit) if lit.value == 0.0)
                    && ValueType::from(&e.left).is_number() =>
            {
                let argument = ctx.ast.move_expression(&mut e.left);
                *expr = if matches!(
                    e.operator,
                    BinaryOperator::StrictInequality | BinaryOperator::Inequality
                ) {
                    // `if ((a | b) !== 0)` -> `if (a | b);`
                    argument
                } else {
                    // `if ((a | b) === 0);", "if (!(a | b));")`
                    ctx.ast.expression_unary(e.span, UnaryOperator::LogicalNot, argument)
                };
                return true;
            }
            // "if (!!a && !!b)" => "if (a && b)"
            Expression::LogicalExpression(e) if e.operator == LogicalOperator::And => {
                Self::try_fold_expr_in_boolean_context(&mut e.left, ctx);
                Self::try_fold_expr_in_boolean_context(&mut e.right, ctx);
                // "if (anything && truthyNoSideEffects)" => "if (anything)"
                if ctx.get_side_free_boolean_value(&e.right) == Some(true) {
                    *expr = ctx.ast.move_expression(&mut e.left);
                    return true;
                }
            }
            // "if (!!a ||!!b)" => "if (a || b)"
            Expression::LogicalExpression(e) if e.operator == LogicalOperator::Or => {
                Self::try_fold_expr_in_boolean_context(&mut e.left, ctx);
                Self::try_fold_expr_in_boolean_context(&mut e.right, ctx);
                // "if (anything || falsyNoSideEffects)" => "if (anything)"
                if ctx.get_side_free_boolean_value(&e.right) == Some(false) {
                    *expr = ctx.ast.move_expression(&mut e.left);
                    return true;
                }
            }
            Expression::ConditionalExpression(e) => {
                // "if (a ? !!b : !!c)" => "if (a ? b : c)"
                Self::try_fold_expr_in_boolean_context(&mut e.consequent, ctx);
                Self::try_fold_expr_in_boolean_context(&mut e.alternate, ctx);
                if let Some(boolean) = ctx.get_side_free_boolean_value(&e.consequent) {
                    let right = ctx.ast.move_expression(&mut e.alternate);
                    let left = ctx.ast.move_expression(&mut e.test);
                    if boolean {
                        // "if (anything1 ? truthyNoSideEffects : anything2)" => "if (anything1 || anything2)"
                        *expr =
                            ctx.ast.expression_logical(e.span(), left, LogicalOperator::Or, right);
                    } else {
                        // "if (anything1 ? falsyNoSideEffects : anything2)" => "if (!anything1 && anything2)"
                        let left =
                            ctx.ast.expression_unary(left.span(), UnaryOperator::LogicalNot, left);
                        *expr =
                            ctx.ast.expression_logical(e.span(), left, LogicalOperator::And, right);
                    }
                    return true;
                }
                if let Some(boolean) = ctx.get_side_free_boolean_value(&e.alternate) {
                    let left = ctx.ast.move_expression(&mut e.test);
                    let right = ctx.ast.move_expression(&mut e.consequent);
                    if boolean {
                        // "if (anything1 ? anything2 : truthyNoSideEffects)" => "if (!anything1 || anything2)"
                        let left =
                            ctx.ast.expression_unary(left.span(), UnaryOperator::LogicalNot, left);
                        *expr =
                            ctx.ast.expression_logical(e.span(), left, LogicalOperator::Or, right);
                    } else {
                        // "if (anything1 ? anything2 : falsyNoSideEffects)" => "if (anything1 && anything2)"
                        *expr =
                            ctx.ast.expression_logical(e.span(), left, LogicalOperator::And, right);
                    }
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    // `typeof foo === 'number'` -> `typeof foo == 'number'`
    //  ^^^^^^^^^^ `ValueType::from(&e.left).is_string()` is `true`.
    // `a instanceof b === true` -> `a instanceof b`
    // `a instanceof b === false` -> `!(a instanceof b)`
    //  ^^^^^^^^^^^^^^ `ValueType::from(&e.left).is_boolean()` is `true`.
    // `x >> y !== 0` -> `x >> y`
    //  ^^^^^^ ValueType::from(&e.left).is_number()` is `true`.
    fn try_minimize_binary(
        e: &mut BinaryExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if !e.operator.is_equality() {
            return None;
        }
        let left = ValueType::from(&e.left);
        let right = ValueType::from(&e.right);
        if left.is_undetermined() || right.is_undetermined() {
            return None;
        }
        if left == right {
            match e.operator {
                BinaryOperator::StrictInequality => {
                    e.operator = BinaryOperator::Inequality;
                }
                BinaryOperator::StrictEquality => {
                    e.operator = BinaryOperator::Equality;
                }
                _ => {}
            }
        }
        match &mut e.right {
            Expression::BooleanLiteral(b) if left.is_boolean() => {
                match e.operator {
                    BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                        e.operator = BinaryOperator::Equality;
                        b.value = !b.value;
                    }
                    BinaryOperator::StrictEquality => {
                        e.operator = BinaryOperator::Equality;
                    }
                    BinaryOperator::Equality => {}
                    _ => return None,
                }
                Some(if b.value {
                    ctx.ast.move_expression(&mut e.left)
                } else {
                    let argument = ctx.ast.move_expression(&mut e.left);
                    ctx.ast.expression_unary(e.span, UnaryOperator::LogicalNot, argument)
                })
            }
            _ => None,
        }
    }
}

/// <https://github.com/google/closure-compiler/blob/v20240609/test/com/google/javascript/jscomp/PeepholeMinimizeConditionsTest.java>
#[cfg(test)]
mod test {
    use crate::tester::{test, test_same};

    /** Check that removing blocks with 1 child works */
    #[test]
    fn test_fold_one_child_blocks() {
        test("function f(){if(x)a();x=3}", "function f(){x&&a(),x=3}");
        test("function f(){if(x)a?.();x=3}", "function f(){x&&a?.(),x=3}");

        test("function f(){if(x){a()}x=3}", "function f(){x&&a(),x=3}");
        test("function f(){if(x){a?.()}x=3}", "function f(){x&&a?.(),x=3}");

        // test("function f(){if(x){return 3}}", "function f(){if(x)return 3}");
        test("function f(){if(x){a()}}", "function f(){x&&a()}");
        // test("function f(){if(x){throw 1}}", "function f(){if(x)throw 1;}");

        // Try it out with functions
        test("function f(){if(x){foo()}}", "function f(){x&&foo()}");
        test("function f(){if(x){foo()}else{bar()}}", "function f(){x?foo():bar()}");

        // Try it out with properties and methods
        test("function f(){if(x){a.b=1}}", "function f(){x&&(a.b=1)}");
        test("function f(){if(x){a.b*=1}}", "function f(){x&&(a.b*=1)}");
        test("function f(){if(x){a.b+=1}}", "function f(){x&&(a.b+=1)}");
        test("function f(){if(x){++a.b}}", "function f(){x&&++a.b}");
        test("function f(){if(x){a.foo()}}", "function f(){x&&a.foo()}");
        test("function f(){if(x){a?.foo()}}", "function f(){x&&a?.foo()}");

        // Try it out with throw/catch/finally [which should not change]
        test_same("function f(){try{foo()}catch(e){bar(e)}finally{baz()}}");

        // Try it out with switch statements
        test_same("function f(){switch(x){case 1:break}}");

        // Do while loops stay in a block if that's where they started
        test(
            "function f(){if(e1){do foo();while(e2)}else foo2()}",
            "function f() { if (e1) do foo(); while (e2); else foo2(); }",
        );
        // Test an obscure case with do and while
        // test("if(x){do{foo()}while(y)}else bar()", "if(x){do foo();while(y)}else bar()");

        // Play with nested IFs
        test("function f(){if(x){if(y)foo()}}", "function f(){x && (y && foo())}");
        test("function f(){if(x){if(y)foo();else bar()}}", "function f(){x&&(y?foo():bar())}");
        test("function f(){if(x){if(y)foo()}else bar()}", "function f(){x?y&&foo():bar()}");
        test(
            "function f(){if(x){if(y)foo();else bar()}else{baz()}}",
            "function f(){x?y?foo():bar():baz()}",
        );

        // test("if(e1){while(e2){if(e3){foo()}}}else{bar()}", "if(e1)while(e2)e3&&foo();else bar()");

        // test("if(e1){with(e2){if(e3){foo()}}}else{bar()}", "if(e1)with(e2)e3&&foo();else bar()");

        // test("if(a||b){if(c||d){var x;}}", "if(a||b)if(c||d)var x");
        // test("if(x){ if(y){var x;}else{var z;} }", "if(x)if(y)var x;else var z");

        // NOTE - technically we can remove the blocks since both the parent
        // and child have elses. But we don't since it causes ambiguities in
        // some cases where not all descendent ifs having elses
        // test(
        // "if(x){ if(y){var x;}else{var z;} }else{var w}",
        // "if(x)if(y)var x;else var z;else var w",
        // );
        // test("if (x) {var x;}else { if (y) { var y;} }", "if(x)var x;else if(y)var y");

        // Here's some of the ambiguous cases
        // test(
        // "if(a){if(b){f1();f2();}else if(c){f3();}}else {if(d){f4();}}",
        // "if(a)if(b){f1();f2()}else c&&f3();else d&&f4()",
        // );

        test_same("function f(){foo()}");
        test_same("switch(x){case y: foo()}");
        test(
            "try{foo()}catch(ex){bar()}finally{baz()}",
            "try { foo(); } catch { bar(); } finally { baz(); }",
        );

        // Dot not fold `let` and `const`.
        // Lexical declaration cannot appear in a single-statement context.
        test_same("if (foo) { const bar = 1 } else { const baz = 1 }");
        test_same("if (foo) { let bar = 1 } else { let baz = 1 }");
        // test(
        // "if (foo) { var bar = 1 } else { var baz = 1 }",
        // "if (foo) var bar = 1; else var baz = 1;",
        // );
    }

    #[test]
    fn test_fold_returns() {
        test("function f(){if(x)return 1;else return 2}", "function f(){return x?1:2}");
        test("function f(){if(x)return 1;return 2}", "function f(){return x?1:2}");
        test("function f(){if(x)return;return 2}", "function f(){return x?void 0:2}");
        test("function f(){if(x)return 1+x;else return 2-x}", "function f(){return x?1+x:2-x}");
        test("function f(){if(x)return 1+x;return 2-x}", "function f(){return x?1+x:2-x}");
        test(
            "function f(){if(x)return y += 1;else return y += 2}",
            "function f(){return x?(y+=1):(y+=2)}",
        );

        test("function f(){if(x)return;else return 2-x}", "function f(){return x?void 0:2-x}");
        test("function f(){if(x)return;return 2-x}", "function f(){return x?void 0:2-x}");
        test("function f(){if(x)return x;else return}", "function f(){if(x)return x;}");
        test("function f(){if(x)return x;return}", "function f(){if(x)return x}");

        test(
            "function f(){for(var x in y) { return x.y; } return k}",
            "function f() { for (var x in y) return x.y; return k; }",
        );
    }

    #[test]
    #[ignore]
    fn test_combine_ifs1() {
        test(
            "function f() {if (x) return 1; if (y) return 1}",
            "function f() {if (x||y) return 1;}",
        );
        test(
            "function f() {if (x) return 1; if (y) foo(); else return 1}",
            "function f() {if ((!x)&&y) foo(); else return 1;}",
        );
    }

    #[test]
    #[ignore]
    fn test_combine_ifs2() {
        // combinable but not yet done
        test_same("function f() {if (x) throw 1; if (y) throw 1}");
        // Can't combine, side-effect
        test("function f(){ if (x) g(); if (y) g() }", "function f(){ x&&g(); y&&g() }");
        test("function f(){ if (x) g?.(); if (y) g?.() }", "function f(){ x&&g?.(); y&&g?.() }");
        // Can't combine, side-effect
        test(
            "function f(){ if (x) y = 0; if (y) y = 0; }",
            "function f(){ x&&(y = 0); y&&(y = 0); }",
        );
    }

    #[test]
    #[ignore]
    fn test_combine_ifs3() {
        test_same("function f() {if (x) return 1; if (y) {g();f()}}");
    }

    /** Try to minimize assignments */
    #[test]
    #[ignore]
    fn test_fold_assignments() {
        test("function f(){if(x)y=3;else y=4;}", "function f(){y=x?3:4}");
        test("function f(){if(x)y=1+a;else y=2+a;}", "function f(){y=x?1+a:2+a}");

        // and operation assignments
        test("function f(){if(x)y+=1;else y+=2;}", "function f(){y+=x?1:2}");
        test("function f(){if(x)y-=1;else y-=2;}", "function f(){y-=x?1:2}");
        test("function f(){if(x)y%=1;else y%=2;}", "function f(){y%=x?1:2}");
        test("function f(){if(x)y|=1;else y|=2;}", "function f(){y|=x?1:2}");

        // Don't fold if the 2 ops don't match.
        test_same("function f(){x ? y-=1 : y+=2}");

        // Don't fold if the 2 LHS don't match.
        test_same("function f(){x ? y-=1 : z-=1}");

        // Don't fold if there are potential effects.
        test_same("function f(){x ? y().a=3 : y().a=4}");
    }

    #[test]
    #[ignore]
    fn test_remove_duplicate_statements() {
        // TODO(bradfordcsmith): Stop normalizing the expected output or document why it is necessary.
        // enableNormalizeExpectedOutput();
        test("if (a) { x = 1; x++ } else { x = 2; x++ }", "x=(a) ? 1 : 2; x++");
        test(
            concat!(
                "if (a) { x = 1; x++; y += 1; z = pi; }",
                " else  { x = 2; x++; y += 1; z = pi; }"
            ),
            "x=(a) ? 1 : 2; x++; y += 1; z = pi;",
        );
        test(
            concat!("function z() {", "if (a) { foo(); return !0 } else { goo(); return !0 }", "}"),
            "function z() {(a) ? foo() : goo(); return !0}",
        );
        test(
            concat!(
                "function z() {if (a) { foo(); x = true; return true ",
                "} else { goo(); x = true; return true }}"
            ),
            "function z() {(a) ? foo() : goo(); x = true; return true}",
        );

        test(
            concat!(
                "function z() {",
                "  if (a) { bar(); foo(); return true }",
                "    else { bar(); goo(); return true }",
                "}"
            ),
            concat!(
                "function z() {",
                "  if (a) { bar(); foo(); }",
                "    else { bar(); goo(); }",
                "  return true;",
                "}"
            ),
        );
    }

    #[test]
    fn test_fold_returns_integration2() {
        // if-then-else duplicate statement removal handles this case:
        test(
            "function test(a) {if (a) {const a = Math.random();if(a) {return a;}} return a; }",
            "function test(a) { if (a) { const a = Math.random(); if (a) return a; } return a; }",
        );
    }

    #[test]
    fn test_dont_remove_duplicate_statements_without_normalization() {
        // In the following test case, we can't remove the duplicate "alert(x);" lines since each "x"
        // refers to a different variable.
        // We only try removing duplicate statements if the AST is normalized and names are unique.
        test_same(
            "if (Math.random() < 0.5) { const x = 3; alert(x); } else { const x = 5; alert(x); }",
        );
    }

    #[test]
    fn test_not_cond() {
        test("function f(){if(!x)foo()}", "function f(){x||foo()}");
        test("function f(){if(!x)b=1}", "function f(){x||(b=1)}");
        // test("if(!x)z=1;else if(y)z=2", "x ? y&&(z=2) : z=1;");
        // test("if(x)y&&(z=2);else z=1;", "x ? y&&(z=2) : z=1");
        test("function f(){if(!(x=1))a.b=1}", "function f(){(x=1)||(a.b=1)}");
    }

    #[test]
    #[ignore]
    fn test_and_parentheses_count() {
        test("function f(){if(x||y)a.foo()}", "function f(){(x||y)&&a.foo()}");
        test("function f(){if(x.a)x.a=0}", "function f(){x.a&&(x.a=0)}");
        test("function f(){if(x?.a)x.a=0}", "function f(){x?.a&&(x.a=0)}");
        test_same("function f(){if(x()||y()){x()||y()}}");
    }

    #[test]
    #[ignore]
    fn test_fold_logical_op_string_compare() {
        // side-effects
        // There is two way to parse two &&'s and both are correct.
        test("if (foo() && false) z()", "(foo(), 0) && z()");
    }

    #[test]
    #[ignore]
    fn test_fold_not() {
        test("while(!(x==y)){a=b;}", "while(x!=y){a=b;}");
        test("while(!(x!=y)){a=b;}", "while(x==y){a=b;}");
        test("while(!(x===y)){a=b;}", "while(x!==y){a=b;}");
        test("while(!(x!==y)){a=b;}", "while(x===y){a=b;}");
        // Because !(x<NaN) != x>=NaN don't fold < and > cases.
        test_same("while(!(x>y)){a=b;}");
        test_same("while(!(x>=y)){a=b;}");
        test_same("while(!(x<y)){a=b;}");
        test_same("while(!(x<=y)){a=b;}");
        test_same("while(!(x<=NaN)){a=b;}");

        // NOT forces a boolean context
        test("x = !(y() && true)", "x = !y()");
        // This will be further optimized by PeepholeFoldConstants.
        test("x = !true", "x = !1");
    }

    #[test]
    fn test_fold_triple_not() {
        test("!!!foo ? bar : baz", "foo ? baz : bar");
    }

    #[test]
    fn test_minimize_expr_condition() {
        test("(x ? true : false) && y()", "x && y()");
        test("(x ? false : true) && y()", "!x && y()");
        test("(x ? true : y) && y()", "(x || y) && y();");
        test("(x ? y : false) && y()", "(x && y) && y()");
        test("var x; (x && true) && y()", "var x; x && y()");
        test("var x; (x && false) && y()", "var x; x && !1");
        test("(x && true) && y()", "x && y()");
        test("(x && false) && y()", "x && !1");
        test("var x; (x || true) && y()", "var x; x || !0, y()");
        test("var x; (x || false) && y()", "var x; x && y()");

        test("(x || true) && y()", "x || !0, y()");
        test("(x || false) && y()", "x && y()");

        test("let x = foo ? true : false", "let x = !!foo");
        test("let x = foo ? true : bar", "let x = !!foo || bar");
        test("let x = foo ? bar : false", "let x = !!foo && bar");
        test("function x () { return a ? true : false }", "function x() { return !!a }");
        test("function x () { return a ? false : true }", "function x() { return !a }");
        test("function x () { return a ? true : b }", "function x() { return !!a || b }");
        // can't be minified e.g. `a = ''` would return `''`
        test("function x() { return a && true }", "function x() { return a && !0 }");

        test("foo ? bar : bar", "foo, bar");
        test_same("foo ? bar : baz");
        test("foo() ? bar : bar", "foo(), bar");

        test_same("var k = () => !!x;");
    }

    #[test]
    #[ignore]
    fn test_minimize_while_condition() {
        // This test uses constant folding logic, so is only here for completeness.
        test("while(!!true) foo()", "while(1) foo()");
        // These test tryMinimizeCondition
        test("while(!!x) foo()", "while(x) foo()");
        test("while(!(!x&&!y)) foo()", "while(x||y) foo()");
        test("while(x||!!y) foo()", "while(x||y) foo()");
        test("while(!(!!x&&y)) foo()", "while(!x||!y) foo()");
        test("while(!(!x&&y)) foo()", "while(x||!y) foo()");
        test("while(!(x||!y)) foo()", "while(!x&&y) foo()");
        test("while(!(x||y)) foo()", "while(!x&&!y) foo()");
        test("while(!(!x||y-z)) foo()", "while(x&&!(y-z)) foo()");
        test("while(!(!(x/y)||z+w)) foo()", "while(x/y&&!(z+w)) foo()");
        test_same("while(!(x+y||z)) foo()");
        test_same("while(!(x&&y*z)) foo()");
        test("while(!(!!x&&y)) foo()", "while(!x||!y) foo()");
        test("while(x&&!0) foo()", "while(x) foo()");
        test("while(x||!1) foo()", "while(x) foo()");
        test("while(!((x,y)&&z)) foo()", "while((x,!y)||!z) foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan_remove_leading_not() {
        test("if(!(!a||!b)&&c) foo()", "((a&&b)&&c)&&foo()");
        test("if(!(x&&y)) foo()", "x&&y||foo()");
        test("if(!(x||y)) foo()", "(x||y)||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan1() {
        test("if(!a&&!b)foo()", "(a||b)||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan2() {
        // Make sure trees with cloned functions are marked as changed
        test("(!(a&&!((function(){})())))||foo()", "!a||(function(){})()||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan2b() {
        // Make sure unchanged trees with functions are not marked as changed
        test_same("!a||(function(){})()||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan3() {
        test("if((!a||!b)&&(c||d)) foo()", "(a&&b||!c&&!d)||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan5() {
        test("if((!a||!b)&&c) foo()", "(a&&b||!c)||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan11() {
        test(
            "if (x && (y===2 || !f()) && (y===3 || !h())) foo()",
            "(!x || y!==2 && f() || y!==3 && h()) || foo()",
        );
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan20a() {
        test(
            "if (0===c && (2===a || 1===a)) f(); else g()",
            "if (0!==c || 2!==a && 1!==a) g(); else f()",
        );
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan20b() {
        test("if (0!==c || 2!==a && 1!==a) g(); else f()", "(0!==c || 2!==a && 1!==a) ? g() : f()");
    }

    #[test]
    fn test_preserve_if() {
        test_same("if(!a&&!b)for(;f(););");
    }

    #[test]
    fn test_no_swap_with_dangling_else() {
        test_same("if(!x) {for(;;)foo(); for(;;)bar()} else if(y) for(;;) f()");
        test_same("if(!a&&!b) {for(;;)foo(); for(;;)bar()} else if(y) for(;;) f()");
    }

    #[test]
    fn test_minimize_hook() {
        test("x ? x : y", "x || y");
        test_same("x.y ? x.y : x.z");
        test_same("x?.y ? x?.y : x.z");
        test_same("x?.y ? x?.y : x?.z");

        test_same("x() ? x() : y()");
        test_same("x?.() ? x?.() : y()");

        test("!x ? foo() : bar()", "x ? bar() : foo()");
        // TODO
        // test("while(!(x ? y : z)) foo();", "while(x ? !y : !z) foo();");
        // test("(x ? !y : !z) ? foo() : bar()", "(x ? y : z) ? bar() : foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_comma() {
        test("while(!(inc(), test())) foo();", "while(inc(), !test()) foo();");
        test("(inc(), !test()) ? foo() : bar()", "(inc(), test()) ? bar() : foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_expr_result() {
        test("!x||!y", "x&&y");
        test("if(!(x&&!y)) foo()", "(!x||y)&&foo()");
        test("if(!x||y) foo()", "(!x||y)&&foo()");
        test("(!x||y)&&foo()", "x&&!y||!foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan21() {
        test("if (0===c && (2===a || 1===a)) f()", "(0!==c || 2!==a && 1!==a) || f()");
    }

    #[test]
    #[ignore]
    fn test_minimize_and_or1() {
        test("if ((!a || !b) && (d || e)) f()", "(a&&b || !d&&!e) || f()");
    }

    #[test]
    fn test_minimize_for_condition() {
        // This test uses constant folding logic, so is only here for completeness.
        // These could be simplified to "for(;;) ..."
        test("for(;!!true;) foo()", "for(;;) foo()");
        // Verify function deletion tracking.
        // test("if(!!true||function(){}) {}", "if(1) {}");
        // Don't bother with FOR inits as there are normalized out.
        test("for(!!true;;) foo()", "for(!0;;) foo()");

        // These test tryMinimizeCondition
        test("for(;!!x;) foo()", "for(;x;) foo()");

        test_same("for(a in b) foo()");
        test_same("for(a in {}) foo()");
        test_same("for(a in []) foo()");
        test("for(a in !!true) foo()", "for(a in !0) foo()");

        test_same("for(a of b) foo()");
        test_same("for(a of {}) foo()");
        test_same("for(a of []) foo()");
        test("for(a of !!true) foo()", "for(a of !0) foo()");
    }

    #[test]
    fn test_minimize_condition_example1() {
        // Based on a real failing code sample.
        test("if(!!(f() > 20)) {foo();foo()}", "f() > 20 && (foo(), foo())");
    }

    #[test]
    #[ignore]
    fn test_fold_loop_break_late() {
        test("for(;;) if (a) break", "for(;!a;);");
        test_same("for(;;) if (a) { f(); break }");
        test("for(;;) if (a) break; else f()", "for(;!a;) { { f(); } }");
        test("for(;a;) if (b) break", "for(;a && !b;);");
        test("for(;a;) { if (b) break; if (c) break; }", "for(;(a && !b);) if (c) break;");
        test("for(;(a && !b);) if (c) break;", "for(;(a && !b) && !c;);");
        test("for(;;) { if (foo) { break; var x; } } x;", "var x; for(;!foo;) {} x;");

        // 'while' is normalized to 'for'
        test("while(true) if (a) break", "for(;1&&!a;);");
    }

    #[test]
    #[ignore]
    fn test_fold_loop_break_early() {
        test_same("for(;;) if (a) break");
        test_same("for(;;) if (a) { f(); break }");
        test_same("for(;;) if (a) break; else f()");
        test_same("for(;a;) if (b) break");
        test_same("for(;a;) { if (b) break; if (c) break; }");

        test_same("while(1) if (a) break");
        test_same("for (; 1; ) if (a) break");
    }

    #[test]
    #[ignore]
    fn test_fold_conditional_var_declaration() {
        test("if(x) var y=1;else y=2", "var y=x?1:2");
        test("if(x) y=1;else var y=2", "var y=x?1:2");

        test_same("if(x) var y = 1; z = 2");
        test_same("if(x||y) y = 1; var z = 2");

        test_same("if(x) { var y = 1; print(y)} else y = 2 ");
        test_same("if(x) var y = 1; else {y = 2; print(y)}");
    }

    #[test]
    #[ignore]
    fn test_fold_if_with_lower_operators_inside() {
        test("if (x + (y=5)) z && (w,z);", "x + (y=5) && (z && (w,z))");
        test("if (!(x+(y=5))) z && (w,z);", "x + (y=5) || z && (w,z)");
        test(
            "if (x + (y=5)) if (z && (w,z)) for(;;) foo();",
            "if (x + (y=5) && (z && (w,z))) for(;;) foo();",
        );
    }

    #[test]
    #[ignore]
    fn test_substitute_return() {
        // TODO(bradfordcsmith): Stop normalizing the expected output or document why it is necessary.
        // enableNormalizeExpectedOutput();

        test("function f() { while(x) { return }}", "function f() { while(x) { break }}");

        test_same("function f() { while(x) { return 5 } }");

        test_same("function f() { a: { return 5 } }");

        test(
            "function f() { while(x) { return 5}  return 5}",
            "function f() { while(x) { break }    return 5}",
        );

        test(
            "function f() { while(x) { return x}  return x}",
            "function f() { while(x) { break }    return x}",
        );

        test(
            "function f() { while(x) { if (y) { return }}}",
            "function f() { while(x) { if (y) { break  }}}",
        );

        test(
            "function f() { while(x) { if (y) { return }} return}",
            "function f() { while(x) { if (y) { break  }}}",
        );

        test(
            "function f() { while(x) { if (y) { return 5 }} return 5}",
            "function f() { while(x) { if (y) { break    }} return 5}",
        );

        // It doesn't matter if x is changed between them. We are still returning
        // x at whatever x value current holds. The whole x = 1 is skipped.
        test(
            "function f() { while(x) { if (y) { return x } x = 1} return x}",
            "function f() { while(x) { if (y) { break    } x = 1} return x}",
        );

        test(
            "function f() { while(x) { if (y) { return x } return x} return x}",
            "function f() { while(x) { if (y) {} break }return x}",
        );

        // A break here only breaks out of the inner loop.
        test_same("function f() { while(x) { while (y) { return } } }");

        test_same("function f() { while(1) { return 7}  return 5}");

        test_same(concat!(
            "function f() {",
            "  try { while(x) {return f()}} catch (e) { } return f()}"
        ));

        test_same(concat!(
            "function f() {",
            "  try { while(x) {return f()}} finally {alert(1)} return f()}"
        ));

        // Both returns has the same handler
        test(
            concat!(
                "function f() {",
                "  try { while(x) { return f() } return f() } catch (e) { } }"
            ),
            concat!("function f() {", "  try { while(x) { break } return f() } catch (e) { } }"),
        );

        // We can't fold this because it'll change the order of when foo is called.
        test_same(concat!(
            "function f() {",
            "  try { while(x) { return foo() } } finally { alert(1) } ",
            "  return foo()}"
        ));

        // This is fine, we have no side effect in the return value.
        test(
            concat!(
                "function f() {",
                "  try { while(x) { return 1 } } finally { alert(1) } return 1}"
            ),
            concat!(
                "function f() {",
                "  try { while(x) { break    } } finally { alert(1) } return 1}"
            ),
        );

        test_same("function f() { try{ return a } finally { a = 2 } return a; }");

        test(
            "function f() { switch(a){ case 1: return a; default: g();} return a;}",
            "function f() { switch(a){ case 1: break; default: g();} return a; }",
        );
    }

    #[test]
    #[ignore]
    fn test_substitute_break_for_throw() {
        // TODO(bradfordcsmith): Stop normalizing the expected output or document why it is necessary.
        // enableNormalizeExpectedOutput();

        test_same("function f() { while(x) { throw Error }}");

        test(
            "function f() { while(x) { throw Error } throw Error }",
            "function f() { while(x) { break } throw Error}",
        );
        test_same("function f() { while(x) { throw Error(1) } throw Error(2)}");
        test_same("function f() { while(x) { throw Error(1) } return Error(2)}");

        test_same("function f() { while(x) { throw 5 } }");

        test_same("function f() { a: { throw 5 } }");

        test(
            "function f() { while(x) { throw 5}  throw 5}",
            "function f() { while(x) { break }   throw 5}",
        );

        test(
            "function f() { while(x) { throw x}  throw x}",
            "function f() { while(x) { break }   throw x}",
        );

        test_same("function f() { while(x) { if (y) { throw Error }}}");

        test(
            "function f() { while(x) { if (y) { throw Error }} throw Error}",
            "function f() { while(x) { if (y) { break }} throw Error}",
        );

        test(
            "function f() { while(x) { if (y) { throw 5 }} throw 5}",
            "function f() { while(x) { if (y) { break    }} throw 5}",
        );

        // It doesn't matter if x is changed between them. We are still throwing
        // x at whatever x value current holds. The whole x = 1 is skipped.
        test(
            "function f() { while(x) { if (y) { throw x } x = 1} throw x}",
            "function f() { while(x) { if (y) { break    } x = 1} throw x}",
        );

        test(
            "function f() { while(x) { if (y) { throw x } throw x} throw x}",
            "function f() { while(x) { if (y) {} break }throw x}",
        );

        // A break here only breaks out of the inner loop.
        test_same("function f() { while(x) { while (y) { throw Error } } }");

        test_same("function f() { while(1) { throw 7}  throw 5}");

        test_same(concat!(
            "function f() {",
            "  try { while(x) {throw f()}} catch (e) { } throw f()}"
        ));

        test_same(concat!(
            "function f() {",
            "  try { while(x) {throw f()}} finally {alert(1)} throw f()}"
        ));

        // Both throws has the same handler
        test(
            concat!("function f() {", "  try { while(x) { throw f() } throw f() } catch (e) { } }"),
            concat!("function f() {", "  try { while(x) { break } throw f() } catch (e) { } }"),
        );

        // We can't fold this because it'll change the order of when foo is called.
        test_same(concat!(
            "function f() {",
            "  try { while(x) { throw foo() } } finally { alert(1) } ",
            "  throw foo()}"
        ));

        // This is fine, we have no side effect in the throw value.
        test(
            concat!(
                "function f() {",
                "  try { while(x) { throw 1 } } finally { alert(1) } throw 1}"
            ),
            concat!(
                "function f() {",
                "  try { while(x) { break    } } finally { alert(1) } throw 1}"
            ),
        );

        test_same("function f() { try{ throw a } finally { a = 2 } throw a; }");

        test(
            "function f() { switch(a){ case 1: throw a; default: g();} throw a;}",
            "function f() { switch(a){ case 1: break; default: g();} throw a; }",
        );
    }

    #[test]
    #[ignore]
    fn test_remove_duplicate_return() {
        test("function f() { return; }", "function f(){}");
        test_same("function f() { return a; }");
        test(
            "function f() { if (x) { return a } return a; }",
            "function f() { if (x) {} return a; }",
        );
        test_same("function f() { try { if (x) { return a } } catch(e) {} return a; }");
        test_same("function f() { try { if (x) {} } catch(e) {} return 1; }");

        // finally clauses may have side effects
        test_same("function f() { try { if (x) { return a } } finally { a++ } return a; }");
        // but they don't matter if the result doesn't have side effects and can't
        // be affect by side-effects.
        test(
            "function f() { try { if (x) { return 1 } } finally {} return 1; }",
            "function f() { try { if (x) {} } finally {} return 1; }",
        );

        test(
            "function f() { switch(a){ case 1: return a; } return a; }",
            "function f() { switch(a){ case 1: } return a; }",
        );

        test(
            concat!(
                "function f() { switch(a){ ",
                "  case 1: return a; case 2: return a; } return a; }"
            ),
            concat!("function f() { switch(a){ ", "  case 1: break; case 2: } return a; }"),
        );
    }

    #[test]
    #[ignore]
    fn test_remove_duplicate_throw() {
        test_same("function f() { throw a; }");
        test("function f() { if (x) { throw a } throw a; }", "function f() { if (x) {} throw a; }");
        test_same("function f() { try { if (x) {throw a} } catch(e) {} throw a; }");
        test_same("function f() { try { if (x) {throw 1} } catch(e) {f()} throw 1; }");
        test_same("function f() { try { if (x) {throw 1} } catch(e) {f()} throw 1; }");
        test_same("function f() { try { if (x) {throw 1} } catch(e) {throw 1}}");
        test(
            "function f() { try { if (x) {throw 1} } catch(e) {throw 1} throw 1; }",
            "function f() { try { if (x) {throw 1} } catch(e) {} throw 1; }",
        );

        // finally clauses may have side effects
        test_same("function f() { try { if (x) { throw a } } finally { a++ } throw a; }");
        // but they don't matter if the result doesn't have side effects and can't
        // be affect by side-effects.
        test(
            "function f() { try { if (x) { throw 1 } } finally {} throw 1; }",
            "function f() { try { if (x) {} } finally {} throw 1; }",
        );

        test(
            "function f() { switch(a){ case 1: throw a; } throw a; }",
            "function f() { switch(a){ case 1: } throw a; }",
        );

        test(
            concat!("function f() { switch(a){ ", "case 1: throw a; case 2: throw a; } throw a; }"),
            concat!("function f() { switch(a){ case 1: break; case 2: } throw a; }"),
        );
    }

    #[test]
    #[ignore]
    fn test_nested_if_combine() {
        test("if(x)if(y){while(1){}}", "if(x&&y){while(1){}}");
        test("if(x||z)if(y){while(1){}}", "if((x||z)&&y){while(1){}}");
        test("if(x)if(y||z){while(1){}}", "if((x)&&(y||z)){while(1){}}");
        test_same("if(x||z)if(y||z){while(1){}}");
        test("if(x)if(y){if(z){while(1){}}}", "if(x&&(y&&z)){while(1){}}");
    }

    // See: http://blickly.github.io/closure-compiler-issues/#291
    #[test]
    #[ignore]
    fn test_issue291() {
        test("if (true) { f.onchange(); }", "if (1) f.onchange();");
        test_same("if (f) { f.onchange(); }");
        test_same("if (f) { f.bar(); } else { f.onchange(); }");
        test("if (f) { f.bonchange(); }", "f && f.bonchange();");
        test_same("if (f) { f['x'](); }");

        // optional versions
        test("if (true) { f?.onchange(); }", "if (1) f?.onchange();");
        test_same("if (f) { f?.onchange(); }");
        test_same("if (f) { f?.bar(); } else { f?.onchange(); }");
        test("if (f) { f?.bonchange(); }", "f && f?.bonchange();");
        test_same("if (f) { f?.['x'](); }");
    }

    #[test]
    #[ignore]
    fn test_remove_else_cause() {
        test(
            concat!(
                "function f() {",
                " if(x) return 1;",
                " else if(x) return 2;",
                " else if(x) return 3 }"
            ),
            concat!(
                "function f() {",
                " if(x) return 1;",
                "{ if(x) return 2;",
                "{ if(x) return 3 } } }"
            ),
        );
    }

    #[test]
    #[ignore]
    fn test_remove_else_cause1() {
        test(
            "function f() { if (x) throw 1; else f() }",
            "function f() { if (x) throw 1; { f() } }",
        );
    }

    #[test]
    #[ignore]
    fn test_remove_else_cause2() {
        test(
            "function f() { if (x) return 1; else f() }",
            "function f() { if (x) return 1; { f() } }",
        );
        test("function f() { if (x) return; else f() }", "function f() { if (x) {} else { f() } }");
        // This case is handled by minimize exit points.
        test_same("function f() { if (x) return; f() }");
    }

    #[test]
    fn test_remove_else_cause3() {
        test(
            "function f() { a:{if (x) break a; else f() } }",
            "function f() { a: if (x) break a; else f() }",
        );
        test("function f() { if (x) { a:{ break a } } else f() }", "function f() { x || f() }");
        test("function f() { if (x) a:{ break a } else f() }", "function f() { x || f() }");
    }

    #[test]
    fn test_remove_else_cause4() {
        test(
            "function f() { if (x) { if (y) { return 1; } } else f() }",
            "function f() { if (x) { if (y) return 1; } else f() }",
        );
    }

    #[test]
    #[ignore]
    fn test_issue925() {
        test(
            concat!(
                "if (x[--y] === 1) {\n",
                "    x[y] = 0;\n",
                "} else {\n",
                "    x[y] = 1;\n",
                "}"
            ),
            "(x[--y] === 1) ? x[y] = 0 : x[y] = 1;",
        );

        test(
            concat!("if (x[--y]) {\n", "    a = 0;\n", "} else {\n", "    a = 1;\n", "}"),
            "a = (x[--y]) ? 0 : 1;",
        );

        test(
            concat!(
                "if (x?.[--y]) {", //
                "    a = 0;",
                "} else {",
                "    a = 1;",
                "}",
            ),
            "a = (x?.[--y]) ? 0 : 1;",
        );

        test("if (x++) { x += 2 } else { x += 3 }", "x++ ? x += 2 : x += 3");

        test("if (x++) { x = x + 2 } else { x = x + 3 }", "x = x++ ? x + 2 : x + 3");
    }

    #[test]
    fn test_coercion_substitution_disabled() {
        test_same("var x = {}; if (x != null) throw 'a';");
        test("var x = {}; var y = x != null;", "var x = {}, y = x != null;");

        test_same("var x = 1; if (x != 0) throw 'a';");
        test("var x = 1; var y = x != 0;", "var x = 1, y = x != 0;");
    }

    #[test]
    fn test_coercion_substitution_boolean_result0() {
        test("var x = {}; var y = x != null;", "var x = {}, y = x != null");
    }

    #[test]
    fn test_coercion_substitution_boolean_result1() {
        test("var x = {}; var y = x == null;", "var x = {}, y = x == null;");
        test("var x = {}; var y = x !== null;", "var x = {}, y = x !== null;");
        test("var x = undefined; var y = x !== null;", "var x = void 0, y = x !== null;");
        test("var x = {}; var y = x === null;", "var x = {}, y = x === null;");
        test("var x = undefined; var y = x === null;", "var x = void 0, y = x === null;");

        test("var x = 1; var y = x != 0;", "var x = 1, y = x != 0;");
        test("var x = 1; var y = x == 0;", "var x = 1, y = x == 0;");
        test("var x = 1; var y = x !== 0;", "var x = 1, y = x !== 0;");
        test("var x = 1; var y = x === 0;", "var x = 1, y = x === 0;");
    }

    #[test]
    fn test_coercion_substitution_if() {
        test("var x = {};\nif (x != null) throw 'a';\n", "var x={}; if (x!=null) throw 'a'");
        test_same("var x = {};\nif (x == null) throw 'a';\n");
        test_same("var x = {};\nif (x != null) throw 'a';\n");
        test_same("var x = {};\nif (x !== null) throw 'a';\n");
        test_same("var x = {};\nif (x === null) throw 'a';\n");

        test_same("var x = 1;\nif (x != 0) throw 'a';\n");
        test_same("var x = 1;\nif (x != 0) throw 'a';\n");
        test_same("var x = 1;\nif (x == 0) throw 'a';\n");
        test_same("var x = 1;\nif (x !== 0) throw 'a';\n");
        test_same("var x = 1;\nif (x === 0) throw 'a';\n");
        test_same("var x = NaN;\nif (x === 0) throw 'a';\n");
    }

    #[test]
    fn test_coercion_substitution_expression() {
        test_same("var x = {}; x != null && alert('b');");
        test_same("var x = 1; x != 0 && alert('b');");
    }

    #[test]
    fn test_coercion_substitution_hook() {
        test("var x = {}; var y = x != null ? 1 : 2;", "var x = {}, y = x == null ? 2 : 1;");
        test("var x = 1; var y = x != 0 ? 1 : 2;", "var x = 1, y = x == 0 ? 2 : 1;");
    }

    #[test]
    fn test_coercion_substitution_not() {
        test("var x = {}; var y = !(x != null) ? 1 : 2;", "var x = {}, y = x == null ? 1 : 2;");
        test("var x = 1; var y = !(x != 0) ? 1 : 2; ", "var x = 1, y = x == 0 ? 1 : 2; ");
    }

    #[test]
    fn test_coercion_substitution_while() {
        test("var x = {}; while (x != null) throw 'a';", "for (var x = {} ;x != null;) throw 'a';");
        test("var x = 1; while (x != 0) throw 'a';", "for (var x = 1; x != 0;) throw 'a';");
    }

    #[test]
    fn test_coercion_substitution_unknown_type() {
        test_same("var x = /** @type {?} */ ({});\nif (x != null) throw 'a';\n");
        test_same("var x = /** @type {?} */ (1);\nif (x != 0) throw 'a';\n");
    }

    #[test]
    fn test_coercion_substitution_all_type() {
        test_same("var x = /** @type {*} */ ({});\nif (x != null) throw 'a';\n");
        test_same("var x = /** @type {*} */ (1);\nif (x != 0) throw 'a';\n");
    }

    #[test]
    fn test_coercion_substitution_primitives_vs_null() {
        test_same("var x = 0;\nif (x != null) throw 'a';\n");
        test_same("var x = '';\nif (x != null) throw 'a';\n");
        test_same("var x = !1;\nif (x != null) throw 'a';\n");
    }

    #[test]
    fn test_coercion_substitution_non_number_vs_zero() {
        test_same("var x = {};\nif (x != 0) throw 'a';\n");
        test_same("var x = '';\nif (x != 0) throw 'a';\n");
        test_same("var x = !1;\nif (x != 0) throw 'a';\n");
    }

    #[test]
    fn test_coercion_substitution_boxed_number_vs_zero() {
        test_same("var x = new Number(0);\nif (x != 0) throw 'a';\n");
    }

    #[test]
    fn test_coercion_substitution_boxed_primitives() {
        test_same("var x = new Number(); if (x != null) throw 'a';");
        test_same("var x = new String(); if (x != null) throw 'a';");
        test_same("var x = new Boolean();\nif (x != null) throw 'a';");
    }

    #[test]
    #[ignore]
    fn test_minimize_if_with_new_target_condition() {
        // Related to https://github.com/google/closure-compiler/issues/3097
        test(
            concat!(
                "function x() {",
                "  if (new.target) {",
                "    return 1;",
                "  } else {",
                "    return 2;",
                "  }",
                "}",
            ),
            concat!("function x() {", "  return new.target ? 1 : 2;", "}"),
        );
    }

    #[test]
    fn compress_conditional() {
        test("foo ? foo : bar", "foo || bar");
        test("foo ? bar : foo", "foo && bar");
        test_same("x.y ? x.y : bar");
        test_same("x.y ? bar : x.y");
    }

    #[test]
    fn compress_binary_boolean() {
        test("a instanceof b === true", "a instanceof b");
        test("a instanceof b == true", "a instanceof b");
        test("a instanceof b === false", "!(a instanceof b)");
        test("a instanceof b == false", "!(a instanceof b)");

        test("a instanceof b !== true", "!(a instanceof b)");
        test("a instanceof b != true", "!(a instanceof b)");
        test("a instanceof b !== false", "a instanceof b");
        test("a instanceof b != false", "a instanceof b");

        test("delete x === true", "delete x");
        test("delete x == true", "delete x");
        test("delete x === false", "!(delete x)");
        test("delete x == false", "!(delete x)");

        test("delete x !== true", "!(delete x)");
        test("delete x != true", "!(delete x)");
        test("delete x !== false", "delete x");
        test("delete x != false", "delete x");
    }

    #[test]
    fn compress_binary_number() {
        test("if(x >> y == 0){}", "!(x >> y)");
        test("if(x >> y === 0){}", "!(x >> y)");
        test("if(x >> y != 0){}", "x >> y");
        test("if(x >> y !== 0){}", "x >> y");
        test("if((-0 != +0) !== false){}", "");
        test_same("foo(x >> y == 0)");

        test("(x = 1) === 1", "(x = 1) == 1");
        test("(x = 1) !== 1", "(x = 1) != 1");
        test("!0 + null !== 1", "!0 + null != 1");
    }

    #[test]
    fn minimize_duplicate_nots() {
        // test("!x", "x"); // TODO: in ExpressionStatement
        test("!!x", "x");
        test("!!!x", "!x");
        test("!!!!x", "x");
        test("!!!(x && y)", "!(x && y)");
        test_same("var k = () => { !!x; }");

        test_same("var k = !!x;");
        test_same("function k () { return !!x; }");
        test("var k = () => { return !!x; }", "var k = () => !!x");
        test_same("var k = () => !!x;");
    }

    #[test]
    fn minimize_nots_with_binary_expressions() {
        test("!(x === undefined)", "x !== void 0");
        test("!(typeof(x) === 'undefined')", "!(typeof x > 'u')");
        test("!(x === void 0)", "x !== void 0");
        test("!!delete x.y", "delete x.y");
        test("!!!delete x.y", "!delete x.y");
        test("!!!!delete x.y", "delete x.y");
        test("var k = !!(foo instanceof bar)", "var k = foo instanceof bar");
    }

    #[test]
    fn minimize_conditional_exprs() {
        test("(a, b) ? c : d", "a, b ? c : d");
        test("!a ? b : c", "a ? c : b");
        // test("/* @__PURE__ */ a() ? b : b", "b");
        test("a ? b : b", "a, b");
        test("a ? true : false", "a");
        test("a ? false : true", "!a");
        test("a ? a : b", "a || b");
        test("a ? b : a", "a && b");
        test("a ? b ? c : d : d", "a && b ? c : d");
        test("a ? b : c ? b : d", "a || c ? b : d");
        test("a ? c : (b, c)", "(a || b), c");
        test("a ? (b, c) : c", "(a && b), c");
        test("a ? b || c : c", "(a && b) || c");
        test("a ? c : b && c", "(a || b) && c");
        test("var a; a ? b(c, d) : b(e, d)", "var a; b(a ? c : e, d)");
        test("var a; a ? b(...c) : b(...e)", "var a; b(...a ? c : e)");
        test("var a; a ? b(c) : b(e)", "var a; b(a ? c : e)");
        test("a() != null ? a() : b", "a() == null ? b : a()");
        // test("a != null ? a : b", "a ?? b");
        // test("a != null ? a.b.c[d](e) : undefined", "a?.b.c[d](e)");
        test("cmp !== 0 ? cmp : (bar, cmp);", "cmp === 0 && bar, cmp;");
        test("cmp === 0 ? cmp : (bar, cmp);", "cmp === 0 || bar, cmp;");
        test("cmp !== 0 ? (bar, cmp) : cmp;", "cmp === 0 || bar, cmp;");
        test("cmp === 0 ? (bar, cmp) : cmp;", "cmp === 0 && bar, cmp;");
    }

    #[test]
    fn test_try_fold_in_boolean_context() {
        test("if (!!a);", "a");
        test("while (!!a);", "for (;a;);");
        test("do; while (!!a);", "do; while (a);");
        test("for (;!!a;);", "for (;a;);");
        test("!!a ? b : c", "a ? b : c");
        test("if (!!!a);", "!a");
        // test("Boolean(!!a)", "Boolean()");
        test("if ((a | b) !== 0);", "a | b");
        test("if ((a | b) === 0);", "!(a | b)");
        test("if (!!a && !!b);", "a && b");
        test("if (!!a || !!b);", "a || b");
        test("if (anything || (0, false));", "anything");
        test("if (a ? !!b : !!c);", "a ? b : c");
        test("if (anything1 ? (0, true) : anything2);", "anything1 || anything2");
        test("if (anything1 ? (0, false) : anything2);", "!anything1 && anything2");
        test("if (anything1 ? anything2 : (0, true));", "!anything1 || anything2");
        test("if (anything1 ? anything2 : (0, false));", "anything1 && anything2");
        test("if(!![]);", "");
    }

    #[test]
    fn test_try_compress_type_of_equal_string() {
        test("typeof foo === 'number'", "typeof foo == 'number'");
        test("'number' === typeof foo", "typeof foo == 'number'");
        test("typeof foo === `number`", "typeof foo == 'number'");
        test("`number` === typeof foo", "typeof foo == 'number'");
        test("typeof foo !== 'number'", "typeof foo != 'number'");
        test("'number' !== typeof foo", "typeof foo != 'number'");
        test("typeof foo !== `number`", "typeof foo != 'number'");
        test("`number` !== typeof foo", "typeof foo != 'number'");
    }

    #[test]
    fn test_negate_empty_if_stmt_consequent() {
        test("if (x) {} else { foo }", "x || foo");
        test("if (x) ;else { foo }", "x || foo");
        test("if (x) {;} else { foo }", "x || foo");

        test("if (x) { var foo } else { bar }", "if (x) { var foo } else bar");
        test_same("if (x) foo; else { var bar }");
    }
}
