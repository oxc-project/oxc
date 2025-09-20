use oxc_ast::ast::*;

use crate::{
    constant_evaluation::DetermineValueType, to_numeric::ToNumeric,
    to_primitive::ToPrimitive,
};

use super::super::{MayHaveSideEffects, MayHaveSideEffectsContext};

impl<'a> MayHaveSideEffects<'a> for BinaryExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self.operator {
            BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan => {
                self.left.may_have_side_effects(ctx) || self.right.may_have_side_effects(ctx)
            }
            BinaryOperator::Instanceof => {
                // When the following conditions are met, instanceof won't throw `TypeError`.
                // - the right hand side is a known global reference which is a function
                // - the left hand side is not a proxy
                if let Expression::Identifier(right_ident) = &self.right {
                    let name = right_ident.name.as_str();
                    // Any known global non-constructor functions can be allowed here.
                    // But because non-constructor functions are not likely to be used, we ignore them.
                    if is_known_global_constructor(name)
                        && ctx.is_global_reference(right_ident)
                        && !self.left.value_type(ctx).is_undetermined()
                    {
                        return false;
                    }
                }
                // instanceof can throw `TypeError`
                true
            }
            BinaryOperator::In => {
                // in can throw `TypeError`
                true
            }
            BinaryOperator::Addition => {
                let left = self.left.to_primitive(ctx);
                let right = self.right.to_primitive(ctx);
                if left.is_string() == Some(true) || right.is_string() == Some(true) {
                    // If either side is a string, ToString is called for both sides.
                    let other_side = if left.is_string() == Some(true) { right } else { left };
                    // ToString() for Symbols throws an error.
                    return other_side.is_symbol() != Some(false)
                        || self.left.may_have_side_effects(ctx)
                        || self.right.may_have_side_effects(ctx);
                }

                let left_to_numeric_type = left.to_numeric(ctx);
                let right_to_numeric_type = right.to_numeric(ctx);
                if (left_to_numeric_type.is_number() && right_to_numeric_type.is_number())
                    || (left_to_numeric_type.is_bigint() && right_to_numeric_type.is_bigint())
                {
                    self.left.may_have_side_effects(ctx) || self.right.may_have_side_effects(ctx)
                } else {
                    true
                }
            }
            BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::ShiftLeft
            | BinaryOperator::BitwiseOR
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::Exponential
            | BinaryOperator::ShiftRightZeroFill => {
                let left_to_numeric_type = self.left.to_numeric(ctx);
                let right_to_numeric_type = self.right.to_numeric(ctx);
                if left_to_numeric_type.is_bigint() && right_to_numeric_type.is_bigint() {
                    if self.operator == BinaryOperator::ShiftRightZeroFill {
                        true
                    } else if matches!(
                        self.operator,
                        BinaryOperator::Exponential
                            | BinaryOperator::Division
                            | BinaryOperator::Remainder
                    ) {
                        if let Expression::BigIntLiteral(right) = &self.right {
                            match self.operator {
                                BinaryOperator::Exponential => {
                                    right.is_negative() || self.left.may_have_side_effects(ctx)
                                }
                                BinaryOperator::Division | BinaryOperator::Remainder => {
                                    right.is_zero() || self.left.may_have_side_effects(ctx)
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            true
                        }
                    } else {
                        self.left.may_have_side_effects(ctx)
                            || self.right.may_have_side_effects(ctx)
                    }
                } else if left_to_numeric_type.is_number() && right_to_numeric_type.is_number() {
                    self.left.may_have_side_effects(ctx) || self.right.may_have_side_effects(ctx)
                } else {
                    true
                }
            }
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for LogicalExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        if self.left.may_have_side_effects(ctx) {
            return true;
        }
        match self.operator {
            LogicalOperator::And => {
                // Pattern: typeof x !== 'undefined' && x
                if is_side_effect_free_unbound_identifier_ref(&self.right, &self.left, true, ctx) {
                    return false;
                }
            }
            LogicalOperator::Or => {
                // Pattern: typeof x === 'undefined' || x
                if is_side_effect_free_unbound_identifier_ref(&self.right, &self.left, false, ctx) {
                    return false;
                }
            }
            LogicalOperator::Coalesce => {}
        }
        self.right.may_have_side_effects(ctx)
    }
}

/// Whether the name matches any known global constructors.
///
/// <https://tc39.es/ecma262/multipage/global-object.html#sec-constructor-properties-of-the-global-object>
pub fn is_known_global_constructor(name: &str) -> bool {
    // technically, we need to exclude the constructors that are not supported by the target
    matches!(
        name,
        "AggregateError"
            | "Array"
            | "ArrayBuffer"
            | "BigInt"
            | "BigInt64Array"
            | "BitUint64Array"
            | "Boolean"
            | "DataView"
            | "Date"
            | "Error"
            | "EvalError"
            | "FinalizationRegistry"
            | "Float32Array"
            | "Float64Array"
            | "Function"
            | "Int8Array"
            | "Int16Array"
            | "Int32Array"
            | "Iterator"
            | "Map"
            | "Number"
            | "Object"
            | "Promise"
            | "Proxy"
            | "RangeError"
            | "ReferenceError"
            | "RegExp"
            | "Set"
            | "SharedArrayBuffer"
            | "String"
            | "Symbol"
            | "SyntaxError"
            | "TypeError"
            | "Uint8Array"
            | "Uint8ClampedArray"
            | "Uint16Array"
            | "Uint32Array"
            | "URIError"
            | "WeakMap"
            | "WeakSet"
    )
}

/// Check if the expression matches the pattern of typeof-guarded global access
/// that is side-effect free. This helps identify patterns like:
/// - `typeof x !== 'undefined' && x` (expectUndefined = true)
/// - `typeof x === 'undefined' || x` (expectUndefined = false)
pub fn is_side_effect_free_unbound_identifier_ref<'a>(
    test_expr: &Expression<'a>,
    condition_expr: &Expression<'a>,
    expect_undefined: bool,
    ctx: &impl MayHaveSideEffectsContext<'a>,
) -> bool {
    if let Expression::Identifier(test_ident) = test_expr {
        if !ctx.is_global_reference(test_ident) {
            return false;
        }

        // Check if condition_expr matches the pattern
        if let Expression::BinaryExpression(bin_expr) = condition_expr {
            // Check for typeof comparison patterns
            if let Expression::UnaryExpression(unary) = &bin_expr.left {
                if unary.operator == UnaryOperator::Typeof {
                    if let Expression::Identifier(typeof_ident) = &unary.argument {
                        if typeof_ident.name == test_ident.name
                            && ctx.is_global_reference(typeof_ident)
                        {
                            // Check if it's comparing to 'undefined'
                            if let Expression::StringLiteral(str_lit) = &bin_expr.right {
                                if str_lit.value == "undefined" {
                                    // Match the operator with expected pattern
                                    return match bin_expr.operator {
                                        BinaryOperator::StrictInequality | BinaryOperator::Inequality => expect_undefined,
                                        BinaryOperator::StrictEquality | BinaryOperator::Equality => !expect_undefined,
                                        _ => false,
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}