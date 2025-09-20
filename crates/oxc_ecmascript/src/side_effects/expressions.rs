mod array_expressions;
mod binary_expressions;
mod call_expressions;
mod class_expressions;
mod literal_expressions;
mod member_expressions;
mod object_expressions;
mod unary_expressions;

use oxc_ast::ast::*;

use super::{MayHaveSideEffects, context::MayHaveSideEffectsContext};
use binary_expressions::is_side_effect_free_unbound_identifier_ref;

impl<'a> MayHaveSideEffects<'a> for Expression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            Expression::Identifier(ident) => ident.may_have_side_effects(ctx),
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
            Expression::TemplateLiteral(e) => e.may_have_side_effects(ctx),
            Expression::UnaryExpression(e) => e.may_have_side_effects(ctx),
            Expression::LogicalExpression(e) => e.may_have_side_effects(ctx),
            Expression::ParenthesizedExpression(e) => e.expression.may_have_side_effects(ctx),
            Expression::ConditionalExpression(e) => {
                if e.test.may_have_side_effects(ctx) {
                    return true;
                }
                // typeof x === 'undefined' ? fallback : x
                if is_side_effect_free_unbound_identifier_ref(&e.alternate, &e.test, false, ctx) {
                    return e.consequent.may_have_side_effects(ctx);
                }
                // typeof x !== 'undefined' ? x : fallback
                if is_side_effect_free_unbound_identifier_ref(&e.consequent, &e.test, true, ctx) {
                    return e.alternate.may_have_side_effects(ctx);
                }
                e.consequent.may_have_side_effects(ctx) || e.alternate.may_have_side_effects(ctx)
            }
            Expression::SequenceExpression(e) => {
                e.expressions.iter().any(|e| e.may_have_side_effects(ctx))
            }
            Expression::BinaryExpression(e) => e.may_have_side_effects(ctx),
            Expression::ObjectExpression(object_expr) => {
                object_expr.properties.iter().any(|property| property.may_have_side_effects(ctx))
            }
            Expression::ArrayExpression(e) => e.may_have_side_effects(ctx),
            Expression::ClassExpression(e) => e.may_have_side_effects(ctx),
            // NOTE: private in can throw `TypeError`
            Expression::ChainExpression(e) => e.expression.may_have_side_effects(ctx),
            match_member_expression!(Expression) => {
                self.to_member_expression().may_have_side_effects(ctx)
            }
            Expression::CallExpression(e) => e.may_have_side_effects(ctx),
            Expression::NewExpression(e) => e.may_have_side_effects(ctx),
            Expression::TaggedTemplateExpression(e) => e.may_have_side_effects(ctx),
            _ => true,
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for AssignmentTarget<'a> {
    /// This only checks the `Evaluation of <AssignmentTarget>`.
    /// The sideeffect of `PutValue(<AssignmentTarget>)` is not considered here.
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            match_simple_assignment_target!(AssignmentTarget) => {
                self.to_simple_assignment_target().may_have_side_effects(ctx)
            }
            match_assignment_target_pattern!(AssignmentTarget) => true,
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for SimpleAssignmentTarget<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(_) => false,
            SimpleAssignmentTarget::StaticMemberExpression(member_expr) => {
                member_expr.object.may_have_side_effects(ctx)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(member_expr) => {
                member_expr.object.may_have_side_effects(ctx)
                    || member_expr.expression.may_have_side_effects(ctx)
            }
            SimpleAssignmentTarget::PrivateFieldExpression(member_expr) => {
                member_expr.object.may_have_side_effects(ctx)
            }
            SimpleAssignmentTarget::TSAsExpression(_)
            | SimpleAssignmentTarget::TSNonNullExpression(_)
            | SimpleAssignmentTarget::TSSatisfiesExpression(_)
            | SimpleAssignmentTarget::TSTypeAssertion(_) => true,
        }
    }
}
