use oxc_ast::{AstKind, ast::*};
use oxc_data_structures::stack;
use oxc_span::GetSpan;
use oxc_syntax::{
    operator,
    precedence::{GetPrecedence, Precedence},
};

use crate::{
    formatter::parent_stack::ParentStack,
    write::{BinaryLikeExpression, should_flatten},
};

use super::NeedsParentheses;

impl<'a> NeedsParentheses<'a> for Expression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        match self {
            // Expression::BooleanLiteral(it) => it.needs_parentheses(stack),
            // Expression::NullLiteral(it) => it.needs_parentheses(stack),
            Expression::NumericLiteral(it) => it.needs_parentheses(stack),
            // Expression::BigIntLiteral(it) => it.needs_parentheses(stack),
            // Expression::RegExpLiteral(it) => it.needs_parentheses(stack),
            Expression::StringLiteral(it) => it.needs_parentheses(stack),
            // Expression::TemplateLiteral(it) => it.needs_parentheses(stack),
            // Expression::Identifier(it) => it.needs_parentheses(stack),
            // Expression::MetaProperty(it) => it.needs_parentheses(stack),
            // Expression::Super(it) => it.needs_parentheses(stack),
            Expression::ArrayExpression(it) => it.needs_parentheses(stack),
            Expression::ArrowFunctionExpression(it) => it.needs_parentheses(stack),
            Expression::AssignmentExpression(it) => it.needs_parentheses(stack),
            Expression::AwaitExpression(it) => it.needs_parentheses(stack),
            Expression::BinaryExpression(it) => it.needs_parentheses(stack),
            Expression::CallExpression(it) => it.needs_parentheses(stack),
            Expression::ChainExpression(it) => it.needs_parentheses(stack),
            Expression::ClassExpression(it) => it.needs_parentheses(stack),
            Expression::ConditionalExpression(it) => it.needs_parentheses(stack),
            Expression::FunctionExpression(it) => it.needs_parentheses(stack),
            Expression::ImportExpression(it) => it.needs_parentheses(stack),
            Expression::LogicalExpression(it) => it.needs_parentheses(stack),
            Expression::NewExpression(it) => it.needs_parentheses(stack),
            Expression::ObjectExpression(it) => it.needs_parentheses(stack),
            Expression::ParenthesizedExpression(it) => it.needs_parentheses(stack),
            Expression::SequenceExpression(it) => it.needs_parentheses(stack),
            Expression::TaggedTemplateExpression(it) => it.needs_parentheses(stack),
            Expression::ThisExpression(it) => it.needs_parentheses(stack),
            Expression::UnaryExpression(it) => it.needs_parentheses(stack),
            Expression::UpdateExpression(it) => it.needs_parentheses(stack),
            Expression::YieldExpression(it) => it.needs_parentheses(stack),
            Expression::PrivateInExpression(it) => it.needs_parentheses(stack),
            // Expression::JSXElement(it) => it.needs_parentheses(stack),
            // Expression::JSXFragment(it) => it.needs_parentheses(stack),
            Expression::TSAsExpression(it) => it.needs_parentheses(stack),
            Expression::TSSatisfiesExpression(it) => it.needs_parentheses(stack),
            Expression::TSTypeAssertion(it) => it.needs_parentheses(stack),
            Expression::TSNonNullExpression(it) => it.needs_parentheses(stack),
            Expression::TSInstantiationExpression(it) => it.needs_parentheses(stack),
            Expression::V8IntrinsicExpression(it) => it.needs_parentheses(stack),
            match_member_expression!(Expression) => {
                self.to_member_expression().needs_parentheses(stack)
            }
            _ => todo!(),
        }
    }
}

impl<'a> NeedsParentheses<'a> for NumericLiteral<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        if let AstKind::MemberExpression(MemberExpression::StaticMemberExpression(e)) =
            stack.parent()
        {
            return e.object.without_parentheses().span() == self.span;
        }
        false
    }
}

impl<'a> NeedsParentheses<'a> for StringLiteral<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        matches!(stack.parent(), AstKind::ExpressionStatement(_))
    }
}

