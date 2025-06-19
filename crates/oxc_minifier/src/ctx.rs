use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

use rustc_hash::FxHashMap;

use oxc_ast::{AstBuilder, ast::*};
use oxc_ecmascript::{
    constant_evaluation::{
        ConstantEvaluation, ConstantEvaluationCtx, ConstantValue, binary_operation_evaluate_value,
    },
    side_effects::{MayHaveSideEffects, PropertyReadSideEffects},
};
use oxc_semantic::{IsGlobalReference, Scoping, SymbolId};
use oxc_span::format_atom;
use oxc_syntax::reference::ReferenceId;

use crate::CompressOptions;

pub struct MinifierState<'a> {
    pub options: Rc<CompressOptions>,

    /// Constant values evaluated from expressions.
    ///
    /// Values are saved during constant evaluation phase.
    /// Values are read during [oxc_ecmascript::is_global_reference::IsGlobalReference::get_constant_value_for_reference_id].
    pub constant_values: FxHashMap<SymbolId, ConstantValue<'a>>,
}

impl MinifierState<'_> {
    pub fn new(options: Rc<CompressOptions>) -> Self {
        Self { options, constant_values: FxHashMap::default() }
    }
}

pub type TraverseCtx<'a> = oxc_traverse::TraverseCtx<'a, MinifierState<'a>>;

pub struct Ctx<'a, 'b>(&'b mut TraverseCtx<'a>);

impl<'a, 'b> Ctx<'a, 'b> {
    pub fn new(ctx: &'b mut TraverseCtx<'a>) -> Self {
        Self(ctx)
    }
}

impl<'a, 'b> Deref for Ctx<'a, 'b> {
    type Target = &'b mut TraverseCtx<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[expect(clippy::mut_mut)]
impl<'a, 'b> DerefMut for Ctx<'a, 'b> {
    fn deref_mut(&mut self) -> &mut &'b mut TraverseCtx<'a> {
        &mut self.0
    }
}

impl<'a> oxc_ecmascript::is_global_reference::IsGlobalReference<'a> for Ctx<'a, '_> {
    fn is_global_reference(&self, ident: &IdentifierReference<'_>) -> Option<bool> {
        Some(ident.is_global_reference(self.0.scoping()))
    }

    fn get_constant_value_for_reference_id(
        &self,
        reference_id: ReferenceId,
    ) -> Option<ConstantValue<'a>> {
        self.scoping()
            .get_reference(reference_id)
            .symbol_id()
            .and_then(|symbol_id| self.state.constant_values.get(&symbol_id))
            .cloned()
    }
}

impl<'a> oxc_ecmascript::side_effects::MayHaveSideEffectsContext<'a> for Ctx<'a, '_> {
    fn annotations(&self) -> bool {
        self.state.options.treeshake.annotations
    }

    fn manual_pure_functions(&self, callee: &Expression) -> bool {
        if let Expression::Identifier(ident) = callee {
            return self
                .state
                .options
                .treeshake
                .manual_pure_functions
                .iter()
                .any(|name| ident.name.as_str() == name);
        }
        false
    }

    fn property_read_side_effects(&self) -> PropertyReadSideEffects {
        self.state.options.treeshake.property_read_side_effects
    }

    fn unknown_global_side_effects(&self) -> bool {
        self.state.options.treeshake.unknown_global_side_effects
    }
}

impl<'a> ConstantEvaluationCtx<'a> for Ctx<'a, '_> {
    fn ast(&self) -> AstBuilder<'a> {
        self.ast
    }
}

pub fn is_exact_int64(num: f64) -> bool {
    num.fract() == 0.0
}

impl<'a> Ctx<'a, '_> {
    fn scoping(&self) -> &Scoping {
        self.0.scoping()
    }

    pub fn is_global_reference(&self, ident: &IdentifierReference<'a>) -> bool {
        ident.is_global_reference(self.0.scoping())
    }

    pub fn eval_binary(&self, e: &BinaryExpression<'a>) -> Option<Expression<'a>> {
        if e.may_have_side_effects(self) {
            None
        } else {
            e.evaluate_value(self).map(|v| self.value_to_expr(e.span, v))
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

    pub fn value_to_expr(&self, span: Span, value: ConstantValue<'a>) -> Expression<'a> {
        match value {
            ConstantValue::Number(n) => {
                let number_base =
                    if is_exact_int64(n) { NumberBase::Decimal } else { NumberBase::Float };
                self.ast.expression_numeric_literal(span, n, None, number_base)
            }
            ConstantValue::BigInt(bigint) => {
                let value = format_atom!(self.ast.allocator, "{bigint}");
                self.ast.expression_big_int_literal(span, value, None, BigintBase::Decimal)
            }
            ConstantValue::String(s) => {
                self.ast.expression_string_literal(span, self.ast.atom_from_cow(&s), None)
            }
            ConstantValue::Boolean(b) => self.ast.expression_boolean_literal(span, b),
            ConstantValue::Undefined => self.ast.void_0(span),
            ConstantValue::Null => self.ast.expression_null_literal(span),
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
}
