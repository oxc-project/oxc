//! TypeScript Enum Constant Evaluation
//!
//! Provides constant evaluation for TypeScript enum members.
//! Based on Babel's implementation: <https://github.com/babel/babel/blob/610897a9a96c5e344e77ca9665df7613d2f88358/packages/babel-plugin-transform-typescript/src/enum.ts#L241C1-L394C2>

use oxc_allocator::StringBuilder;
use oxc_ast::{AstBuilder, ast::*};
use oxc_span::Atom;
use oxc_syntax::{
    number::ToJsString,
    operator::{BinaryOperator, UnaryOperator},
};
use rustc_hash::FxHashMap;

use crate::{ToInt32, ToUint32};

/// Enum member values (or None if it can't be evaluated at build time) keyed by names
pub type EnumMembers<'a> = FxHashMap<Atom<'a>, Option<ConstantEnumValue<'a>>>;

/// TypeScript enum constant evaluator
pub struct EnumConstantEvaluator<'a> {
    /// Map of enum names to their members
    enums: FxHashMap<Atom<'a>, EnumMembers<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub enum ConstantEnumValue<'a> {
    Number(f64),
    String(Atom<'a>),
}

impl<'a> EnumConstantEvaluator<'a> {
    pub fn new() -> Self {
        Self { enums: FxHashMap::default() }
    }


    /// Get mutable access to the enums map
    pub fn enums_mut(&mut self) -> &mut FxHashMap<Atom<'a>, EnumMembers<'a>> {
        &mut self.enums
    }

    /// Evaluate the expression to a constant value.
    /// Refer to [babel](https://github.com/babel/babel/blob/610897a9a96c5e344e77ca9665df7613d2f88358/packages/babel-plugin-transform-typescript/src/enum.ts#L241C1-L394C2)
    pub fn computed_constant_value(
        &self,
        expr: &Expression<'a>,
        prev_members: &EnumMembers<'a>,
        ast: AstBuilder<'a>,
    ) -> Option<ConstantEnumValue<'a>> {
        self.evaluate(expr, prev_members, ast)
    }

    fn evaluate_ref(
        &self,
        expr: &Expression<'a>,
        prev_members: &EnumMembers<'a>,
    ) -> Option<ConstantEnumValue<'a>> {
        match expr {
            match_member_expression!(Expression) => {
                let expr = expr.to_member_expression();
                let Expression::Identifier(ident) = expr.object() else { return None };
                let members = self.enums.get(&ident.name)?;
                let property = expr.static_property_name()?;
                *members.get(property)?
            }
            Expression::Identifier(ident) => {
                if ident.name == "Infinity" {
                    return Some(ConstantEnumValue::Number(f64::INFINITY));
                } else if ident.name == "NaN" {
                    return Some(ConstantEnumValue::Number(f64::NAN));
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
        prev_members: &EnumMembers<'a>,
        ast: AstBuilder<'a>,
    ) -> Option<ConstantEnumValue<'a>> {
        match expr {
            Expression::Identifier(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::PrivateFieldExpression(_) => self.evaluate_ref(expr, prev_members),
            Expression::BinaryExpression(expr) => {
                self.eval_binary_expression(expr, prev_members, ast)
            }
            Expression::UnaryExpression(expr) => {
                self.eval_unary_expression(expr, prev_members, ast)
            }
            Expression::NumericLiteral(lit) => Some(ConstantEnumValue::Number(lit.value)),
            Expression::StringLiteral(lit) => Some(ConstantEnumValue::String(lit.value)),
            Expression::TemplateLiteral(lit) => {
                let value = if let Some(quasi) = lit.single_quasi() {
                    quasi
                } else {
                    let mut value = StringBuilder::new_in(ast.allocator);
                    for (i, quasi) in lit.quasis.iter().enumerate() {
                        value.push_str(&quasi.value.cooked.unwrap_or(quasi.value.raw));
                        if i < lit.expressions.len() {
                            match self.evaluate(&lit.expressions[i], prev_members, ast)? {
                                ConstantEnumValue::String(str) => value.push_str(&str),
                                ConstantEnumValue::Number(num) => {
                                    value.push_str(&num.to_js_string());
                                }
                            }
                        }
                    }
                    Atom::from(value.into_str())
                };
                Some(ConstantEnumValue::String(value))
            }
            Expression::ParenthesizedExpression(expr) => {
                self.evaluate(&expr.expression, prev_members, ast)
            }
            _ => None,
        }
    }

    fn eval_binary_expression(
        &self,
        expr: &BinaryExpression<'a>,
        prev_members: &EnumMembers<'a>,
        ast: AstBuilder<'a>,
    ) -> Option<ConstantEnumValue<'a>> {
        let left = self.evaluate(&expr.left, prev_members, ast)?;
        let right = self.evaluate(&expr.right, prev_members, ast)?;

        if matches!(expr.operator, BinaryOperator::Addition)
            && (matches!(left, ConstantEnumValue::String(_))
                || matches!(right, ConstantEnumValue::String(_)))
        {
            let left_string = match left {
                ConstantEnumValue::String(str) => str,
                ConstantEnumValue::Number(v) => ast.atom(&v.to_js_string()),
            };

            let right_string = match right {
                ConstantEnumValue::String(str) => str,
                ConstantEnumValue::Number(v) => ast.atom(&v.to_js_string()),
            };

            return Some(ConstantEnumValue::String(
                ast.atom_from_strs_array([&left_string, &right_string]),
            ));
        }

        let ConstantEnumValue::Number(left) = left else { return None };

        let ConstantEnumValue::Number(right) = right else { return None };

        match expr.operator {
            BinaryOperator::ShiftRight => Some(ConstantEnumValue::Number(f64::from(
                left.to_int_32().wrapping_shr(right.to_uint_32()),
            ))),
            BinaryOperator::ShiftRightZeroFill => Some(ConstantEnumValue::Number(f64::from(
                (left.to_uint_32()).wrapping_shr(right.to_uint_32()),
            ))),
            BinaryOperator::ShiftLeft => Some(ConstantEnumValue::Number(f64::from(
                left.to_int_32().wrapping_shl(right.to_uint_32()),
            ))),
            BinaryOperator::BitwiseXOR => {
                Some(ConstantEnumValue::Number(f64::from(left.to_int_32() ^ right.to_int_32())))
            }
            BinaryOperator::BitwiseOR => {
                Some(ConstantEnumValue::Number(f64::from(left.to_int_32() | right.to_int_32())))
            }
            BinaryOperator::BitwiseAnd => {
                Some(ConstantEnumValue::Number(f64::from(left.to_int_32() & right.to_int_32())))
            }
            BinaryOperator::Multiplication => Some(ConstantEnumValue::Number(left * right)),
            BinaryOperator::Division => Some(ConstantEnumValue::Number(left / right)),
            BinaryOperator::Addition => Some(ConstantEnumValue::Number(left + right)),
            BinaryOperator::Subtraction => Some(ConstantEnumValue::Number(left - right)),
            BinaryOperator::Remainder => Some(ConstantEnumValue::Number(left % right)),
            BinaryOperator::Exponential => Some(ConstantEnumValue::Number(left.powf(right))),
            _ => None,
        }
    }

    fn eval_unary_expression(
        &self,
        expr: &UnaryExpression<'a>,
        prev_members: &EnumMembers<'a>,
        ast: AstBuilder<'a>,
    ) -> Option<ConstantEnumValue<'a>> {
        let value = self.evaluate(&expr.argument, prev_members, ast)?;

        let value = match value {
            ConstantEnumValue::Number(value) => value,
            ConstantEnumValue::String(_) => {
                let value = if expr.operator == UnaryOperator::UnaryNegation {
                    ConstantEnumValue::Number(f64::NAN)
                } else if expr.operator == UnaryOperator::BitwiseNot {
                    ConstantEnumValue::Number(-1.0)
                } else {
                    value
                };
                return Some(value);
            }
        };

        match expr.operator {
            UnaryOperator::UnaryPlus => Some(ConstantEnumValue::Number(value)),
            UnaryOperator::UnaryNegation => Some(ConstantEnumValue::Number(-value)),
            UnaryOperator::BitwiseNot => {
                Some(ConstantEnumValue::Number(f64::from(!value.to_int_32())))
            }
            _ => None,
        }
    }
}

impl Default for EnumConstantEvaluator<'_> {
    fn default() -> Self {
        Self::new()
    }
}
