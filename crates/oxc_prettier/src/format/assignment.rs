use oxc_ast::{
    ast::{
        AccessorProperty, Argument, AssignmentExpression, AssignmentTarget,
        AssignmentTargetPattern, AssignmentTargetProperty, BindingPatternKind, Expression,
        ObjectProperty, PropertyDefinition, PropertyKind, Statement, TSTypeParameterInstantiation,
        VariableDeclarator,
    },
    AstKind,
};

use crate::{
    array,
    doc::{Doc, DocBuilder, Group, IndentIfBreak},
    group, indent, line, ss, Format, Prettier,
};

use super::{binaryish::should_inline_logical_expression, class::ClassMemberish};

pub(super) fn print_assignment_expression<'a>(
    p: &mut Prettier<'a>,
    assignment_expr: &AssignmentExpression<'a>,
) -> Doc<'a> {
    let left_doc = assignment_expr.left.format(p);
    print_assignment(
        p,
        AssignmentLikeNode::AssignmentExpression(assignment_expr),
        left_doc,
        array![p, ss!(" "), Doc::Str(assignment_expr.operator.as_str())],
        Some(&assignment_expr.right),
    )
}

pub(super) fn print_variable_declarator<'a>(
    p: &mut Prettier<'a>,
    variable_declarator: &VariableDeclarator<'a>,
) -> Doc<'a> {
    let left_doc = variable_declarator.id.format(p);
    print_assignment(
        p,
        AssignmentLikeNode::VariableDeclarator(variable_declarator),
        left_doc,
        Doc::Str(" ="),
        variable_declarator.init.as_ref(),
    )
}

#[derive(Debug, Clone, Copy)]
pub(super) enum AssignmentLikeNode<'a, 'b> {
    AssignmentExpression(&'b AssignmentExpression<'a>),
    VariableDeclarator(&'b VariableDeclarator<'a>),
    PropertyDefinition(&'b PropertyDefinition<'a>),
    AccessorProperty(&'b AccessorProperty<'a>),
    ObjectProperty(&'b ObjectProperty<'a>),
}

impl<'a, 'b> From<ClassMemberish<'a, 'b>> for AssignmentLikeNode<'a, 'b> {
    fn from(class_memberish: ClassMemberish<'a, 'b>) -> Self {
        match class_memberish {
            ClassMemberish::PropertyDefinition(property_def) => {
                Self::PropertyDefinition(property_def)
            }
            ClassMemberish::AccessorProperty(accessor_prop) => {
                Self::AccessorProperty(accessor_prop)
            }
        }
    }
}

pub(super) fn print_assignment<'a>(
    p: &mut Prettier<'a>,
    node: AssignmentLikeNode<'a, '_>,
    left_doc: Doc<'a>,
    op: Doc<'a>,
    right_expr: Option<&Expression<'a>>,
) -> Doc<'a> {
    let layout = choose_layout(p, &node, &left_doc, right_expr);

    // TODO: set the layout in options so that when we print the right-hand side, we can refer to it.
    let right_doc = if let Some(expr) = right_expr { expr.format(p) } else { Doc::Array(p.vec()) };

    match layout {
        Layout::BreakAfterOperator => {
            group!(p, group!(p, left_doc), op, group!(p, indent!(p, line!(), right_doc)))
        }
        Layout::NeverBreakAfterOperator => {
            group!(p, group!(p, left_doc), op, ss!(" "), group!(p, right_doc))
        }
        // First break right-hand side, then after operator
        Layout::Fluid => {
            let group_id = p.next_id();

            let after_op = {
                let mut parts = p.vec();
                parts.push(indent!(p, line!()));
                Doc::Group(Group::new(parts, false).with_id(group_id))
            };

            let right_doc = {
                let mut parts = p.vec();
                parts.push(group!(p, right_doc));
                Doc::IndentIfBreak(IndentIfBreak::new(parts).with_id(group_id))
            };

            group!(p, group!(p, left_doc), op, after_op, right_doc)
        }
        Layout::BreakLhs => {
            group!(p, left_doc, op, ss!(" "), group!(p, right_doc))
        }
        // Parts of assignment chains aren't wrapped in groups.
        // Once one of them breaks, the chain breaks too.
        Layout::Chain => {
            array!(p, group!(p, left_doc), op, line!(), right_doc)
        }
        Layout::ChainTail => {
            array!(p, group!(p, left_doc), op, indent!(p, line!(), right_doc))
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

    let should_use_chain_formatting =
        matches!(assignment_like_node, AssignmentLikeNode::AssignmentExpression(_))
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
        } else if let Expression::ArrowExpression(arrow_expr) = right_expr {
            if let Some(Statement::ExpressionStatement(expr_stmt)) =
                arrow_expr.body.statements.first()
            {
                if let Expression::ArrowExpression(_) = expr_stmt.expression {
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
                    | Expression::NumberLiteral(_)
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

fn is_complex_destructuring(expr: &AssignmentLikeNode) -> bool {
    match expr {
        AssignmentLikeNode::AssignmentExpression(assignment_expr) => {
            if let AssignmentTarget::AssignmentTargetPattern(
                AssignmentTargetPattern::ObjectAssignmentTarget(obj_assignment_target),
            ) = &assignment_expr.left
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
        _ => false,
    }
}

fn is_complex_type_alias_params(expr: &AssignmentLikeNode) -> bool {
    false
}

fn has_complex_type_annotation(expr: &AssignmentLikeNode) -> bool {
    false
}

fn is_arrow_function_variable_declarator(expr: &AssignmentLikeNode) -> bool {
    match expr {
        AssignmentLikeNode::VariableDeclarator(variable_declarator) => {
            if let Some(Expression::ArrowExpression(_)) = &variable_declarator.init {
                return true;
            }
            false
        }
        AssignmentLikeNode::AssignmentExpression(_)
        | AssignmentLikeNode::PropertyDefinition(_)
        | AssignmentLikeNode::ObjectProperty(_)
        | AssignmentLikeNode::AccessorProperty(_) => false,
    }
}

fn is_object_property_with_short_key<'a>(
    p: &Prettier<'a>,
    expr: &AssignmentLikeNode<'a, '_>,
    left_doc: &Doc<'a>,
) -> bool {
    let AssignmentLikeNode::ObjectProperty(object_prop) = expr else { return false };

    if object_prop.method || object_prop.kind != PropertyKind::Init {
        return false;
    }

    true
}

/// https://github.com/prettier/prettier/blob/eebf0e4b5ec8ac24393c56ced4b4819d4c551f31/src/language-js/print/assignment.js#L182
fn should_break_after_operator<'a>(
    p: &Prettier<'a>,
    expr: &Expression<'a>,
    has_short_key: bool,
) -> bool {
    if matches!(expr, Expression::BinaryExpression(_) | Expression::LogicalExpression(_))
        && !should_inline_logical_expression(expr)
    {
        return true;
    }

    match expr {
        Expression::SequenceExpression(_) => return true,
        Expression::ConditionalExpression(conditional_expr) => {
            return matches!(
                conditional_expr.test,
                Expression::LogicalExpression(_) | Expression::BinaryExpression(_)
            ) && !should_inline_logical_expression(&conditional_expr.test);
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
            Expression::MemberExpression(member_expr) => {
                is_chain_expression = true;
                Some(member_expr.object())
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
