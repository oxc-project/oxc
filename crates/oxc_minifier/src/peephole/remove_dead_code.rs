use crate::generated::ancestor::Ancestor;
use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue},
    side_effects::MayHaveSideEffects,
};
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeId;

use crate::{TraverseCtx, keep_var::KeepVar, state::FunctionSummary};

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
                let mut block = BlockStatement::boxed(finalizer.span, ArenaVec::new_in(ctx), ctx);
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
            let exprs = ArenaVec::from_array_in(
                [
                    {
                        let mut test = e.test.take_in(ctx);
                        Self::remove_unused_expression(&mut test, ctx);
                        test
                    },
                    if v { e.consequent.take_in(ctx) } else { e.alternate.take_in(ctx) },
                ],
                ctx,
            );
            Expression::new_sequence_expression(e.span, exprs, ctx)
        } else {
            let result_expr = if v { e.consequent.take_in(ctx) } else { e.alternate.take_in(ctx) };
            let should_keep_as_sequence_expr = Self::should_keep_indirect_access(&result_expr, ctx);
            // "(1 ? a.b : 0)()" => "(0, a.b)()"
            if should_keep_as_sequence_expr {
                Expression::new_sequence_expression(
                    e.span,
                    ArenaVec::from_array_in(
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
                    ),
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
                        false,
                        f.scope_id.get(),
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
                                    true,
                                    a.scope_id.get(),
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
                                        false,
                                        f.scope_id.get(),
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

    /// Record what we know about a named function literal: whether it is pure
    /// (`pure_return`) and, independently, how many trailing arguments call
    /// sites may drop (`dead_arg_prefix`). Each fact has its own gates on top of
    /// a shared set; an entry is inserted when either fact is present and any
    /// stale entry is removed when neither is, so a fact recorded for an earlier
    /// pass or an earlier declaration of the same symbol never survives.
    #[expect(clippy::too_many_arguments)]
    fn try_save_pure_function(
        id: Option<&BindingIdentifier<'a>>,
        params: &FormalParameters<'a>,
        body: &FunctionBody<'a>,
        r#async: bool,
        generator: bool,
        is_arrow: bool,
        scope_id: Option<ScopeId>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // ── Shared gates ──
        let Some(symbol_id) = id.and_then(|id| id.symbol_id.get()) else { return };
        // Redeclarations are span-only in oxc_semantic and create no references, so
        // they're invisible to the read-only-refs check below. The runtime-winning
        // declaration may be an impure one that never reaches this function, so a
        // fact recorded for another declaration of the same symbol cannot be
        // trusted and must not survive.
        if !ctx.scoping().symbol_redeclarations(symbol_id).is_empty() {
            ctx.state.pure_functions.remove(&symbol_id);
            return;
        }
        // Params must be plain identifiers with no default. `function foo({}) {}
        // foo(null)` is a runtime type error, and — because in oxc a parameter
        // default lives in `FormalParameter::initializer`, NOT an
        // `AssignmentPattern` — `is_binding_identifier()` alone would accept
        // `(u = g())`, whose default runs on `undefined`; the explicit
        // `initializer.is_none()` excludes it.
        if !params
            .items
            .iter()
            .all(|pat| pat.pattern.is_binding_identifier() && pat.initializer.is_none())
        {
            ctx.state.pure_functions.remove(&symbol_id);
            return;
        }
        // A writable binding may be reassigned to a different (impure / arg-using)
        // function that never reaches this recorder, so its facts can't be trusted.
        if !ctx.scoping().get_resolved_references(symbol_id).all(|r| r.flags().is_read_only()) {
            ctx.state.pure_functions.remove(&symbol_id);
            return;
        }

        // ── `pure_return` fact — exactly the original gates and semantics ──
        let pure_return = if !r#async
            && !generator
            && !body.statements.iter().any(|stmt| stmt.may_have_side_effects(ctx))
        {
            Some(if body.is_empty() { Some(ConstantValue::Undefined) } else { None })
        } else {
            None
        };

        // ── `dead_arg_prefix` fact — async / generator / effectful bodies OK ──
        let dead_arg_prefix = Self::compute_dead_arg_prefix(params, is_arrow, scope_id, ctx);

        if pure_return.is_some() || dead_arg_prefix.is_some() {
            ctx.state
                .pure_functions
                .insert(symbol_id, FunctionSummary { pure_return, dead_arg_prefix });
        } else {
            ctx.state.pure_functions.remove(&symbol_id);
        }
    }

    /// Smallest `N` such that every argument at index `>= N` is safe to drop at
    /// a call site: its parameter is unused, or it is beyond the declared params
    /// (which a function that ignores extra args never observes). `None` when a
    /// soundness gate fails. Shared gates (plain params, read-only binding, no
    /// redeclaration) are already checked by the caller.
    fn compute_dead_arg_prefix(
        params: &FormalParameters<'a>,
        is_arrow: bool,
        scope_id: Option<ScopeId>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<usize> {
        // A rest param collects trailing args, so an argument's index no longer
        // maps to a fixed parameter — never drop.
        if params.rest.is_some() {
            return None;
        }
        // Direct eval anywhere can read parameters by name / reflect on the call.
        // `DirectEval` propagates to the root scope, so one global check suffices.
        if ctx.scoping().root_scope_flags().contains_direct_eval() {
            return None;
        }
        // A binding at a script's root scope is aliased on `globalThis` and can be
        // reassigned by another script (or indirect eval) with no reference we can
        // see, so its parameter-usage facts are untrustworthy.
        if Self::keep_top_level_var_in_script_mode(ctx) {
            return None;
        }
        // A non-arrow function exposes `arguments`, which reflects the *actual*
        // argument list regardless of the declared params, so dropping an arg an
        // `arguments[i]` read observes would change behavior. `arguments` mentions
        // surface as an unresolved root reference, so one program-wide lookup
        // covers every non-arrow body — except sloppy `var arguments`, which binds
        // a real symbol aliasing the arguments object; that shape is a SyntaxError
        // in strict code, so requiring the function scope to be strict closes it.
        // Arrows never bind `arguments`; a nested non-arrow's `arguments` is its
        // own object, unrelated to this call's args.
        if !is_arrow {
            let is_strict = scope_id
                .is_some_and(|scope_id| ctx.scoping().scope_flags(scope_id).is_strict_mode());
            if !is_strict || ctx.scoping().root_unresolved_references().contains_key("arguments") {
                return None;
            }
        }
        // Scan params from the end while unused; `n` is the count that must be
        // kept. Recorded even when `n == params.len()` (no trailing unused param)
        // so call sites can still drop arguments passed beyond the declared params.
        let mut n = params.items.len();
        while n > 0 {
            let BindingPattern::BindingIdentifier(id) = &params.items[n - 1].pattern else { break };
            let Some(symbol_id) = id.symbol_id.get() else { break };
            if !ctx.scoping().symbol_is_unused(symbol_id) {
                break;
            }
            n -= 1;
        }
        Some(n)
    }

    pub fn remove_dead_code_call_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::CallExpression(e) = expr else { return };
        let Expression::Identifier(ident) = &e.callee else { return };
        let reference_id = ident.reference_id();
        let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else { return };
        let Some((is_empty_body_pure, dead_arg_prefix)) =
            ctx.state.pure_functions.get(&symbol_id).map(|summary| {
                (
                    matches!(summary.pure_return, Some(Some(ConstantValue::Undefined))),
                    summary.dead_arg_prefix,
                )
            })
        else {
            return;
        };

        // An empty-bodied pure function returns `undefined`, so the whole call is
        // just its (effectful) arguments followed by `void 0`.
        if is_empty_body_pure {
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

        // Otherwise drop trailing arguments the callee never reads.
        let Some(n) = dead_arg_prefix else { return };
        // A spread contributes an unknown number of values, so an argument's
        // index no longer maps to a fixed parameter position; a trailing pop past
        // an earlier spread could drop a value that actually lands on a *used*
        // parameter. Bail if any argument is a spread.
        if e.arguments.iter().any(|arg| matches!(arg, Argument::SpreadElement(_))) {
            return;
        }
        while e.arguments.len() > n {
            // Stop at the first side-effectful trailing arg: it must still run.
            if e.arguments.last().unwrap().may_have_side_effects(ctx) {
                break;
            }
            // Spreads were excluded above, so `into_expression` cannot panic.
            let dropped = e.arguments.pop().unwrap().into_expression();
            ctx.drop_expression(&dropped);
        }
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
            ArenaVec::from_array_in(
                [Expression::new_numeric_literal(span, 0.0, None, NumberBase::Decimal, ctx), expr],
                ctx,
            ),
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
