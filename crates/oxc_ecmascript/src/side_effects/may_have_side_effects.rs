use oxc_ast::ast::*;

use crate::{
    ToBigInt, ToIntegerIndex, constant_evaluation::DetermineValueType,
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
            Expression::TemplateLiteral(e) => e.may_have_side_effects(is_global_reference),
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
            Expression::ChainExpression(e) => {
                e.expression.may_have_side_effects(is_global_reference)
            }
            match_member_expression!(Expression) => {
                self.to_member_expression().may_have_side_effects(is_global_reference)
            }
            Expression::CallExpression(e) => e.may_have_side_effects(is_global_reference),
            Expression::NewExpression(e) => e.may_have_side_effects(is_global_reference),
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

impl MayHaveSideEffects for TemplateLiteral<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        self.expressions.iter().any(|e| {
            // ToString is called for each expression.
            // If the expression is a Symbol or ToPrimitive returns a Symbol, an error is thrown.
            // ToPrimitive returns the value as-is for non-Object values, so we can use it instead of ToString here.
            e.to_primitive(is_global_reference).is_symbol() != Some(false)
                || e.may_have_side_effects(is_global_reference)
        })
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
            BinaryOperator::Instanceof => {
                // When the following conditions are met, instanceof won't throw `TypeError`.
                // - the right hand side is a known global reference which is a function
                // - the left hand side is not a proxy
                if let Expression::Identifier(right_ident) = &self.right {
                    let name = right_ident.name.as_str();
                    // Any known global non-constructor functions can be allowed here.
                    // But because non-constructor functions are not likely to be used, we ignore them.
                    if is_known_global_constructor(name)
                        && is_global_reference.is_global_reference(right_ident) == Some(true)
                        && !self.left.value_type(is_global_reference).is_undetermined()
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

/// Whether the name matches any known global constructors.
///
/// <https://tc39.es/ecma262/multipage/global-object.html#sec-constructor-properties-of-the-global-object>
fn is_known_global_constructor(name: &str) -> bool {
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
                Expression::TemplateLiteral(t) => t.may_have_side_effects(is_global_reference),
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
                Expression::TemplateLiteral(t) => t.may_have_side_effects(is_global_reference),
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
                // ToPropertyKey(key) throws an error when ToPrimitive(key) throws an Error
                // But we can ignore that by using the assumption.
                self.to_expression().may_have_side_effects(is_global_reference)
            }
        }
    }
}

impl MayHaveSideEffects for Class<'_> {
    /// Based on <https://github.com/evanw/esbuild/blob/v0.25.0/internal/js_ast/js_ast_helpers.go#L2320>
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        if !self.decorators.is_empty() {
            return true;
        }

        // NOTE: extending a value that is neither constructors nor null, throws an error
        // but that error is ignored here (it is included in the assumption)
        // Example cases: `class A extends 0 {}`, `class A extends (async function() {}) {}`
        // Considering these cases is difficult and requires to de-opt most classes with a super class.
        // To allow classes with a super class to be removed, we ignore this side effect.
        if self
            .super_class
            .as_ref()
            .is_some_and(|sup| sup.may_have_side_effects(is_global_reference))
        {
            return true;
        }

        self.body.body.iter().any(|element| element.may_have_side_effects(is_global_reference))
    }
}

impl MayHaveSideEffects for ClassElement<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        match self {
            // TODO: check side effects inside the block
            ClassElement::StaticBlock(block) => !block.body.is_empty(),
            ClassElement::MethodDefinition(e) => {
                !e.decorators.is_empty() || e.key.may_have_side_effects(is_global_reference)
            }
            ClassElement::PropertyDefinition(e) => {
                !e.decorators.is_empty()
                    || e.key.may_have_side_effects(is_global_reference)
                    || (e.r#static
                        && e.value
                            .as_ref()
                            .is_some_and(|v| v.may_have_side_effects(is_global_reference)))
            }
            ClassElement::AccessorProperty(e) => {
                !e.decorators.is_empty() || e.key.may_have_side_effects(is_global_reference)
            }
            ClassElement::TSIndexSignature(_) => false,
        }
    }
}

