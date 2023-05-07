#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_span::{GetSpan, Span};

use crate::{builder::SemanticBuilder, AstNode};

pub struct EarlyErrorTypeScript;

impl EarlyErrorTypeScript {
    pub fn run<'a>(node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) {
        let kind = node.get().kind();

        // should be removed when add more matches.
        #[allow(clippy::single_match)]
        match kind {
            AstKind::SimpleAssignmentTarget(target) => check_simple_assignment_target(target, ctx),
            AstKind::AssignmentTarget(target) => check_assignment_target(target, ctx),
            _ => {}
        }
    }
}

fn check_assignment_target<'a>(target: &AssignmentTarget<'a>, ctx: &SemanticBuilder<'a>) {
    match target {
        AssignmentTarget::AssignmentTargetPattern(assignment_pattern) => {
            if let Some(rest) = match assignment_pattern {
                AssignmentTargetPattern::ObjectAssignmentTarget(t) => &t.rest,
                AssignmentTargetPattern::ArrayAssignmentTarget(t) => &t.rest,
            } {
                if let AssignmentTarget::SimpleAssignmentTarget(simple_assign_target) = rest {
                    if let Some(expression) = simple_assign_target.get_expression() {
                        match expression.get_inner_expression() {
                            Expression::Identifier(_) | Expression::MemberExpression(_) => {}
                            _ => {
                                #[derive(Debug, Error, Diagnostic)]
                                #[error(
                                    "The target of an object rest assignment must be a variable or a property access."
                                )]
                                #[diagnostic()]
                                struct UnexpectedRestAssignment(#[label] Span);

                                ctx.error(UnexpectedRestAssignment(target.span()));
                            }
                        }
                    }
                }
            }
        }
        AssignmentTarget::SimpleAssignmentTarget(_) => {}
    }
}

fn check_simple_assignment_target<'a>(
    target: &SimpleAssignmentTarget<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    if let Some(expression) = target.get_expression() {
        match expression.get_inner_expression() {
            Expression::Identifier(_) | Expression::MemberExpression(_) => {}
            _ => {
                if let AstKind::AssignmentTarget(_) = ctx.parent_kind() {
                    return;
                }
                #[derive(Debug, Error, Diagnostic)]
                #[error(
                    "The left-hand side of an assignment expression must be a variable or a property access."
                )]
                #[diagnostic()]
                struct UnexpectedAssignment(#[label] Span);

                ctx.error(UnexpectedAssignment(target.span()));
            }
        }
    }
}
