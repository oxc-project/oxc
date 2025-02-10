use oxc_ast::ast::*;

/// Returns true if subtree changes application state.
///
/// This trait assumes the following:
/// - `.toString()`, `.valueOf()`, and `[Symbol.toPrimitive]()` are side-effect free.
/// - Errors thrown when creating a String or an Array that exceeds the maximum length does not happen.
/// - TDZ errors does not happen.
///
/// Ported from [closure-compiler](https://github.com/google/closure-compiler/blob/f3ce5ed8b630428e311fe9aa2e20d36560d975e2/src/com/google/javascript/jscomp/AstAnalyzer.java#L94)
pub trait MayHaveSideEffects {
    fn is_global_reference(&self, ident: &IdentifierReference<'_>) -> bool;

    fn expression_may_have_side_effects(&self, e: &Expression<'_>) -> bool {
        match e {
            Expression::Identifier(ident) => match ident.name.as_str() {
                "NaN" | "Infinity" | "undefined" => false,
                // Reading global variables may have a side effect.
                // NOTE: It should also return true when the reference might refer to a reference value created by a with statement
                // NOTE: we ignore TDZ errors
                _ => self.is_global_reference(ident),
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
                match &e.argument {
                    Expression::NumericLiteral(_)
                    | Expression::NullLiteral(_)
                    | Expression::BooleanLiteral(_)
                    | Expression::StringLiteral(_) => false,
                    Expression::Identifier(ident) => {
                        !(matches!(ident.name.as_str(), "Infinity" | "NaN" | "undefined")
                            && self.is_global_reference(ident))
                    }
                    Expression::ArrayExpression(arr) => {
                        self.array_expression_may_have_side_effects(arr)
                    }
                    // unless `Symbol.toPrimitive`, `valueOf`, `toString` is overridden,
                    // ToPrimitive for an object returns `"[object Object]"`
                    Expression::ObjectExpression(obj) => !obj.properties.is_empty(),
                    // ToNumber throws an error when the argument is Symbol / BigInt / an object that
                    // returns Symbol or BigInt from ToPrimitive
                    _ => true,
                }
            }
            UnaryOperator::UnaryNegation | UnaryOperator::BitwiseNot => {
                match &e.argument {
                    Expression::BigIntLiteral(_)
                    | Expression::NumericLiteral(_)
                    | Expression::NullLiteral(_)
                    | Expression::BooleanLiteral(_)
                    | Expression::StringLiteral(_) => false,
                    Expression::Identifier(ident) => {
                        !(matches!(ident.name.as_str(), "Infinity" | "NaN" | "undefined")
                            && self.is_global_reference(ident))
                    }
                    Expression::ArrayExpression(arr) => {
                        self.array_expression_may_have_side_effects(arr)
                    }
                    // unless `Symbol.toPrimitive`, `valueOf`, `toString` is overridden,
                    // ToPrimitive for an object returns `"[object Object]"`
                    Expression::ObjectExpression(obj) => !obj.properties.is_empty(),
                    // ToNumber throws an error when the argument is Symbol an object that
                    // returns Symbol from ToPrimitive
                    _ => true,
                }
            }
        }
    }

    fn binary_expression_may_have_side_effects(&self, e: &BinaryExpression<'_>) -> bool {
        // `instanceof` and `in` can throw `TypeError`
        if matches!(e.operator, BinaryOperator::In | BinaryOperator::Instanceof) {
            return true;
        }
        self.expression_may_have_side_effects(&e.left)
            || self.expression_may_have_side_effects(&e.right)
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