impl<'a> NeedsParentheses<'a> for ThisExpression {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for ArrayExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for ObjectExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        is_class_extends(stack.parent(), self.span)
            || is_first_in_statement(stack, FirstInStatementMode::ExpressionStatementOrArrow)
    }
}

impl<'a> NeedsParentheses<'a> for TaggedTemplateExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for MemberExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for ComputedMemberExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for StaticMemberExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for PrivateFieldExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for CallExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        // TODO
        matches!(stack.parent(), AstKind::NewExpression(_) | AstKind::ExportDefaultDeclaration(_))
    }
}

impl<'a> NeedsParentheses<'a> for NewExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        is_class_extends(stack.parent(), self.span)
    }
}

impl<'a> NeedsParentheses<'a> for UpdateExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        if self.prefix {
            if let AstKind::UnaryExpression(unary) = stack.parent() {
                let parent_operator = unary.operator;
                let operator = self.operator;
                return (parent_operator == UnaryOperator::UnaryPlus
                    && operator == UpdateOperator::Increment)
                    || (parent_operator == UnaryOperator::UnaryNegation
                        && operator == UpdateOperator::Decrement);
            }
        }
        unary_like_expression_needs_parens(UnaryLike::UpdateExpression(self), stack)
    }
}

impl<'a> NeedsParentheses<'a> for UnaryExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        match stack.parent() {
            AstKind::UnaryExpression(parent_unary) => {
                let parent_operator = parent_unary.operator;
                let operator = self.operator;
                matches!(operator, UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation)
                    && parent_operator == operator
            }
            // A user typing `!foo instanceof Bar` probably intended `!(foo instanceof Bar)`,
            // so format to `(!foo) instance Bar` to what is really happening
            // A user typing `!foo in bar` probably intended `!(foo instanceof Bar)`,
            // so format to `(!foo) in bar` to what is really happening
            AstKind::BinaryExpression(e) if e.operator.is_relational() => true,
            _ => unary_like_expression_needs_parens(UnaryLike::UnaryExpression(self), stack),
        }
    }
}

impl<'a> NeedsParentheses<'a> for BinaryExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        binary_like_needs_parens(BinaryLikeExpression::BinaryExpression(self), stack)
    }
}

impl<'a> NeedsParentheses<'a> for PrivateInExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for LogicalExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        let parent = stack.parent();
        if let AstKind::LogicalExpression(parent) = parent {
            parent.operator != self.operator
        } else if self.operator.is_coalesce() && matches!(parent, AstKind::ConditionalExpression(_))
        {
            true
        } else {
            binary_like_needs_parens(BinaryLikeExpression::LogicalExpression(self), stack)
        }
    }
}

impl<'a> NeedsParentheses<'a> for ConditionalExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        let parent = stack.parent();
        if matches!(
            parent,
            AstKind::UnaryExpression(_)
                | AstKind::AwaitExpression(_)
                | AstKind::TSTypeAssertion(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::SpreadElement(_)
                | AstKind::LogicalExpression(_)
                | AstKind::BinaryExpression(_)
        ) {
            return true;
        }
        if let AstKind::ConditionalExpression(e) = parent {
            e.test.without_parentheses().span() == self.span
        } else {
            update_or_lower_expression_needs_parens(self.span, parent)
        }
    }
}

impl<'a> NeedsParentheses<'a> for Function<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        if self.r#type != FunctionType::FunctionExpression {
            return false;
        }
        matches!(
            stack.parent(),
            AstKind::CallExpression(_) | AstKind::NewExpression(_) | AstKind::TemplateLiteral(_)
        ) || is_first_in_statement(stack, FirstInStatementMode::ExpressionOrExportDefault)
    }
}

impl<'a> NeedsParentheses<'a> for AssignmentExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        // TODO
        match stack.parent() {
            AstKind::ArrowFunctionExpression(arrow) => arrow.body.span == self.span,
            AstKind::ExportDefaultDeclaration(_) => true,
            _ => false,
        }
    }
}

impl<'a> NeedsParentheses<'a> for SequenceExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        !matches!(
            stack.parent(),
            AstKind::ReturnStatement(_)
                | AstKind::ForStatement(_)
                | AstKind::ExpressionStatement(_)
                | AstKind::SequenceExpression(_)
        )
    }
}

