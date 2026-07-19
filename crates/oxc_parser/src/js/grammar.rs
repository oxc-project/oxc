//! Cover Grammar for Destructuring Assignment

use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{ParserConfig as Config, ParserImpl, diagnostics};

pub trait CoverGrammar<'a, T, C: Config>: Sized {
    fn cover(value: T, p: &mut ParserImpl<'a, C>) -> Self;
}

impl<'a, C: Config> CoverGrammar<'a, Expression<'a>, C> for AssignmentTarget<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        match expr.into_kind() {
            ExpressionKindOwned::ArrayExpression(array_expr) => {
                let pat = ArrayAssignmentTarget::cover(array_expr.unbox(), p);
                AssignmentTarget::ArrayAssignmentTarget(p.alloc(pat))
            }
            ExpressionKindOwned::ObjectExpression(object_expr) => {
                let pat = ObjectAssignmentTarget::cover(object_expr.unbox(), p);
                AssignmentTarget::ObjectAssignmentTarget(p.alloc(pat))
            }
            other => AssignmentTarget::from(SimpleAssignmentTarget::cover(
                Expression::from_kind(other),
                p,
            )),
        }
    }
}

impl<'a, C: Config> CoverGrammar<'a, Expression<'a>, C> for SimpleAssignmentTarget<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        /// `true` if the inner expression is a valid simple assignment target for a
        /// TS expression wrapper (`x as T`, `x!`, etc.)
        fn is_valid_ts_target_inner(expr: &Expression<'_>) -> bool {
            matches!(
                expr.get_inner_expression().tag(),
                ExpressionTag::Identifier
                    | ExpressionTag::StaticMemberExpression
                    | ExpressionTag::ComputedMemberExpression
                    | ExpressionTag::PrivateFieldExpression
            )
        }

        if expr.is_member_expression() {
            return SimpleAssignmentTarget::from(expr.into_member_expression());
        }
        match expr.into_kind() {
            ExpressionKindOwned::Identifier(ident) => {
                SimpleAssignmentTarget::AssignmentTargetIdentifier(ident)
            }
            ExpressionKindOwned::ParenthesizedExpression(expr) => {
                let span = expr.span;
                let inner = expr.unbox().expression;
                match inner.tag() {
                    ExpressionTag::ObjectExpression | ExpressionTag::ArrayExpression => {
                        p.fatal_error(diagnostics::invalid_assignment(span))
                    }
                    _ => SimpleAssignmentTarget::cover(inner, p),
                }
            }
            ExpressionKindOwned::TSAsExpression(expr) => {
                if is_valid_ts_target_inner(&expr.expression) {
                    SimpleAssignmentTarget::TSAsExpression(expr)
                } else {
                    p.fatal_error(diagnostics::invalid_assignment(expr.span()))
                }
            }
            ExpressionKindOwned::TSSatisfiesExpression(expr) => {
                if is_valid_ts_target_inner(&expr.expression) {
                    SimpleAssignmentTarget::TSSatisfiesExpression(expr)
                } else {
                    p.fatal_error(diagnostics::invalid_assignment(expr.span()))
                }
            }
            ExpressionKindOwned::TSNonNullExpression(expr) => {
                if is_valid_ts_target_inner(&expr.expression) {
                    SimpleAssignmentTarget::TSNonNullExpression(expr)
                } else {
                    p.fatal_error(diagnostics::invalid_assignment(expr.span()))
                }
            }
            ExpressionKindOwned::TSTypeAssertion(expr) => {
                if is_valid_ts_target_inner(&expr.expression) {
                    SimpleAssignmentTarget::TSTypeAssertion(expr)
                } else {
                    p.fatal_error(diagnostics::invalid_assignment(expr.span()))
                }
            }
            ExpressionKindOwned::TSInstantiationExpression(expr) => {
                p.fatal_error(diagnostics::invalid_lhs_assignment(expr.span()))
            }
            other => {
                p.fatal_error(diagnostics::invalid_assignment(Expression::from_kind(other).span()))
            }
        }
    }
}

