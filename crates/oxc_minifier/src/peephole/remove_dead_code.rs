use oxc_allocator::{TakeIn, Vec};
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue},
    side_effects::MayHaveSideEffects,
};
use oxc_span::GetSpan;
use oxc_traverse::Ancestor;

use crate::{ctx::Ctx, keep_var::KeepVar};

use super::PeepholeOptimizations;

/// Remove Dead Code from the AST.
///
/// Terser option: `dead_code: true`.
///
/// See `KeepVar` at the end of this file for `var` hoisting logic.
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeRemoveDeadCode.java>
impl<'a> PeepholeOptimizations {
    /// Remove block from single line blocks
    /// `{ block } -> block`
    pub fn try_optimize_block(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::BlockStatement(s) = stmt else { return };
        match s.body.len() {
            0 => {
                let parent = ctx.parent();
                if parent.is_while_statement()
                    || parent.is_do_while_statement()
                    || parent.is_for_statement()
                    || parent.is_for_in_statement()
                    || parent.is_for_of_statement()
                    || parent.is_block_statement()
                    || parent.is_program()
                {
                    // Remove the block if it is empty and the parent is a block statement.
                    *stmt = ctx.ast.statement_empty(s.span);
                    ctx.state.changed = true;
                }
            }
            1 => {
                let first = &s.body[0];
                if matches!(first, Statement::VariableDeclaration(decl) if !decl.kind.is_var())
                    || matches!(first, Statement::ClassDeclaration(_))
                    || matches!(first, Statement::FunctionDeclaration(_))
                {
                    return;
                }
                *stmt = s.body.remove(0);
                ctx.state.changed = true;
            }
            _ => {}
        }
    }

    pub fn try_fold_if(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::IfStatement(if_stmt) = stmt else { return };
        // Descend and remove `else` blocks first.
        match &mut if_stmt.alternate {
            Some(Statement::IfStatement(_)) => {
                Self::try_fold_if(if_stmt.alternate.as_mut().unwrap(), ctx);
            }
            Some(Statement::BlockStatement(s)) if s.body.is_empty() => {
                if_stmt.alternate = None;
            }
            Some(Statement::EmptyStatement(_)) => {
                if_stmt.alternate = None;
            }
            _ => {}
        }

        if let Some(boolean) = if_stmt.test.evaluate_value_to_boolean(ctx) {
            let test_has_side_effects = if_stmt.test.may_have_side_effects(ctx);
            // Use "1" and "0" instead of "true" and "false" to be shorter.
            // And also prevent swapping consequent and alternate when `!0` is encountered.
            if !test_has_side_effects {
                if_stmt.test = ctx.ast.expression_numeric_literal(
                    if_stmt.test.span(),
                    if boolean { 1.0 } else { 0.0 },
                    None,
                    NumberBase::Decimal,
                );
            }
            let mut keep_var = KeepVar::new(ctx.ast);
            if boolean {
                if let Some(alternate) = &if_stmt.alternate {
                    keep_var.visit_statement(alternate);
                }
            } else {
                keep_var.visit_statement(&if_stmt.consequent);
            }
            let var_stmt = keep_var
                .get_variable_declaration_statement()
                .and_then(|stmt| Self::remove_unused_variable_declaration(stmt, ctx));
            let has_var_stmt = var_stmt.is_some();
            if let Some(var_stmt) = var_stmt {
                if boolean {
                    if_stmt.alternate = Some(var_stmt);
                } else {
                    if_stmt.consequent = var_stmt;
                }
                return;
            }
            if test_has_side_effects {
                if !has_var_stmt {
                    if boolean {
                        if_stmt.alternate = None;
                    } else {
                        if_stmt.consequent = ctx.ast.statement_empty(if_stmt.consequent.span());
                    }
                }
                return;
            }
            *stmt = if boolean {
                if_stmt.consequent.take_in(ctx.ast)
            } else if let Some(alternate) = if_stmt.alternate.take() {
                alternate
            } else {
                ctx.ast.statement_empty(if_stmt.span)
            };
            ctx.state.changed = true;
        }
    }

    pub fn try_fold_for(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::ForStatement(for_stmt) = stmt else { return };
        if let Some(init) = &mut for_stmt.init
            && let Some(init) = init.as_expression_mut()
            && Self::remove_unused_expression(init, ctx)
        {
            for_stmt.init = None;
            ctx.state.changed = true;
        }
        if let Some(update) = &mut for_stmt.update
            && Self::remove_unused_expression(update, ctx)
        {
            for_stmt.update = None;
            ctx.state.changed = true;
        }

        let test_boolean =
            for_stmt.test.as_ref().and_then(|test| test.evaluate_value_to_boolean(ctx));
        if for_stmt.test.as_ref().is_some_and(|test| test.may_have_side_effects(ctx)) {
            return;
        }
        match test_boolean {
            Some(false) => match &for_stmt.init {
                Some(ForStatementInit::VariableDeclaration(_)) => {
                    let mut keep_var = KeepVar::new(ctx.ast);
                    keep_var.visit_statement(&for_stmt.body);
                    let mut var_decl = keep_var.get_variable_declaration();
                    let Some(ForStatementInit::VariableDeclaration(var_init)) = &mut for_stmt.init
                    else {
                        return;
                    };
                    if var_init.kind.is_var() {
                        if let Some(var_decl) = &mut var_decl {
                            var_decl
                                .declarations
                                .splice(0..0, var_init.declarations.take_in(ctx.ast));
                        } else {
                            var_decl = Some(var_init.take_in_box(ctx.ast));
                        }
                    }
                    *stmt = var_decl.map_or_else(
                        || ctx.ast.statement_empty(for_stmt.span),
                        Statement::VariableDeclaration,
                    );
                    ctx.state.changed = true;
                }
                None => {
                    let mut keep_var = KeepVar::new(ctx.ast);
                    keep_var.visit_statement(&for_stmt.body);
                    *stmt = keep_var.get_variable_declaration().map_or_else(
                        || ctx.ast.statement_empty(for_stmt.span),
                        Statement::VariableDeclaration,
                    );
                    ctx.state.changed = true;
                }
                _ => {}
            },
            Some(true) => {
                // Remove the test expression.
                for_stmt.test = None;
                ctx.state.changed = true;
            }
            None => {}
        }
    }

    /// Remove meaningless labeled statements.
    ///
    /// ```js
    /// a: break a;
    /// ```
    pub fn try_fold_labeled(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::LabeledStatement(s) = stmt else { return };
        let id = s.label.name.as_str();

        if ctx.options().drop_labels.contains(id) {
            *stmt = ctx.ast.statement_empty(s.span);
            ctx.state.changed = true;
            return;
        }

        // Check the first statement in the block, or just the `break [id] ` statement.
        // Check if we need to remove the whole block.
        match &mut s.body {
            Statement::BreakStatement(break_stmt)
                if break_stmt.label.as_ref().is_some_and(|l| l.name.as_str() == id) => {}
            Statement::BlockStatement(block) if block.body.first().is_some_and(|first| matches!(first, Statement::BreakStatement(break_stmt) if break_stmt.label.as_ref().is_some_and(|l| l.name.as_str() == id))) => {}
            Statement::EmptyStatement(_) => {
                *stmt = ctx.ast.statement_empty(s.span);
                ctx.state.changed = true;
                return;
            }
            _ => return
        }
        let mut var = KeepVar::new(ctx.ast);
        var.visit_statement(&s.body);
        let var_decl = var.get_variable_declaration_statement();
        *stmt = var_decl.unwrap_or_else(|| ctx.ast.statement_empty(s.span));
        ctx.state.changed = true;
    }

    pub fn try_fold_expression_stmt(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::ExpressionStatement(expr_stmt) = stmt else { return };
        // We need to check if it is in arrow function with `expression: true`.
        // This is the only scenario where we can't remove it even if `ExpressionStatement`.
        if let Ancestor::ArrowFunctionExpressionBody(body) = ctx.ancestry.ancestor(1)
            && *body.expression()
        {
            return;
        }

        if Self::remove_unused_expression(&mut expr_stmt.expression, ctx) {
            *stmt = ctx.ast.statement_empty(expr_stmt.span);
            ctx.state.changed = true;
        }
    }

    pub fn try_fold_try(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::TryStatement(s) = stmt else { return };
        if let Some(handler) = &s.handler
            && s.block.body.is_empty()
        {
            let mut var = KeepVar::new(ctx.ast);
            var.visit_block_statement(&handler.body);
            let Some(handler) = &mut s.handler else { return };
            handler.body.body.clear();
            if let Some(var_decl) = var.get_variable_declaration_statement() {
                handler.body.body.push(var_decl);
            }
        }

        if let Some(finalizer) = &s.finalizer
            && finalizer.body.is_empty()
            && s.handler.is_some()
        {
            s.finalizer = None;
        }

        if s.block.body.is_empty()
            && s.handler.as_ref().is_none_or(|handler| handler.body.body.is_empty())
        {
            *stmt = if let Some(finalizer) = &mut s.finalizer {
                let mut block = ctx.ast.block_statement(finalizer.span, ctx.ast.vec());
                std::mem::swap(&mut **finalizer, &mut block);
                Statement::BlockStatement(ctx.ast.alloc(block))
            } else {
                ctx.ast.statement_empty(s.span)
            };
            ctx.state.changed = true;
        }
    }

    /// Try folding conditional expression (?:) if the condition results of the condition is known.
    pub fn try_fold_conditional_expression(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::ConditionalExpression(e) = expr else { return };
        let Some(v) = e.test.evaluate_value_to_boolean(ctx) else { return };
        ctx.state.changed = true;
        *expr = if e.test.may_have_side_effects(ctx) {
            // "(a, true) ? b : c" => "a, b"
            let exprs = ctx.ast.vec_from_array([
                {
                    let mut test = e.test.take_in(ctx.ast);
                    Self::remove_unused_expression(&mut test, ctx);
                    test
                },
                if v { e.consequent.take_in(ctx.ast) } else { e.alternate.take_in(ctx.ast) },
            ]);
            ctx.ast.expression_sequence(e.span, exprs)
        } else {
            let result_expr =
                if v { e.consequent.take_in(ctx.ast) } else { e.alternate.take_in(ctx.ast) };
            let should_keep_as_sequence_expr = Self::should_keep_indirect_access(&result_expr, ctx);
            // "(1 ? a.b : 0)()" => "(0, a.b)()"
            if should_keep_as_sequence_expr {
                ctx.ast.expression_sequence(
                    e.span,
                    ctx.ast.vec_from_array([
                        ctx.ast.expression_numeric_literal(e.span, 0.0, None, NumberBase::Decimal),
                        result_expr,
                    ]),
                )
            } else {
                result_expr
            }
        };
    }

    pub fn remove_sequence_expression(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::SequenceExpression(e) = expr else { return };
        let should_keep_as_sequence_expr = e
            .expressions
            .last()
            .is_some_and(|last_expr| Self::should_keep_indirect_access(last_expr, ctx));
        if should_keep_as_sequence_expr
            && e.expressions.len() == 2
            && e.expressions.first().unwrap().is_number_0()
        {
            return;
        }
        let old_len = e.expressions.len();
        let mut i = 0;
        e.expressions.retain_mut(|e| {
            i += 1;
            if should_keep_as_sequence_expr && i == old_len - 1 {
                if Self::remove_unused_expression(e, ctx) {
                    *e = ctx.ast.expression_numeric_literal(
                        e.span(),
                        0.0,
                        None,
                        NumberBase::Decimal,
                    );
                    ctx.state.changed = true;
                }
                return true;
            }
            if i == old_len {
                return true;
            }
            !Self::remove_unused_expression(e, ctx)
        });
        if e.expressions.len() != old_len {
            ctx.state.changed = true;
        }
        if e.expressions.len() == 1 {
            *expr = e.expressions.pop().unwrap();
            ctx.state.changed = true;
        }
    }

    pub fn keep_track_of_pure_functions(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        match stmt {
            Statement::FunctionDeclaration(f) => {
                if let Some(body) = &f.body {
                    Self::try_save_pure_function(
                        f.id.as_ref(),
                        &f.params,
                        body,
                        f.r#async,
                        f.generator,
                        ctx,
                    );
                }
            }
            Statement::VariableDeclaration(decl) => {
                for d in &decl.declarations {
                    if let BindingPattern::BindingIdentifier(id) = &d.id {
                        match &d.init {
                            Some(Expression::ArrowFunctionExpression(a)) => {
                                Self::try_save_pure_function(
                                    Some(id),
                                    &a.params,
                                    &a.body,
                                    a.r#async,
                                    false,
                                    ctx,
                                );
                            }
                            Some(Expression::FunctionExpression(f)) => {
                                if let Some(body) = &f.body {
                                    Self::try_save_pure_function(
                                        Some(id),
                                        &f.params,
                                        body,
                                        f.r#async,
                                        f.generator,
                                        ctx,
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn try_save_pure_function(
        id: Option<&BindingIdentifier<'a>>,
        params: &FormalParameters<'a>,
        body: &FunctionBody<'a>,
        r#async: bool,
        generator: bool,
        ctx: &mut Ctx<'a, '_>,
    ) {
        if r#async || generator {
            return;
        }
        // `function foo({}) {} foo(null)` is runtime type error.
        if !params.items.iter().all(|pat| pat.pattern.is_binding_identifier()) {
            return;
        }
        if body.statements.iter().any(|stmt| stmt.may_have_side_effects(ctx)) {
            return;
        }
        let Some(symbol_id) = id.and_then(|id| id.symbol_id.get()) else { return };
        if ctx.scoping().get_resolved_references(symbol_id).all(|r| r.flags().is_read_only()) {
            ctx.state.pure_functions.insert(
                symbol_id,
                if body.is_empty() { Some(ConstantValue::Undefined) } else { None },
            );
        }
    }

    pub fn remove_dead_code_call_expression(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::CallExpression(e) = expr else { return };
        if let Expression::Identifier(ident) = &e.callee {
            let reference_id = ident.reference_id();
            if let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id()
                && matches!(
                    ctx.state.pure_functions.get(&symbol_id),
                    Some(Some(ConstantValue::Undefined))
                )
            {
                let mut exprs = Self::fold_arguments_into_needed_expressions(&mut e.arguments, ctx);
                if exprs.is_empty() {
                    *expr = ctx.ast.void_0(e.span);
                    ctx.state.changed = true;
                    return;
                }
                exprs.push(ctx.ast.void_0(e.span));
                *expr = ctx.ast.expression_sequence(e.span, exprs);
                ctx.state.changed = true;
            }
        }
    }

    /// Whether the indirect access should be kept.
    /// For example, `(0, foo.bar)()` should not be transformed to `foo.bar()`.
    /// Example case: `let o = { f() { assert.ok(this !== o); } }; (true && o.f)(); (true && o.f)``;`
    ///
    /// * `access_value` - The expression that may need to be kept as indirect reference (`foo.bar` in the example above)
    pub fn should_keep_indirect_access(access_value: &Expression<'a>, ctx: &Ctx<'a, '_>) -> bool {
        match ctx.parent() {
            Ancestor::CallExpressionCallee(_) | Ancestor::TaggedTemplateExpressionTag(_) => {
                match access_value {
                    Expression::Identifier(id) => id.name == "eval" && ctx.is_global_reference(id),
                    match_member_expression!(Expression) => true,
                    _ => false,
                }
            }
            Ancestor::UnaryExpressionArgument(unary) => match unary.operator() {
                UnaryOperator::Typeof => {
                    // Example case: `typeof (0, foo)` (error) -> `typeof foo` (no error)
                    if let Expression::Identifier(id) = access_value {
                        ctx.is_global_reference(id)
                    } else {
                        false
                    }
                }
                UnaryOperator::Delete => {
                    match access_value {
                        // Example case: `delete (0, foo)` (no error) -> `delete foo` (error)
                        Expression::Identifier(_)
                        // Example case: `delete (0, foo.#a)` (no error) -> `delete foo.#a` (error)
                        | Expression::PrivateFieldExpression(_)
                        // Example case: `typeof (0, foo.bar)` (noop) -> `typeof foo.bar` (deletes bar)
                        | Expression::ComputedMemberExpression(_)
                        | Expression::StaticMemberExpression(_) => true,
                        // Example case: `typeof (0, foo?.bar)` (noop) -> `typeof foo?.bar` (deletes bar)
                        Expression::ChainExpression(chain) => {
                            matches!(&chain.expression, match_member_expression!(ChainElement))
                        }
                        _ => false,
                    }
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn remove_dead_code_exit_class_body(body: &mut ClassBody<'a>, _ctx: &mut Ctx<'a, '_>) {
        body.body.retain(|e| !matches!(e, ClassElement::StaticBlock(s) if s.body.is_empty()));
    }

    pub fn remove_empty_spread_arguments(args: &mut Vec<'a, Argument<'a>>) {
        if args.len() != 1 {
            return;
        }
        let Argument::SpreadElement(e) = &args[0] else { return };
        let Expression::ArrayExpression(e) = &e.argument else { return };
        if e.elements.is_empty() {
            args.drain(..);
        }
    }
}
