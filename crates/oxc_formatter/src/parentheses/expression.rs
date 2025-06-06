use oxc_allocator::Address;
use oxc_ast::ast::*;
use oxc_data_structures::stack;
use oxc_span::GetSpan;
use oxc_syntax::{
    operator,
    precedence::{GetPrecedence, Precedence},
};

use crate::{
    Format,
    formatter::{Formatter, parent_stack::ParentStack},
    generated::ast_nodes::{AstNode, AstNodes},
    write::{BinaryLikeExpression, should_flatten},
};

use super::NeedsParentheses;

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, Expression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        match self.as_ast_nodes() {
            // AstNodes::BooleanLiteral(it) => it.needs_parentheses(f),
            // AstNodes::NullLiteral(it) => it.needs_parentheses(f),
            AstNodes::NumericLiteral(it) => it.needs_parentheses(f),
            // AstNodes::BigIntLiteral(it) => it.needs_parentheses(f),
            // AstNodes::RegExpLiteral(it) => it.needs_parentheses(f),
            AstNodes::StringLiteral(it) => it.needs_parentheses(f),
            // AstNodes::TemplateLiteral(it) => it.needs_parentheses(f),
            // AstNodes::Identifier(it) => it.needs_parentheses(f),
            // AstNodes::MetaProperty(it) => it.needs_parentheses(f),
            // AstNodes::Super(it) => it.needs_parentheses(f),
            AstNodes::ArrayExpression(it) => it.needs_parentheses(f),
            AstNodes::ArrowFunctionExpression(it) => it.needs_parentheses(f),
            AstNodes::AssignmentExpression(it) => it.needs_parentheses(f),
            AstNodes::AwaitExpression(it) => it.needs_parentheses(f),
            AstNodes::BinaryExpression(it) => it.needs_parentheses(f),
            AstNodes::CallExpression(it) => it.needs_parentheses(f),
            AstNodes::ChainExpression(it) => it.needs_parentheses(f),
            AstNodes::Class(it) => it.needs_parentheses(f),
            AstNodes::ConditionalExpression(it) => it.needs_parentheses(f),
            AstNodes::Function(it) => it.needs_parentheses(f),
            AstNodes::ImportExpression(it) => it.needs_parentheses(f),
            AstNodes::LogicalExpression(it) => it.needs_parentheses(f),
            AstNodes::NewExpression(it) => it.needs_parentheses(f),
            AstNodes::ObjectExpression(it) => it.needs_parentheses(f),
            AstNodes::ParenthesizedExpression(it) => it.needs_parentheses(f),
            AstNodes::SequenceExpression(it) => it.needs_parentheses(f),
            AstNodes::TaggedTemplateExpression(it) => it.needs_parentheses(f),
            AstNodes::ThisExpression(it) => it.needs_parentheses(f),
            AstNodes::UnaryExpression(it) => it.needs_parentheses(f),
            AstNodes::UpdateExpression(it) => it.needs_parentheses(f),
            AstNodes::YieldExpression(it) => it.needs_parentheses(f),
            AstNodes::PrivateInExpression(it) => it.needs_parentheses(f),
            // AstNodes::JSXElement(it) => it.needs_parentheses(f),
            // AstNodes::JSXFragment(it) => it.needs_parentheses(f),
            AstNodes::TSAsExpression(it) => it.needs_parentheses(f),
            AstNodes::TSSatisfiesExpression(it) => it.needs_parentheses(f),
            AstNodes::TSTypeAssertion(it) => it.needs_parentheses(f),
            AstNodes::TSNonNullExpression(it) => it.needs_parentheses(f),
            AstNodes::TSInstantiationExpression(it) => it.needs_parentheses(f),
            AstNodes::V8IntrinsicExpression(it) => it.needs_parentheses(f),
            AstNodes::MemberExpression(it) => it.needs_parentheses(f),
            _ => {
                // TODO: incomplete
                false
            }
        }
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, NumericLiteral<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        if let AstNodes::MemberExpression(member) = self.parent() {
            if let MemberExpression::StaticMemberExpression(e) = member.inner() {
                return e.object.without_parentheses().span() == self.span();
            }
        }
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, StringLiteral<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(self.parent(), AstNodes::ExpressionStatement(_))
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, ThisExpression> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, ArrayExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, ObjectExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent();
        is_class_extends(parent, self.span())
            || is_first_in_statement(parent, FirstInStatementMode::ExpressionStatementOrArrow)
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, TaggedTemplateExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, MemberExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, ComputedMemberExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, StaticMemberExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, PrivateFieldExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, CallExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        // TODO
        matches!(self.parent(), AstNodes::NewExpression(_) | AstNodes::ExportDefaultDeclaration(_))
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, NewExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        is_class_extends(self.parent(), self.span())
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, UpdateExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent();
        if self.prefix() {
            if let AstNodes::UnaryExpression(unary) = parent {
                let parent_operator = unary.operator();
                let operator = self.operator();
                return (parent_operator == UnaryOperator::UnaryPlus
                    && operator == UpdateOperator::Increment)
                    || (parent_operator == UnaryOperator::UnaryNegation
                        && operator == UpdateOperator::Decrement);
            }
        }
        unary_like_expression_needs_parens(UnaryLike::UpdateExpression(self))
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, UnaryExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent();
        match parent {
            AstNodes::UnaryExpression(parent_unary) => {
                let parent_operator = parent_unary.operator();
                let operator = self.operator();
                matches!(operator, UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation)
                    && parent_operator == operator
            }
            // A user typing `!foo instanceof Bar` probably intended `!(foo instanceof Bar)`,
            // so format to `(!foo) instance Bar` to what is really happening
            // A user typing `!foo in bar` probably intended `!(foo instanceof Bar)`,
            // so format to `(!foo) in bar` to what is really happening
            AstNodes::BinaryExpression(e) if e.operator().is_relational() => true,
            _ => unary_like_expression_needs_parens(UnaryLike::UnaryExpression(self)),
        }
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, BinaryExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        binary_like_needs_parens(BinaryLikeExpression::BinaryExpression(self))
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, PrivateInExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, LogicalExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent();
        if let AstNodes::LogicalExpression(parent) = parent {
            parent.operator() != self.operator()
        } else if self.operator().is_coalesce()
            && matches!(parent, AstNodes::ConditionalExpression(_))
        {
            true
        } else {
            binary_like_needs_parens(BinaryLikeExpression::LogicalExpression(self))
        }
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, ConditionalExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent();
        if matches!(
            parent,
            AstNodes::UnaryExpression(_)
                | AstNodes::AwaitExpression(_)
                | AstNodes::TSTypeAssertion(_)
                | AstNodes::TSAsExpression(_)
                | AstNodes::TSSatisfiesExpression(_)
                | AstNodes::SpreadElement(_)
                | AstNodes::LogicalExpression(_)
                | AstNodes::BinaryExpression(_)
        ) {
            return true;
        }
        if let AstNodes::ConditionalExpression(e) = parent {
            e.inner().test.without_parentheses().span() == self.span()
        } else {
            update_or_lower_expression_needs_parens(self.span(), parent)
        }
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, Function<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        if self.r#type() != FunctionType::FunctionExpression {
            return false;
        }
        let parent = self.parent();
        matches!(
            parent,
            AstNodes::CallExpression(_) | AstNodes::NewExpression(_) | AstNodes::TemplateLiteral(_)
        ) || is_first_in_statement(parent, FirstInStatementMode::ExpressionOrExportDefault)
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, AssignmentExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        // TODO
        match self.parent() {
            AstNodes::ExpressionStatement(parent) => {
                let parent_parent = parent.parent();
                if let AstNodes::FunctionBody(body) = parent_parent {
                    let parent_parent_parent = body.parent();
                    return matches!(parent_parent_parent, AstNodes::ArrowFunctionExpression(arrow) if arrow.expression());
                }
                false
            }
            AstNodes::ExportDefaultDeclaration(_) => true,
            _ => false,
        }
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, SequenceExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        !matches!(
            self.parent(),
            AstNodes::ReturnStatement(_)
                | AstNodes::ForStatement(_)
                | AstNodes::ExpressionStatement(_)
                | AstNodes::SequenceExpression(_)
        )
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, AwaitExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        await_or_yield_needs_parens(self.span(), self.parent())
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, ChainExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, Class<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        if self.r#type() != ClassType::ClassExpression {
            return false;
        }
        let parent = self.parent();
        match parent {
            AstNodes::CallExpression(_)
            | AstNodes::NewExpression(_)
            | AstNodes::ExportDefaultDeclaration(_) => true,
            parent if is_class_extends(parent, self.span()) => true,
            _ => is_first_in_statement(parent, FirstInStatementMode::ExpressionOrExportDefault),
        }
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, ParenthesizedExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, ArrowFunctionExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent();
        if matches!(
            parent,
            AstNodes::TSAsExpression(_)
                | AstNodes::TSSatisfiesExpression(_)
                | AstNodes::TSTypeAssertion(_)
                | AstNodes::UnaryExpression(_)
                | AstNodes::AwaitExpression(_)
                | AstNodes::LogicalExpression(_)
                | AstNodes::BinaryExpression(_)
        ) {
            return true;
        }
        if let AstNodes::ConditionalExpression(e) = parent {
            e.inner().test.without_parentheses().span() == self.span()
        } else {
            update_or_lower_expression_needs_parens(self.span(), parent)
        }
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, YieldExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent();
        matches!(parent, AstNodes::AwaitExpression(_) | AstNodes::TSTypeAssertion(_))
            || await_or_yield_needs_parens(self.span(), parent)
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, ImportExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(self.parent(), AstNodes::NewExpression(_))
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, V8IntrinsicExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, JSXMemberExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, JSXExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, JSXEmptyExpression> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, TSAsExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        ts_as_or_satisfies_needs_parens(self.parent())
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, TSSatisfiesExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        ts_as_or_satisfies_needs_parens(self.parent())
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, TSTypeAssertion<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(self.parent(), AstNodes::SimpleAssignmentTarget(_))
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, TSNonNullExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = self.parent();
        is_class_extends(parent, self.span())
            || (matches!(parent, AstNodes::NewExpression(_))
                && member_chain_callee_needs_parens(self.expression().inner()))
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, TSInstantiationExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        if let AstNodes::MemberExpression(e) = self.parent() {
            return e.inner().object().without_parentheses().span() == self.span();
        }
        false
    }
}

