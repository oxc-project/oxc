use super::misc;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{
    array, conditional_group,
    doc::{Doc, DocBuilder, Group},
    group_break, hardline, if_break, indent, line, softline, ss,
    utils::will_break,
    Format, Prettier,
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

    Doc::Group(Group::new(parts, false))
}

fn print_call_expression_arguments<'a>(
    p: &mut Prettier<'a>,
    expression: &CallExpressionLike<'a, '_>,
) -> Doc<'a> {
    let mut parts = p.vec();
    parts.push(ss!("("));

    let callee = expression.callee();
    let arguments = expression.arguments();
    let should_break = if matches!(expression, CallExpressionLike::CallExpression(_)) {
        !is_commons_js_or_amd_call(expression.callee(), arguments)
    } else {
        true
    };

    if arguments.is_empty() {
        parts.extend(p.print_inner_comment(Span::new(callee.span().end, expression.span().end)));
        parts.push(ss!(")"));
        return Doc::Array(parts);
    }

    #[allow(clippy::cast_sign_loss)]
    let get_printed_arguments = |p: &mut Prettier<'a>, skip_index: isize| {
        let mut printed_arguments = p.vec();
        let mut len = arguments.len();
        let arguments: Box<dyn Iterator<Item = (usize, &Argument)>> = match skip_index {
            _ if skip_index > 0 => {
                len -= skip_index as usize;
                Box::new(arguments.iter().skip(skip_index as usize).enumerate())
            }
            _ if skip_index < 0 => {
                len -= (-skip_index) as usize;
                Box::new(
                    arguments.iter().take(arguments.len() - (-skip_index) as usize).enumerate(),
                )
            }
            _ => Box::new(arguments.iter().enumerate()),
        };

        for (i, element) in arguments {
            let doc = element.format(p);
            let mut arg = p.vec();
            arg.push(doc);

            if i < len - 1 {
                arg.push(ss!(","));
                if p.is_next_line_empty(element.span()) {
                    arg.extend(hardline!());
                    arg.extend(hardline!());
                } else {
                    arg.push(line!());
                }
            }
            printed_arguments.push(Doc::Array(arg));
        }
        printed_arguments
    };

    let all_args_broken_out = |p: &mut Prettier<'a>| {
        let mut parts = p.vec();
        parts.push(ss!("("));
        parts.push(indent!(
            p,
            line!(),
            Doc::Array(get_printed_arguments(p, 0)),
            if p.should_print_all_comma() { ss!(",") } else { ss!("") }
        ));
        parts.push(line!());
        parts.push(ss!(")"));
        Doc::Group(Group::new(parts, true))
    };

    if should_expand_first_arg(arguments) {
        p.args.expand_first_arg = true;
        let mut first_doc = arguments[0].format(p);
        p.args.expand_first_arg = false;

        if will_break(&mut first_doc) {
            let last_doc = get_printed_arguments(p, 1).pop().unwrap();
            return array![
                p,
                Doc::BreakParent,
                conditional_group!(
                    p,
                    array!(p, ss!("("), group_break!(p, first_doc), ss!(", "), last_doc, ss!(")")),
                    all_args_broken_out(p)
                )
            ];
        }
    }

    if should_expand_last_arg(arguments) {
        let mut printed_arguments = get_printed_arguments(p, -1);
        if printed_arguments.iter_mut().any(will_break) {
            return all_args_broken_out(p);
        }

        if !printed_arguments.is_empty() {
            printed_arguments.push(ss!(","));
            printed_arguments.push(line!());
        }

        let get_last_doc = |p: &mut Prettier<'a>| {
            p.args.expand_last_arg = true;
            let last_doc = arguments.last().unwrap().format(p);
            p.args.expand_last_arg = false;
            last_doc
        };

        let mut last_doc = get_last_doc(p);

        if will_break(&mut last_doc) {
            return array![
                p,
                Doc::BreakParent,
                conditional_group!(
                    p,
                    array!(
                        p,
                        ss!("("),
                        Doc::Array(printed_arguments),
                        group_break!(p, last_doc),
                        ss!(")")
                    ),
                    all_args_broken_out(p)
                ),
            ];
        }

        return conditional_group!(
            p,
            array!(p, ss!("("), Doc::Array(printed_arguments), last_doc, ss!(")")),
            array!(
                p,
                ss!("("),
                Doc::Array(get_printed_arguments(p, -1)),
                group_break!(p, get_last_doc(p)),
                ss!(")")
            ),
            all_args_broken_out(p)
        );
    }

    let mut printed_arguments = get_printed_arguments(p, 0);

    if should_break {
        printed_arguments.insert(0, softline!());
        parts.push(Doc::Indent(printed_arguments));
        parts.push(if_break!(p, ",", "", None));
        parts.push(softline!());
    } else {
        parts.extend(printed_arguments);
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

/// * Reference https://github.com/prettier/prettier/blob/main/src/language-js/print/call-arguments.js#L247-L272
fn should_expand_first_arg<'a>(arguments: &Vec<'a, Argument<'a>>) -> bool {
    if arguments.len() != 2 {
        return false;
    }

    let Argument::Expression(first_arg) = &arguments[0] else { return false };
    let Argument::Expression(second_arg) = &arguments[1] else { return false };

    let first_check = match first_arg {
        Expression::FunctionExpression(_) => true,
        Expression::ArrowExpression(arrow) => !arrow.expression,
        _ => false,
    };

    first_check
        && !matches!(
            second_arg,
            Expression::FunctionExpression(_)
                | Expression::ArrowExpression(_)
                | Expression::ConditionalExpression(_)
        )
        && is_hopefully_short_call_argument(second_arg)
        && !could_expand_arg(second_arg, false)
}

