use oxc_allocator::GetAllocator;
use oxc_ast::ast::*;
use oxc_compat::ESFeature;
use oxc_ecmascript::{
    GlobalContext,
    constant_evaluation::{
        ConstantEvaluationCtx, ConstantValue, ValueType, binary_operation_evaluate_value,
    },
    side_effects::{
        MayHaveSideEffects, MayHaveSideEffectsContext, PropertyReadSideEffects, is_pure_function,
    },
};
use oxc_semantic::{IsGlobalReference, SymbolId};
use oxc_str::format_str;
use oxc_syntax::{reference::ReferenceId, scope::ScopeFlags};

use crate::{
    generated::ancestor::Ancestor,
    options::CompressOptions,
    state::MinifierState,
    symbol_value::{FreshValueKind, ReferenceCounts, SymbolValue},
};

use oxc_ast_visit::Visit;

use super::{TraverseCtx, dropped_subtree_collector::DroppedSubtreeCollector};

pub fn is_exact_int64(num: f64) -> bool {
    num.fract() == 0.0
}

impl<'a> GlobalContext<'a> for TraverseCtx<'a, MinifierState<'a>> {
    fn is_global_reference(&self, ident: &IdentifierReference<'_>) -> bool {
        ident.is_global_reference(self.scoping())
    }

    fn get_constant_value_for_reference_id(
        &self,
        reference_id: ReferenceId,
    ) -> Option<ConstantValue<'a>> {
        self.tracked_constant_for_reference_id(reference_id).cloned()
    }

    fn value_type_for_reference_id(&self, reference_id: ReferenceId) -> Option<ValueType> {
        self.tracked_constant_for_reference_id(reference_id).map(ConstantValue::value_type)
    }
}

impl<'a> GlobalContext<'a> for &TraverseCtx<'a, MinifierState<'a>> {
    fn is_global_reference(&self, ident: &IdentifierReference<'_>) -> bool {
        (*self).is_global_reference(ident)
    }

    fn get_constant_value_for_reference_id(
        &self,
        reference_id: ReferenceId,
    ) -> Option<ConstantValue<'a>> {
        (*self).get_constant_value_for_reference_id(reference_id)
    }

    fn value_type_for_reference_id(&self, reference_id: ReferenceId) -> Option<ValueType> {
        (*self).value_type_for_reference_id(reference_id)
    }
}

impl<'a> GlobalContext<'a> for &mut TraverseCtx<'a, MinifierState<'a>> {
    fn is_global_reference(&self, ident: &IdentifierReference<'_>) -> bool {
        (**self).is_global_reference(ident)
    }

    fn get_constant_value_for_reference_id(
        &self,
        reference_id: ReferenceId,
    ) -> Option<ConstantValue<'a>> {
        (**self).get_constant_value_for_reference_id(reference_id)
    }

    fn value_type_for_reference_id(&self, reference_id: ReferenceId) -> Option<ValueType> {
        (**self).value_type_for_reference_id(reference_id)
    }
}

impl<'a> MayHaveSideEffectsContext<'a> for TraverseCtx<'a, MinifierState<'a>> {
    fn annotations(&self) -> bool {
        self.options().treeshake.annotations
    }

    fn manual_pure_functions(&self, callee: &Expression) -> bool {
        let pure_functions = &self.options().treeshake.manual_pure_functions;
        if pure_functions.is_empty() {
            return false;
        }
        is_pure_function(callee, pure_functions)
    }

    fn property_read_side_effects(&self) -> PropertyReadSideEffects {
        self.options().treeshake.property_read_side_effects
    }

    fn property_write_side_effects(&self) -> bool {
        self.options().treeshake.property_write_side_effects
    }

    fn unknown_global_side_effects(&self) -> bool {
        self.options().treeshake.unknown_global_side_effects
    }
}

impl<'a> MayHaveSideEffectsContext<'a> for &TraverseCtx<'a, MinifierState<'a>> {
    fn annotations(&self) -> bool {
        (*self).annotations()
    }

    fn manual_pure_functions(&self, callee: &Expression) -> bool {
        (*self).manual_pure_functions(callee)
    }

