use oxc_ast::ast::*;

use crate::is_global_reference::IsGlobalReference;

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
pub trait MayHaveSideEffects: Sized + IsGlobalReference {
    fn expression_may_have_side_effects(&self, e: &Expression<'_>) -> bool {
        match e {
            Expression::Identifier(ident) => match ident.name.as_str() {
                "NaN" | "Infinity" | "undefined" => false,
                // Reading global variables may have a side effect.
                // NOTE: It should also return true when the reference might refer to a reference value created by a with statement
                // NOTE: we ignore TDZ errors
                _ => self.is_global_reference(ident) != Some(false),
            },
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
                template.expressions.iter().any(|e| self.expression_may_have_side_effects(e))
            }
            Expression::UnaryExpression(e) => self.unary_expression_may_have_side_effects(e),
            Expression::LogicalExpression(e) => self.logical_expression_may_have_side_effects(e),
            Expression::ParenthesizedExpression(e) => {
                self.expression_may_have_side_effects(&e.expression)
            }
            Expression::ConditionalExpression(e) => {
                self.expression_may_have_side_effects(&e.test)
                    || self.expression_may_have_side_effects(&e.consequent)
                    || self.expression_may_have_side_effects(&e.alternate)
            }
            Expression::SequenceExpression(e) => {
                e.expressions.iter().any(|e| self.expression_may_have_side_effects(e))
            }
            Expression::BinaryExpression(e) => self.binary_expression_may_have_side_effects(e),
            Expression::ObjectExpression(object_expr) => object_expr
                .properties
                .iter()
                .any(|property| self.object_property_kind_may_have_side_effects(property)),
            Expression::ArrayExpression(e) => self.array_expression_may_have_side_effects(e),
            Expression::ClassExpression(e) => self.class_may_have_side_effects(e),
            // NOTE: private in can throw `TypeError`
            _ => true,
        }
    }

    fn unary_expression_may_have_side_effects(&self, e: &UnaryExpression<'_>) -> bool {
        match e.operator {
            UnaryOperator::Delete => true,
            UnaryOperator::Void | UnaryOperator::LogicalNot => {
                self.expression_may_have_side_effects(&e.argument)
            }
            UnaryOperator::Typeof => {
                if matches!(&e.argument, Expression::Identifier(_)) {
                    false
                } else {
                    self.expression_may_have_side_effects(&e.argument)
                }
            }
            UnaryOperator::UnaryPlus => {
                // ToNumber throws an error when the argument is Symbol / BigInt / an object that
                // returns Symbol or BigInt from ToPrimitive
                maybe_symbol_or_bigint_or_to_primitive_may_return_symbol_or_bigint(
                    self,
                    &e.argument,
                ) || self.expression_may_have_side_effects(&e.argument)
            }
            UnaryOperator::UnaryNegation | UnaryOperator::BitwiseNot => {
                // ToNumeric throws an error when the argument is Symbol / an object that
                // returns Symbol from ToPrimitive
                maybe_symbol_or_to_primitive_may_return_symbol(self, &e.argument)
                    || self.expression_may_have_side_effects(&e.argument)
            }
        }
    }

    fn binary_expression_may_have_side_effects(&self, e: &BinaryExpression<'_>) -> bool {
        match e.operator {
            BinaryOperator::Equality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::LessThan
            | BinaryOperator::LessEqualThan
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterEqualThan => {
                self.expression_may_have_side_effects(&e.left)
                    || self.expression_may_have_side_effects(&e.right)
            }
            BinaryOperator::In | BinaryOperator::Instanceof => {
                // instanceof and in can throw `TypeError`
                true
            }
            BinaryOperator::Addition => {
                if is_string_or_to_primitive_returns_string(&e.left)
                    || is_string_or_to_primitive_returns_string(&e.right)
                {
                    let other_side = if is_string_or_to_primitive_returns_string(&e.left) {
                        &e.right
                    } else {
                        &e.left
                    };
                    maybe_symbol_or_to_primitive_may_return_symbol(self, other_side)
                        || self.expression_may_have_side_effects(&e.left)
                        || self.expression_may_have_side_effects(&e.right)
                } else if e.left.is_number() || e.right.is_number() {
                    let other_side = if e.left.is_number() { &e.right } else { &e.left };
                    !matches!(
                        other_side,
                        Expression::NullLiteral(_)
                            | Expression::NumericLiteral(_)
                            | Expression::BooleanLiteral(_)
                    )
                } else {
                    !(e.left.is_big_int_literal() && e.right.is_big_int_literal())
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
                if e.left.is_big_int_literal() || e.right.is_big_int_literal() {
                    if let (Expression::BigIntLiteral(_), Expression::BigIntLiteral(right)) =
                        (&e.left, &e.right)
                    {
                        match e.operator {
                            BinaryOperator::Exponential => right.is_negative(),
                            BinaryOperator::Division | BinaryOperator::Remainder => right.is_zero(),
                            BinaryOperator::ShiftRightZeroFill => true,
                            _ => false,
                        }
                    } else {
                        true
                    }
                } else if !(maybe_symbol_or_bigint_or_to_primitive_may_return_symbol_or_bigint(
                    self, &e.left,
                ) || maybe_symbol_or_bigint_or_to_primitive_may_return_symbol_or_bigint(
                    self, &e.right,
                )) {
                    self.expression_may_have_side_effects(&e.left)
                        || self.expression_may_have_side_effects(&e.right)
                } else {
                    true
                }
            }
        }
    }

    fn logical_expression_may_have_side_effects(&self, e: &LogicalExpression<'_>) -> bool {
        self.expression_may_have_side_effects(&e.left)
            || self.expression_may_have_side_effects(&e.right)
    }

    fn array_expression_may_have_side_effects(&self, e: &ArrayExpression<'_>) -> bool {
        e.elements
            .iter()
            .any(|element| self.array_expression_element_may_have_side_effects(element))
    }

    fn array_expression_element_may_have_side_effects(
        &self,
        e: &ArrayExpressionElement<'_>,
    ) -> bool {
        match e {
            ArrayExpressionElement::SpreadElement(e) => match &e.argument {
                Expression::ArrayExpression(arr) => {
                    self.array_expression_may_have_side_effects(arr)
                }
                Expression::StringLiteral(_) => false,
                Expression::TemplateLiteral(t) => {
                    t.expressions.iter().any(|e| self.expression_may_have_side_effects(e))
                }
                _ => true,
            },
            match_expression!(ArrayExpressionElement) => {
                self.expression_may_have_side_effects(e.to_expression())
            }
            ArrayExpressionElement::Elision(_) => false,
        }
    }

    fn object_property_kind_may_have_side_effects(&self, e: &ObjectPropertyKind<'_>) -> bool {
        match e {
            ObjectPropertyKind::ObjectProperty(o) => self.object_property_may_have_side_effects(o),
            ObjectPropertyKind::SpreadProperty(e) => match &e.argument {
                Expression::ArrayExpression(arr) => {
                    self.array_expression_may_have_side_effects(arr)
                }
                Expression::StringLiteral(_) => false,
                Expression::TemplateLiteral(t) => {
                    t.expressions.iter().any(|e| self.expression_may_have_side_effects(e))
                }
                _ => true,
            },
        }
    }

    fn object_property_may_have_side_effects(&self, e: &ObjectProperty<'_>) -> bool {
        self.property_key_may_have_side_effects(&e.key)
            || self.expression_may_have_side_effects(&e.value)
    }

    fn property_key_may_have_side_effects(&self, key: &PropertyKey<'_>) -> bool {
        match key {
            PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => false,
            match_expression!(PropertyKey) => {
                self.expression_may_have_side_effects(key.to_expression())
            }
        }
    }

    fn class_may_have_side_effects(&self, class: &Class<'_>) -> bool {
        class.body.body.iter().any(|element| self.class_element_may_have_side_effects(element))
    }

    fn class_element_may_have_side_effects(&self, e: &ClassElement<'_>) -> bool {
        match e {
            // TODO: check side effects inside the block
            ClassElement::StaticBlock(block) => !block.body.is_empty(),
            ClassElement::MethodDefinition(e) => {
                e.r#static && self.property_key_may_have_side_effects(&e.key)
            }
            ClassElement::PropertyDefinition(e) => {
                e.r#static
                    && (self.property_key_may_have_side_effects(&e.key)
                        || e.value
                            .as_ref()
                            .is_some_and(|v| self.expression_may_have_side_effects(v)))
            }
            ClassElement::AccessorProperty(e) => {
                e.r#static && self.property_key_may_have_side_effects(&e.key)
            }
            ClassElement::TSIndexSignature(_) => false,
        }
    }
}

