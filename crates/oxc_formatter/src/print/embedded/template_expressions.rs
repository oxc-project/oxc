//! [`crate::JsEmbeddedIn::template_expressions_only`]: the syntax a template language's host reads as JS does.

use oxc_ast::ast::*;

/// `expression` holds only what a template language reads as JS does ([`crate::JsEmbeddedIn::template_expressions_only`]):
/// Angular's expression grammar (`@angular/compiler`'s `Parser`), which it reads with a parser of its own, not TS's.
///
/// An allowlist: a kind not named here is outside it, whatever JS or TS adds;
/// one the grammar gains (an arrow function in 22) prints as written until it is named here.
pub(super) fn is_template_expression(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(_)
        | Expression::ThisExpression(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::RegExpLiteral(_) => true,
        Expression::TemplateLiteral(template) => {
            template.expressions.iter().all(is_template_expression)
        }
        Expression::TaggedTemplateExpression(tagged) => {
            tagged.type_arguments.is_none()
                && is_template_expression(&tagged.tag)
                && tagged.quasi.expressions.iter().all(is_template_expression)
        }
        Expression::ParenthesizedExpression(parens) => is_template_expression(&parens.expression),
        Expression::TSNonNullExpression(non_null) => is_template_expression(&non_null.expression),
        Expression::CallExpression(call) => is_template_call(call),
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => is_template_call(call),
            ChainElement::TSNonNullExpression(non_null) => {
                is_template_expression(&non_null.expression)
            }
            element => element.as_member_expression().is_some_and(is_template_member),
        },
        expression if let Some(member) = expression.as_member_expression() => {
            is_template_member(member)
        }
        Expression::UnaryExpression(unary) => {
            matches!(
                unary.operator,
                UnaryOperator::LogicalNot
                    | UnaryOperator::UnaryNegation
                    | UnaryOperator::UnaryPlus
                    | UnaryOperator::Typeof
                    | UnaryOperator::Void
            ) && is_template_expression(&unary.argument)
        }
        Expression::BinaryExpression(binary) => {
            !binary.operator.is_bitwise()
                && is_template_expression(&binary.left)
                && is_template_expression(&binary.right)
        }
        Expression::LogicalExpression(logical) => {
            is_template_expression(&logical.left) && is_template_expression(&logical.right)
        }
        Expression::ConditionalExpression(conditional) => {
            is_template_expression(&conditional.test)
                && is_template_expression(&conditional.consequent)
                && is_template_expression(&conditional.alternate)
        }
        Expression::AssignmentExpression(assignment) => {
            !assignment.operator.is_bitwise()
                && matches!(
                    &assignment.left,
                    AssignmentTarget::AssignmentTargetIdentifier(_)
                        | AssignmentTarget::StaticMemberExpression(_)
                        | AssignmentTarget::ComputedMemberExpression(_)
                )
                && assignment.left.as_member_expression().is_none_or(is_template_member)
                && is_template_expression(&assignment.right)
        }
        Expression::ArrayExpression(array) => array.elements.iter().all(|element| match element {
            ArrayExpressionElement::SpreadElement(spread) => {
                is_template_expression(&spread.argument)
            }
            ArrayExpressionElement::Elision(_) => false,
            element => element.as_expression().is_some_and(is_template_expression),
        }),
        Expression::ObjectExpression(object) => {
            object.properties.iter().all(|property| match property {
                ObjectPropertyKind::SpreadProperty(spread) => {
                    is_template_expression(&spread.argument)
                }
                ObjectPropertyKind::ObjectProperty(property) => {
                    property.kind == PropertyKind::Init
                        && !property.method
                        && !property.key.is_computed()
                        && is_template_expression(&property.value)
                }
            })
        }
        Expression::ArrowFunctionExpression(arrow) => {
            !arrow.r#async
                && arrow.type_parameters.is_none()
                && arrow.return_type.is_none()
                && arrow.params.rest.is_none()
                && arrow.params.items.iter().all(|param| {
                    param.pattern.is_binding_identifier()
                        && param.type_annotation.is_none()
                        && param.initializer.is_none()
                })
                && arrow.get_expression().is_some_and(is_template_expression)
        }
        _ => false,
    }
}

fn is_template_member(member: &MemberExpression<'_>) -> bool {
    match member {
        MemberExpression::StaticMemberExpression(member) => is_template_expression(&member.object),
        MemberExpression::ComputedMemberExpression(member) => {
            is_template_expression(&member.object) && is_template_expression(&member.expression)
        }
        MemberExpression::PrivateFieldExpression(_) => false,
    }
}

fn is_template_call(call: &CallExpression<'_>) -> bool {
    call.type_arguments.is_none()
        && is_template_expression(&call.callee)
        && call.arguments.iter().all(is_template_argument)
}

fn is_template_argument(argument: &Argument<'_>) -> bool {
    match argument {
        Argument::SpreadElement(spread) => is_template_expression(&spread.argument),
        argument => argument.as_expression().is_some_and(is_template_expression),
    }
}
