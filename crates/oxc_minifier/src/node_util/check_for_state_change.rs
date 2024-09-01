use oxc_ast::ast::*;

use oxc_syntax::operator::UnaryOperator;

/// A "simple" operator is one whose children are expressions, has no direct side-effects.
fn is_simple_unary_operator(operator: UnaryOperator) -> bool {
    operator != UnaryOperator::Delete
}

/// port from [closure-compiler](https://github.com/google/closure-compiler/blob/f3ce5ed8b630428e311fe9aa2e20d36560d975e2/src/com/google/javascript/jscomp/AstAnalyzer.java#L241)
/// Returns true if some node in n's subtree changes application state. If
/// `check_for_new_objects` is true, we assume that newly created mutable objects (like object
/// literals) change state. Otherwise, we assume that they have no side effects.
pub trait CheckForStateChange<'a, 'b> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool;
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for Expression<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        match self {
            Self::NumericLiteral(_)
            | Self::BooleanLiteral(_)
            | Self::StringLiteral(_)
            | Self::BigIntLiteral(_)
            | Self::NullLiteral(_)
            | Self::RegExpLiteral(_)
            | Self::MetaProperty(_)
            | Self::ThisExpression(_)
            | Self::ClassExpression(_)
            | Self::FunctionExpression(_) => false,
            Self::TemplateLiteral(template) => template
                .expressions
                .iter()
                .any(|expr| expr.check_for_state_change(check_for_new_objects)),
            Self::Identifier(_ident) =>
            /* TODO: ident.reference_flags == ReferenceFlags::Write */
            {
                false
            }
            Self::UnaryExpression(unary_expr) => {
                unary_expr.check_for_state_change(check_for_new_objects)
            }
            Self::ParenthesizedExpression(p) => {
                p.expression.check_for_state_change(check_for_new_objects)
            }
            Self::ConditionalExpression(p) => {
                p.test.check_for_state_change(check_for_new_objects)
                    || p.consequent.check_for_state_change(check_for_new_objects)
                    || p.alternate.check_for_state_change(check_for_new_objects)
            }
            Self::SequenceExpression(s) => {
                s.expressions.iter().any(|expr| expr.check_for_state_change(check_for_new_objects))
            }
            Self::BinaryExpression(binary_expr) => {
                binary_expr.check_for_state_change(check_for_new_objects)
            }
            Self::ObjectExpression(object_expr) => {
                if check_for_new_objects {
                    return true;
                }

                object_expr
                    .properties
                    .iter()
                    .any(|property| property.check_for_state_change(check_for_new_objects))
            }
            Self::ArrayExpression(array_expr) => {
                if check_for_new_objects {
                    return true;
                }
                array_expr
                    .elements
                    .iter()
                    .any(|element| element.check_for_state_change(check_for_new_objects))
            }
            _ => true,
        }
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for UnaryExpression<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        if is_simple_unary_operator(self.operator) {
            return self.argument.check_for_state_change(check_for_new_objects);
        }
        true
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for BinaryExpression<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        let left = self.left.check_for_state_change(check_for_new_objects);
        let right = self.right.check_for_state_change(check_for_new_objects);

        left || right
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for ArrayExpressionElement<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        match self {
            Self::SpreadElement(element) => element.check_for_state_change(check_for_new_objects),
            match_expression!(Self) => {
                self.to_expression().check_for_state_change(check_for_new_objects)
            }
            Self::Elision(_) => false,
        }
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for ObjectPropertyKind<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        match self {
            Self::ObjectProperty(method) => method.check_for_state_change(check_for_new_objects),
            Self::SpreadProperty(spread_element) => {
                spread_element.check_for_state_change(check_for_new_objects)
            }
        }
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for SpreadElement<'a> {
    fn check_for_state_change(&self, _check_for_new_objects: bool) -> bool {
        // Object-rest and object-spread may trigger a getter.
        // TODO: Closure Compiler assumes that getters may side-free when set `assumeGettersArePure`.
        // https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/AstAnalyzer.java#L282
        true
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for ObjectProperty<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        self.key.check_for_state_change(check_for_new_objects)
            || self.value.check_for_state_change(check_for_new_objects)
    }
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for PropertyKey<'a> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool {
        match self {
            Self::StaticIdentifier(_) | Self::PrivateIdentifier(_) => false,
            match_expression!(Self) => {
                self.to_expression().check_for_state_change(check_for_new_objects)
            }
        }
    }
}
