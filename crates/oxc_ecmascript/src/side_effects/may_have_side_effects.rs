use oxc_ast::ast::*;

use crate::{
    is_global_reference::IsGlobalReference, to_numeric::ToNumeric, to_primitive::ToPrimitive,
};

/// Returns true if subtree changes application state.
///
/// This trait assumes the following:
/// - `.toString()`, `.valueOf()`, and `[Symbol.toPrimitive]()` are side-effect free.
///   - This is mainly to assume `ToPrimitive` is side-effect free.
///   - Note that the builtin `Array::toString` has a side-effect when a value contains a Symbol as `ToString(Symbol)` throws an error. Maybe we should revisit this assumption and remove it.
///     - For example, `"" == [Symbol()]` returns an error, but this trait returns `false`.
/// - Errors thrown when creating a String or an Array that exceeds the maximum length does not happen.
/// - TDZ errors does not happen.
///
/// Ported from [closure-compiler](https://github.com/google/closure-compiler/blob/f3ce5ed8b630428e311fe9aa2e20d36560d975e2/src/com/google/javascript/jscomp/AstAnalyzer.java#L94)
pub trait MayHaveSideEffects {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool;
}

impl MayHaveSideEffects for Expression<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        match self {
            Expression::Identifier(ident) => ident.may_have_side_effects(is_global_reference),
            Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::MetaProperty(_)
            | Expression::ThisExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::Super(_) => false,
            Expression::TemplateLiteral(template) => {
                template.expressions.iter().any(|e| e.may_have_side_effects(is_global_reference))
            }
            Expression::UnaryExpression(e) => e.may_have_side_effects(is_global_reference),
            Expression::LogicalExpression(e) => e.may_have_side_effects(is_global_reference),
            Expression::ParenthesizedExpression(e) => {
                e.expression.may_have_side_effects(is_global_reference)
            }
            Expression::ConditionalExpression(e) => {
                e.test.may_have_side_effects(is_global_reference)
                    || e.consequent.may_have_side_effects(is_global_reference)
                    || e.alternate.may_have_side_effects(is_global_reference)
            }
            Expression::SequenceExpression(e) => {
                e.expressions.iter().any(|e| e.may_have_side_effects(is_global_reference))
            }
            Expression::BinaryExpression(e) => e.may_have_side_effects(is_global_reference),
            Expression::ObjectExpression(object_expr) => object_expr
                .properties
                .iter()
                .any(|property| property.may_have_side_effects(is_global_reference)),
            Expression::ArrayExpression(e) => e.may_have_side_effects(is_global_reference),
            Expression::ClassExpression(e) => e.may_have_side_effects(is_global_reference),
            // NOTE: private in can throw `TypeError`
            Expression::StaticMemberExpression(e) => e.may_have_side_effects(is_global_reference),
            Expression::ComputedMemberExpression(e) => e.may_have_side_effects(is_global_reference),
            _ => true,
        }
    }
}

impl MayHaveSideEffects for IdentifierReference<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        match self.name.as_str() {
            "NaN" | "Infinity" | "undefined" => false,
            // Reading global variables may have a side effect.
            // NOTE: It should also return true when the reference might refer to a reference value created by a with statement
            // NOTE: we ignore TDZ errors
            _ => is_global_reference.is_global_reference(self) != Some(false),
        }
    }
}

impl MayHaveSideEffects for UnaryExpression<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        match self.operator {
            UnaryOperator::Delete => true,
            UnaryOperator::Void | UnaryOperator::LogicalNot => {
                self.argument.may_have_side_effects(is_global_reference)
            }
            UnaryOperator::Typeof => {
                if matches!(&self.argument, Expression::Identifier(_)) {
                    false
                } else {
                    self.argument.may_have_side_effects(is_global_reference)
                }
            }
            UnaryOperator::UnaryPlus => {
                // ToNumber throws an error when the argument is Symbol / BigInt / an object that
                // returns Symbol or BigInt from ToPrimitive
                self.argument.to_primitive(is_global_reference).is_symbol_or_bigint() != Some(false)
                    || self.argument.may_have_side_effects(is_global_reference)
            }
            UnaryOperator::UnaryNegation | UnaryOperator::BitwiseNot => {
                // ToNumeric throws an error when the argument is Symbol / an object that
                // returns Symbol from ToPrimitive
                self.argument.to_primitive(is_global_reference).is_symbol() != Some(false)
                    || self.argument.may_have_side_effects(is_global_reference)
            }
        }
    }
}

