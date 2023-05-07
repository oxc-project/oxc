#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_span::{Atom, GetSpan, Span};

use super::javascript::{ClassStatickBlockAwait, ReservedKeyword, STRICT_MODE_NAMES};
use crate::{builder::SemanticBuilder, scope::ScopeFlags, AstNode};

pub struct EarlyErrorTypeScript;

impl EarlyErrorTypeScript {
    pub fn run<'a>(node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) {
        let kind = node.get().kind();

        match kind {
            AstKind::SimpleAssignmentTarget(target) => check_simple_assignment_target(target, ctx),
            _ => {}
        }
    }
}

fn check_simple_assignment_target<'a>(
    target: &SimpleAssignmentTarget<'a>,
    ctx: &SemanticBuilder<'a>,
) {
    // `AssignmentTargetIdentifier` and `MemberAssignmentTarget` doesn't has expression
    if let Some(expression) = target.get_expression() {
        match expression.get_inner_expression() {
            Expression::Identifier(_) | Expression::MemberExpression(_) => {}
            _ => {
                #[derive(Debug, Error, Diagnostic)]
                #[error(
                    "The left-hand side of an assignment expression must be a variable or a property access."
                )]
                #[diagnostic()]
                struct UnexpectedAssignmentTarget(#[label] Span);

                return ctx.error(UnexpectedAssignmentTarget(expression.span()));
            }
        }
    }
}