fn should_expand_last_arg(args: &Vec<'_, Argument<'_>>) -> bool {
    let Some(Argument::Expression(last_arg)) = args.last() else { return false };

    let penultimate_arg = if args.len() >= 2 { Some(&args[args.len() - 2]) } else { None };

    could_expand_arg(last_arg, false)
        && (penultimate_arg.is_none() || matches!(last_arg, arg))
        && (args.len() != 2
            || !matches!(
                penultimate_arg,
                Some(Argument::Expression(Expression::ArrowExpression(_)))
            )
            || !matches!(last_arg, Expression::ArrayExpression(_)))
}

fn is_hopefully_short_call_argument(node: &Expression) -> bool {
    if let Expression::ParenthesizedExpression(expr) = node {
        return is_hopefully_short_call_argument(&expr.expression);
    }

    if node.is_call_like_expression() {
        return !match node {
            Expression::CallExpression(call) => call.arguments.len() > 1,
            Expression::NewExpression(call) => call.arguments.len() > 1,
            Expression::ImportExpression(call) => call.arguments.len() > 0,
            _ => false,
        };
    }

    if let Expression::BinaryExpression(expr) = node {
        return is_simple_call_argument(&expr.left, 1) && is_simple_call_argument(&expr.right, 1);
    }

    matches!(node, Expression::RegExpLiteral(_)) || is_simple_call_argument(node, 2)
}

fn is_simple_call_argument(node: &Expression, depth: usize) -> bool {
    if let Expression::RegExpLiteral(literal) = node {
        return literal.regex.pattern.len() <= 5;
    }

    if node.is_literal() || is_string_word_type(node) {
        return true;
    }

    let is_child_simple = |node: &Expression| {
        if depth <= 1 {
            return false;
        }
        is_simple_call_argument(node, depth - 1)
    };

    if let Expression::TemplateLiteral(literal) = node {
        return literal.quasis.iter().all(|element| !element.value.raw.contains('\n'))
            && literal.expressions.iter().all(|expr| is_child_simple(expr));
    }

    if let Expression::ObjectExpression(expr) = node {
        return expr.properties.iter().all(|p| {
            if let ObjectPropertyKind::ObjectProperty(property) = p {
                !property.computed && (property.shorthand || is_child_simple(&property.value))
            } else {
                false
            }
        });
    }

    if let Expression::ArrayExpression(expr) = node {
        return expr.elements.iter().all(
            |x| matches!(x, ArrayExpressionElement::Expression(expr) if is_child_simple(expr)),
        );
    }

    if node.is_call_expression() {
        if let Expression::ImportExpression(expr) = node {
            return expr.arguments.len() <= depth && expr.arguments.iter().all(is_child_simple);
        } else if let Expression::CallExpression(expr) = node {
            if is_simple_call_argument(&expr.callee, depth) {
                return expr.arguments.len() <= depth
                    && expr.arguments.iter().all(|arg| {
                        if let Argument::Expression(expr) = arg {
                            is_child_simple(expr)
                        } else {
                            false
                        }
                    });
            }
        } else if let Expression::NewExpression(expr) = node {
            if is_simple_call_argument(&expr.callee, depth) {
                return expr.arguments.len() <= depth
                    && expr.arguments.iter().all(|arg| {
                        if let Argument::Expression(expr) = arg {
                            is_child_simple(expr)
                        } else {
                            false
                        }
                    });
            }
        }
        return false;
    }

    let check_member_expression = |expr: &MemberExpression<'_>| {
        if let MemberExpression::StaticMemberExpression(expr) = expr {
            return is_simple_call_argument(&expr.object, depth);
        }
        if let MemberExpression::ComputedMemberExpression(expr) = expr {
            return is_simple_call_argument(&expr.object, depth)
                && is_simple_call_argument(&expr.expression, depth);
        }
        if let MemberExpression::PrivateFieldExpression(expr) = expr {
            return is_simple_call_argument(&expr.object, depth);
        }
        false
    };

    if let Expression::MemberExpression(expr) = node {
        return check_member_expression(expr);
    }

    if let Expression::UnaryExpression(expr) = node {
        return matches!(
            expr.operator,
            UnaryOperator::LogicalNot
                | UnaryOperator::UnaryNegation
                | UnaryOperator::UnaryPlus
                | UnaryOperator::BitwiseNot
        ) && is_simple_call_argument(&expr.argument, depth);
    }

    if let Expression::UpdateExpression(expr) = node {
        return match &expr.argument {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(target) => true,
            SimpleAssignmentTarget::MemberAssignmentTarget(target) => {
                check_member_expression(target)
            }
            _ => return false,
        };
    }

    false
}

fn could_expand_arg(arg: &Expression, arrow_chain_recursion: bool) -> bool {
    match arg {
        Expression::ObjectExpression(expr) => expr.properties.len() > 0,
        Expression::ArrayExpression(expr) => expr.elements.len() > 0,
        Expression::BinaryExpression(expr) => could_expand_arg(&expr.left, arrow_chain_recursion),
        Expression::FunctionExpression(_) => true,
        Expression::ArrowExpression(expr) => {
            if !expr.expression {
                return true;
            }
            let Statement::ExpressionStatement(statement) = &expr.body.statements[0] else {
                return false;
            };

            match &statement.expression {
                Expression::ArrayExpression(expr) => could_expand_arg(&statement.expression, true),
                Expression::ObjectExpression(_) => true,
                Expression::CallExpression(_) | Expression::ConditionalExpression(_) => {
                    !arrow_chain_recursion
                }
                _ => false,
            }
        }
        _ => false,
    }
}

fn is_string_word_type(node: &Expression) -> bool {
    matches!(node, Expression::Identifier(_) | Expression::ThisExpression(_) | Expression::Super(_))
}
