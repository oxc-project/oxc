use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};

use super::call_arguments::print_call_arguments;
use crate::{
    doc::{Doc, DocBuilder, Group},
    ss, Format, Prettier,
};

pub(super) enum CallExpressionLike<'a, 'b> {
    CallExpression(&'b CallExpression<'a>),
    NewExpression(&'b NewExpression<'a>),
}

impl<'a, 'b> CallExpressionLike<'a, 'b> {
    pub fn is_new(&self) -> bool {
        matches!(self, CallExpressionLike::NewExpression(_))
    }

    pub fn callee(&self) -> &Expression<'a> {
        match self {
            CallExpressionLike::CallExpression(call) => &call.callee,
            CallExpressionLike::NewExpression(new) => &new.callee,
        }
    }

    pub fn optional(&self) -> bool {
        match self {
            CallExpressionLike::CallExpression(call) => call.optional,
            CallExpressionLike::NewExpression(new) => false,
        }
    }

    pub fn arguments(&self) -> &Vec<'a, Argument<'a>> {
        match self {
            CallExpressionLike::CallExpression(call) => &call.arguments,
            CallExpressionLike::NewExpression(new) => &new.arguments,
        }
    }

    pub fn type_parameters(
        &self,
    ) -> &Option<oxc_allocator::Box<'a, TSTypeParameterInstantiation<'a>>> {
        match self {
            CallExpressionLike::CallExpression(call) => &call.type_parameters,
            CallExpressionLike::NewExpression(new) => &new.type_parameters,
        }
    }
}

impl GetSpan for CallExpressionLike<'_, '_> {
    fn span(&self) -> Span {
        match self {
            CallExpressionLike::CallExpression(call) => call.span,
            CallExpressionLike::NewExpression(new) => new.span,
        }
    }
}

pub(super) fn print_call_expression<'a>(
    p: &mut Prettier<'a>,
    expression: &CallExpressionLike<'a, '_>,
) -> Doc<'a> {
    let mut parts = p.vec();

    if expression.is_new() {
        parts.push(ss!("new "));
    };

    parts.push(expression.callee().format(p));

    if let Some(type_parameters) = expression.type_parameters() {
        parts.push(type_parameters.format(p));
    }

    if expression.optional() {
        parts.push(ss!("?."));
    }

    parts.push(print_call_arguments(p, expression));

    Doc::Group(Group::new(parts))
}

/// <https://github.com/prettier/prettier/blob/7aecca5d6473d73f562ca3af874831315f8f2581/src/language-js/print/call-expression.js#L93-L116>
pub fn is_commons_js_or_amd_call<'a>(
    callee: &Expression<'a>,
    arguments: &Vec<'a, Argument<'a>>,
) -> bool {
    if let Expression::Identifier(callee) = callee {
        if callee.name == "require" {
            return arguments.len() == 1 && matches!(arguments[0], Argument::StringLiteral(_))
                || arguments.len() > 1;
        }
        if callee.name == "define" {
            // TODO: the parent node is ExpressionStatement
            return arguments.len() == 1
                || (arguments.len() == 2 && matches!(arguments[1], Argument::ArrayExpression(_)))
                || (arguments.len() == 3
                    && matches!(arguments[0], Argument::StringLiteral(_))
                    && matches!(arguments[1], Argument::ArrayExpression(_)));
        }
    }
    false
}
