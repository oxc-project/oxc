use oxc_ast::ast::*;

/// Returns true if subtree changes application state.
///
/// Ported from [closure-compiler](https://github.com/google/closure-compiler/blob/f3ce5ed8b630428e311fe9aa2e20d36560d975e2/src/com/google/javascript/jscomp/AstAnalyzer.java#L94)
pub trait MayHaveSideEffects {
    fn is_global_reference(&self, ident: &IdentifierReference<'_>) -> bool;

    fn expression_may_have_side_effects(&self, e: &Expression<'_>) -> bool {
        match e {
            // Reference read can have a side effect.
            Expression::Identifier(ident) => match ident.name.as_str() {
                "NaN" | "Infinity" | "undefined" => !self.is_global_reference(ident),
                _ => true,
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
            | Expression::FunctionExpression(_) => false,
            Expression::TemplateLiteral(template) => {
                template.expressions.iter().any(|e| self.expression_may_have_side_effects(e))
            }
            Expression::UnaryExpression(e) => self.unary_expression_may_have_side_effects(e),
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
            Expression::ArrayExpression(e) => e
                .elements
                .iter()
                .any(|element| self.array_expression_element_may_have_side_effects(element)),
            _ => true,
        }
    }

    fn unary_expression_may_have_side_effects(&self, e: &UnaryExpression<'_>) -> bool {
        /// A "simple" operator is one whose children are expressions, has no direct side-effects.
        fn is_simple_unary_operator(operator: UnaryOperator) -> bool {
            operator != UnaryOperator::Delete
        }
        if is_simple_unary_operator(e.operator) {
            return self.expression_may_have_side_effects(&e.argument);
        }
        true
    }

    fn binary_expression_may_have_side_effects(&self, e: &BinaryExpression<'_>) -> bool {
        // `instanceof` and `in` can throw `TypeError`
        if matches!(e.operator, BinaryOperator::In | BinaryOperator::Instanceof) {
            return true;
        }
        self.expression_may_have_side_effects(&e.left)
            || self.expression_may_have_side_effects(&e.right)
    }

    fn array_expression_element_may_have_side_effects(
        &self,
        e: &ArrayExpressionElement<'_>,
    ) -> bool {
        match e {
            ArrayExpressionElement::SpreadElement(e) => {
                self.expression_may_have_side_effects(&e.argument)
            }
            match_expression!(ArrayExpressionElement) => {
                self.expression_may_have_side_effects(e.to_expression())
            }
            ArrayExpressionElement::Elision(_) => false,
        }
    }

    fn object_property_kind_may_have_side_effects(&self, e: &ObjectPropertyKind<'_>) -> bool {
        match e {
            ObjectPropertyKind::ObjectProperty(o) => self.object_property_may_have_side_effects(o),
            ObjectPropertyKind::SpreadProperty(e) => {
                self.expression_may_have_side_effects(&e.argument)
            }
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
}
