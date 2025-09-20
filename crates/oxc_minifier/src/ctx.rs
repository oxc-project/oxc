use std::ops::{Deref, DerefMut};

use oxc_ast::{AstBuilder, ast::*};
use oxc_ecmascript::{
    constant_evaluation::{
        ConstantEvaluation, ConstantEvaluationCtx, ConstantValue, binary_operation_evaluate_value,
    },
    side_effects::{MayHaveSideEffects, PropertyReadSideEffects},
};
use oxc_semantic::{IsGlobalReference, Scoping, SymbolId};
use oxc_span::format_atom;
use oxc_syntax::{
    identifier::{is_identifier_part, is_identifier_start},
    reference::ReferenceId,
};
use oxc_traverse::Ancestor;

use crate::{options::CompressOptions, state::MinifierState, symbol_value::SymbolValue};
use oxc_compat::ESFeature;

pub type TraverseCtx<'a> = oxc_traverse::TraverseCtx<'a, MinifierState<'a>>;

pub struct Ctx<'a, 'b>(&'b mut TraverseCtx<'a>);

impl<'a, 'b> Ctx<'a, 'b> {
    pub fn new(ctx: &'b mut TraverseCtx<'a>) -> Self {
        Self(ctx)
    }
}

impl<'a> Deref for Ctx<'a, '_> {
    type Target = TraverseCtx<'a>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> DerefMut for Ctx<'a, '_> {
    fn deref_mut(&mut self) -> &mut TraverseCtx<'a> {
        self.0
    }
}

impl<'a> oxc_ecmascript::GlobalContext<'a> for Ctx<'a, '_> {
    fn is_global_reference(&self, ident: &IdentifierReference<'_>) -> bool {
        ident.is_global_reference(self.0.scoping())
    }

    fn get_constant_value_for_reference_id(
        &self,
        reference_id: ReferenceId,
    ) -> Option<ConstantValue<'a>> {
        self.scoping()
            .get_reference(reference_id)
            .symbol_id()
            .and_then(|symbol_id| self.state.symbol_values.get_symbol_value(symbol_id))
            .filter(|sv| sv.write_references_count == 0)
            .and_then(|sv| sv.initialized_constant.as_ref())
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
    pub fn scoping(&self) -> &Scoping {
        self.0.scoping()
    }

    pub fn options(&self) -> &CompressOptions {
        &self.0.state.options
    }

    /// Check if the target engines supports a feature.
    ///
    /// Returns `true` if the feature is supported.
    pub fn supports_feature(&self, feature: ESFeature) -> bool {
        !self.options().target.has_feature(feature)
    }

    pub fn source_type(&self) -> SourceType {
        self.0.state.source_type
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

    pub fn init_value(&mut self, symbol_id: SymbolId, constant: Option<ConstantValue<'a>>) {
        let mut exported = false;
        if self.scoping.current_scope_id() == self.scoping().root_scope_id() {
            for ancestor in self.ancestors() {
                if ancestor.is_export_named_declaration()
                    || ancestor.is_export_all_declaration()
                    || ancestor.is_export_default_declaration()
                {
                    exported = true;
                }
            }
        }

        let mut read_references_count = 0;
        let mut write_references_count = 0;
        for r in self.scoping().get_resolved_references(symbol_id) {
            if r.is_read() {
                read_references_count += 1;
            }
            if r.is_write() {
                write_references_count += 1;
            }
        }

        let scope_id = self.scoping.current_scope_id();
        let symbol_value = SymbolValue {
            initialized_constant: constant,
            exported,
            read_references_count,
            write_references_count,
            scope_id,
        };
        self.state.symbol_values.init_value(symbol_id, symbol_value);
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

    /// `is_identifier_name` patched with KATAKANA MIDDLE DOT and HALFWIDTH KATAKANA MIDDLE DOT
    /// Otherwise `({ 'x・': 0 })` gets converted to `({ x・: 0 })`, which breaks in Unicode 4.1 to
    /// 15.
    /// <https://github.com/oxc-project/unicode-id-start/pull/3>
    pub fn is_identifier_name_patched(s: &str) -> bool {
        let mut chars = s.chars();
        chars.next().is_some_and(is_identifier_start)
            && chars.all(|c| is_identifier_part(c) && c != '・' && c != '･')
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
}