fn binary_like_needs_parens(binary_like: BinaryLikeExpression<'_, '_, '_>) -> bool {
    let parent = match binary_like.parent() {
        AstNodes::TSAsExpression(_)
        | AstNodes::TSSatisfiesExpression(_)
        | AstNodes::TSTypeAssertion(_)
        | AstNodes::UnaryExpression(_)
        | AstNodes::AwaitExpression(_)
        | AstNodes::TSNonNullExpression(_)
        | AstNodes::SpreadElement(_)
        | AstNodes::CallExpression(_)
        | AstNodes::NewExpression(_)
        | AstNodes::MemberExpression(_)
        | AstNodes::TaggedTemplateExpression(_) => return true,
        AstNodes::BinaryExpression(binary) => BinaryLikeExpression::BinaryExpression(binary),
        AstNodes::LogicalExpression(logical) => BinaryLikeExpression::LogicalExpression(logical),
        _ => return false,
    };

    let parent_operator = parent.operator();
    let operator = binary_like.operator();
    let parent_precedence = parent_operator.precedence();
    let precedence = operator.precedence();

    // If the parent has a higher precedence than parentheses are necessary to not change the semantic meaning
    // when re-parsing.
    if parent_precedence > precedence {
        return true;
    }

    let is_right = parent.right().span() == binary_like.span();

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
enum UnaryLike<'a, 'b, 'c> {
    UpdateExpression(&'c AstNode<'a, 'b, UpdateExpression<'a>>),
    UnaryExpression(&'b AstNode<'a, 'b, UnaryExpression<'a>>),
}