    fn property_read_side_effects(&self) -> PropertyReadSideEffects {
        (*self).property_read_side_effects()
    }

    fn property_write_side_effects(&self) -> bool {
        (*self).property_write_side_effects()
    }

    fn unknown_global_side_effects(&self) -> bool {
        (*self).unknown_global_side_effects()
    }
}

impl<'a> MayHaveSideEffectsContext<'a> for &mut TraverseCtx<'a, MinifierState<'a>> {
    fn annotations(&self) -> bool {
        (**self).annotations()
    }

    fn manual_pure_functions(&self, callee: &Expression) -> bool {
        (**self).manual_pure_functions(callee)
    }

    fn property_read_side_effects(&self) -> PropertyReadSideEffects {
        (**self).property_read_side_effects()
    }

    fn property_write_side_effects(&self) -> bool {
        (**self).property_write_side_effects()
    }

    fn unknown_global_side_effects(&self) -> bool {
        (**self).unknown_global_side_effects()
    }
}

impl<'a> ConstantEvaluationCtx<'a> for TraverseCtx<'a, MinifierState<'a>> {}

impl<'a> TraverseCtx<'a, MinifierState<'a>> {
    pub fn options(&self) -> &CompressOptions {
        self.state.options()
    }

    /// Check if the target engines supports a feature.
    ///
    /// Returns `true` if the feature is supported.
    pub fn supports_feature(&self, feature: ESFeature) -> bool {
        !self.options().target.has_feature(feature)
    }

    pub fn source_type(&self) -> SourceType {
        self.state.source_type()
    }

    /// Whether this run removes dead code without applying size-only rewrites.
    pub fn is_tree_shake_only(&self) -> bool {
        self.state.is_tree_shake_only()
    }

    /// Whether `Normalize` should seed persistent member-write metadata for a
    /// consumer enabled by this configuration.
    pub fn should_track_member_write_effects(&self) -> bool {
        self.state.should_track_member_write_effects()
    }

    pub fn is_global_reference(&self, ident: &IdentifierReference<'a>) -> bool {
        ident.is_global_reference(self.scoping())
    }

    fn tracked_constant_for_reference_id(
        &self,
        reference_id: ReferenceId,
    ) -> Option<&ConstantValue<'a>> {
        self.scoping()
            .get_reference(reference_id)
            .symbol_id()
            .and_then(|symbol_id| self.state.symbols.value(symbol_id))
            .filter(|sv| !sv.references.has_writes())
            .and_then(|sv| sv.initialized_constant.as_ref())
    }

    pub fn eval_binary(&self, e: &BinaryExpression<'a>) -> Option<Expression<'a>> {
        if e.may_have_side_effects(self) {
            None
        } else {
            let v = self.eval_binary_operation(e.operator, &e.left, &e.right)?;
            // Bail instead of materializing the shadow-safe division form:
            // replacing `0 / 0` with `0 / 0` would set the changed flag on
            // every pass and the fixed-point loop would never converge. The
            // other `eval_binary_operation` callers fold bitwise operators
            // only, whose results are always finite, so the guard lives here.
            if let ConstantValue::Number(n) = &v
                && self.non_finite_global_shadowed(*n)
            {
                return None;
            }
            Some(self.value_to_expr(e.span, v))
        }
    }

    pub fn eval_binary_operation(
        &self,
        operator: BinaryOperator,
        left: &Expression<'a>,
        right: &Expression<'a>,
    ) -> Option<ConstantValue<'a>> {
        binary_operation_evaluate_value(operator, left, right, self)
    }

    /// Whether materializing `n` prints a global name (`NaN`, `Infinity`)
    /// that a binding in the current scope chain captures: in
    /// `function f() { let NaN = 1; return 0 / 0; }`, folding the division
    /// to a NaN literal makes `f` return `1`.
    fn non_finite_global_shadowed(&self, n: f64) -> bool {
        let name = if n.is_nan() {
            "NaN"
        } else if n.is_infinite() {
            "Infinity"
        } else {
            return false;
        };
        self.scoping().find_binding(self.scoping.current_scope_id(), name.into()).is_some()
    }

    /// `NaN` → `0 / 0`, `Infinity` → `1 / 0`, `-Infinity` → `-1 / 0`: the
    /// same value with no capturable name attached. Used when a non-finite
    /// constant must be materialized where its global name is shadowed.
    fn non_finite_to_division_expr(&self, span: Span, n: f64) -> Expression<'a> {
        let numerator = if n.is_nan() { 0.0 } else { 1.0 };
        let mut left =
            Expression::new_numeric_literal(span, numerator, None, NumberBase::Decimal, self);
        if n == f64::NEG_INFINITY {
            left = Expression::new_unary_expression(span, UnaryOperator::UnaryNegation, left, self);
        }
        let right = Expression::new_numeric_literal(span, 0.0, None, NumberBase::Decimal, self);
        Expression::new_binary_expression(span, left, BinaryOperator::Division, right, self)
    }

    pub fn value_to_expr(&self, span: Span, value: ConstantValue<'a>) -> Expression<'a> {
        match value {
            ConstantValue::Number(n) => {
                if !n.is_finite() && self.non_finite_global_shadowed(n) {
                    return self.non_finite_to_division_expr(span, n);
                }
                let number_base =
                    if is_exact_int64(n) { NumberBase::Decimal } else { NumberBase::Float };
                Expression::new_numeric_literal(span, n, None, number_base, self)
            }
            ConstantValue::BigInt(bigint) => {
                let value = format_str!(self.allocator(), "{bigint}");
                Expression::new_big_int_literal(span, value, None, BigintBase::Decimal, self)
            }
            ConstantValue::String(s) => {
                Expression::new_string_literal(span, Str::from_cow_in(&s, self), None, self)
            }
            ConstantValue::Boolean(b) => Expression::new_boolean_literal(span, b, self),
            ConstantValue::Undefined => Expression::new_void_0(span, self),
            ConstantValue::Null => Expression::new_null_literal(span, self),
        }
    }

    pub fn is_expression_undefined(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(ident) if self.is_identifier_undefined(ident) => true,
            Expression::UnaryExpression(e) if e.operator.is_void() && e.argument.is_number() => {
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn is_identifier_undefined(&self, ident: &IdentifierReference) -> bool {
        if ident.name == "undefined" && ident.is_global_reference(self.scoping()) {
            return true;
        }
        false
    }

    pub fn init_value(
        &mut self,
        symbol_id: SymbolId,
        constant: Option<ConstantValue<'a>>,
        kind: FreshValueKind,
        falsy_init: bool,
        init_absent: bool,
    ) {
        let mut references = ReferenceCounts::default();
        for reference in self.scoping().get_resolved_references(symbol_id) {
            references.record(reference.flags());
        }

        let scope_id = self.scoping().symbol_scope_id(symbol_id);
        let scope_flags = self.scoping().scope_flags(scope_id);
        // Declaration traversal is source-ordered, but a nested function can
        // consume the first declaration's facts before a later declaration of
        // the same symbol is visited. Disable every declaration-derived fact
        // from the outset instead of trying to repair an already-consumed value.
        let has_multiple_value_declarations = self
            .scoping()
            .symbol_redeclarations(symbol_id)
            .iter()
            .filter(|declaration| declaration.flags.is_value())
            .nth(1)
            .is_some();

        // `constant` is the value-context value, `None` when withheld (e.g. a hoisted
        // `var` past a dirty prelude). Capture before it's moved just below.
        let value_withheld = constant.is_none();
        let initialized_constant =
            if has_multiple_value_declarations || scope_flags.contains(ScopeFlags::DirectEval) {
                None
            } else {
                constant
            };

        // `boolean_falsy` (see `SymbolValue::boolean_falsy`) gated to a sound subset:
        // one value declaration, no resolved writes, outside a direct-`eval` scope,
        // and not a script's top-level global (another script could reassign it, so
        // a 0 in-module write count doesn't prove write-once).
        let boolean_falsy = if has_multiple_value_declarations {
            false
        } else {
            falsy_init
                && value_withheld
                && !references.has_writes()
                && !scope_flags.contains(ScopeFlags::DirectEval)
                && !(self.source_type().is_script() && scope_id == self.scoping().root_scope_id())
        };

        // See `SymbolValue::implicit_undefined` — only meaningful when the
        // recorded constant is the hoist-produced `undefined` of `let x;`.
        let implicit_undefined =
            init_absent && initialized_constant.as_ref().is_some_and(ConstantValue::is_undefined);

        let symbol_value = SymbolValue {
            initialized_constant,
            implicit_undefined,
            references,
            kind: if has_multiple_value_declarations { FreshValueKind::None } else { kind },
            boolean_falsy,
        };
        self.state.symbols.init_value(symbol_id, symbol_value);
    }

    /// If two expressions are equal.
    /// Special case `undefined` == `void 0`
    pub fn expr_eq(&self, a: &Expression<'a>, b: &Expression<'a>) -> bool {
        use oxc_span::ContentEq;
        a.content_eq(b) || (self.is_expression_undefined(a) && self.is_expression_undefined(b))
    }

    // https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L2641
    pub fn string_to_equivalent_number_value(s: &str) -> Option<f64> {
        if s.is_empty() {
            return None;
        }
        let mut is_negative = false;
        let mut int_value = 0i32;
        let mut start = 0;
        let bytes = s.as_bytes();
        if bytes[0] == b'-' && s.len() > 1 {
            is_negative = true;
            int_value = -int_value;
            start += 1;
        }
        if bytes[start] == b'0' && s.len() > 1 {
            return None;
        }
        for b in &bytes[start..] {
            if !b.is_ascii_digit() {
                return None;
            }
            int_value = int_value.checked_mul(10).and_then(|v| {
                let n = i32::from(b & 15);
                if is_negative { v.checked_sub(n) } else { v.checked_add(n) }
            })?;
        }
        Some(f64::from(int_value))
    }

    /// Whether the closest function scope is created by an async generator
    pub fn is_closest_function_scope_an_async_generator(&self) -> bool {
        self.ancestors()
            .find_map(|ancestor| match ancestor {
                Ancestor::FunctionBody(body) => Some(*body.r#async() && *body.generator()),
                Ancestor::ArrowFunctionExpressionBody(_) => Some(false),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// Whether the assignment expression needs to be kept to preserve the name
    pub fn is_expression_whose_name_needs_to_be_kept(&self, expr: &Expression) -> bool {
        let options = &self.options().keep_names;
        if !options.class && !options.function {
            return false;
        }
        if !expr.is_anonymous_function_definition() {
            return false;
        }
        let is_class = matches!(expr.without_parentheses(), Expression::ClassExpression(_));
        (options.class && is_class) || (options.function && !is_class)
    }

    /// Construct a `DroppedSubtreeCollector` borrowing the per-pass change accumulator.
    /// Used by the `replace_*` / `drop_*` helpers.
    #[inline]
    fn dropped_subtree_collector(&mut self) -> DroppedSubtreeCollector<'a, '_> {
        DroppedSubtreeCollector::new(&mut self.state.pass_changes)
    }

    /// Replace an expression slot. Marks the pass as having mutated the AST.
    ///
    /// Prefer this over a direct `*slot = new; ctx.notice_change();` pair —
    /// the typed helper keeps dropped-subtree bookkeeping, the slot update,
    /// and the pass revisit request together.
    #[inline]
    pub fn replace_expression(&mut self, slot: &mut Expression<'a>, new: Expression<'a>) {
        self.dropped_subtree_collector().visit_expression(slot);
        *slot = new;
        self.state.record_ast_change();
    }

    /// Replace a statement slot. Marks the pass as having mutated the AST.
    #[inline]
    pub fn replace_statement(&mut self, slot: &mut Statement<'a>, new: Statement<'a>) {
        self.dropped_subtree_collector().visit_statement(slot);
        *slot = new;
        self.state.record_ast_change();
    }

    /// Replace an assignment-target-property slot. Marks the pass as having mutated the AST.
    #[inline]
    pub fn replace_assignment_target_property(
        &mut self,
        slot: &mut AssignmentTargetProperty<'a>,
        new: AssignmentTargetProperty<'a>,
    ) {
        self.dropped_subtree_collector().visit_assignment_target_property(slot);
        *slot = new;
        self.state.record_ast_change();
    }

    /// Replace a property-key slot. Marks the pass as having mutated the AST.
    #[inline]
    pub fn replace_property_key(&mut self, slot: &mut PropertyKey<'a>, new: PropertyKey<'a>) {
        self.dropped_subtree_collector().visit_property_key(slot);
        *slot = new;
        self.state.record_ast_change();
    }

    /// Replace a `for-in` / `for-of` statement's `left` slot. Same contract
    /// as `replace_expression`.
    #[inline]
    pub fn replace_for_statement_left(
        &mut self,
        slot: &mut ForStatementLeft<'a>,
        new: ForStatementLeft<'a>,
    ) {
        self.dropped_subtree_collector().visit_for_statement_left(slot);
        *slot = new;
        self.state.record_ast_change();
    }

    /// Replace a `VariableDeclarator` slot. Same contract as `replace_expression`.
    #[inline]
    pub fn replace_variable_declarator(
        &mut self,
        slot: &mut VariableDeclarator<'a>,
        new: VariableDeclarator<'a>,
    ) {
        self.dropped_subtree_collector().visit_variable_declarator(slot);
        *slot = new;
        self.state.record_ast_change();
    }

    /// Mark the pass as having mutated the AST in place (operand swap, in-place
    /// field flip, collection element removal, etc.) where no slot replacement
    /// happened. Prefer the `replace_*` helpers when the mutation IS a slot
    /// replacement.
    #[inline]
    pub fn notice_change(&mut self) {
        self.state.record_ast_change();
    }

    /// Mark an expression subtree as about to be dropped (popped from a collection,
    /// taken out of an Option, etc.). Walks the subtree to record dead references
    /// and dropped direct-eval calls into the per-pass `PassChanges` accumulator.
    ///
    /// Use this helper at every site where a subtree is being removed from the AST
    /// without an immediate slot-replacement helper (e.g. inside a `retain_mut`
    /// predicate, before `field = None`, after `vec.pop()`).
    #[inline]
    pub fn drop_expression(&mut self, expr: &Expression<'a>) {
        self.dropped_subtree_collector().visit_expression(expr);
        self.state.record_ast_change();
    }

    /// Mark a statement subtree as about to be dropped. Same contract as
    /// `drop_expression`.
    #[inline]
    pub fn drop_statement(&mut self, stmt: &Statement<'a>) {
        self.dropped_subtree_collector().visit_statement(stmt);
        self.state.record_ast_change();
    }

    /// Mark a class element subtree as about to be dropped. Same contract as
    /// `drop_expression`.
    #[inline]
    pub fn drop_class_element(&mut self, element: &ClassElement<'a>) {
        self.dropped_subtree_collector().visit_class_element(element);
        self.state.record_ast_change();
    }

    /// Mark a variable declarator as about to be dropped. Walks the whole
    /// declarator — binding pattern, TS type annotation (which can contain
    /// references, e.g. computed keys in a type literal), and init if still
    /// attached. Same contract as `drop_expression`. If the init is kept
    /// alive elsewhere, `take()` it out of the declarator before calling this.
    #[inline]
    pub fn drop_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        self.dropped_subtree_collector().visit_variable_declarator(decl);
        self.state.record_ast_change();
    }

    /// Mark a switch case subtree as about to be dropped. Walks the entire case —
    /// test expression (if present) and all statements in the consequent. Same contract
    /// as `drop_expression`. Use this helper when removing a case from a switch statement's
    /// case vector to properly notify the scope tracking system about dropped references.
    #[inline]
    pub fn drop_switch_case(&mut self, switch_case: &SwitchCase<'a>) {
        self.dropped_subtree_collector().visit_switch_case(switch_case);
        self.state.record_ast_change();
    }
}
