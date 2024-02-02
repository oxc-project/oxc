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

        match kind {
            AstKind::VariableDeclarator(decl) => check_variable_declarator(decl, ctx),
            AstKind::SimpleAssignmentTarget(target) => check_simple_assignment_target(target, ctx),
            AstKind::FormalParameters(params) => check_formal_parameters(params, ctx),
            _ => {}
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn check_variable_declarator(decl: &VariableDeclarator, ctx: &SemanticBuilder<'_>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("Unexpected `?` operator")]
    #[diagnostic()]
    struct UnexpectedOptional(#[label] Span);
    if decl.id.optional {
        let start = decl.id.span().end;
        let Some(offset) = ctx.source_text[start as usize..].find('?') else { return };
        let offset = start + offset as u32;
        ctx.error(UnexpectedOptional(Span::new(offset, offset)));
    }
}

fn check_formal_parameters(params: &FormalParameters, ctx: &SemanticBuilder<'_>) {
    #[derive(Debug, Error, Diagnostic)]
    #[error("A required parameter cannot follow an optional parameter.")]
    #[diagnostic()]
    struct RequiredParameterAfterOptionalParameter(#[label] Span);
    #[derive(Debug, Error, Diagnostic)]
    #[error("A parameter property is only allowed in a constructor implementation.")]
    #[diagnostic()]
    struct ParameterPropertyOutsideConstructor(#[label] Span);

    if !params.is_empty() && params.kind == FormalParameterKind::Signature {
        check_duplicate_bound_names(params, ctx);
    }

    let is_inside_constructor = ctx.current_scope_flags().is_constructor();
    let mut has_optional = false;

    for item in &params.items {
        // function a(optional?: number, required: number) { }
        if has_optional && !item.pattern.optional && !item.pattern.kind.is_assignment_pattern() {
            ctx.error(RequiredParameterAfterOptionalParameter(item.span));
        }
        if item.pattern.optional {
            has_optional = true;
        }

        // function a(public x: number) { }
        if !is_inside_constructor && item.accessibility.is_some() {
            ctx.error(ParameterPropertyOutsideConstructor(item.span));
        }
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
