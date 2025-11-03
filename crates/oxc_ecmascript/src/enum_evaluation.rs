//! Enum constant value evaluation
//!
//! This module provides reusable logic for evaluating TypeScript enum member values.
//! It's used by both the TypeScript transformer and the semantic analyzer.
//!
//! Based on TypeScript's and Babel's enum transformation implementation.

use oxc_allocator::StringBuilder;
use oxc_ast::{
    AstBuilder,
    ast::{BinaryExpression, Expression, UnaryExpression, match_member_expression},
};
use oxc_span::Atom;
use oxc_syntax::{
    number::ToJsString,
    operator::{BinaryOperator, UnaryOperator},
};
use rustc_hash::FxHashMap;

use crate::{ToInt32, ToUint32};

/// Constant value for enum members during evaluation.
#[derive(Debug, Clone, Copy)]
pub enum ConstantValue<'a> {
    Number(f64),
    String(Atom<'a>),
}

/// Enum member values (or None if it can't be evaluated at build time) keyed by member names
pub type PrevMembers<'a> = FxHashMap<Atom<'a>, Option<ConstantValue<'a>>>;

/// Evaluator for enum constant values.
/// This is a port of TypeScript's enum value evaluation logic.
pub struct EnumEvaluator<'b, 'a: 'b> {
    ast: AstBuilder<'a>,
    /// Map of enum names to their members (for cross-enum references)
    enums: Option<&'b FxHashMap<Atom<'a>, PrevMembers<'a>>>,
}

impl<'b, 'a> EnumEvaluator<'b, 'a> {
    /// Create a new evaluator with access to all enum definitions (for cross-enum references)
    pub fn new_with_enums(
        ast: AstBuilder<'a>,
        enums: &'b FxHashMap<Atom<'a>, PrevMembers<'a>>,
    ) -> Self {
        Self { ast, enums: Some(enums) }
    }

    /// Create a new evaluator without cross-enum reference support
    pub fn new(ast: AstBuilder<'a>) -> Self {
        Self { ast, enums: None }
    }

