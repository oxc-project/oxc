//! Constant Folding
//!
//! <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeFoldConstants.java>

use oxc_ast::BigUint;
#[allow(clippy::wildcard_imports)]
use oxc_hir::hir::*;
use oxc_hir::hir_util::{IsLiteralValue, MayHaveSideEffects};
use oxc_span::{Atom, Span};
use oxc_syntax::{operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator}, NumberBase};

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
        let has_side_effects = expr.may_have_side_effects();

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
                UnaryOperator::UnaryPlus
                | UnaryOperator::UnaryNegation
                | UnaryOperator::LogicalNot
                | UnaryOperator::BitwiseNot
                    if !has_side_effects =>
                {
                    self.try_fold_unary_operator(unary_expr)
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

    fn try_fold_unary_operator<'b>(
        &mut self,
        unary_expr: &'b mut UnaryExpression<'a>,
    ) -> Option<Expression<'a>> {
        // TODO: I want compress children frist, so we can fold expression like `- - 4`
        // But our ast will vistor more than once, is there better way?
        self.fold_expression(&mut unary_expr.argument);

        let tri_kind = get_boolean_value(&unary_expr.argument);

        if tri_kind == Tri::Unknown {
            return None;
        }

        match unary_expr.operator {
            UnaryOperator::LogicalNot => {
                if let Expression::NumberLiteral(number_literal) = &unary_expr.argument {
                    let value = number_literal.value;
                    if value == 0.0 || value == 1.0 {
                        None
                    } else {
                        // Tri::Unknown has been already filtered out
                        let inverted_boolean = if tri_kind == Tri::True { false } else { true };
                        let bool_literal =
                            self.hir.boolean_literal(unary_expr.span, inverted_boolean);
                        Some(self.hir.literal_boolean_expression(bool_literal))
                    }
                } else {
                    None
                }
            }
            UnaryOperator::UnaryPlus => match &unary_expr.argument {
                Expression::NumberLiteral(number_literal) => {
                    let number_literal = self.hir.number_literal(
                        unary_expr.span,
                        number_literal.value,
                        number_literal.raw,
                        number_literal.base,
                    );
                    Some(self.hir.literal_number_expression(number_literal))
                },
                Expression::Identifier(ident) => {
                    if matches!(ident.name.as_str(), "NaN" | "Infinity") {
                        self.try_detach_unary_op(unary_expr)
                    } else {
                        None
                    }
                }, 
                _ => {
                    if let Some(value) = get_number_value(&unary_expr.argument) {
                        let raw = self.hir.new_str(value.to_string().as_str());
                        let number_literal = self.hir.number_literal(
                            unary_expr.span,
                            value,
                            raw,
                            NumberBase::Decimal,
                        );
                        Some(self.hir.literal_number_expression(number_literal))
                    } else {
                        None
                    }
                },
                
            }
            UnaryOperator::UnaryNegation => match &unary_expr.argument {
                Expression::NumberLiteral(number_literal) => {
                    let number_literal = self.hir.number_literal(
                        unary_expr.span,
                        -number_literal.value,
                        number_literal.raw,
                        number_literal.base,
                    );
                    Some(self.hir.literal_number_expression(number_literal))
                }
                Expression::Identifier(ident) => {
                    if ident.name == "NaN" {
                        self.try_detach_unary_op(unary_expr)
                    } else {
                        None
                    }
                }, 
                _ => None,
            },
            UnaryOperator::BitwiseNot => {
                if let Expression::NumberLiteral(number_literal) = &unary_expr.argument {
                    let value = number_literal.value;
                    if value.fract() == 0.0 {
                        let int_value = ecmascript_to_int32(value);
                        let number_literal = self.hir.number_literal(
                            unary_expr.span,
                            (!int_value) as f64,
                            number_literal.raw,
                            number_literal.base,
                        );
                        Some(self.hir.literal_number_expression(number_literal))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    fn try_detach_unary_op(&mut self, unary_expr: & mut UnaryExpression<'a>) -> Option<Expression<'a>> {
        match &unary_expr.argument {
            Expression::Identifier(ident) => {
                if matches!(ident.name.as_str(), "NaN" | "Infinity") {
                    let ident = self.hir.identifier_reference(
                        unary_expr.span,
                        ident.name.clone(),
                        ident.reference_id,
                    );
                    Some(self.hir.identifier_reference_expression(ident))
                } else {
                    None
                }
            },
            _ => None
        }
        
    }
}

fn get_number_value(expr: &Expression) -> Option<f64> {
    match expr {
        Expression::NumberLiteral(number_literal) => Some(number_literal.value),
        Expression::UnaryExpression(unary_expr) => {
            match unary_expr.operator {
                UnaryOperator::UnaryPlus => get_number_value(&unary_expr.argument),
                UnaryOperator::UnaryNegation => get_number_value(&unary_expr.argument).map(|v| -v),
                UnaryOperator::BitwiseNot => {
                    if let Some(value) = get_number_value(&unary_expr.argument) {
                        Some(!ecmascript_to_int32(value) as f64)
                    } else {
                        None
                    }
                },
                UnaryOperator::LogicalNot => {
                    match get_boolean_value(expr) {
                        Tri::True => Some(1.0),
                        Tri::False => Some(0.0),
                        Tri::Unknown => None,
                    }
                }, 
                _ => None
            }
        }
        Expression::BooleanLiteral(bool_literal) => {
            if bool_literal.value {
                Some(1.0)
            } else {
                Some(0.0)
            }
        }
        Expression::NullLiteral(_) => Some(0.0),
        _ => None,
    }
}

/// code port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/base/JSCompDoubles.java#L113)
/// <https://262.ecma-international.org/5.1/#sec-9.5>
fn ecmascript_to_int32(num: f64) -> i32 {
    let int32_value = num as i32;
    if int32_value as f64 == num {
        return int32_value;
    }

    let pos_int = num.signum() * num.abs().floor();
    let int32bit = pos_int % 2f64.powi(32);

    if int32bit >= 2f64.powi(31) {
        return (int32bit - 2f64.powi(32)) as i32;
    } else {
        return int32bit as i32;
    }
}

fn get_boolean_value<'a, 'b>(expr: &'b Expression<'a>) -> Tri {
    let be_tri_boolean = |boolean: bool| {
        if boolean { Tri::True } else { Tri::False }
    };

    match expr {
        Expression::BooleanLiteral(boolean_literal) => {
            be_tri_boolean(boolean_literal.value)
        },
        Expression::NullLiteral(_) => Tri::False,
        Expression::NumberLiteral(number_literal) => {
            be_tri_boolean(number_literal.value != 0.0)
        },
        Expression::BigintLiteral(big_int_literal) => be_tri_boolean(big_int_literal.value == BigUint::default()),
        Expression::RegExpLiteral(_) => Tri::True,
        Expression::StringLiteral(string_literal) => be_tri_boolean(!string_literal.value.is_empty()),
        Expression::TemplateLiteral(template_literal) => {
            if let Some(quasi) = template_literal.quasis.get(0) && quasi.tail {
                if quasi.value.cooked.as_ref().map_or(false, |cooked| !cooked.is_empty()) {
                    Tri::True
                } else {
                    Tri::False
                }
            } else {
                Tri::Unknown
            }
        },
        Expression::Identifier(ident) => {
            if expr.is_undefined() {
                Tri::False
            } else if  ident.name == "Infinity" {
                Tri::True
            } else if ident.name == "NaN" {
                Tri::False
            } else {
                Tri::Unknown
            }
        }, 
        Expression::ArrayExpression(_) => Tri::True,
        Expression::ArrowExpression(_) => Tri::True,
        Expression::AssignmentExpression(assign_expr) => {
            match assign_expr.operator {
                // TODO: Is there possible to be true or fase ?
                // https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/NodeUtil.java#L186 
                AssignmentOperator::LogicalAnd | AssignmentOperator::LogicalOr => {
                    Tri::Unknown
                }
                // For ASSIGN, the value is the value of the RHS.
                _ =>  get_boolean_value(&assign_expr.right)
            }
        },
        Expression::ClassExpression(_) => Tri::True,
        Expression::FunctionExpression(_) => Tri::True,
        Expression::LogicalExpression(logical_expr) => {
            match logical_expr.operator {
                LogicalOperator::And => {
                    if get_boolean_value(&logical_expr.left) == Tri::True && get_boolean_value(&logical_expr.right) == Tri:: True {
                        Tri::True
                    } else {
                        Tri::False
                    }
                },
                LogicalOperator::Or => {
                    if get_boolean_value(&logical_expr.left) == Tri::True || get_boolean_value(&logical_expr.right) == Tri:: True {
                        Tri::True
                    } else {
                        Tri::False
                    }
                },
                _ => Tri::Unknown
            }
        },
        Expression::NewExpression(_) => Tri::True,
        Expression::ObjectExpression(_) => Tri::True,
        Expression::SequenceExpression(sequence_expr) => {
            // For sequence expression, the value is the value of the RHS.
            if let Some(expr) = sequence_expr.expressions.last() {
                get_boolean_value(expr)
            } else {
                Tri::Unknown
            }
        },
        Expression::UnaryExpression(unary_expr) => {
            if unary_expr.operator == UnaryOperator::Void {
                Tri::False
            } else if matches!(unary_expr.operator, UnaryOperator::BitwiseNot | UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation) {
                match &unary_expr.argument {
                   Expression::NumberLiteral(number_literal) => be_tri_boolean(number_literal.value != 0.0),
                   Expression::BigintLiteral(big_int_literal) => be_tri_boolean(big_int_literal.value == BigUint::default()),
                   _ => Tri::Unknown
                }
            } else if unary_expr.operator == UnaryOperator::LogicalNot {
                match get_boolean_value(&unary_expr.argument) {
                    Tri::True => Tri::False,
                    Tri::False => Tri::True,
                    Tri::Unknown => Tri::Unknown
                }
            } else {
                Tri::Unknown
            }
        },
        _ => Tri::Unknown

    }
}