impl MayHaveSideEffects for ChainElement<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        match self {
            ChainElement::CallExpression(e) => e.may_have_side_effects(is_global_reference),
            ChainElement::TSNonNullExpression(e) => {
                e.expression.may_have_side_effects(is_global_reference)
            }
            match_member_expression!(ChainElement) => {
                self.to_member_expression().may_have_side_effects(is_global_reference)
            }
        }
    }
}

impl MayHaveSideEffects for MemberExpression<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        match self {
            MemberExpression::ComputedMemberExpression(e) => {
                e.may_have_side_effects(is_global_reference)
            }
            MemberExpression::StaticMemberExpression(e) => {
                e.may_have_side_effects(is_global_reference)
            }
            MemberExpression::PrivateFieldExpression(_) => true,
        }
    }
}

impl MayHaveSideEffects for StaticMemberExpression<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        property_access_may_have_side_effects(
            &self.object,
            &self.property.name,
            is_global_reference,
        )
    }
}

impl MayHaveSideEffects for ComputedMemberExpression<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        match &self.expression {
            Expression::StringLiteral(s) => {
                property_access_may_have_side_effects(&self.object, &s.value, is_global_reference)
            }
            Expression::TemplateLiteral(t) if t.is_no_substitution_template() => {
                property_access_may_have_side_effects(
                    &self.object,
                    &t.quasi().expect("template literal must have at least one quasi"),
                    is_global_reference,
                )
            }
            Expression::NumericLiteral(n) => !n.value.to_integer_index().is_some_and(|n| {
                !integer_index_property_access_may_have_side_effects(
                    &self.object,
                    n,
                    is_global_reference,
                )
            }),
            Expression::BigIntLiteral(b) => {
                if b.is_negative() {
                    return true;
                }
                !b.to_big_int(is_global_reference)
                    .and_then(ToIntegerIndex::to_integer_index)
                    .is_some_and(|b| {
                        !integer_index_property_access_may_have_side_effects(
                            &self.object,
                            b,
                            is_global_reference,
                        )
                    })
            }
            _ => true,
        }
    }
}

fn property_access_may_have_side_effects(
    object: &Expression,
    property: &str,
    is_global_reference: &impl IsGlobalReference,
) -> bool {
    match property {
        "length" => {
            !(matches!(object, Expression::ArrayExpression(_))
                || object.value_type(is_global_reference).is_string())
        }
        _ => true,
    }
}

fn integer_index_property_access_may_have_side_effects(
    object: &Expression,
    property: u32,
    is_global_reference: &impl IsGlobalReference,
) -> bool {
    if object.may_have_side_effects(is_global_reference) {
        return true;
    }
    match object {
        Expression::StringLiteral(s) => property as usize >= s.value.encode_utf16().count(),
        Expression::ArrayExpression(arr) => property as usize >= get_array_minimum_length(arr),
        _ => true,
    }
}

fn get_array_minimum_length(arr: &ArrayExpression) -> usize {
    arr.elements
        .iter()
        .map(|e| match e {
            ArrayExpressionElement::SpreadElement(spread) => match &spread.argument {
                Expression::ArrayExpression(arr) => get_array_minimum_length(arr),
                Expression::StringLiteral(str) => str.value.chars().count(),
                _ => 0,
            },
            _ => 1,
        })
        .sum()
}

impl MayHaveSideEffects for CallExpression<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        if self.pure {
            self.arguments.iter().any(|e| e.may_have_side_effects(is_global_reference))
        } else {
            true
        }
    }
}

impl MayHaveSideEffects for NewExpression<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        if self.pure {
            self.arguments.iter().any(|e| e.may_have_side_effects(is_global_reference))
        } else {
            true
        }
    }
}

impl MayHaveSideEffects for Argument<'_> {
    fn may_have_side_effects(&self, is_global_reference: &impl IsGlobalReference) -> bool {
        match self {
            Argument::SpreadElement(e) => match &e.argument {
                Expression::ArrayExpression(arr) => arr.may_have_side_effects(is_global_reference),
                Expression::StringLiteral(_) => false,
                Expression::TemplateLiteral(t) => t.may_have_side_effects(is_global_reference),
                _ => true,
            },
            match_expression!(Argument) => {
                self.to_expression().may_have_side_effects(is_global_reference)
            }
        }
    }
}
