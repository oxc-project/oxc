use oxc_ast::ast::{
    ArrayExpressionElement, AssignmentTarget, ComputedMemberExpression, Expression,
    IdentifierReference, MemberExpression, PrivateFieldExpression, Program, SimpleAssignmentTarget,
    Statement, StaticMemberExpression,
};

use crate::{ast_util::get_declaration_of_variable, utils::Value, LintContext};

use super::NoSideEffectsDiagnostic;

pub trait ListenerMap {
    fn report_effects(&self, _ctx: &LintContext) {}
    fn report_effects_when_assigned(&self, _ctx: &LintContext) {}
    fn report_effects_when_called(&self, _ctx: &LintContext) {}
    fn report_effects_when_mutated(&self, _ctx: &LintContext) {}
    fn get_value_and_report_effects(&self, _ctx: &LintContext) -> Option<Value> {
        None
    }
}

impl<'a> ListenerMap for Program<'a> {
    fn report_effects(&self, ctx: &LintContext) {
        self.body.iter().for_each(|stmt| stmt.report_effects(ctx));
    }
}

impl<'a> ListenerMap for Statement<'a> {
    fn report_effects(&self, ctx: &LintContext) {
        if let Self::ExpressionStatement(expr_stmt) = self {
            expr_stmt.expression.report_effects(ctx);
        }
    }
}

impl<'a> ListenerMap for Expression<'a> {
    fn report_effects(&self, ctx: &LintContext) {
        match self {
            Self::ArrayExpression(array_expr) => {
                array_expr.elements.iter().for_each(|el| el.report_effects(ctx));
            }
            Self::AssignmentExpression(assign_expr) => {
                assign_expr.left.report_effects_when_assigned(ctx);
                assign_expr.right.report_effects(ctx);
            }
            Self::Identifier(ident) => {
                ident.report_effects(ctx);
            }
            _ => {}
        }
    }
    fn report_effects_when_mutated(&self, ctx: &LintContext) {
        #[allow(clippy::single_match)]
        match self {
            Self::Identifier(ident) => {
                ident.report_effects_when_mutated(ctx);
            }
            _ => {}
        }
    }
}

impl<'a> ListenerMap for AssignmentTarget<'a> {
    fn report_effects_when_assigned(&self, ctx: &LintContext) {
        match self {
            Self::SimpleAssignmentTarget(target) => {
                target.report_effects_when_assigned(ctx);
            }
            Self::AssignmentTargetPattern(_pattern) => {}
        }
    }
}

impl<'a> ListenerMap for SimpleAssignmentTarget<'a> {
    fn report_effects_when_assigned(&self, ctx: &LintContext) {
        match self {
            Self::AssignmentTargetIdentifier(ident) => {
                ident.report_effects_when_assigned(ctx);
            }
            Self::MemberAssignmentTarget(member) => {
                member.report_effects_when_assigned(ctx);
            }
            _ => {
                // For remain TypeScript AST, just visit its expression
                if let Some(expr) = self.get_expression() {
                    expr.report_effects_when_assigned(ctx);
                }
            }
        }
    }
}

impl<'a> ListenerMap for IdentifierReference<'a> {
    fn report_effects_when_assigned(&self, ctx: &LintContext) {
        if get_declaration_of_variable(self, ctx).is_none() {
            ctx.diagnostic(NoSideEffectsDiagnostic::Assignment(
                self.name.to_compact_str(),
                self.span,
            ));
        }
    }

    fn report_effects_when_mutated(&self, ctx: &LintContext) {
        // TODO: check mutation of local variable.
        if get_declaration_of_variable(self, ctx).is_none() {
            ctx.diagnostic(NoSideEffectsDiagnostic::Mutation(
                self.name.to_compact_str(),
                self.span,
            ));
        }
    }
}

impl<'a> ListenerMap for MemberExpression<'a> {
    fn report_effects_when_assigned(&self, ctx: &LintContext) {
        match self {
            Self::ComputedMemberExpression(expr) => {
                expr.report_effects(ctx);
                expr.object.report_effects_when_mutated(ctx);
            }
            Self::StaticMemberExpression(expr) => {
                expr.report_effects(ctx);
                expr.object.report_effects_when_mutated(ctx);
            }
            Self::PrivateFieldExpression(expr) => {
                expr.report_effects(ctx);
                expr.object.report_effects_when_mutated(ctx);
            }
        }
    }
}

impl<'a> ListenerMap for ComputedMemberExpression<'a> {}

impl<'a> ListenerMap for StaticMemberExpression<'a> {}

impl<'a> ListenerMap for PrivateFieldExpression<'a> {}

impl<'a> ListenerMap for ArrayExpressionElement<'a> {
    fn report_effects(&self, ctx: &LintContext) {
        match self {
            Self::Expression(expr) => expr.report_effects(ctx),
            Self::SpreadElement(spreed) => {
                spreed.argument.report_effects(ctx);
            }
            Self::Elision(_) => {}
        }
    }
}