impl UnaryLike<'_, '_, '_> {
    fn parent(&self) -> &AstNodes<'_, '_> {
        match self {
            Self::UpdateExpression(e) => e.parent(),
            Self::UnaryExpression(e) => e.parent(),
        }
    }
}

impl GetSpan for UnaryLike<'_, '_, '_> {
    fn span(&self) -> Span {
        match self {
            Self::UpdateExpression(e) => e.span(),
            Self::UnaryExpression(e) => e.span(),
        }
    }
}

fn unary_like_expression_needs_parens(node: UnaryLike<'_, '_, '_>) -> bool {
    match node.parent() {
        AstNodes::BinaryExpression(e) => {
            e.operator() == BinaryOperator::Exponential && e.left().span() == node.span()
        }
        parent => update_or_lower_expression_needs_parens(node.span(), parent),
    }
}

/// Returns `true` if an expression with lower precedence than an update expression needs parentheses.
///
/// This is generally the case if the expression is used in a left hand side, or primary expression context.
fn update_or_lower_expression_needs_parens(span: Span, parent: &AstNodes<'_, '_>) -> bool {
    if matches!(
        parent,
        // JsSyntaxKind::JS_EXTENDS_CLAUSE
        // | JsSyntaxKind::JS_TEMPLATE_EXPRESSION
        AstNodes::TSNonNullExpression(_) | AstNodes::CallExpression(_) | AstNodes::NewExpression(_)
    ) {
        return true;
    }
    if let AstNodes::MemberExpression(member_expr) = parent {
        return member_expr.inner().object().get_inner_expression().span() == span;
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
fn is_first_in_statement(parent: &AstNodes<'_, '_>, mode: FirstInStatementMode) -> bool {
    // TODO: incomplete
    // https://github.com/biomejs/biome/blob/4a5ef84930344ae54f3877da36888a954711f4a6/crates/biome_js_syntax/src/parentheses/expression.rs#L979-L1105

    if let AstNodes::ExpressionStatement(parent) = parent {
        let parent_parent = parent.parent();
        if let AstNodes::FunctionBody(body) = parent_parent {
            let parent_parent_parent = body.parent();
            return !matches!(parent_parent_parent, AstNodes::ArrowFunctionExpression(arrow) if arrow.expression());
        }
        return true;
    }
    matches!(parent, AstNodes::ExportDefaultDeclaration(_))
}

fn await_or_yield_needs_parens(span: Span, node: &AstNodes<'_, '_>) -> bool {
    if matches!(
        node,
        AstNodes::UnaryExpression(_)
            | AstNodes::TSAsExpression(_)
            | AstNodes::TSSatisfiesExpression(_)
            | AstNodes::SpreadElement(_)
            | AstNodes::LogicalExpression(_)
            | AstNodes::BinaryExpression(_),
    ) {
        return true;
    }
    if let AstNodes::ConditionalExpression(e) = node {
        e.inner().test.without_parentheses().span() == span
    } else {
        update_or_lower_expression_needs_parens(span, &node)
    }
}

fn ts_as_or_satisfies_needs_parens(parent: &AstNodes<'_, '_>) -> bool {
    matches!(parent, AstNodes::SimpleAssignmentTarget(_))
}

fn is_class_extends(parent: &AstNodes<'_, '_>, span: Span) -> bool {
    if let AstNodes::Class(c) = parent {
        return c
            .inner()
            .super_class
            .as_ref()
            .is_some_and(|c| c.without_parentheses().span() == span);
    }
    false
}
