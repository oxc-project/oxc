use super::misc;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};

use crate::{
    doc::{Doc, Group},
    if_break, ss, Format, Prettier,
};

pub(super) enum CallExpressionLike<'a, 'b> {
    CallExpression(&'b CallExpression<'a>),
    NewExpression(&'b NewExpression<'a>),
}

impl<'a, 'b> CallExpressionLike<'a, 'b> {
    fn is_new(&self) -> bool {
        matches!(self, CallExpressionLike::NewExpression(_))
    }
    fn callee(&self) -> &Expression<'a> {
        match self {
            CallExpressionLike::CallExpression(call) => &call.callee,
            CallExpressionLike::NewExpression(new) => &new.callee,
        }
    }
    fn optional(&self) -> bool {
        match self {
            CallExpressionLike::CallExpression(call) => call.optional,
            CallExpressionLike::NewExpression(new) => false,
        }
    }
    fn arguments(&self) -> &Vec<'a, Argument<'a>> {
        match self {
            CallExpressionLike::CallExpression(call) => &call.arguments,
            CallExpressionLike::NewExpression(new) => &new.arguments,
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

    if expression.optional() {
        parts.push(ss!("?."));
    }

    parts.push(print_call_expression_arguments(p, expression));

    Doc::Array(parts)
}

fn print_call_expression_arguments<'a>(
    p: &mut Prettier<'a>,
    expression: &CallExpressionLike<'a, '_>,
) -> Doc<'a> {
    let mut parts = p.vec();
    parts.push(ss!("("));

    let mut parts_inner = p.vec();

    let callee = expression.callee();
    let arguments = expression.arguments();
    let should_break = !is_commons_js_or_amd_call(expression.callee(), arguments);

    if arguments.is_empty() {
        parts.extend(p.print_inner_comment(Span::new(callee.span().end, expression.span().end)));
    }

    for (i, element) in arguments.iter().enumerate() {
        let doc = element.format(p);
        parts_inner.push(doc);

        if i < arguments.len() - 1 {
            parts_inner.push(ss!(","));
            parts_inner.push(Doc::Line);
        }
    }
    if should_break {
        parts_inner.insert(0, Doc::Softline);
        parts.push(Doc::Indent(parts_inner));
        parts.push(if_break!(p, ","));
        parts.push(Doc::Softline);
    } else {
        parts.extend(parts_inner);
    }
    parts.push(ss!(")"));

    let should_break = should_break
        && arguments.iter().any(|arg| {
            misc::has_new_line_in_range(p.source_text, arg.span().start, arg.span().end)
        });
    Doc::Group(Group::new(parts, should_break))
}

/// https://github.com/prettier/prettier/blob/7aecca5d6473d73f562ca3af874831315f8f2581/src/language-js/print/call-expression.js#L93-L116
fn is_commons_js_or_amd_call<'a>(
    callee: &Expression<'a>,
    arguments: &Vec<'a, Argument<'a>>,
) -> bool {
    if let Expression::Identifier(callee) = callee {
        if callee.name == "require" {
            return arguments.len() == 1
                && matches!(arguments[0], Argument::Expression(Expression::StringLiteral(_)))
                || arguments.len() > 1;
        }
        if callee.name == "define" {
            // TODO: the parent node is ExpressionStatement
            return arguments.len() == 1
                || (arguments.len() == 2
                    && matches!(
                        arguments[1],
                        Argument::Expression(Expression::ArrayExpression(_))
                    ))
                || (arguments.len() == 3
                    && matches!(arguments[0], Argument::Expression(Expression::StringLiteral(_)))
                    && matches!(
                        arguments[1],
                        Argument::Expression(Expression::ArrayExpression(_))
                    ));
        }
    }
    false
}
