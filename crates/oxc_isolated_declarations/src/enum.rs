use rustc_hash::FxHashMap;

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_ecmascript::ToInt32;
use oxc_span::{Atom, GetSpan, SPAN};
use oxc_syntax::{
    number::{NumberBase, ToJsString},
    operator::{BinaryOperator, UnaryOperator},
};

use crate::{diagnostics::enum_member_initializers, IsolatedDeclarations};

#[derive(Debug, Clone)]
enum ConstantValue {
    Number(f64),
    String(String),
}

impl<'a> IsolatedDeclarations<'a> {
    /// # Panics
    /// if the enum member is a template literal with substitutions.
    pub fn transform_ts_enum_declaration(
        &mut self,
        decl: &TSEnumDeclaration<'a>,
    ) -> Option<Declaration<'a>> {
        let mut members = self.ast.vec();
        let mut prev_initializer_value = Some(ConstantValue::Number(-1.0));
        let mut prev_members = FxHashMap::default();
        for member in &decl.members {
            let value = if let Some(initializer) = &member.initializer {
                let computed_value =
                    self.computed_constant_value(initializer, &decl.id.name, &prev_members);

                if computed_value.is_none() {
                    self.error(enum_member_initializers(member.id.span()));
                }

                computed_value
            } else if let Some(ConstantValue::Number(v)) = prev_initializer_value {
                Some(ConstantValue::Number(v + 1.0))
            } else {
                None
            };

            prev_initializer_value.clone_from(&value);

            if let Some(value) = &value {
                let member_name = match &member.id {
                    TSEnumMemberName::StaticIdentifier(id) => &id.name,
                    TSEnumMemberName::StaticStringLiteral(str) => &str.value,
                    TSEnumMemberName::StaticTemplateLiteral(template) => {
                        &template.quasi().expect("Template enum members cannot have substitutions.")
                    }
                    #[allow(clippy::unnested_or_patterns)] // Clippy is wrong
                    TSEnumMemberName::StaticNumericLiteral(_)
                    | match_expression!(TSEnumMemberName) => {
                        unreachable!()
                    }
                };
                prev_members.insert(member_name.clone(), value.clone());
            }

            let member = self.ast.ts_enum_member(
                member.span,
                // SAFETY: `ast.copy` is unsound! We need to fix.
                unsafe { self.ast.copy(&member.id) },
                value.map(|v| match v {
                    ConstantValue::Number(v) => {
                        let is_negative = v < 0.0;

                        // Infinity
                        let expr = if v.is_infinite() {
                            self.ast.expression_identifier_reference(SPAN, "Infinity")
                        } else {
                            let value = if is_negative { -v } else { v };
                            self.ast.expression_numeric_literal(
                                SPAN,
                                value,
                                value.to_string(),
                                NumberBase::Decimal,
                            )
                        };

                        if is_negative {
                            self.ast.expression_unary(SPAN, UnaryOperator::UnaryNegation, expr)
                        } else {
                            expr
                        }
                    }
                    ConstantValue::String(v) => self.ast.expression_string_literal(SPAN, v),
                }),
            );

            members.push(member);
        }

        Some(self.ast.declaration_ts_enum(
            decl.span,
            // SAFETY: `ast.copy` is unsound! We need to fix.
            unsafe { self.ast.copy(&decl.id) },
            members,
            decl.r#const,
            self.is_declare(),
        ))
    }

    /// Evaluate the expression to a constant value.
    /// Refer to [babel](https://github.com/babel/babel/blob/610897a9a96c5e344e77ca9665df7613d2f88358/packages/babel-plugin-transform-typescript/src/enum.ts#L241C1-L394C2)
    fn computed_constant_value(
        &self,
        expr: &Expression<'a>,
        enum_name: &str,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        self.evaluate(expr, enum_name, prev_members)
    }

    #[allow(clippy::unused_self, clippy::needless_pass_by_value)]
    fn evaluate_ref(
        &self,
        expr: &Expression<'a>,
        enum_name: &str,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        match expr {
            match_member_expression!(Expression) => {
                let expr = expr.to_member_expression();
                let Expression::Identifier(ident) = expr.object() else { return None };
                if ident.name == enum_name {
                    let property = expr.static_property_name()?;
                    prev_members.get(property).cloned()
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
                    return Some(value.clone());
                }

                None
            }
            _ => None,
        }
    }

    fn evaluate(
        &self,
        expr: &Expression<'a>,
        enum_name: &str,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        match expr {
            Expression::Identifier(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::PrivateFieldExpression(_) => {
                self.evaluate_ref(expr, enum_name, prev_members)
            }
            Expression::BinaryExpression(expr) => {
                self.eval_binary_expression(expr, enum_name, prev_members)
            }
            Expression::UnaryExpression(expr) => {
                self.eval_unary_expression(expr, enum_name, prev_members)
            }
            Expression::NumericLiteral(lit) => Some(ConstantValue::Number(lit.value)),
            Expression::StringLiteral(lit) => Some(ConstantValue::String(lit.value.to_string())),
            Expression::TemplateLiteral(lit) => {
                let mut value = String::new();
                for part in &lit.quasis {
                    value.push_str(&part.value.raw);
                }
                Some(ConstantValue::String(value))
            }
            Expression::ParenthesizedExpression(expr) => {
                self.evaluate(&expr.expression, enum_name, prev_members)
            }
            _ => None,
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss)]
    fn eval_binary_expression(
        &self,
        expr: &BinaryExpression<'a>,
        enum_name: &str,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        let left = self.evaluate(&expr.left, enum_name, prev_members)?;
        let right = self.evaluate(&expr.right, enum_name, prev_members)?;

        if matches!(expr.operator, BinaryOperator::Addition)
            && (matches!(left, ConstantValue::String(_))
                || matches!(right, ConstantValue::String(_)))
        {
            let left_string = match left {
                ConstantValue::String(str) => str,
                ConstantValue::Number(v) => v.to_js_string(),
            };

            let right_string = match right {
                ConstantValue::String(str) => str,
                ConstantValue::Number(v) => v.to_js_string(),
            };

            return Some(ConstantValue::String(format!("{left_string}{right_string}")));
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
                left.to_int_32().wrapping_shr(right.to_int_32() as u32),
            ))),
            BinaryOperator::ShiftRightZeroFill => Some(ConstantValue::Number(f64::from(
                (left.to_int_32() as u32).wrapping_shr(right.to_int_32() as u32),
            ))),
            BinaryOperator::ShiftLeft => Some(ConstantValue::Number(f64::from(
                left.to_int_32().wrapping_shl(right.to_int_32() as u32),
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

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn eval_unary_expression(
        &self,
        expr: &UnaryExpression<'a>,
        enum_name: &str,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        let value = self.evaluate(&expr.argument, enum_name, prev_members)?;

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
