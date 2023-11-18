use std::num::NonZeroU32;

use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, AssignmentTargetPattern, AssignmentTargetProperty,
    BindingPatternKind, Expression, TSType, VariableDeclarator,
};

use crate::{
    array,
    doc::{Doc, Group, IndentIfBreak},
    group, indent, indent_if_break, ss, Format, Prettier,
};

pub fn print_assignment_expression<'a>(
    p: &mut Prettier<'a>,
    expr: &AssignmentExpression<'a>,
) -> Doc<'a> {
    let left_doc = expr.left.format(p);
    print_assignment(
        p,
        AssignmentLikeNode::AssignmentExpression(expr),
        left_doc,
        array![p, ss!(" "), Doc::Str(expr.operator.as_str())],
        Some(&expr.right),
    )
}

pub fn print_variable_declarator<'a>(
    p: &mut Prettier<'a>,
    var_decl: &VariableDeclarator<'a>,
) -> Doc<'a> {
    let left_doc = var_decl.id.format(p);
    print_assignment(
        p,
        AssignmentLikeNode::VariableDeclarator(var_decl),
        left_doc,
        Doc::Str(" ="),
        var_decl.init.as_ref(),
    )
}

#[derive(Debug, Clone, Copy)]
enum AssignmentLikeNode<'a, 'b> {
    AssignmentExpression(&'b AssignmentExpression<'a>),
    VariableDeclarator(&'b VariableDeclarator<'a>),
}

fn print_assignment<'a>(
    p: &mut Prettier<'a>,
    node: AssignmentLikeNode<'a, '_>,
    left_doc: Doc<'a>,
    op: Doc<'a>,
    right_expr: Option<&Expression<'a>>,
) -> Doc<'a> {
    let layout = choose_layout(p, &node, &left_doc, right_expr);

    // TODO: set the layout in options so that when we print the right-hand side, we can refer to it.
    let right_doc = if let Some(expr) = right_expr { expr.format(p) } else { array!(p,) };

    match layout {
        Layout::BreakAfterOperator => {
            group!(p, group!(p, left_doc), op, group!(p, indent!(p, Doc::Line, right_doc)))
        }
        Layout::NeverBreakAfterOperator => {
            group!(p, group!(p, left_doc), op, ss!(" "), group!(p, right_doc))
        }
        // First break right-hand side, then after operator
        Layout::Fluid => {
            let group_id = p.new_group_id();

            let g = {
                let mut parts = p.vec();
                // group!(p, indent!(p, Doc::Line));
                parts.push(indent!(p, Doc::Line));

                Group { docs: parts, group_id: Some(group_id) }
            };

            group!(
                p,
                group!(p, left_doc),
                op,
                Doc::Group(g),
                Doc::IndentIfBreak(IndentIfBreak {
                    contents: p.boxed(array!(p, right_doc)),
                    group_id: Some(group_id)
                })
            )
        }
        Layout::BreakLhs => {
            group!(p, left_doc, op, ss!(" "), group!(p, right_doc))
        }
        // Parts of assignment chains aren't wrapped in groups.
        // Once one of them breaks, the chain breaks too.
        Layout::Chain => {
            array!(p, group!(p, left_doc), op, Doc::Line, right_doc)
        }
        Layout::ChainTail => {
            array!(p, group!(p, left_doc), op, indent!(p, Doc::Line, right_doc))
        }
        Layout::ChainTailArrowChain => {
            array!(p, group!(p, left_doc), op, right_doc)
        }
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
    assignment_like_node: &AssignmentLikeNode<'a, '_>,
    left_doc: &Doc<'a>,
    right_expr: Option<&Expression<'a>>,
) -> Layout {
    let Some(right_expr) = right_expr else { return Layout::OnlyLeft };

    // Short assignment chains (only 2 segments) are NOT formatted as chains.
    //   1) a = b = c; (expression statements)
    //   2) var/let/const a = b = c;

    let is_tail = !is_assignment(right_expr);

    let should_use_chain_formatting = {
        // TODO, we need

        //     const shouldUseChainFormatting = path.match(
        //       isAssignment,
        //       isAssignmentOrVariableDeclarator,
        //       (node) =>
        //         !isTail ||
        //         (node.type !== "ExpressionStatement" &&
        //           node.type !== "VariableDeclaration"),
        //     );
        false
        // true
    };

    if should_use_chain_formatting {
        if !is_tail {
            return Layout::Chain;
        }

        if matches!(right_expr, Expression::ArrowExpression(_)) {
            return Layout::ChainTailArrowChain;
        }
        return Layout::ChainTail;
    }
    let is_head_of_long_chain = !is_tail
        && if let Expression::AssignmentExpression(assignment_expr) = right_expr {
            is_assignment(&assignment_expr.right)
        } else {
            false
        };

    // TODO: check `has_leading_own_line_comment`
    if is_head_of_long_chain {
        return Layout::BreakAfterOperator;
    }

    if let Expression::CallExpression(call_expr) = &right_expr {
        if let Expression::Identifier(ident) = &call_expr.callee {
            if ident.name == "require" {
                return Layout::NeverBreakAfterOperator;
            }
        }
    }

    let can_break_left_doc = can_break(left_doc);

    if is_complex_destructuring(assignment_like_node)
        || is_complex_type_alias_params(assignment_like_node)
        || has_complex_type_annotation(assignment_like_node)
        || (is_arrow_function_variable_declarator(assignment_like_node) && can_break_left_doc)
    {
        return Layout::BreakLhs;
    }

    // wrapping object properties with very short keys usually doesn't add much value
    let has_short_key = is_object_property_with_short_key(assignment_like_node); //, left_doc, options);

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
                    | Expression::ClassExpression(_)
            )
            || is_numeric_literal(right_expr))
    {
        return Layout::NeverBreakAfterOperator;
    }

    Layout::Fluid
}