fn is_string_or_to_primitive_returns_string(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::StringLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::ArrayExpression(_) => true,
        // unless `Symbol.toPrimitive`, `valueOf`, `toString` is overridden,
        // ToPrimitive for an object returns `"[object Object]"`
        Expression::ObjectExpression(obj) => {
            !maybe_object_with_to_primitive_related_properties_overridden(obj)
        }
        _ => false,
    }
}

/// Whether the given expression may be a `Symbol` or converted to a `Symbol` when passed to `toPrimitive`.
fn maybe_symbol_or_to_primitive_may_return_symbol(
    m: &impl MayHaveSideEffects,
    expr: &Expression<'_>,
) -> bool {
    match expr {
        Expression::Identifier(ident) => {
            !(matches!(ident.name.as_str(), "Infinity" | "NaN" | "undefined")
                && m.is_global_reference(ident) == Some(true))
        }
        Expression::StringLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::ArrayExpression(_) => false,
        // unless `Symbol.toPrimitive`, `valueOf`, `toString` is overridden,
        // ToPrimitive for an object returns `"[object Object]"`
        Expression::ObjectExpression(obj) => {
            maybe_object_with_to_primitive_related_properties_overridden(obj)
        }
        _ => true,
    }
}

/// Whether the given expression may be a `Symbol`/`BigInt` or converted to a `Symbol`/`BigInt` when passed to `toPrimitive`.
fn maybe_symbol_or_bigint_or_to_primitive_may_return_symbol_or_bigint(
    m: &impl MayHaveSideEffects,
    expr: &Expression<'_>,
) -> bool {
    match expr {
        Expression::Identifier(ident) => {
            !(matches!(ident.name.as_str(), "Infinity" | "NaN" | "undefined")
                && m.is_global_reference(ident) == Some(true))
        }
        Expression::StringLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::ArrayExpression(_) => false,
        // unless `Symbol.toPrimitive`, `valueOf`, `toString` is overridden,
        // ToPrimitive for an object returns `"[object Object]"`
        Expression::ObjectExpression(obj) => {
            maybe_object_with_to_primitive_related_properties_overridden(obj)
        }
        _ => true,
    }
}

fn maybe_object_with_to_primitive_related_properties_overridden(
    obj: &ObjectExpression<'_>,
) -> bool {
    obj.properties.iter().any(|prop| match prop {
        ObjectPropertyKind::ObjectProperty(prop) => match &prop.key {
            PropertyKey::StaticIdentifier(id) => {
                matches!(id.name.as_str(), "toString" | "valueOf")
            }
            PropertyKey::PrivateIdentifier(_) => false,
            PropertyKey::StringLiteral(str) => {
                matches!(str.value.as_str(), "toString" | "valueOf")
            }
            PropertyKey::TemplateLiteral(temp) => {
                !temp.is_no_substitution_template()
                    || temp
                        .quasi()
                        .is_some_and(|val| matches!(val.as_str(), "toString" | "valueOf"))
            }
            _ => true,
        },
        ObjectPropertyKind::SpreadProperty(e) => match &e.argument {
            Expression::ObjectExpression(obj) => {
                maybe_object_with_to_primitive_related_properties_overridden(obj)
            }
            Expression::ArrayExpression(_)
            | Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_) => false,
            _ => true,
        },
    })
}