impl<'a, C: Config> CoverGrammar<'a, ArrayExpression<'a>, C> for ArrayAssignmentTarget<'a> {
    // Destructuring-target conversion is comparatively rare and large. Keeping it out of line
    // stops it being inlined into `AssignmentTarget::cover`, whose common arm (simple targets)
    // would otherwise carry this body's large stack frame + callee-saved spills on every call.
    #[inline(never)]
    fn cover(expr: ArrayExpression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        let mut elements = ArenaVec::new_in(p);
        let mut rest = None;

        let len = expr.elements.len();
        for (i, elem) in expr.elements.into_iter().enumerate() {
            match elem {
                ArrayExpressionElement::Expression(expr) => {
                    let target = AssignmentTargetMaybeDefault::cover(expr, p);
                    elements.push(Some(target));
                }
                ArrayExpressionElement::SpreadElement(elem) => {
                    if i == len - 1 {
                        let span = elem.span;
                        let argument = elem.unbox().argument;
                        if !matches!(
                            argument.get_inner_expression().tag(),
                            ExpressionTag::Identifier
                                | ExpressionTag::ArrayExpression
                                | ExpressionTag::ObjectExpression
                                | ExpressionTag::StaticMemberExpression
                                | ExpressionTag::ComputedMemberExpression
                                | ExpressionTag::PrivateFieldExpression
                        ) {
                            p.error(diagnostics::invalid_rest_assignment_target(argument.span()));
                        }
                        let target = AssignmentTarget::cover(argument, p);
                        rest = Some(AssignmentTargetRest::boxed(span, target, p));
                        if let Some(span) = p.state.trailing_commas.get(&expr.span.start) {
                            p.error(diagnostics::rest_element_trailing_comma(*span));
                        }
                    } else {
                        let error = diagnostics::spread_last_element(elem.span);
                        return p.fatal_error(error);
                    }
                }
                ArrayExpressionElement::Elision(_) => elements.push(None),
            }
        }

        ArrayAssignmentTarget::new(expr.span, elements, rest, p)
    }
}

impl<'a, C: Config> CoverGrammar<'a, Expression<'a>, C> for AssignmentTargetMaybeDefault<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        if expr.is_assignment_expression() {
            let assignment_expr = expr.into_assignment_expression().unwrap();
            if assignment_expr.operator != AssignmentOperator::Assign {
                p.error(diagnostics::invalid_assignment_target_default_value_operator(
                    assignment_expr.span,
                ));
            }
            let target = AssignmentTargetWithDefault::cover(assignment_expr.unbox(), p);
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(p.alloc(target))
        } else {
            let target = AssignmentTarget::cover(expr, p);
            AssignmentTargetMaybeDefault::from(target)
        }
    }
}

impl<'a, C: Config> CoverGrammar<'a, AssignmentExpression<'a>, C>
    for AssignmentTargetWithDefault<'a>
{
    fn cover(expr: AssignmentExpression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        AssignmentTargetWithDefault::new(expr.span, expr.left, expr.right, p)
    }
}

impl<'a, C: Config> CoverGrammar<'a, ObjectExpression<'a>, C> for ObjectAssignmentTarget<'a> {
    // Kept out of line for the same reason as `ArrayAssignmentTarget::cover` above: avoid
    // inlining this large body into the hot `AssignmentTarget::cover` dispatcher.
    #[inline(never)]
    fn cover(expr: ObjectExpression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        let mut properties = ArenaVec::new_in(p);
        let mut rest = None;

        let len = expr.properties.len();
        for (i, elem) in expr.properties.into_iter().enumerate() {
            match elem {
                ObjectPropertyKind::ObjectProperty(property) => {
                    let target = AssignmentTargetProperty::cover(property.unbox(), p);
                    properties.push(target);
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    if i == len - 1 {
                        let span = spread.span;
                        let argument = spread.unbox().argument;
                        if !matches!(
                            argument.get_inner_expression().tag(),
                            ExpressionTag::Identifier
                                | ExpressionTag::StaticMemberExpression
                                | ExpressionTag::ComputedMemberExpression
                                | ExpressionTag::PrivateFieldExpression
                        ) {
                            p.error(diagnostics::invalid_rest_assignment_target(argument.span()));
                        }
                        if let Some(span) = p.state.trailing_commas.get(&expr.span.start) {
                            p.error(diagnostics::rest_element_trailing_comma(*span));
                        }
                        let target = AssignmentTarget::cover(argument, p);
                        rest = Some(AssignmentTargetRest::boxed(span, target, p));
                    } else {
                        return p.fatal_error(diagnostics::spread_last_element(spread.span));
                    }
                }
            }
        }

        ObjectAssignmentTarget::new(expr.span, properties, rest, p)
    }
}

impl<'a, C: Config> CoverGrammar<'a, ObjectProperty<'a>, C> for AssignmentTargetProperty<'a> {
    fn cover(property: ObjectProperty<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        if property.shorthand {
            let binding = match property.key {
                PropertyKey::StaticIdentifier(ident) => {
                    let ident = ident.unbox();
                    IdentifierReference::new(ident.span, ident.name, p)
                }
                _ => return p.unexpected(),
            };
            // convert `CoverInitializedName`
            let init = p.state.cover_initialized_name.remove(&property.span.start).map(|e| e.right);
            AssignmentTargetProperty::new_assignment_target_property_identifier(
                property.span,
                binding,
                init,
                p,
            )
        } else {
            let binding = AssignmentTargetMaybeDefault::cover(property.value, p);
            AssignmentTargetProperty::new_assignment_target_property_property(
                property.span,
                property.key,
                binding,
                property.computed,
                p,
            )
        }
    }
}