fn is_assignment(expr: &Expression) -> bool {
    matches!(expr, Expression::AssignmentExpression(_))
}

fn is_complex_destructuring(expr: &AssignmentLikeNode) -> bool {
    // TODO: can we refactor, remove duplication, remove `todo!()`
    match expr {
        AssignmentLikeNode::AssignmentExpression(assignment_expr) => {
            if let AssignmentTarget::AssignmentTargetPattern(
                AssignmentTargetPattern::ObjectAssignmentTarget(obj_assignment_target),
            ) = &assignment_expr.left
            {
                if obj_assignment_target.properties.len() > 2
                    && obj_assignment_target.properties.iter().any(|property| {
                        if let AssignmentTargetProperty::AssignmentTargetPropertyProperty(v) =
                            property
                        {
                            // TODO: FIXME
                            //(!property.shorthand || property.value?.type === "AssignmentPattern"),
                        }

                        true
                    })
                {
                    return true;
                }
            }

            false
        }
        AssignmentLikeNode::VariableDeclarator(variable_declarator) => {
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
    }
}

fn get_type_parameters_from_type_alias<'a, 'b>(
    r#type: &'a TSType<'b>,
) -> Option<&'a oxc_allocator::Vec<'b, TSType<'b>>> {
    let TSType::TSTypeReference(type_ref) = &r#type else { return None };

    type_ref.type_parameters.as_ref().map(|type_params| &type_params.params)
}

fn is_complex_type_alias_params(node: &AssignmentLikeNode) -> bool {
    // TODO
    false
}

fn has_complex_type_annotation(node: &AssignmentLikeNode) -> bool {
    let AssignmentLikeNode::VariableDeclarator(v) = node else { return false };

    let Some(type_annotation) = &v.id.type_annotation else { return false };

    let Some(type_params) = get_type_parameters_from_type_alias(&type_annotation.type_annotation)
    else {
        return false;
    };

    return type_params.len() > 1
        && type_params.iter().any(|param| {
            if matches!(param, TSType::TSConditionalType(_)) {
                return true;
            }

            let Some(type_params) =
                get_type_parameters_from_type_alias(&type_annotation.type_annotation)
            else {
                return false;
            };

            type_params.len() > 0
        });
}

fn is_arrow_function_variable_declarator(node: &AssignmentLikeNode) -> bool {
    let AssignmentLikeNode::VariableDeclarator(v) = node else { return false };

    matches!(v.init, Some(Expression::ArrowExpression(_)))
}

fn is_numeric_literal(expr: &Expression) -> bool {
    matches!(expr, Expression::NumberLiteral(_))
}

fn can_break(expr: &Doc) -> bool {
    true
}

fn is_object_property_with_short_key(node: &AssignmentLikeNode) -> bool {
    true
}

fn should_break_after_operator(
    p: &Prettier<'_>,
    right_expr: &Expression<'_>,
    has_short_key: bool,
) -> bool {
    false
}
