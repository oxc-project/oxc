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
        let kind = node.kind();

        // should be removed when add more matches.
        #[allow(clippy::single_match)]
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
    if let Some(expression) = target.get_expression() {
        match expression.get_inner_expression() {
            Expression::Identifier(_) | Expression::MemberExpression(_) => {}
            _ => {
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