impl<'a> NeedsParentheses<'a> for AwaitExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        let parent = stack.parent();
        await_or_yield_needs_parens(self.span, parent)
    }
}

impl<'a> NeedsParentheses<'a> for ChainExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for Class<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        if self.r#type != ClassType::ClassExpression {
            return false;
        }
        match stack.parent() {
            AstKind::CallExpression(_)
            | AstKind::NewExpression(_)
            | AstKind::ExportDefaultDeclaration(_) => true,
            parent if is_class_extends(parent, self.span) => true,
            _ => is_first_in_statement(stack, FirstInStatementMode::ExpressionOrExportDefault),
        }
    }
}

impl<'a> NeedsParentheses<'a> for ParenthesizedExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for ArrowFunctionExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        let parent = stack.parent();
        if matches!(
            parent,
            AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSTypeAssertion(_)
                | AstKind::UnaryExpression(_)
                | AstKind::AwaitExpression(_)
                | AstKind::LogicalExpression(_)
                | AstKind::BinaryExpression(_)
        ) {
            return true;
        }
        if let AstKind::ConditionalExpression(e) = parent {
            e.test.without_parentheses().span() == self.span
        } else {
            update_or_lower_expression_needs_parens(self.span, parent)
        }
    }
}

impl<'a> NeedsParentheses<'a> for YieldExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        let parent = stack.parent();
        matches!(parent, AstKind::AwaitExpression(_) | AstKind::TSTypeAssertion(_))
            || await_or_yield_needs_parens(self.span, parent)
    }
}

impl<'a> NeedsParentheses<'a> for ImportExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        matches!(stack.parent(), AstKind::NewExpression(_))
    }
}

impl<'a> NeedsParentheses<'a> for V8IntrinsicExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for JSXMemberExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for JSXExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for JSXEmptyExpression {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for TSAsExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        ts_as_or_satisfies_needs_parens(stack)
    }
}

impl<'a> NeedsParentheses<'a> for TSSatisfiesExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        ts_as_or_satisfies_needs_parens(stack)
    }
}

impl<'a> NeedsParentheses<'a> for TSTypeAssertion<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        matches!(stack.parent(), AstKind::SimpleAssignmentTarget(_))
    }
}

impl<'a> NeedsParentheses<'a> for TSNonNullExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        let parent = stack.parent();
        is_class_extends(parent, self.span)
            || (matches!(parent, AstKind::NewExpression(_))
                && member_chain_callee_needs_parens(&self.expression))
    }
}

impl<'a> NeedsParentheses<'a> for TSInstantiationExpression<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        if let AstKind::MemberExpression(e) = stack.parent() {
            return e.object().without_parentheses().span() == self.span;
        }
        false
    }
}

fn binary_like_needs_parens<'a>(
    current: BinaryLikeExpression<'a, '_>,
    stack: &ParentStack<'a>,
) -> bool {
    let parent = match stack.parent() {
        AstKind::TSAsExpression(_)
        | AstKind::TSSatisfiesExpression(_)
        | AstKind::TSTypeAssertion(_)
        | AstKind::UnaryExpression(_)
        | AstKind::AwaitExpression(_)
        | AstKind::TSNonNullExpression(_)
        | AstKind::SpreadElement(_)
        | AstKind::CallExpression(_)
        | AstKind::NewExpression(_)
        | AstKind::MemberExpression(_)
        | AstKind::TaggedTemplateExpression(_) => return true,
        AstKind::BinaryExpression(binary) => BinaryLikeExpression::BinaryExpression(binary),
        AstKind::LogicalExpression(logical) => BinaryLikeExpression::LogicalExpression(logical),
        _ => return false,
    };

    let parent_operator = parent.operator();
    let operator = current.operator();
    let parent_precedence = parent_operator.precedence();
    let precedence = operator.precedence();

    // If the parent has a higher precedence than parentheses are necessary to not change the semantic meaning
    // when re-parsing.
    if parent_precedence > precedence {
        return true;
    }

    let is_right = parent.right().span() == current.span();

    // `a ** b ** c`
    if is_right && parent_precedence == precedence {
        return true;
    }

    // Add parentheses around bitwise and bit shift operators
    // `a * 3 >> 5` -> `(a * 3) >> 5`
    if parent_precedence.is_bitwise() || parent_precedence.is_shift() {
        return true;
    }

    // `a % 4 + 4` -> `(a % 4) + 4)`
    if parent_precedence < precedence && operator.is_remainder() {
        return parent_precedence.is_additive();
    }

    parent_precedence == precedence && !should_flatten(parent_operator, operator)
}

