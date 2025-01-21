use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstKind};

use crate::{
    array,
    format::print::{binaryish, class},
    group, indent, indent_if_break,
    ir::Doc,
    line, text, Format, Prettier,
};

#[derive(Debug, Clone, Copy)]
pub enum AssignmentLike<'a, 'b> {
    AssignmentExpression(&'b AssignmentExpression<'a>),
    VariableDeclarator(&'b VariableDeclarator<'a>),
    PropertyDefinition(&'b PropertyDefinition<'a>),
    AccessorProperty(&'b AccessorProperty<'a>),
    ObjectProperty(&'b ObjectProperty<'a>),
    ImportAttribute(&'b ImportAttribute<'a>),
}

pub fn print_assignment<'a>(
    p: &mut Prettier<'a>,
    node: AssignmentLike<'a, '_>,
    left_doc: Doc<'a>,
    op: Doc<'a>,
    right_expr: Option<&Expression<'a>>,
) -> Doc<'a> {
    let layout = choose_layout(p, &node, &left_doc, right_expr);

    // TODO: set the layout in options so that when we print the right-hand side, we can refer to it.
    let right_doc = if let Some(expr) = right_expr { expr.format(p) } else { text!("") };

    match layout {
        Layout::BreakAfterOperator => {
            group!(p, [group!(p, [left_doc]), op, group!(p, [indent!(p, [line!(), right_doc])])])
        }
        Layout::NeverBreakAfterOperator => {
            group!(p, [group!(p, [left_doc]), op, text!(" "), right_doc])
        }
        // First break right-hand side, then after operator
        Layout::Fluid => {
            let group_id = p.next_id();

            let after_op = {
                let mut parts = Vec::new_in(p.allocator);
                parts.push(indent!(p, [line!()]));
                group!(p, parts, false, Some(group_id))
            };

            let right_doc = { indent_if_break!(p, group!(p, [right_doc]), group_id) };

            group!(p, [group!(p, [left_doc]), op, after_op, right_doc])
        }
        Layout::BreakLhs => group!(p, [left_doc, op, text!(" "), group!(p, [right_doc])]),
        // Parts of assignment chains aren't wrapped in groups.
        // Once one of them breaks, the chain breaks too.
        Layout::Chain => array!(p, [group!(p, [left_doc]), op, line!(), right_doc]),
        Layout::ChainTail => {
            array!(p, [group!(p, [left_doc]), op, indent!(p, [line!(), right_doc])])
        }
        Layout::ChainTailArrowChain => array!(p, [group!(p, [left_doc]), op, right_doc]),
        Layout::OnlyLeft => left_doc,
    }
}

#[derive(Debug)]
enum Layout {
    OnlyLeft,
    Chain,
    ChainTailArrowChain,
    ChainTail,
    BreakAfterOperator,
    NeverBreakAfterOperator,
    BreakLhs,
    Fluid,
}

fn choose_layout<'a>(
    p: &Prettier<'a>,
    assignment_like_node: &AssignmentLike<'a, '_>,
    left_doc: &Doc<'a>,
    right_expr: Option<&Expression<'a>>,
) -> Layout {
    let Some(right_expr) = right_expr else { return Layout::OnlyLeft };

    // Short assignment chains (only 2 segments) are NOT formatted as chains.
    //   1) a = b = c; (expression statements)
    //   2) var/let/const a = b = c;

    let is_tail = !is_assignment(right_expr);

    let should_use_chain_formatting =
        matches!(assignment_like_node, AssignmentLike::AssignmentExpression(_))
            && matches!(
                p.parent_kind(),
                AstKind::AssignmentExpression(_) | AstKind::VariableDeclarator(_)
            )
            && (!is_tail
                || !matches!(
                    p.parent_parent_kind(),
                    Some(AstKind::ExpressionStatement(_) | AstKind::VariableDeclaration(_))
                ));

    if should_use_chain_formatting {
        if !is_tail {
            return Layout::Chain;
        } else if let Expression::ArrowFunctionExpression(arrow_expr) = right_expr {
            if let Some(Statement::ExpressionStatement(expr_stmt)) =
                arrow_expr.body.statements.first()
            {
                if let Expression::ArrowFunctionExpression(_) = expr_stmt.expression {
                    return Layout::ChainTailArrowChain;
                }
            }
        }
        return Layout::ChainTail;
    }

    let is_head_of_long_chain = !is_tail && is_assignment(right_expr);

    if is_head_of_long_chain {
        return Layout::BreakAfterOperator;
    }

    if let AssignmentLike::ImportAttribute(import_attr) = assignment_like_node {
        return Layout::NeverBreakAfterOperator;
    }
    if let Expression::CallExpression(call_expr) = right_expr {
        if let Expression::Identifier(ident) = &call_expr.callee {
            if ident.name == "require" {
                return Layout::NeverBreakAfterOperator;
            }
        }
    }

    let can_break_left_doc = false; // = canBreak(leftDoc);

    if is_complex_destructuring(assignment_like_node)
        || is_complex_type_alias_params(assignment_like_node)
        || has_complex_type_annotation(assignment_like_node)
        || (is_arrow_function_variable_declarator(assignment_like_node) && can_break_left_doc)
    {
        return Layout::BreakLhs;
    }

    // wrapping object properties with very short keys usually doesn't add much value
    let has_short_key = is_object_property_with_short_key(p, assignment_like_node, left_doc);

    if should_break_after_operator(p, right_expr, has_short_key) {
        return Layout::BreakAfterOperator;
    }

    if !can_break_left_doc
        && (has_short_key
            || matches!(
                right_expr,
                Expression::TemplateLiteral(_)
                    | Expression::TaggedTemplateExpression(_)
                    | Expression::BooleanLiteral(_)
                    | Expression::NumericLiteral(_)
                    | Expression::ClassExpression(_)
            ))
    {
        return Layout::NeverBreakAfterOperator;
    }

    Layout::Fluid
}

