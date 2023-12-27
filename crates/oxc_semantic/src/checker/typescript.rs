use oxc_ast::syntax_directed_operations::BoundNames;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_span::{Atom, GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{builder::SemanticBuilder, diagnostics::Redeclaration, AstNode};

pub struct EarlyErrorTypeScript;

impl EarlyErrorTypeScript {
    pub fn run<'a>(node: &AstNode<'a>, ctx: &SemanticBuilder<'a>) {
        let kind = node.kind();

        // should be removed when add more matches.
        #[allow(clippy::single_match)]
        match kind {
            AstKind::SimpleAssignmentTarget(target) => check_simple_assignment_target(target, ctx),
            AstKind::FormalParameters(params) => check_formal_parameters(params, ctx),
            _ => {}
        }
    }
}

fn check_formal_parameters(params: &FormalParameters, ctx: &SemanticBuilder<'_>) {
    if !params.is_empty() && params.kind == FormalParameterKind::Signature {
        check_duplicate_bound_names(params, ctx);
    }
}

fn check_duplicate_bound_names<T: BoundNames>(bound_names: &T, ctx: &SemanticBuilder<'_>) {
    let mut idents: FxHashMap<Atom, Span> = FxHashMap::default();
    bound_names.bound_names(&mut |ident| {
        if let Some(old_span) = idents.insert(ident.name.clone(), ident.span) {
            ctx.error(Redeclaration(ident.name.clone(), old_span, ident.span));
        }
    });
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
