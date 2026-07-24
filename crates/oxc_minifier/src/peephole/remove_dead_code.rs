use crate::generated::ancestor::Ancestor;
use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::ast::*;
use oxc_ast_visit::VisitJs;
use oxc_ecmascript::{constant_evaluation::ConstantEvaluation, side_effects::MayHaveSideEffects};
use oxc_span::GetSpan;
use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
};

use crate::{TraverseCtx, keep_var::KeepVar, symbol_metadata::FunctionSummary};

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
    pub fn try_optimize_block(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
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
                    let new_stmt = Statement::new_empty_statement(s.span, ctx);
                    ctx.replace_statement(stmt, new_stmt);
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
                let new_stmt = s.body.remove(0);
                ctx.replace_statement(stmt, new_stmt);
            }
            _ => {}
        }
    }

    #[expect(clippy::float_cmp)]
    pub fn try_fold_if(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
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
            //
            // Idempotency: skip the rewrite when `if_stmt.test` is already the canonical
            // numeric form (NumericLiteral 1.0 / 0.0). Without this gate, the typed
            // `replace_expression` helper would re-record a mutation on every loop
            // iteration (NumericLiteral 1.0 still evaluates to Some(true) via the line-73
            // predicate), preventing fixed-point convergence.
            if !test_has_side_effects
                && !matches!(
                    &if_stmt.test,
                    Expression::NumericLiteral(n)
                        if (boolean && n.value == 1.0) || (!boolean && n.value == 0.0)
                )
            {
                let new_test = Expression::new_numeric_literal(
                    if_stmt.test.span(),
                    if boolean { 1.0 } else { 0.0 },
                    None,
                    NumberBase::Decimal,
                    ctx,
                );
                ctx.replace_expression(&mut if_stmt.test, new_test);
            }
            let mut keep_var = KeepVar::new();
            if boolean {
                if let Some(alternate) = &if_stmt.alternate {
                    keep_var.visit_statement(alternate);
                }
            } else {
                keep_var.visit_statement(&if_stmt.consequent);
            }
            let var_stmt = keep_var
                .get_variable_declaration_statement(&ctx.ast)
                .and_then(|stmt| Self::remove_unused_variable_declaration(stmt, ctx));
            let has_var_stmt = var_stmt.is_some();
            if let Some(var_stmt) = var_stmt {
                // Idempotency: skip when the target slot is already in canonical KeepVar
                // output shape (a `var` declaration whose declarators all lack initializers).
                // On the next loop iteration `KeepVar` would re-extract the same names from
                // that slot and produce a structurally-equivalent fresh allocation; routing
                // through `replace_statement` would re-record a mutation indefinitely.
                if boolean {
                    let already_canonical =
                        if_stmt.alternate.as_ref().is_some_and(Self::is_keep_var_canonical);
                    if !already_canonical {
                        if let Some(alternate) = if_stmt.alternate.as_mut() {
                            ctx.replace_statement(alternate, var_stmt);
                        } else {
                            // `KeepVar` only produced a stmt because it visited the alternate,
                            // so the alternate must be Some. Defensive fall-through preserves
                            // historical behaviour: install it without dropping anything.
                            if_stmt.alternate = Some(var_stmt);
                            ctx.notice_change();
                        }
                    }
                } else if !Self::is_keep_var_canonical(&if_stmt.consequent) {
                    ctx.replace_statement(&mut if_stmt.consequent, var_stmt);
                }
                return;
            }
            if test_has_side_effects {
                if !has_var_stmt {
                    if boolean {
                        // Idempotent: `Option::take` only fires when the slot still holds a value.
                        if let Some(old) = if_stmt.alternate.take() {
                            ctx.drop_statement(&old);
                        }
                    } else if !matches!(&if_stmt.consequent, Statement::EmptyStatement(_)) {
                        // Idempotency: skip when consequent is already empty.
                        let new_consequent =
                            Statement::new_empty_statement(if_stmt.consequent.span(), ctx);
                        ctx.replace_statement(&mut if_stmt.consequent, new_consequent);
                    }
                }
                return;
            }
            let new_stmt = if boolean {
                if_stmt.consequent.take_in(ctx)
            } else if let Some(alternate) = if_stmt.alternate.take() {
                alternate
            } else {
                Statement::new_empty_statement(if_stmt.span, ctx)
            };
            ctx.replace_statement(stmt, new_stmt);
        }
    }

    /// True when `stmt` is already in the canonical shape produced by
    /// `KeepVar::get_variable_declaration_statement`: a single `var` declaration
    /// whose declarators all lack initializers. Used as an idempotency gate in
    /// `try_fold_if` so the var-hoisting rewrite doesn't re-fire across loop
    /// iterations when `KeepVar` would just re-emit the same shape.
    fn is_keep_var_canonical(stmt: &Statement<'a>) -> bool {
        matches!(
            stmt,
            Statement::VariableDeclaration(decl)
                if decl.kind.is_var() && decl.declarations.iter().all(|d| d.init.is_none())
        )
    }

    pub fn try_fold_for(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::ForStatement(for_stmt) = stmt else { return };
        if let Some(init) = &mut for_stmt.init
            && let Some(init_expr) = init.as_expression_mut()
            && Self::remove_unused_expression(init_expr, ctx)
        {
            ctx.drop_expression(init_expr);
            for_stmt.init = None;
        }
        if let Some(update) = &mut for_stmt.update
            && Self::remove_unused_expression(update, ctx)
        {
            ctx.drop_expression(update);
            for_stmt.update = None;
        }

        let test_boolean =
            for_stmt.test.as_ref().and_then(|test| test.evaluate_value_to_boolean(ctx));
        if for_stmt.test.as_ref().is_some_and(|test| test.may_have_side_effects(ctx)) {
            return;
        }
        match test_boolean {
            Some(false) => match &for_stmt.init {
                Some(ForStatementInit::VariableDeclaration(_)) => {
                    let mut keep_var = KeepVar::new();
                    keep_var.visit_statement(&for_stmt.body);
                    let mut var_decl = keep_var.get_variable_declaration(&ctx.ast);
                    let Some(ForStatementInit::VariableDeclaration(var_init)) = &mut for_stmt.init
                    else {
                        return;
                    };
                    if var_init.kind.is_var() {
                        if let Some(var_decl) = &mut var_decl {
                            var_decl.declarations.splice(0..0, var_init.declarations.take_in(ctx));
                        } else {
                            var_decl = Some(var_init.take_in_box(ctx));
                        }
                    }
                    let new_stmt = var_decl.map_or_else(
                        || Statement::new_empty_statement(for_stmt.span, ctx),
                        Statement::VariableDeclaration,
                    );
                    ctx.replace_statement(stmt, new_stmt);
                }
                None => {
                    let mut keep_var = KeepVar::new();
                    keep_var.visit_statement(&for_stmt.body);
                    let new_stmt = keep_var.get_variable_declaration(&ctx.ast).map_or_else(
                        || Statement::new_empty_statement(for_stmt.span, ctx),
                        Statement::VariableDeclaration,
                    );
                    ctx.replace_statement(stmt, new_stmt);
                }
                _ => {}
            },
            Some(true) => {
                // Remove the test expression.
                if let Some(old) = for_stmt.test.take() {
                    ctx.drop_expression(&old);
                }
            }
            None => {}
        }
    }

    /// Remove meaningless labeled statements.
    ///
    /// ```js
    /// a: break a;
    /// ```
    pub fn try_fold_labeled(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::LabeledStatement(s) = stmt else { return };
        let id = s.label.name.as_str();

        if ctx.options().drop_labels.contains(id) {
            let new_stmt = Statement::new_empty_statement(s.span, ctx);
            ctx.replace_statement(stmt, new_stmt);
            return;
        }

        // Check the first statement in the block, or just the `break [id] ` statement.
        // Check if we need to remove the whole block.
        match &mut s.body {
            Statement::BreakStatement(break_stmt)
                if break_stmt.label.as_ref().is_some_and(|l| l.name.as_str() == id) => {}
            Statement::BlockStatement(block) if block.body.first().is_some_and(|first| matches!(first, Statement::BreakStatement(break_stmt) if break_stmt.label.as_ref().is_some_and(|l| l.name.as_str() == id))) => {}
            Statement::EmptyStatement(_) => {
                let new_stmt = Statement::new_empty_statement(s.span, ctx);
                ctx.replace_statement(stmt, new_stmt);
                return;
            }
            _ => return
        }
        let mut var = KeepVar::new();
        var.visit_statement(&s.body);
        let var_decl = var.get_variable_declaration_statement(&ctx.ast);
        let new_stmt = var_decl.unwrap_or_else(|| Statement::new_empty_statement(s.span, ctx));
        ctx.replace_statement(stmt, new_stmt);
    }

    pub fn try_fold_expression_stmt(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::ExpressionStatement(expr_stmt) = stmt else { return };
        // We need to check if it is in arrow function with `expression: true`.
        // This is the only scenario where we can't remove it even if `ExpressionStatement`.
        if let Ancestor::ArrowFunctionExpressionBody(body) = ctx.ancestry.ancestor(1)
            && *body.expression()
        {
            return;
        }

        if Self::remove_unused_expression(&mut expr_stmt.expression, ctx) {
            let new_stmt = Statement::new_empty_statement(expr_stmt.span, ctx);
            ctx.replace_statement(stmt, new_stmt);
        }
    }

    pub fn try_fold_try(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::TryStatement(s) = stmt else { return };
        if let Some(handler) = &s.handler
            && s.block.body.is_empty()
        {
            let body = &handler.body.body;
            let is_canonical_body =
                body.is_empty() || (body.len() == 1 && Self::is_keep_var_canonical(&body[0]));
            if !is_canonical_body {
                let mut var = KeepVar::new();
                var.visit_block_statement(&handler.body);
                let Some(handler) = &mut s.handler else { return };

                for dropped in handler.body.body.take_in(ctx) {
                    ctx.drop_statement(&dropped);
                }
                if let Some(var_decl) = var.get_variable_declaration_statement(&ctx.ast) {
                    handler.body.body.push(var_decl);
                }
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
            let new_stmt = if let Some(finalizer) = &mut s.finalizer {
                let mut block = BlockStatement::boxed(finalizer.span, [], ctx);
                std::mem::swap(finalizer, &mut block);
                Statement::BlockStatement(block)
            } else {
                Statement::new_empty_statement(s.span, ctx)
            };
            ctx.replace_statement(stmt, new_stmt);
        }
    }

    /// Try folding conditional expression (?:) if the condition results of the condition is known.
    pub fn try_fold_conditional_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::ConditionalExpression(e) = expr else { return };
        let Some(v) = e.test.evaluate_value_to_boolean(ctx) else { return };
        let new_expr = if e.test.may_have_side_effects(ctx) {
            // "(a, true) ? b : c" => "a, b"
            Expression::new_sequence_expression(
                e.span,
                [
                    {
                        let mut test = e.test.take_in(ctx);
                        Self::remove_unused_expression(&mut test, ctx);
                        test
                    },
                    if v { e.consequent.take_in(ctx) } else { e.alternate.take_in(ctx) },
                ],
                ctx,
            )
        } else {
            let result_expr = if v { e.consequent.take_in(ctx) } else { e.alternate.take_in(ctx) };
            let should_keep_as_sequence_expr = Self::should_keep_indirect_access(&result_expr, ctx);
            // "(1 ? a.b : 0)()" => "(0, a.b)()"
            if should_keep_as_sequence_expr {
                Expression::new_sequence_expression(
                    e.span,
                    [
                        Expression::new_numeric_literal(
                            e.span,
                            0.0,
                            None,
                            NumberBase::Decimal,
                            ctx,
                        ),
                        result_expr,
                    ],
                    ctx,
                )
            } else {
                result_expr
            }
        };
        ctx.replace_expression(expr, new_expr);
    }

    pub fn remove_sequence_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
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
                // Idempotency: skip the rewrite when `e` is already the canonical
                // `0` placeholder. Without this gate the re-wrap re-bumps the
                // mutation counter every iteration (mirrors the `len == 2` guard
                // above), spinning the fixed-point loop. `0` is side-effect-free,
                // so `remove_unused_expression` would return `true` and produce a
                // structurally-identical fresh `0`.
                if !e.is_number_0() && Self::remove_unused_expression(e, ctx) {
                    let new_expr = Expression::new_numeric_literal(
                        e.span(),
                        0.0,
                        None,
                        NumberBase::Decimal,
                        ctx,
                    );
                    ctx.replace_expression(e, new_expr);
                }
                return true;
            }
            if i == old_len {
                return true;
            }
            if Self::remove_unused_expression(e, ctx) {
                ctx.drop_expression(e);
                false
            } else {
                true
            }
        });
        if e.expressions.len() == 1 {
            let new_expr = e.expressions.pop().unwrap();
            ctx.replace_expression(expr, new_expr);
        }
    }

    pub fn keep_track_of_pure_functions(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
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
        ctx: &mut TraverseCtx<'a>,
    ) {
        if r#async || generator {
            return;
        }
        // Destructuring can throw. Default initializers run for missing or `undefined`
        // arguments, and function summaries are call-independent, so reject an initializer
        // that may have side effects. TDZ-only throws follow the minifier's documented
        // `No TDZ Violation` assumption.
        if !params.items.iter().all(|param| {
            param.pattern.is_binding_identifier()
                && param.initializer.as_ref().is_none_or(|init| !init.may_have_side_effects(ctx))
        }) {
            return;
        }
        if body.statements.iter().any(|stmt| stmt.may_have_side_effects(ctx)) {
            return;
        }
        let Some(symbol_id) = id.and_then(|id| id.symbol_id.get()) else { return };
        let binding_scope_id = ctx.scoping().symbol_scope_id(symbol_id);
        let binding_scope_flags = ctx.scoping().scope_flags(binding_scope_id);
        // Redeclarations are span-only in semantic and create no references,
        // so the read-only-reference check below cannot see them. A different
        // declaration of the same symbol may be impure and win at runtime.
        if !ctx.scoping().symbol_redeclarations(symbol_id).is_empty() {
            ctx.state.symbols.clear_function_summary(symbol_id);
            return;
        }
        // Direct eval and Script global properties can replace the binding
        // without producing a resolved write reference. Discard any summary
        // from an earlier pass; removing the last eval may make the binding
        // safe later, while Script roots are rejected again on every pass.
        if binding_scope_flags.contains_direct_eval()
            || (ctx.source_type().is_script() && binding_scope_id == ctx.scoping().root_scope_id())
        {
            ctx.state.symbols.clear_function_summary(symbol_id);
            return;
        }
        if ctx.scoping().get_resolved_references(symbol_id).all(|r| r.flags().is_read_only()) {
            ctx.state.symbols.set_function_summary(
                symbol_id,
                if body.is_empty() {
                    FunctionSummary::SideEffectFreeReturnsUndefined
                } else {
                    FunctionSummary::SideEffectFree
                },
            );
        }
    }

    pub fn keep_track_of_function_declaration_dead_arguments(
        function: &Function<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if function.r#type == FunctionType::FunctionDeclaration {
            Self::try_save_dead_argument_prefix(
                function.id.as_ref(),
                &function.params,
                function.scope_id.get(),
                ctx,
            );
        }
    }

    pub fn keep_track_of_variable_function_dead_arguments(
        decl: &VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let BindingPattern::BindingIdentifier(id) = &decl.id else { return };
        match &decl.init {
            Some(Expression::ArrowFunctionExpression(function)) => {
                Self::try_save_dead_argument_prefix(
                    Some(id),
                    &function.params,
                    function.scope_id.get(),
                    ctx,
                );
            }
            Some(Expression::FunctionExpression(function)) => {
                Self::try_save_dead_argument_prefix(
                    Some(id),
                    &function.params,
                    function.scope_id.get(),
                    ctx,
                );
            }
            _ => {}
        }
    }

    fn try_save_dead_argument_prefix(
        id: Option<&BindingIdentifier<'a>>,
        params: &FormalParameters<'a>,
        function_scope_id: Option<ScopeId>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Some(symbol_id) = id.and_then(|id| id.symbol_id.get()) else { return };
        if !ctx.state.symbols.is_dead_argument_candidate(symbol_id) {
            return;
        }
        let binding_scope_id = ctx.scoping().symbol_scope_id(symbol_id);
        let binding_scope_flags = ctx.scoping().scope_flags(binding_scope_id);
        let binding_is_stable = ctx.scoping().symbol_redeclarations(symbol_id).is_empty()
            && !binding_scope_flags.contains_direct_eval()
            && !(ctx.source_type().is_script()
                && binding_scope_id == ctx.scoping().root_scope_id());
        let has_only_read_references = ctx.state.symbols.value(symbol_id).map_or_else(
            || {
                let mut references = ctx.scoping().get_resolved_references(symbol_id);
                references.next().is_some_and(|first_reference| {
                    first_reference.flags().is_read_only()
                        && references.all(|reference| reference.flags().is_read_only())
                })
            },
            |value| value.references.has_reads() && !value.references.has_writes(),
        );
        if !binding_is_stable || !has_only_read_references {
            if ctx.state.symbols.clear_dead_argument_prefix(symbol_id) {
                ctx.state.request_revisit();
            }
            return;
        }

        if let Some(prefix) = Self::compute_dead_argument_prefix(params, function_scope_id, ctx) {
            if ctx.state.symbols.set_dead_argument_prefix(symbol_id, prefix) {
                ctx.state.request_revisit();
            }
        } else if ctx.state.symbols.clear_dead_argument_prefix(symbol_id) {
            ctx.state.request_revisit();
        }
    }

    /// Return the first argument index whose value the function cannot observe.
    /// Arguments at and after this index may be removed when their evaluation is
    /// also unobservable.
    fn compute_dead_argument_prefix(
        params: &FormalParameters<'a>,
        function_scope_id: Option<ScopeId>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<usize> {
        if params.rest.is_some()
            || !params
                .items
                .iter()
                .all(|param| param.pattern.is_binding_identifier() && param.initializer.is_none())
        {
            return None;
        }

        let function_scope_id = function_scope_id?;
        let function_scope_flags = ctx.scoping().scope_flags(function_scope_id);
        if !function_scope_flags.is_arrow()
            && (!function_scope_flags.is_strict_mode()
                || ctx.scoping().root_unresolved_references().contains_key("arguments"))
        {
            return None;
        }

        let mut prefix = params.items.len();
        while prefix > 0 {
            let BindingPattern::BindingIdentifier(id) = &params.items[prefix - 1].pattern else {
                break;
            };
            let Some(symbol_id) = id.symbol_id.get() else { break };
            if !ctx.scoping().symbol_is_unused(symbol_id) {
                break;
            }
            prefix -= 1;
        }
        Some(prefix)
    }

    pub fn remove_dead_code_call_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::CallExpression(e) = expr else { return };
        if let Expression::Identifier(ident) = &e.callee {
            let reference_id = ident.reference_id();
            if let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id()
                && ctx.state.symbols.function_summary(symbol_id).returns_undefined()
            {
                if ctx.is_tree_shake_only()
                    && Self::reference_may_be_shadowed_by_with(
                        ctx.scoping().get_reference(reference_id).scope_id(),
                        ctx,
                    )
                {
                    return;
                }
                let mut exprs = Self::fold_arguments_into_needed_expressions(&mut e.arguments, ctx);
                if exprs.is_empty() {
                    let new_expr = Expression::new_void_0(e.span, ctx);
                    ctx.replace_expression(expr, new_expr);
                    return;
                }
                exprs.push(Expression::new_void_0(e.span, ctx));
                let new_expr = Expression::new_sequence_expression(e.span, exprs, ctx);
                ctx.replace_expression(expr, new_expr);
                return;
            }
        }

        if !ctx.is_tree_shake_only() {
            return;
        }
        let Expression::Identifier(ident) = &e.callee else { return };
        let reference_id = ident.reference_id();
        let reference = ctx.scoping().get_reference(reference_id);
        let Some(symbol_id) = reference.symbol_id() else { return };
        if !ctx.state.symbols.is_dead_argument_candidate(symbol_id) {
            return;
        }
        let Some(prefix) = ctx.state.symbols.dead_argument_prefix(symbol_id) else {
            let flags = ctx.scoping().symbol_flags(symbol_id);
            if flags.is_function() && !flags.contains(SymbolFlags::FunctionExpression) {
                ctx.state.symbols.record_missed_dead_argument_call(symbol_id);
            }
            return;
        };
        if Self::reference_may_be_shadowed_by_with(reference.scope_id(), ctx) {
            return;
        }
        if e.arguments.len() <= prefix
            || e.arguments.iter().any(|argument| matches!(argument, Argument::SpreadElement(_)))
        {
            return;
        }

        while e.arguments.len() > prefix {
            let Some(argument) = e.arguments.last_mut().and_then(Argument::as_expression_mut)
            else {
                return;
            };
            // Reuse unused-expression analysis so hidden effects and derived-
            // constructor `this` before `super()` remain observable.
            if !Self::remove_unused_expression(argument, ctx) {
                break;
            }
            let dropped = e.arguments.pop().unwrap().into_expression();
            ctx.drop_expression(&dropped);
        }
    }

    /// A `with` object can dynamically replace a statically resolved identifier.
    pub(super) fn reference_may_be_shadowed_by_with(
        reference_scope_id: ScopeId,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        ctx.scoping()
            .scope_ancestors(reference_scope_id)
            .any(|scope_id| ctx.scoping().scope_flags(scope_id).contains(ScopeFlags::With))
    }

    /// Whether the indirect access should be kept.
    /// For example, `(0, foo.bar)()` should not be transformed to `foo.bar()`.
    /// Example case: `let o = { f() { assert.ok(this !== o); } }; (true && o.f)(); (true && o.f)``;`
    ///
    /// * `access_value` - The expression that may need to be kept as indirect reference (`foo.bar` in the example above)
    pub fn should_keep_indirect_access(
        access_value: &Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
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

    /// Wrap `expr` as `(0, expr)` so an access that
    /// [`Self::should_keep_indirect_access`] flagged stays indirect.
    ///
    /// The shape is load-bearing: `remove_sequence_expression`'s idempotency
    /// gate recognizes exactly a 2-element sequence whose head `is_number_0`.
    /// Sibling fold sites in `fold_constants.rs` / `try_fold_conditional_expression`
    /// still build it inline; migrating them here is welcome.
    pub fn preserve_indirect_access(
        span: Span,
        expr: Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        Expression::new_sequence_expression(
            span,
            [Expression::new_numeric_literal(span, 0.0, None, NumberBase::Decimal, ctx), expr],
            ctx,
        )
    }

    pub fn remove_dead_code_exit_class_body(body: &mut ClassBody<'a>, _ctx: &mut TraverseCtx<'a>) {
        body.body.retain(|e| !matches!(e, ClassElement::StaticBlock(s) if s.body.is_empty()));
    }

    pub fn remove_empty_spread_arguments(args: &mut ArenaVec<'a, Argument<'a>>) {
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
