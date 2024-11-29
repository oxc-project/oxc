use std::ops::Deref;

use oxc_allocator::IntoIn;
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, ConstantValue};
use oxc_semantic::{IsGlobalReference, SymbolTable};
use oxc_traverse::TraverseCtx;

#[derive(Clone, Copy)]
pub struct Ctx<'a, 'b>(pub &'b TraverseCtx<'a>);

impl<'a, 'b> Deref for Ctx<'a, 'b> {
    type Target = &'b TraverseCtx<'a>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> ConstantEvaluation<'a> for Ctx<'a, '_> {
    fn is_global_reference(&self, ident: &oxc_ast::ast::IdentifierReference<'a>) -> bool {
        ident.is_global_reference(self.0.symbols())
    }
}

pub fn is_exact_int64(num: f64) -> bool {
    num.fract() == 0.0
}

impl<'a> Ctx<'a, '_> {
    fn symbols(&self) -> &SymbolTable {
        self.0.symbols()
    }

    pub fn value_to_expr(self, span: Span, value: ConstantValue<'a>) -> Expression<'a> {
        match value {
            ConstantValue::Number(n) => {
                let number_base =
                    if is_exact_int64(n) { NumberBase::Decimal } else { NumberBase::Float };
                self.ast.expression_numeric_literal(span, n, Some(Atom::from("")), number_base)
            }
            ConstantValue::BigInt(n) => {
                let raw: &str = (n.to_string() + "n").into_in(&self.ast.allocator);
                self.ast.expression_big_int_literal(
                    span,
                    Some(Atom::from(raw)),
                    BigintBase::Decimal,
                )
            }
            ConstantValue::String(s) => self.ast.expression_string_literal(span, s, None),
            ConstantValue::Boolean(b) => self.ast.expression_boolean_literal(span, b),
            ConstantValue::Undefined => self.ast.void_0(span),
            ConstantValue::Null => self.ast.expression_null_literal(span),
        }
    }

    pub fn is_expression_undefined(self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(ident) if self.is_identifier_undefined(ident) => true,
            Expression::UnaryExpression(e) if e.operator.is_void() && e.argument.is_number() => {
                true
            }
            _ => false,
        }
    }

    pub fn is_identifier_undefined(self, ident: &IdentifierReference) -> bool {
        if ident.name == "undefined" && ident.is_global_reference(self.symbols()) {
            return true;
        }
        false
    }
}
