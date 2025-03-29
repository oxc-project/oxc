use oxc_ast::{AstKind, ast::*};

pub fn has_naked_left_side(kind: AstKind<'_>) -> bool {
    matches!(
        kind,
        AstKind::AssignmentExpression(_)
            | AstKind::BinaryExpression(_)
            | AstKind::LogicalExpression(_)
            | AstKind::ConditionalExpression(_)
            | AstKind::CallExpression(_)
            | AstKind::MemberExpression(_)
            | AstKind::SequenceExpression(_)
            | AstKind::TaggedTemplateExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::ChainExpression(_)
    ) || matches!(kind, AstKind::UpdateExpression(e) if !e.prefix)
}

pub fn get_left_side_path_name(kind: AstKind<'_>) -> AstKind<'_> {
    match kind {
        AstKind::AssignmentExpression(e) => AstKind::AssignmentTarget(&e.left),
        AstKind::BinaryExpression(e) => AstKind::from_expression(&e.left),
        AstKind::CallExpression(e) => AstKind::from_expression(&e.callee),
        AstKind::ChainExpression(e) => match &e.expression {
            ChainElement::CallExpression(e) => AstKind::from_expression(&e.callee),
            ChainElement::StaticMemberExpression(e) => AstKind::from_expression(&e.object),
            ChainElement::ComputedMemberExpression(e) => AstKind::from_expression(&e.object),
            ChainElement::PrivateFieldExpression(e) => AstKind::from_expression(&e.object),
            ChainElement::TSNonNullExpression(e) => AstKind::from_expression(&e.expression),
        },
        AstKind::ConditionalExpression(e) => AstKind::from_expression(&e.test),
        AstKind::LogicalExpression(e) => AstKind::from_expression(&e.left),
        AstKind::MemberExpression(e) => AstKind::from_expression(e.object()),
        AstKind::SequenceExpression(e) => AstKind::from_expression(&e.expressions[0]),
        AstKind::TaggedTemplateExpression(e) => AstKind::from_expression(&e.tag),
        AstKind::UpdateExpression(e) => AstKind::from_expression(
            // TODO: If this causes a panic, make sure to handle `Option`
            e.argument.get_expression().expect("UpdateExpression.argument is missing"),
        ),
        AstKind::TSAsExpression(e) => AstKind::from_expression(&e.expression),
        AstKind::TSNonNullExpression(e) => AstKind::from_expression(&e.expression),
        AstKind::TSSatisfiesExpression(e) => AstKind::from_expression(&e.expression),
        _ => unreachable!("get_left_side_path_name(): need to handle {}", kind.debug_name()),
    }
}

pub fn get_left_side_expression<'a>(expr: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    match expr {
        Expression::AssignmentExpression(e) => e.left.get_expression(),
        Expression::BinaryExpression(e) => Some(&e.left),
        Expression::CallExpression(e) => Some(&e.callee),
        Expression::ChainExpression(e) => match &e.expression {
            ChainElement::CallExpression(e) => Some(&e.callee),
            ChainElement::StaticMemberExpression(e) => Some(&e.object),
            ChainElement::ComputedMemberExpression(e) => Some(&e.object),
            ChainElement::PrivateFieldExpression(e) => Some(&e.object),
            ChainElement::TSNonNullExpression(e) => Some(&e.expression),
        },
        Expression::ConditionalExpression(e) => Some(&e.test),
        Expression::LogicalExpression(e) => Some(&e.left),
        Expression::StaticMemberExpression(e) => Some(&e.object),
        Expression::ComputedMemberExpression(e) => Some(&e.object),
        Expression::PrivateFieldExpression(e) => Some(&e.object),
        Expression::SequenceExpression(e) => Some(&e.expressions[0]),
        Expression::TaggedTemplateExpression(e) => Some(&e.tag),
        Expression::UpdateExpression(e) => e.argument.get_expression(),
        Expression::TSAsExpression(e) => Some(&e.expression),
        Expression::TSNonNullExpression(e) => Some(&e.expression),
        Expression::TSSatisfiesExpression(e) => Some(&e.expression),
        _ => unreachable!("get_left_side_expression: need to handle"),
    }
}