fn member_chain_callee_needs_parens(e: &Expression) -> bool {
    std::iter::successors(Some(e), |e| match e {
        Expression::ComputedMemberExpression(e) => Some(&e.object),
        Expression::StaticMemberExpression(e) => Some(&e.object),
        Expression::TSNonNullExpression(e) => Some(&e.expression),
        _ => None,
    })
    .any(|object| matches!(object, Expression::CallExpression(_)))
}

#[derive(Clone, Copy)]
enum UnaryLike<'a, 'b> {
    UpdateExpression(&'b UpdateExpression<'a>),
    UnaryExpression(&'b UnaryExpression<'a>),
}

impl GetSpan for UnaryLike<'_, '_> {
    fn span(&self) -> Span {
        match self {
            Self::UpdateExpression(e) => e.span,
            Self::UnaryExpression(e) => e.span,
        }
    }
}

fn unary_like_expression_needs_parens<'a>(
    current: UnaryLike<'a, '_>,
    stack: &ParentStack<'a>,
) -> bool {
    match stack.parent() {
        AstKind::BinaryExpression(e) => {
            e.operator == BinaryOperator::Exponential && e.left.span() == current.span()
        }
        parent => update_or_lower_expression_needs_parens(current.span(), parent),
    }
}

/// Returns `true` if an expression with lower precedence than an update expression needs parentheses.
///
/// This is generally the case if the expression is used in a left hand side, or primary expression context.
fn update_or_lower_expression_needs_parens(span: Span, parent: AstKind<'_>) -> bool {
    if matches!(
        parent,
        // JsSyntaxKind::JS_EXTENDS_CLAUSE
        // | JsSyntaxKind::JS_TEMPLATE_EXPRESSION
        AstKind::TSNonNullExpression(_) | AstKind::CallExpression(_) | AstKind::NewExpression(_)
    ) {
        return true;
    }
    if let AstKind::MemberExpression(member_expr) = parent {
        return member_expr.object().get_inner_expression().span() == span;
    }
    false
}

#[derive(Debug, Clone, Copy)]
pub enum FirstInStatementMode {
    /// Considers [ExpressionStatement] and the body of [ArrowFunctionExpression] as the first statement.
    ExpressionStatementOrArrow,
    /// Considers [ExpressionStatement] and [ExportDefaultDeclaration] as the first statement.
    ExpressionOrExportDefault,
}

/// Returns `true` if this node is at the start of an expression (depends on the passed `mode`).
///
/// Traverses upwards the tree for as long as the `node` is the left most expression until the node isn't
/// the left most node or reached a statement.
fn is_first_in_statement(stack: &ParentStack, mode: FirstInStatementMode) -> bool {
    matches!(stack.parent(), AstKind::ExpressionStatement(_) | AstKind::ExportDefaultDeclaration(_))
}

fn await_or_yield_needs_parens(span: Span, parent: AstKind<'_>) -> bool {
    if matches!(
        parent,
        AstKind::UnaryExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::SpreadElement(_)
            | AstKind::LogicalExpression(_)
            | AstKind::BinaryExpression(_),
    ) {
        return true;
    }
    if let AstKind::ConditionalExpression(e) = parent {
        e.test.without_parentheses().span() == span
    } else {
        update_or_lower_expression_needs_parens(span, parent)
    }
}

fn ts_as_or_satisfies_needs_parens(stack: &ParentStack<'_>) -> bool {
    matches!(stack.parent(), AstKind::SimpleAssignmentTarget(_))
}

fn is_class_extends(parent: AstKind, span: Span) -> bool {
    if let AstKind::Class(c) = parent {
        return c.super_class.as_ref().is_some_and(|c| c.without_parentheses().span() == span);
    }
    false
}
