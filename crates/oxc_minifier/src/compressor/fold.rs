//! Constant Folding
//!
//! <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeFoldConstants.java>

#[allow(clippy::wildcard_imports)]
use oxc_hir::hir::*;
use oxc_hir::hir_util::IsLiteralValue;
use oxc_span::{Atom, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use super::Compressor;

/// Tri state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tri {
    True,
    False,
    Unknown,
}

/// JavaScript Language Type
///
/// <https://tc39.es/ecma262/#sec-ecmascript-language-types>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ty {
    BigInt,
    Boolean,
    Null,
    Number,
    Object,
    Str,
    Void,
    Undetermined,
}

impl<'a> From<&Expression<'a>> for Ty {
    fn from(expr: &Expression<'a>) -> Self {
        // TODO: complete this
        match expr {
            Expression::BigintLiteral(_) => Self::BigInt,
            Expression::BooleanLiteral(_) => Self::Boolean,
            Expression::NullLiteral(_) => Self::Null,
            Expression::NumberLiteral(_) => Self::Number,
            Expression::ObjectExpression(_) => Self::Object,
            Expression::StringLiteral(_) => Self::Str,
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" => Self::Void,
                _ => Self::Undetermined,
            },
            _ => Self::Undetermined,
        }
    }
}

impl<'a> Compressor<'a> {
    pub(crate) fn fold_expression<'b>(&mut self, expr: &'b mut Expression<'a>) {
        let folded_expr = match expr {
            Expression::BinaryExpression(binary_expr) => match binary_expr.operator {
                BinaryOperator::Equality => self.try_fold_comparison(
                    binary_expr.span,
                    binary_expr.operator,
                    &binary_expr.left,
                    &binary_expr.right,
                ),
                _ => None,
            },
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::Typeof => {
                    self.try_fold_typeof(unary_expr.span, &unary_expr.argument)
                }
                _ => None,
            },
            _ => None,
        };
        if let Some(folded_expr) = folded_expr {
            *expr = folded_expr;
        }
    }

    fn try_fold_comparison<'b>(
        &mut self,
        span: Span,
        op: BinaryOperator,
        left: &'b Expression<'a>,
        right: &'b Expression<'a>,
    ) -> Option<Expression<'a>> {
        let value = match self.evaluate_comparison(op, left, right) {
            Tri::True => true,
            Tri::False => false,
            Tri::Unknown => return None,
        };
        let boolean_literal = self.hir.boolean_literal(span, value);
        Some(self.hir.literal_boolean_expression(boolean_literal))
    }

    fn evaluate_comparison<'b>(
        &self,
        op: BinaryOperator,
        left: &'b Expression<'a>,
        right: &'b Expression<'a>,
    ) -> Tri {
        match op {
            BinaryOperator::Equality => self.try_abstract_equality_comparison(left, right),
            _ => Tri::Unknown,
        }
    }

    /// <https://tc39.es/ecma262/#sec-abstract-equality-comparison>
    fn try_abstract_equality_comparison<'b>(
        &self,
        left_expr: &'b Expression<'a>,
        right_expr: &'b Expression<'a>,
    ) -> Tri {
        let left = Ty::from(left_expr);
        let right = Ty::from(right_expr);
        if left != Ty::Undetermined && right != Ty::Undetermined {
            if left == right {
                return self.try_strict_equality_comparison(left_expr, right_expr);
            }
            if matches!((left, right), (Ty::Null, Ty::Void) | (Ty::Void, Ty::Null)) {
                return Tri::True;
            }
        }
        Tri::Unknown
    }

    /// <https://tc39.es/ecma262/#sec-strict-equality-comparison>
    fn try_strict_equality_comparison<'b>(
        &self,
        left_expr: &'b Expression<'a>,
        right_expr: &'b Expression<'a>,
    ) -> Tri {
        let left = Ty::from(left_expr);
        let right = Ty::from(right_expr);
        if left != Ty::Undetermined && right != Ty::Undetermined {
            if left != right {
                return Tri::False;
            }
            return match left {
                Ty::Void | Ty::Null => Tri::True,
                _ => Tri::Unknown,
            };
        }
        Tri::Unknown
    }

    /// Folds 'typeof(foo)' if foo is a literal, e.g.
    /// typeof("bar") --> "string"
    /// typeof(6) --> "number"
    fn try_fold_typeof<'b>(
        &mut self,
        span: Span,
        argument: &'b Expression<'a>,
    ) -> Option<Expression<'a>> {
        if argument.is_literal_value(true) {
            let type_name = match argument {
                Expression::FunctionExpression(_) | Expression::ArrowExpression(_) => {
                    Some("function")
                }
                Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => Some("string"),
                Expression::NumberLiteral(_) => Some("number"),
                Expression::BooleanLiteral(_) => Some("boolean"),
                Expression::NullLiteral(_)
                | Expression::ObjectExpression(_)
                | Expression::ArrayExpression(_) => Some("object"),
                Expression::Identifier(_) if argument.is_undefined() => Some("undefined"),
                Expression::UnaryExpression(unary_expr)
                    if unary_expr.operator == UnaryOperator::Void =>
                {
                    Some("undefined")
                }
                _ => None,
            };

            if let Some(type_name) = type_name {
                let string_literal = self.hir.string_literal(span, Atom::from(type_name));
                return Some(self.hir.literal_string_expression(string_literal));
            }
        }

        None
    }
}