fn is_assignment(expr: &Expression) -> bool {
    matches!(expr, Expression::AssignmentExpression(_))
}

fn is_complex_destructuring(expr: &AssignmentLike) -> bool {
    match expr {
        AssignmentLike::AssignmentExpression(assignment_expr) => {
            if let AssignmentTarget::ObjectAssignmentTarget(obj_assignment_target) =
                &assignment_expr.left
            {
                if obj_assignment_target.properties.len() > 2
                    && obj_assignment_target.properties.iter().any(|property| {
                        matches!(
                            property,
                            AssignmentTargetProperty::AssignmentTargetPropertyProperty(v)
                        )
                    })
                {
                    return true;
                }
            }

            false
        }
        AssignmentLike::VariableDeclarator(variable_declarator) => {
            if let BindingPatternKind::ObjectPattern(object_pat) = &variable_declarator.id.kind {
                if object_pat.properties.len() > 2
                    && object_pat.properties.iter().any(|property| {
                        !property.shorthand
                            || matches!(
                                property.value.kind,
                                BindingPatternKind::AssignmentPattern(_)
                            )
                    })
                {
                    return true;
                }
            }

            false
        }
        _ => false,
    }
}

fn is_complex_type_alias_params(expr: &AssignmentLike) -> bool {
    false
}

fn has_complex_type_annotation(expr: &AssignmentLike) -> bool {
    false
}

pub fn is_arrow_function_variable_declarator(expr: &AssignmentLike) -> bool {
    match expr {
        AssignmentLike::VariableDeclarator(variable_declarator) => {
            if let Some(Expression::ArrowFunctionExpression(_)) = &variable_declarator.init {
                return true;
            }
            false
        }
        _ => false,
    }
}

fn is_object_property_with_short_key<'a>(
    p: &Prettier<'a>,
    expr: &AssignmentLike<'a, '_>,
    left_doc: &Doc<'a>,
) -> bool {
    let AssignmentLike::ObjectProperty(object_prop) = expr else { return false };

    if object_prop.method || object_prop.kind != PropertyKind::Init {
        return false;
    }

    true
}

/// <https://github.com/prettier/prettier/blob/eebf0e4b5ec8ac24393c56ced4b4819d4c551f31/src/language-js/print/assignment.js#L182>
fn should_break_after_operator<'a>(
    p: &Prettier<'a>,
    expr: &Expression<'a>,
    has_short_key: bool,
) -> bool {
    if matches!(expr, Expression::BinaryExpression(_) | Expression::LogicalExpression(_))
        && !binaryish::should_inline_logical_expression(expr)
    {
        return true;
    }

    match expr {
        Expression::SequenceExpression(_) => return true,
        Expression::ConditionalExpression(conditional_expr) => {
            return matches!(
                conditional_expr.test,
                Expression::LogicalExpression(_) | Expression::BinaryExpression(_)
            ) && !binaryish::should_inline_logical_expression(&conditional_expr.test);
        }
        Expression::ClassExpression(class_expr) => {
            if class_expr.decorators.len() > 0 {
                return true;
            }
        }
        _ => {}
    }

    if has_short_key {
        return false;
    }

    let mut current_expr = expr;

    while let Expression::UnaryExpression(_) | Expression::TSNonNullExpression(_) = current_expr {
        current_expr = match current_expr {
            Expression::UnaryExpression(unary) => &unary.argument,
            Expression::TSNonNullExpression(non_null_expr) => &non_null_expr.expression,
            // SAFETY: the `while` loop above ensures that `current_expr` is either a `UnaryExpression` or a `TSNonNullExpression`.
            _ => unreachable!(),
        };
    }

    if current_expr.is_string_literal() || is_poorly_breakable_member_or_call_chain(p, expr) {
        return true;
    };

    false
}

fn is_poorly_breakable_member_or_call_chain<'a>(p: &Prettier<'a>, expr: &Expression<'a>) -> bool {
    let mut is_chain_expression = false;
    let mut is_ident_or_this_expr = false;
    let mut call_expressions = vec![];

    let mut expression = Some(expr);

    while let Some(node) = expression.take() {
        expression = match node {
            Expression::TSNonNullExpression(non_null_expr) => Some(&non_null_expr.expression),
            Expression::CallExpression(call_expr) => {
                is_chain_expression = true;
                let callee = &call_expr.callee;
                call_expressions.push(call_expr);
                Some(callee)
            }
            match_member_expression!(Expression) => {
                is_chain_expression = true;
                Some(node.to_member_expression().object())
            }
            Expression::Identifier(_) | Expression::ThisExpression(_) => {
                is_ident_or_this_expr = true;
                break;
            }
            _ => {
                break;
            }
        }
    }

    if !is_chain_expression || !is_ident_or_this_expr {
        return false;
    }

    for call_expression in call_expressions {
        let is_poorly_breakable_call = call_expression.arguments.len() == 0
            || (call_expression.arguments.len() == 1
                && is_lone_short_argument(&call_expression.arguments[0]));

        if !is_poorly_breakable_call {
            return false;
        }

        if let Some(type_parameters) = &call_expression.type_parameters {
            return is_complex_type_arguments(type_parameters);
        }
    }

    true
}

fn is_lone_short_argument(arg: &Argument) -> bool {
    false
}

fn is_complex_type_arguments(type_parameters: &TSTypeParameterInstantiation) -> bool {
    false
}
