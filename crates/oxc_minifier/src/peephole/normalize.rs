use crate::generated::ancestor::Ancestor;
use oxc_allocator::{ArenaVec, ReplaceWith, TakeIn};
use oxc_ast::ast::*;
use oxc_ecmascript::{
    constant_evaluation::{DetermineValueType, ValueType},
    side_effects::{is_typed_array_constructor, is_valid_regexp},
};
use oxc_semantic::IsGlobalReference;
use oxc_syntax::scope::ScopeFlags;

use super::PeepholeOptimizations;
use crate::{
    ReusableTraverseCtx, Traverse, TraverseCtx, minifier_traverse::traverse_mut_with_ctx,
    symbol_facts::SymbolFact, symbol_liveness,
};

#[derive(Default)]
pub struct NormalizeOptions {
    pub convert_while_to_fors: bool,
    pub convert_const_to_let: bool,
    pub remove_unnecessary_use_strict: bool,
}

/// Normalize AST
///
/// Make subsequent AST passes easier to analyze:
///
/// * remove `Statement::EmptyStatement`
/// * remove `ParenthesizedExpression`
/// * convert whiles to fors
/// * convert `const` to `let` for non-exported variables
/// * convert `Infinity` to `f64::INFINITY`
/// * convert `NaN` and `Number.NaN` to `f64::NaN`
/// * convert `var x; void x` to `void 0`
/// * convert `undefined` to `void 0`
/// * apply `pure` to side-effect free global constructors (e.g. `new WeakMap()`)
/// * remove unnecessary 'use strict' directive
///
/// Also
///
/// * remove `debugger` and `console.log` (optional)
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/Normalize.java>
pub struct Normalize {
    options: NormalizeOptions,
}

impl<'a> Normalize {
    pub fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for Normalize {
    fn exit_program(&mut self, node: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        if self.options.remove_unnecessary_use_strict && node.source_type.is_module() {
            node.directives.drain_filter(|d| d.directive.as_str() == "use strict");
        }
    }

    // Normalize is the only traversal that builds stable liveness metadata:
    // registered function declarations and implicitly observable bindings.
    // Each post-flush analysis derives ownership and reachability from the
    // current semantic references.
    fn enter_function(&mut self, node: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        symbol_liveness::register_function(node, ctx);
    }

    fn enter_variable_declaration(
        &mut self,
        node: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        symbol_liveness::register_using_declaration(node, ctx);
    }

    fn enter_export_named_declaration(
        &mut self,
        node: &mut ExportNamedDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        symbol_liveness::register_named_export(node, ctx);
    }

    fn enter_export_default_declaration(
        &mut self,
        node: &mut ExportDefaultDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        symbol_liveness::register_default_export(node, ctx);
    }

    fn exit_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // No console handling here: `exit_expression` has already rewritten
        // every console call (statement position included) to `void 0`.
        stmts.retain(|stmt| match stmt {
            Statement::EmptyStatement(_) => false,
            Statement::DebuggerStatement(_) if ctx.state.options.drop_debugger => false,
            _ => true,
        });
    }

    fn exit_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.convert_const_to_let {
            Self::convert_const_to_let(decl, ctx);
        }
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match stmt {
            Statement::WhileStatement(_) if self.options.convert_while_to_fors => {
                Self::convert_while_to_for(stmt, ctx);
            }
            _ => {}
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Expression::ParenthesizedExpression(paren_expr) = expr {
            *expr = paren_expr.expression.take_in(ctx);
        }
        // Handled outside the match below so the replacement can go through
        // `ctx.replace_expression`, which walks the dropped call (its
        // argument subtrees may contain resolved references) into `PassDirty`.
        if ctx.state.options.drop_console
            && let Expression::CallExpression(call_expr) = &*expr
            && Self::is_console_call_expression(call_expr)
        {
            let new_expr = Expression::new_void_0(call_expr.span, ctx);
            ctx.replace_expression(expr, new_expr);
            return;
        }
        if let Some(e) = match expr {
            Expression::Identifier(ident) => Self::try_compress_identifier(ident, ctx),
            Expression::UnaryExpression(e) if e.operator.is_void() => {
                Self::fold_void_ident(e, ctx);
                None
            }
            Expression::StaticMemberExpression(e) => Self::fold_number_nan_to_nan(e, ctx),
            _ => None,
        } {
            *expr = e;
        }
    }

    fn exit_call_expression(&mut self, e: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::set_no_side_effects_to_call_expr(e, ctx);
    }

    // The three hooks below seed `MinifierState::symbol_facts` (see its docs)
    // with a program-wide, execution-order-independent view of member writes
    // before the fixed-point loop runs. `MinifierState::seeds_symbol_facts`
    // says when any consumer is live.

    /// Covers `=` / compound / logical assignment lefts, destructuring member
    /// targets (`[o.x] = arr`), and for-in/of lefts (`for (o.x in y)`).
    fn exit_assignment_target(
        &mut self,
        node: &mut AssignmentTarget<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !ctx.state.seeds_symbol_facts() {
            return;
        }
        let Some(target) = node.as_simple_assignment_target() else { return };
        // Compound and logical assignments (`o.x += 1`, `o.y ||= 2`) READ the
        // property before writing; every other position is a plain write.
        let is_read_modify = matches!(
            ctx.parent(),
            Ancestor::AssignmentExpressionLeft(e) if *e.operator() != AssignmentOperator::Assign
        );
        Self::record_simple_target_member_write_hazard(target, is_read_modify, ctx);
    }

    /// `o.x++` / `--o.x` read the property before writing.
    fn exit_update_expression(&mut self, e: &mut UpdateExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !ctx.state.seeds_symbol_facts() {
            return;
        }
        Self::record_simple_target_member_write_hazard(
            &e.argument,
            /* is_read_modify */ true,
            ctx,
        );
    }

    /// `delete o.x` neither reads the property nor can trigger setters, so a
    /// single-level delete is harmless — but a CHAINED delete (`delete a.b.c`)
    /// reads the intermediate object `a.b`, hazarding the base `a`.
    fn exit_unary_expression(&mut self, e: &mut UnaryExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !ctx.state.seeds_symbol_facts() {
            return;
        }
        if e.operator.is_delete()
            && let Some(member) = e.argument.get_inner_expression().as_member_expression()
        {
            // Both flags false: `delete` installs no setter, so the hazard applies
            // only when depth > 1 (the chained intermediate object).
            Self::record_member_write_hazard(
                member.object(),
                /* key_is_unsafe */ false,
                /* is_read_modify */ false,
                ctx,
            );
        }
    }

    fn exit_new_expression(&mut self, e: &mut NewExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::set_pure_or_no_side_effects_to_new_expr(e, ctx);
    }

    fn exit_function_body(&mut self, body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.remove_unnecessary_use_strict {
            Self::remove_unused_use_strict_directive(body, ctx);
        }
    }
}

