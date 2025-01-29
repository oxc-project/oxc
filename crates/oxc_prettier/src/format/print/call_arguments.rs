use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{
    array, break_parent, conditional_group,
    format::print::{
        array::{is_concisely_printed_array, ArrayLike},
        call_expression::{is_commons_js_or_amd_call, CallExpressionLike},
        misc,
    },
    group, hardline, if_break, indent,
    ir::Doc,
    line, softline, text,
    utils::will_break,
    Format, Prettier,
};

pub fn print_call_arguments<'a>(
    p: &mut Prettier<'a>,
    expression: &CallExpressionLike<'a, '_>,
) -> Doc<'a> {
    let arguments = expression.arguments();

    if arguments.is_empty() {
        return array!(p, [text!("("), text!(")")]);
    }

    let mut parts = Vec::new_in(p.allocator);
    parts.push(text!("("));

    #[allow(clippy::cast_sign_loss)]
    let get_printed_arguments = |p: &mut Prettier<'a>, skip_index: isize| {
        let mut printed_arguments = Vec::new_in(p.allocator);
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
            let mut arg = Vec::new_in(p.allocator);
            arg.push(doc);

            if i < len - 1 {
                arg.push(text!(","));
                if p.is_next_line_empty(element.span()) {
                    arg.push(hardline!(p));
                    arg.push(hardline!(p));
                } else {
                    arg.push(line!());
                }
            }
            printed_arguments.push(array!(p, arg));
        }
        printed_arguments
    };

    let all_args_broken_out = |p: &mut Prettier<'a>| {
        let mut parts = Vec::new_in(p.allocator);
        parts.push(text!("("));
        let arguments_doc = get_printed_arguments(p, 0);
        parts.push(indent!(
            p,
            [
                line!(),
                array!(p, arguments_doc),
                if p.should_print_all_comma() { text!(",") } else { text!("") }
            ]
        ));
        parts.push(line!());
        parts.push(text!(")"));
        group!(p, parts, true, None)
    };

    if should_expand_first_arg(arguments) {
        p.args.expand_first_arg = true;
        let mut first_doc = arguments[0].format(p);
        p.args.expand_first_arg = false;

        if will_break(&mut first_doc) {
            let last_doc = get_printed_arguments(p, 1).pop().unwrap();
            let all_args_broken_out_doc = all_args_broken_out(p);

            return array!(
                p,
                [
                    break_parent!(),
                    conditional_group!(
                        p,
                        [
                            array!(
                                p,
                                [
                                    text!("("),
                                    group!(p, [first_doc], true, None),
                                    text!(", "),
                                    last_doc,
                                    text!(")"),
                                ]
                            ),
                            all_args_broken_out_doc
                        ]
                    )
                ]
            );
        }
    }

    if should_expand_last_arg(arguments) {
        let mut printed_arguments = get_printed_arguments(p, -1);
        if printed_arguments.iter_mut().any(will_break) {
            return all_args_broken_out(p);
        }

        if !printed_arguments.is_empty() {
            printed_arguments.push(text!(","));
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
            let all_args_broken_out_doc = all_args_broken_out(p);
            return array!(
                p,
                [
                    break_parent!(),
                    conditional_group!(
                        p,
                        [
                            array!(
                                p,
                                [
                                    text!("("),
                                    array!(p, printed_arguments),
                                    group!(p, [last_doc], true, None),
                                    text!(")")
                                ]
                            ),
                            all_args_broken_out_doc
                        ]
                    ),
                ]
            );
        }

        let printed_arguments2 = get_printed_arguments(p, -1);
        let last_doc2 = get_last_doc(p);
        let all_args_broken_out_doc = all_args_broken_out(p);
        return conditional_group!(
            p,
            [
                array!(p, [text!("("), array!(p, printed_arguments), last_doc, text!(")")]),
                array!(
                    p,
                    [
                        text!("("),
                        array!(p, printed_arguments2),
                        group!(p, [last_doc2], true, None),
                        text!(")")
                    ]
                ),
                all_args_broken_out_doc,
            ]
        );
    }

    let mut printed_arguments = get_printed_arguments(p, 0);

    let should_break = if matches!(expression, CallExpressionLike::CallExpression(_)) {
        !is_commons_js_or_amd_call(expression.callee(), arguments)
    } else {
        true
    };

    if should_break {
        printed_arguments.insert(0, softline!());
        parts.push(indent!(p, printed_arguments));
        parts.push(if_break!(p, text!(",")));
        parts.push(softline!());
    } else {
        parts.extend(printed_arguments);
    }
    parts.push(text!(")"));

    let should_break = should_break
        && arguments.iter().any(|arg| {
            misc::has_new_line_in_range(p.source_text, arg.span().start, arg.span().end)
        });

    group!(p, parts, should_break, None)
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
        && !(args.len() > 1 && is_concisely_printed_array(last_arg))
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
