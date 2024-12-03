use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{
    format::{
        call_expression::{is_commons_js_or_amd_call, CallExpressionLike},
        misc,
    },
    ir::{Doc, DocBuilder},
    p_vec,
    utils::will_break,
    Format, Prettier,
};

pub fn print_call_arguments<'a>(
    p: &mut Prettier<'a>,
    expression: &CallExpressionLike<'a, '_>,
) -> Doc<'a> {
    let mut parts = p.vec();
    parts.push(p.text("("));

    let callee = expression.callee();
    let arguments = expression.arguments();
    let should_break = if matches!(expression, CallExpressionLike::CallExpression(_)) {
        !is_commons_js_or_amd_call(expression.callee(), arguments)
    } else {
        true
    };

    if arguments.is_empty() {
        parts.extend(p.print_inner_comment(Span::new(callee.span().end, expression.span().end)));
        parts.push(p.text(")"));
        return p.array(parts);
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
                arg.push(p.text(","));
                if p.is_next_line_empty(element.span()) {
                    arg.extend(p.hardline());
                    arg.extend(p.hardline());
                } else {
                    arg.push(p.line());
                }
            }
            printed_arguments.push(p.array(arg));
        }
        printed_arguments
    };

    let all_args_broken_out = |p: &mut Prettier<'a>| {
        let mut parts = p.vec();
        parts.push(p.text("("));
        let arguments_doc = get_printed_arguments(p, 0);
        parts.push(p.indent(p_vec!(
            p,
            p.line(),
            p.array(arguments_doc),
            if p.should_print_all_comma() { p.text(",") } else { p.text("") }
        )));
        parts.push(p.line());
        parts.push(p.text(")"));
        p.group_with_opts(p.array(parts), true, None)
    };

    if should_expand_first_arg(arguments) {
        p.args.expand_first_arg = true;
        let mut first_doc = arguments[0].format(p);
        p.args.expand_first_arg = false;

        if will_break(&mut first_doc) {
            let last_doc = get_printed_arguments(p, 1).pop().unwrap();
            let all_args_broken_out_doc = all_args_broken_out(p);

            return p.array(p_vec!(
                p,
                p.break_parent(),
                p.conditional_group(
                    p.array(p_vec!(
                        p,
                        p.text("("),
                        p.group_with_opts(first_doc, true, None),
                        p.text(", "),
                        last_doc,
                        p.text(")"),
                    )),
                    vec![all_args_broken_out_doc],
                    None
                )
            ));
        }
    }

    if should_expand_last_arg(arguments) {
        let mut printed_arguments = get_printed_arguments(p, -1);
        if printed_arguments.iter_mut().any(will_break) {
            return all_args_broken_out(p);
        }

        if !printed_arguments.is_empty() {
            printed_arguments.push(p.text(","));
            printed_arguments.push(p.line());
        }

        let get_last_doc = |p: &mut Prettier<'a>| {
            p.args.expand_last_arg = true;
            let last_doc = arguments.last().unwrap().format(p);
            p.args.expand_last_arg = false;
            last_doc
        };

        let mut last_doc = get_last_doc(p);

        if will_break(&mut last_doc) {
            let all_args_broken_out_doc = all_args_broken_out(p);
            return p.array(p_vec!(
                p,
                p.break_parent(),
                p.conditional_group(
                    p.array(p_vec!(
                        p,
                        p.text("("),
                        p.array(printed_arguments),
                        p.group_with_opts(last_doc, true, None),
                        p.text(")")
                    )),
                    vec![all_args_broken_out_doc],
                    None
                ),
            ));
        }

        let printed_arguments2 = get_printed_arguments(p, -1);
        let last_doc2 = get_last_doc(p);
        let all_args_broken_out_doc = all_args_broken_out(p);
        return p.conditional_group(
            p.array(p_vec!(p, p.text("("), p.array(printed_arguments), last_doc, p.text(")"))),
            vec![
                p.array(p_vec!(
                    p,
                    p.text("("),
                    p.array(printed_arguments2),
                    p.group_with_opts(last_doc2, true, None),
                    p.text(")")
                )),
                all_args_broken_out_doc,
            ],
            None,
        );
    }

    let mut printed_arguments = get_printed_arguments(p, 0);

    if should_break {
        printed_arguments.insert(0, p.softline());
        parts.push(p.indent(printed_arguments));
        parts.push(p.if_break(p.text(","), p.text(""), None));
        parts.push(p.softline());
    } else {
        parts.extend(printed_arguments);
    }
    parts.push(p.text(")"));

    let should_break = should_break
        && arguments.iter().any(|arg| {
            misc::has_new_line_in_range(p.source_text, arg.span().start, arg.span().end)
        });

    p.group_with_opts(p.array(parts), should_break, None)
}

/// * Reference <https://github.com/prettier/prettier/blob/3.3.3/src/language-js/print/call-arguments.js#L247-L272>
fn should_expand_first_arg<'a>(arguments: &Vec<'a, Argument<'a>>) -> bool {
    if arguments.len() != 2 {
        return false;
    }

    match &arguments[0] {
        Argument::FunctionExpression(_) => {}
        Argument::ArrowFunctionExpression(arrow) if !arrow.expression => {}
        _ => return false,
    }

    match &arguments[1] {
        Argument::FunctionExpression(_)
        | Argument::ArrowFunctionExpression(_)
        | Argument::ConditionalExpression(_) => false,
        second_arg if second_arg.is_expression() => {
            let second_arg = second_arg.to_expression();
            is_hopefully_short_call_argument(second_arg) && !could_expand_arg(second_arg, false)
        }
        _ => false,
    }
}

fn should_expand_last_arg(args: &Vec<'_, Argument<'_>>) -> bool {
    let Some(last_arg) = args.last() else { return false };
    let Some(last_arg) = last_arg.as_expression() else { return false };

    let penultimate_arg = if args.len() >= 2 { Some(&args[args.len() - 2]) } else { None };

    could_expand_arg(last_arg, false)
        && (penultimate_arg.is_none() || matches!(last_arg, arg))
        && (args.len() != 2
            || !matches!(penultimate_arg, Some(Argument::ArrowFunctionExpression(_)))
            || !matches!(last_arg, Expression::ArrayExpression(_)))
}

fn is_hopefully_short_call_argument(mut node: &Expression) -> bool {
    while let Expression::ParenthesizedExpression(expr) = node {
        node = &expr.expression;
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
        return expr.elements.iter().all(|elem| elem.as_expression().is_some_and(is_child_simple));
    }

    if node.is_call_expression() {
        if let Expression::ImportExpression(expr) = node {
            return expr.arguments.len() <= depth && expr.arguments.iter().all(is_child_simple);
        } else if let Expression::CallExpression(expr) = node {
            if is_simple_call_argument(&expr.callee, depth) {
                return expr.arguments.len() <= depth
                    && expr.arguments.iter().all(|arg| {
                        if let Some(expr) = arg.as_expression() {
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
                        if let Some(expr) = arg.as_expression() {
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

    if let Some(expr) = node.as_member_expression() {
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
            target @ match_member_expression!(SimpleAssignmentTarget) => {
                check_member_expression(target.to_member_expression())
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
        Expression::ArrowFunctionExpression(expr) => {
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