impl<'a> Normalize {
    pub fn new(options: NormalizeOptions) -> Self {
        Self { options }
    }

    fn is_console_call_expression(call_expr: &CallExpression<'_>) -> bool {
        let Some(member_expr) = call_expr.callee.as_member_expression() else { return false };
        let obj = member_expr.object();
        let Some(ident) = obj.get_identifier_reference() else { return false };
        ident.name == "console"
    }

    fn convert_while_to_for(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        // Caller (`exit_statement`) only calls this for a `WhileStatement`, so the `else` is dead.
        stmt.replace_with(|stmt| {
            let Statement::WhileStatement(while_stmt) = stmt else { unreachable!() };
            let while_stmt = while_stmt.unbox();
            let for_stmt = ForStatement::boxed_with_scope_id(
                while_stmt.span,
                None,
                Some(while_stmt.test),
                None,
                while_stmt.body,
                ctx.create_child_scope_of_current(ScopeFlags::empty()),
                ctx,
            );
            Statement::ForStatement(for_stmt)
        });
    }

    fn convert_const_to_let(decl: &mut VariableDeclaration<'a>, ctx: &TraverseCtx<'a>) {
        // checking whether the current scope is the root scope instead of
        // checking whether any variables are exposed to outside (e.g. `export` in ESM)
        if decl.kind.is_const()
            && ctx.current_scope_id() != ctx.scoping().root_scope_id()
            // direct eval may have a assignment inside
            && !ctx.current_scope_flags().contains_direct_eval()
        {
            let all_declarations_are_only_read = decl.declarations.iter().all(|d| {
                d.id.all_binding_identifiers(&mut |id| {
                    ctx.scoping()
                        .get_resolved_references(id.symbol_id())
                        .all(|reference| reference.flags().is_read_only())
                })
            });
            if all_declarations_are_only_read {
                // mark all declarations as `let`
                decl.kind = VariableDeclarationKind::Let;
                for decl in &mut decl.declarations {
                    decl.kind = VariableDeclarationKind::Let;
                }
            }
        }
    }

    /// Transforms `undefined` => `void 0`, `Infinity` => `f64::Infinity`, `NaN` -> `f64::NaN`.
    /// So subsequent passes don't need to look up whether these variables are shadowed or not.
    fn try_compress_identifier(
        ident: &IdentifierReference<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        match ident.name.as_str() {
            "undefined" if ident.is_global_reference(ctx.scoping()) => {
                // `delete undefined` returns `false`
                // `delete void 0` returns `true`
                if Self::is_unary_delete_ancestor(ctx.ancestors()) {
                    return None;
                }
                Some(Expression::new_void_0(ident.span, ctx))
            }
            "Infinity" if ident.is_global_reference(ctx.scoping()) => {
                // `delete Infinity` returns `false`
                // `delete 1/0` returns `true`
                if Self::is_unary_delete_ancestor(ctx.ancestors()) {
                    return None;
                }
                Some(Expression::new_numeric_literal(
                    ident.span,
                    f64::INFINITY,
                    None,
                    NumberBase::Decimal,
                    ctx,
                ))
            }
            "NaN" if ident.is_global_reference(ctx.scoping()) => {
                // `delete NaN` returns `false`
                // `delete 0/0` returns `true`
                if Self::is_unary_delete_ancestor(ctx.ancestors()) {
                    return None;
                }
                Some(Expression::new_nan(ident.span, ctx))
            }
            _ => None,
        }
    }

    fn is_unary_delete_ancestor<'t>(ancestors: impl Iterator<Item = Ancestor<'a, 't>>) -> bool {
        for ancestor in ancestors {
            match ancestor {
                Ancestor::UnaryExpressionArgument(e) if e.operator().is_delete() => {
                    return true;
                }
                Ancestor::ParenthesizedExpressionExpression(_)
                | Ancestor::SequenceExpressionExpressions(_) => {}
                _ => return false,
            }
        }
        false
    }

    fn fold_void_ident(e: &mut UnaryExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        debug_assert!(e.operator.is_void());
        let Expression::Identifier(ident) = &e.argument else { return };
        if ident.is_global_reference(ctx.scoping()) {
            return;
        }
        // `replace_expression` walks the dropped ident into `PassDirty`, so
        // its resolved reference is pruned by the pre-loop `end_pass` flush,
        // before pass 1 — otherwise the symbol would look referenced forever.
        let new_arg =
            Expression::new_numeric_literal(ident.span, 0.0, None, NumberBase::Decimal, ctx);
        ctx.replace_expression(&mut e.argument, new_arg);
    }

    fn fold_number_nan_to_nan(
        e: &StaticMemberExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let Expression::Identifier(ident) = &e.object else { return None };
        if ident.name != "Number" {
            return None;
        }
        if e.property.name != "NaN" {
            return None;
        }
        if !ctx.is_global_reference(ident) {
            return None;
        }
        Some(Expression::new_nan(ident.span, ctx))
    }

    pub(crate) fn set_no_side_effects_to_call_expr(
        call_expr: &mut CallExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        if call_expr.pure {
            return;
        }
        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return;
        };
        if let Some(symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id() {
            // Apply `/* #__NO_SIDE_EFFECTS__ */`
            if ctx.scoping().no_side_effects().contains(&symbol_id) {
                call_expr.pure = true;
            }
        }
    }

    /// Set `pure` on side effect free `new Expr()`s.
    /// `PC` or `PC_WITH_ARRAY` in <https://github.com/rollup/rollup/blob/v4.42.0/src/ast/nodes/shared/knownGlobals.ts>
    pub(crate) fn set_pure_or_no_side_effects_to_new_expr(
        new_expr: &mut NewExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        if new_expr.pure {
            return;
        }
        let Some(ident) = new_expr.callee.get_identifier_reference() else {
            return;
        };
        if let Some(symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id() {
            // Apply `/* #__NO_SIDE_EFFECTS__ */`
            if ctx.scoping().no_side_effects().contains(&symbol_id) {
                new_expr.pure = true;
            }
            return;
        }
        // callee is a global reference.
        let len = new_expr.arguments.len();

        let (zero_arg_throws_error, one_arg_array_throws_error, one_arg_throws_error): (
            bool,
            bool,
            &'static [ValueType],
        ) = match ident.name.as_str() {
            "AggregateError" => (
                true,
                false,
                &[
                    ValueType::Undefined,
                    ValueType::Null,
                    ValueType::Number,
                    ValueType::BigInt,
                    ValueType::Boolean,
                    ValueType::Object,
                ],
            ),
            "DataView" => (
                true,
                true,
                &[
                    ValueType::Undefined,
                    ValueType::Null,
                    ValueType::Number,
                    ValueType::Boolean,
                    ValueType::BigInt,
                    ValueType::String,
                    ValueType::Boolean,
                    ValueType::Object,
                ],
            ),
            // `Set` accepts any iterable of values, so a string argument is pure.
            "Set" => (
                false,
                false,
                &[ValueType::Number, ValueType::Boolean, ValueType::BigInt, ValueType::Object],
            ),
            // `Map`/`WeakSet`/`WeakMap` need `[k, v]` entries / object keys, so a string
            // argument throws (`new Map("ab")` — `"a"` is not an entry; `new WeakSet("a")`
            // — `"a"` is not an object). Only `null`/`undefined` (empty) and the
            // array-literal forms handled above are pure for these.
            "Map" | "WeakSet" | "WeakMap" => (
                false,
                false,
                &[
                    ValueType::Number,
                    ValueType::Boolean,
                    ValueType::BigInt,
                    ValueType::String,
                    ValueType::Object,
                ],
            ),
            "ArrayBuffer" | "Date" => (false, false, &[ValueType::BigInt]),
            "Boolean" | "Error" | "EvalError" | "RangeError" | "ReferenceError" | "SyntaxError"
            | "TypeError" | "URIError" | "Number" | "Object" | "String" => (false, false, &[]),
            // RegExp needs special validation using the regex parser
            "RegExp" => {
                if Self::can_set_pure(ident, ctx) && is_valid_regexp(&new_expr.arguments) {
                    new_expr.pure = true;
                }
                return;
            }
            // Typed arrays allocate a zeroed buffer and run no user code when the
            // length argument is a non-negative numeric literal: it is either a valid
            // length (pure) or too large, which throws a maximum-length `RangeError`
            // that the minifier is allowed to drop (see `docs/ASSUMPTIONS.md`). The
            // value must be checked explicitly because constant folding can turn `-1`
            // into a negative-valued `NumericLiteral`. Other arguments are kept:
            // `new Int8Array(-1)` throws a negative-length `RangeError`,
            // `new Int8Array(0n)` throws a `TypeError` (BigInt), and an object argument
            // can run user code via `Symbol.iterator` / `valueOf`. The 0-arg and
            // `0`-literal forms (the latter folded to 0-arg by
            // `substitute_typed_array_constructor`) are both covered, so the result is
            // idempotent regardless of fold order.
            name if is_typed_array_constructor(name) => {
                let safe_length = new_expr.arguments.is_empty()
                    || (new_expr.arguments.len() == 1
                        && matches!(
                            new_expr.arguments[0].as_expression(),
                            Some(Expression::NumericLiteral(lit)) if lit.value >= 0.0
                        ));
                if safe_length && Self::can_set_pure(ident, ctx) {
                    new_expr.pure = true;
                }
                return;
            }
            _ => return,
        };

        if !Self::can_set_pure(ident, ctx) {
            return;
        }

        if match len {
            0 if !zero_arg_throws_error => true,
            1 => match new_expr.arguments[0].as_expression() {
                Some(Expression::ArrayExpression(array_expr)) => {
                    if one_arg_array_throws_error {
                        false
                    } else if array_expr.elements.is_empty() {
                        true
                    } else {
                        // A non-empty array literal is iterated via the built-in
                        // array iterator (side-effect-free; element side effects are
                        // preserved when the call is dropped). Only `Set`/`Map` accept
                        // arbitrary entries: `Set` takes any values; `Map` requires
                        // every entry to be an array literal (`new Map([1])` throws —
                        // `1` is not iterable). `WeakSet`/`WeakMap` are excluded —
                        // their keys must be objects, so `new WeakSet([1])` /
                        // `new WeakMap([[1, 2]])` throw.
                        match ident.name.as_str() {
                            "Set" => true,
                            "Map" => array_expr
                                .elements
                                .iter()
                                .all(|el| matches!(el, ArrayExpressionElement::ArrayExpression(_))),
                            _ => false,
                        }
                    }
                }
                Some(Expression::StringLiteral(str_lit)) => {
                    // A string is an iterable of its characters. For constructors that
                    // don't list `String` as throwing (e.g. `Set`, `AggregateError`,
                    // `Number`) it is pure. `Map`/`WeakSet`/`WeakMap` list `String`
                    // because their entries must be `[k, v]` pairs / object keys, so a
                    // non-empty string throws -- but an empty string yields no entries
                    // and is still pure. (`DataView` also lists `String` and throws even
                    // on the empty string, as it needs an `ArrayBuffer`.)
                    if one_arg_throws_error.contains(&ValueType::String) {
                        str_lit.value.is_empty()
                            && matches!(ident.name.as_str(), "Map" | "WeakSet" | "WeakMap")
                    } else {
                        true
                    }
                }
                Some(e) => {
                    if let Expression::NewExpression(new_expr) = e {
                        new_expr.pure
                    } else {
                        let value_type = e.value_type(&ctx);
                        if value_type.is_undetermined() {
                            false
                        } else {
                            !one_arg_throws_error.contains(&value_type)
                        }
                    }
                }
                _ => false,
            },
            _ => false,
        } {
            new_expr.pure = true;
        }
    }

    fn can_set_pure(ident: &IdentifierReference<'a>, ctx: &TraverseCtx<'a>) -> bool {
        ctx.is_global_reference(ident)
            // Throw is never pure.
            && !matches!(ctx.parent(), Ancestor::ThrowStatementArgument(_))
    }

    /// Extract the member-write parts of a (simple) assignment target and
    /// record a hazard for its base symbol when one applies.
    fn record_simple_target_member_write_hazard(
        target: &SimpleAssignmentTarget<'a>,
        is_read_modify: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Direct member targets, or the inner member of TS wrappers
        // (`(o.x as any) = 1`); plain identifier targets have no member write.
        let Some(member) = target.as_member_expression().or_else(|| {
            target
                .get_expression()
                .map(Expression::get_inner_expression)
                .and_then(Expression::as_member_expression)
        }) else {
            return;
        };
        let key_is_unsafe = match member {
            MemberExpression::StaticMemberExpression(e) => e.property.name == "__proto__",
            MemberExpression::ComputedMemberExpression(e) => {
                !PeepholeOptimizations::member_key_is_safe(&e.expression)
            }
            // Private fields can't be `__proto__` and aren't affected by the
            // prototype chain.
            MemberExpression::PrivateFieldExpression(_) => false,
        };
        Self::record_member_write_hazard(member.object(), key_is_unsafe, is_read_modify, ctx);
    }

    /// Record the base symbol of a hazardous member write in
    /// `MinifierState::symbol_facts`. `object` is the member expression's object;
    /// walking it to the base identifier determines the chain depth.
    ///
    /// Sets `MEMBER_WRITE_HAZARD` iff the op reads the property
    /// (`is_read_modify`), the write is chained (`a.b.c = 1` — dropping the
    /// intermediate `a.b = {}` would throw), or the key may be `"__proto__"` or a
    /// non-literal computed value (`key_is_unsafe` — the write may install
    /// setters).
    ///
    /// Additionally sets `PROTO_WRITTEN` for the same unsafe keys — a purely
    /// syntactic fact; the sole-reference exemption is applied at the consumer
    /// (`is_member_assign_to_unused_binding`), where the reference count is
    /// current rather than frozen at seed time.
    fn record_member_write_hazard(
        object: &Expression<'a>,
        key_is_unsafe: bool,
        is_read_modify: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // In DCE mode only `PROTO_WRITTEN` has a live consumer (the opt-in
        // drop); `MEMBER_WRITE_HAZARD`'s default-path reader is full-minify
        // only. Skip hazard-only work (safe-key read-modify ops, chained
        // writes, chained deletes) before walking the chain.
        if ctx.state.dce && !key_is_unsafe {
            return;
        }
        let mut depth = 1u32;
        let mut object = object;
        let base = loop {
            let inner = object.get_inner_expression();
            if let Expression::Identifier(ident) = inner {
                break ident;
            }
            let Some(member) = inner.as_member_expression() else {
                // No identifier base (`f().x`, `(a?.b).c`, `this.x`, …) —
                // nothing droppable resolves through these, so no hazard to
                // record.
                return;
            };
            depth += 1;
            object = member.object();
        };
        if !(is_read_modify || depth > 1 || key_is_unsafe) {
            return;
        }
        let Some(symbol_id) = ctx.scoping().get_reference(base.reference_id()).symbol_id() else {
            return;
        };
        let mut facts = SymbolFact::MEMBER_WRITE_HAZARD;
        if key_is_unsafe {
            facts |= SymbolFact::PROTO_WRITTEN;
        }
        ctx.state.symbol_facts.insert(symbol_id, facts);
    }

    fn remove_unused_use_strict_directive(body: &mut FunctionBody<'a>, ctx: &TraverseCtx<'a>) {
        if !body.directives.is_empty()
            && ctx
                .scoping()
                .scope_parent_id(ctx.current_scope_id())
                .map(|scope_id| ctx.scoping().scope_flags(scope_id))
                .is_some_and(ScopeFlags::is_strict_mode)
        {
            body.directives.drain_filter(|d| d.directive.as_str() == "use strict");
        }
    }
}