    /// Evaluate the expression to a constant value.
    /// Refer to [babel](https://github.com/babel/babel/blob/610897a9a96c5e344e77ca9665df7613d2f88358/packages/babel-plugin-transform-typescript/src/enum.ts#L241C1-L394C2)
    pub fn computed_constant_value(
        &self,
        expr: &Expression<'a>,
        prev_members: &PrevMembers<'a>,
    ) -> Option<ConstantValue<'a>> {
        self.evaluate(expr, prev_members)
    }

    fn evaluate_ref(
        &self,
        expr: &Expression<'a>,
        prev_members: &PrevMembers<'a>,
    ) -> Option<ConstantValue<'a>> {
        match expr {
            match_member_expression!(Expression) => {
                let expr = expr.to_member_expression();
                let Expression::Identifier(ident) = expr.object() else { return None };

                // Look up in all enums if available (for cross-enum references)
                if let Some(enums) = self.enums {
                    let members = enums.get(&ident.name)?;
                    let property = expr.static_property_name()?;
                    *members.get(property)?
                } else {
                    None
                }
            }
            Expression::Identifier(ident) => {
                if ident.name == "Infinity" {
                    return Some(ConstantValue::Number(f64::INFINITY));
                } else if ident.name == "NaN" {
                    return Some(ConstantValue::Number(f64::NAN));
                }

                if let Some(value) = prev_members.get(&ident.name) {
                    return *value;
                }

                // TODO:
                // This is a bit tricky because we need to find the BindingIdentifier that corresponds to the identifier reference.
                // and then we may to evaluate the initializer of the BindingIdentifier.
                // finally, we can get the value of the identifier and call the `computed_constant_value` function.
                // See https://github.com/babel/babel/blob/610897a9a96c5e344e77ca9665df7613d2f88358/packages/babel-plugin-transform-typescript/src/enum.ts#L327-L329
                None
            }
            _ => None,
        }
    }

    fn evaluate(
        &self,
        expr: &Expression<'a>,
        prev_members: &PrevMembers<'a>,
    ) -> Option<ConstantValue<'a>> {
        match expr {
            Expression::Identifier(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::PrivateFieldExpression(_) => self.evaluate_ref(expr, prev_members),
            Expression::BinaryExpression(expr) => self.eval_binary_expression(expr, prev_members),
            Expression::UnaryExpression(expr) => self.eval_unary_expression(expr, prev_members),
            Expression::NumericLiteral(lit) => Some(ConstantValue::Number(lit.value)),
            Expression::StringLiteral(lit) => Some(ConstantValue::String(lit.value)),
            Expression::TemplateLiteral(lit) => {
                let value = if let Some(quasi) = lit.single_quasi() {
                    quasi
                } else {
                    let mut value = StringBuilder::new_in(self.ast.allocator);
                    for (quasi, expr) in lit.quasis.iter().zip(&lit.expressions) {
                        value.push_str(&quasi.value.cooked.unwrap_or(quasi.value.raw));
                        if let Some(ConstantValue::String(str)) = self.evaluate(expr, prev_members)
                        {
                            value.push_str(&str);
                        }
                    }
                    self.ast.atom(value.into_str())
                };
                Some(ConstantValue::String(value))
            }
            Expression::ParenthesizedExpression(expr) => {
                self.evaluate(&expr.expression, prev_members)
            }
            _ => None,
        }
    }

    fn eval_binary_expression(
        &self,
        expr: &BinaryExpression<'a>,
        prev_members: &PrevMembers<'a>,
    ) -> Option<ConstantValue<'a>> {
        let left = self.evaluate(&expr.left, prev_members)?;
        let right = self.evaluate(&expr.right, prev_members)?;

        if matches!(expr.operator, BinaryOperator::Addition)
            && (matches!(left, ConstantValue::String(_))
                || matches!(right, ConstantValue::String(_)))
        {
            let left_string = match left {
                ConstantValue::String(str) => str,
                ConstantValue::Number(v) => self.ast.atom(&v.to_js_string()),
            };

            let right_string = match right {
                ConstantValue::String(str) => str,
                ConstantValue::Number(v) => self.ast.atom(&v.to_js_string()),
            };

            return Some(ConstantValue::String(
                self.ast.atom_from_strs_array([&left_string, &right_string]),
            ));
        }

        let left = match left {
            ConstantValue::Number(v) => v,
            ConstantValue::String(_) => return None,
        };

        let right = match right {
            ConstantValue::Number(v) => v,
            ConstantValue::String(_) => return None,
        };

        match expr.operator {
            BinaryOperator::ShiftRight => Some(ConstantValue::Number(f64::from(
                left.to_int_32().wrapping_shr(right.to_uint_32()),
            ))),
            BinaryOperator::ShiftRightZeroFill => Some(ConstantValue::Number(f64::from(
                (left.to_uint_32()).wrapping_shr(right.to_uint_32()),
            ))),
            BinaryOperator::ShiftLeft => Some(ConstantValue::Number(f64::from(
                left.to_int_32().wrapping_shl(right.to_uint_32()),
            ))),
            BinaryOperator::BitwiseXOR => {
                Some(ConstantValue::Number(f64::from(left.to_int_32() ^ right.to_int_32())))
            }
            BinaryOperator::BitwiseOR => {
                Some(ConstantValue::Number(f64::from(left.to_int_32() | right.to_int_32())))
            }
            BinaryOperator::BitwiseAnd => {
                Some(ConstantValue::Number(f64::from(left.to_int_32() & right.to_int_32())))
            }
            BinaryOperator::Multiplication => Some(ConstantValue::Number(left * right)),
            BinaryOperator::Division => Some(ConstantValue::Number(left / right)),
            BinaryOperator::Addition => Some(ConstantValue::Number(left + right)),
            BinaryOperator::Subtraction => Some(ConstantValue::Number(left - right)),
            BinaryOperator::Remainder => Some(ConstantValue::Number(left % right)),
            BinaryOperator::Exponential => Some(ConstantValue::Number(left.powf(right))),
            _ => None,
        }
    }

    fn eval_unary_expression(
        &self,
        expr: &UnaryExpression<'a>,
        prev_members: &PrevMembers<'a>,
    ) -> Option<ConstantValue<'a>> {
        let value = self.evaluate(&expr.argument, prev_members)?;

        let value = match value {
            ConstantValue::Number(value) => value,
            ConstantValue::String(_) => {
                let value = if expr.operator == UnaryOperator::UnaryNegation {
                    ConstantValue::Number(f64::NAN)
                } else if expr.operator == UnaryOperator::BitwiseNot {
                    ConstantValue::Number(-1.0)
                } else {
                    value
                };
                return Some(value);
            }
        };

        match expr.operator {
            UnaryOperator::UnaryPlus => Some(ConstantValue::Number(value)),
            UnaryOperator::UnaryNegation => Some(ConstantValue::Number(-value)),
            UnaryOperator::BitwiseNot => Some(ConstantValue::Number(f64::from(!value.to_int_32()))),
            _ => None,
        }
    }
}