impl MayHaveSideEffects for BinaryExpression<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        match self.operator {
            BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan => {
                self.left.may_have_side_effects(is_global_reference)
                    || self.right.may_have_side_effects(is_global_reference)
            }
            BinaryOperator::In | BinaryOperator::Instanceof => {
                // instanceof and in can throw `TypeError`
                true
            }
            BinaryOperator::Addition => {
                let left = self.left.to_primitive(is_global_reference);
                let right = self.right.to_primitive(is_global_reference);
                if left.is_string() == Some(true) || right.is_string() == Some(true) {
                    // If either side is a string, ToString is called for both sides.
                    let other_side = if left.is_string() == Some(true) { right } else { left };
                    // ToString() for Symbols throws an error.
                    return other_side.is_symbol() != Some(false)
                        || self.left.may_have_side_effects(is_global_reference)
                        || self.right.may_have_side_effects(is_global_reference);
                }

                let left_to_numeric_type = left.to_numeric(is_global_reference);
                let right_to_numeric_type = right.to_numeric(is_global_reference);
                if (left_to_numeric_type.is_number() && right_to_numeric_type.is_number())
                    || (left_to_numeric_type.is_bigint() && right_to_numeric_type.is_bigint())
                {
                    self.left.may_have_side_effects(is_global_reference)
                        || self.right.may_have_side_effects(is_global_reference)
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
                let left_to_numeric_type = self.left.to_numeric(is_global_reference);
                let right_to_numeric_type = self.right.to_numeric(is_global_reference);
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
                                    right.is_negative()
                                        || self.left.may_have_side_effects(is_global_reference)
                                }
                                BinaryOperator::Division | BinaryOperator::Remainder => {
                                    right.is_zero()
                                        || self.left.may_have_side_effects(is_global_reference)
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            true
                        }
                    } else {
                        self.left.may_have_side_effects(is_global_reference)
                            || self.right.may_have_side_effects(is_global_reference)
                    }
                } else if left_to_numeric_type.is_number() && right_to_numeric_type.is_number() {
                    self.left.may_have_side_effects(is_global_reference)
                        || self.right.may_have_side_effects(is_global_reference)
                } else {
                    true
                }
            }
        }
    }
}

impl MayHaveSideEffects for LogicalExpression<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        self.left.may_have_side_effects(is_global_reference)
            || self.right.may_have_side_effects(is_global_reference)
    }
}

impl MayHaveSideEffects for ArrayExpression<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        self.elements.iter().any(|element| element.may_have_side_effects(is_global_reference))
    }
}

impl MayHaveSideEffects for ArrayExpressionElement<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        match self {
            ArrayExpressionElement::SpreadElement(e) => match &e.argument {
                Expression::ArrayExpression(arr) => arr.may_have_side_effects(is_global_reference),
                Expression::StringLiteral(_) => false,
                Expression::TemplateLiteral(t) => {
                    t.expressions.iter().any(|e| e.may_have_side_effects(is_global_reference))
                }
                _ => true,
            },
            match_expression!(ArrayExpressionElement) => {
                self.to_expression().may_have_side_effects(is_global_reference)
            }
            ArrayExpressionElement::Elision(_) => false,
        }
    }
}

impl MayHaveSideEffects for ObjectPropertyKind<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        match self {
            ObjectPropertyKind::ObjectProperty(o) => o.may_have_side_effects(is_global_reference),
            ObjectPropertyKind::SpreadProperty(e) => match &e.argument {
                Expression::ArrayExpression(arr) => arr.may_have_side_effects(is_global_reference),
                Expression::StringLiteral(_) => false,
                Expression::TemplateLiteral(t) => {
                    t.expressions.iter().any(|e| e.may_have_side_effects(is_global_reference))
                }
                _ => true,
            },
        }
    }
}

impl MayHaveSideEffects for ObjectProperty<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        self.key.may_have_side_effects(is_global_reference)
            || self.value.may_have_side_effects(is_global_reference)
    }
}

impl MayHaveSideEffects for PropertyKey<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        match self {
            PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => false,
            match_expression!(PropertyKey) => {
                self.to_expression().may_have_side_effects(is_global_reference)
            }
        }
    }
}

impl MayHaveSideEffects for Class<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        self.body.body.iter().any(|element| element.may_have_side_effects(is_global_reference))
    }
}

impl MayHaveSideEffects for ClassElement<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        match self {
            // TODO: check side effects inside the block
            ClassElement::StaticBlock(block) => !block.body.is_empty(),
            ClassElement::MethodDefinition(e) => {
                e.r#static && e.key.may_have_side_effects(is_global_reference)
            }
            ClassElement::PropertyDefinition(e) => {
                e.r#static
                    && (e.key.may_have_side_effects(is_global_reference)
                        || e.value
                            .as_ref()
                            .is_some_and(|v| v.may_have_side_effects(is_global_reference)))
            }
            ClassElement::AccessorProperty(e) => {
                e.r#static && e.key.may_have_side_effects(is_global_reference)
            }
            ClassElement::TSIndexSignature(_) => false,
        }
    }
}

impl MayHaveSideEffects for StaticMemberExpression<'_> {
    fn may_have_side_effects(&self, _is_global_reference: &impl IsGlobalReference) -> bool {
        true
    }
}

impl MayHaveSideEffects for ComputedMemberExpression<'_> {
    fn may_have_side_effects(&self, _is_global_reference: &impl IsGlobalReference) -> bool {
        true
    }
}
