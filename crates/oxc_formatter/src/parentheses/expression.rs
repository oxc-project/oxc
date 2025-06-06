use oxc_allocator::Address;
use oxc_ast::{AstKind, ast::*};
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
            _ => todo!(),
        }
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, NumericLiteral<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        if let AstKind::MemberExpression(MemberExpression::StaticMemberExpression(e)) =
            f.parent_kind_of(Address::from_ptr(self))
        {
            return e.object.without_parentheses().span() == self.span();
        }
        false
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, StringLiteral<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(f.parent_kind_of(Address::from_ptr(self)), AstKind::ExpressionStatement(_))
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
        let parent_kind = f.parent_kind_of(Address::from_ptr(self));
        is_class_extends(parent_kind, self.span())
            || is_first_in_statement(
                parent_kind,
                f,
                FirstInStatementMode::ExpressionStatementOrArrow,
            )
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
        matches!(
            f.parent_kind_of(Address::from_ptr(self)),
            AstKind::NewExpression(_) | AstKind::ExportDefaultDeclaration(_)
        )
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, NewExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        is_class_extends(f.parent_kind_of(Address::from_ptr(self)), self.span())
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, UpdateExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent_kind = f.parent_kind_of(Address::from_ptr(self));
        if self.prefix() {
            if let AstKind::UnaryExpression(unary) = parent_kind {
                let parent_operator = unary.operator;
                let operator = self.operator();
                return (parent_operator == UnaryOperator::UnaryPlus
                    && operator == UpdateOperator::Increment)
                    || (parent_operator == UnaryOperator::UnaryNegation
                        && operator == UpdateOperator::Decrement);
            }
        }
        unary_like_expression_needs_parens(UnaryLike::UpdateExpression(self.inner()), parent_kind)
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, UnaryExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent_kind = f.parent_kind_of(Address::from_ptr(self));
        match parent_kind {
            AstKind::UnaryExpression(parent_unary) => {
                let parent_operator = parent_unary.operator;
                let operator = self.operator();
                matches!(operator, UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation)
                    && parent_operator == operator
            }
            // A user typing `!foo instanceof Bar` probably intended `!(foo instanceof Bar)`,
            // so format to `(!foo) instance Bar` to what is really happening
            // A user typing `!foo in bar` probably intended `!(foo instanceof Bar)`,
            // so format to `(!foo) in bar` to what is really happening
            AstKind::BinaryExpression(e) if e.operator.is_relational() => true,
            _ => unary_like_expression_needs_parens(
                UnaryLike::UnaryExpression(self.inner()),
                parent_kind,
            ),
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
        let parent_kind = f.parent_kind_of(Address::from_ptr(self));
        if let AstKind::LogicalExpression(parent) = parent_kind {
            parent.operator != self.operator()
        } else if self.operator().is_coalesce()
            && matches!(parent_kind, AstKind::ConditionalExpression(_))
        {
            true
        } else {
            binary_like_needs_parens(BinaryLikeExpression::LogicalExpression(self))
        }
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, ConditionalExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = f.parent_kind_of(Address::from_ptr(self));
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
            e.test.without_parentheses().span() == self.span()
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
        let parent_kind = f.parent_kind_of(Address::from_ptr(self));
        matches!(
            parent_kind,
            AstKind::CallExpression(_) | AstKind::NewExpression(_) | AstKind::TemplateLiteral(_)
        ) || is_first_in_statement(parent_kind, f, FirstInStatementMode::ExpressionOrExportDefault)
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, AssignmentExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        // TODO
        match f.parent_kind_of(Address::from_ptr(self)) {
            AstKind::ExpressionStatement(parent) => {
                let parent_parent = f.parent_kind_of(Address::from_ptr(parent));
                if let Some(body) = parent_parent.as_function_body() {
                    let parent_parent_parent = f.parent_kind_of(Address::from_ptr(body));
                    return matches!(parent_parent_parent, AstKind::ArrowFunctionExpression(arrow) if arrow.expression);
                }
                false
            }
            AstKind::ExportDefaultDeclaration(_) => true,
            _ => false,
        }
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, SequenceExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        !matches!(
            f.parent_kind_of(Address::from_ptr(self)),
            AstKind::ReturnStatement(_)
                | AstKind::ForStatement(_)
                | AstKind::ExpressionStatement(_)
                | AstKind::SequenceExpression(_)
        )
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, AwaitExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = f.parent_kind_of(Address::from_ptr(self));
        await_or_yield_needs_parens(self.span(), parent)
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
        let parent_kind = f.parent_kind_of(Address::from_ptr(self));
        match parent_kind {
            AstKind::CallExpression(_)
            | AstKind::NewExpression(_)
            | AstKind::ExportDefaultDeclaration(_) => true,
            parent if is_class_extends(parent, self.span()) => true,
            _ => is_first_in_statement(
                parent_kind,
                f,
                FirstInStatementMode::ExpressionOrExportDefault,
            ),
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
        let parent = f.parent_kind_of(Address::from_ptr(self));
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
            e.test.without_parentheses().span() == self.span()
        } else {
            update_or_lower_expression_needs_parens(self.span(), parent)
        }
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, YieldExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = f.parent_kind_of(Address::from_ptr(self));
        matches!(parent, AstKind::AwaitExpression(_) | AstKind::TSTypeAssertion(_))
            || await_or_yield_needs_parens(self.span(), parent)
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, ImportExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(f.parent_kind_of(Address::from_ptr(self)), AstKind::NewExpression(_))
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
        let parent_kind = f.parent_kind_of(Address::from_ptr(self));
        ts_as_or_satisfies_needs_parens(parent_kind)
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, TSSatisfiesExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent_kind = f.parent_kind_of(Address::from_ptr(self));
        ts_as_or_satisfies_needs_parens(parent_kind)
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, TSTypeAssertion<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(f.parent_kind_of(Address::from_ptr(self)), AstKind::SimpleAssignmentTarget(_))
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, TSNonNullExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        let parent = f.parent_kind_of(Address::from_ptr(self));
        is_class_extends(parent, self.span())
            || (matches!(parent, AstKind::NewExpression(_))
                && member_chain_callee_needs_parens(self.expression().inner()))
    }
}

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, TSInstantiationExpression<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        if let AstKind::MemberExpression(e) = f.parent_kind_of(Address::from_ptr(self)) {
            return e.object().without_parentheses().span() == self.span();
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

fn unary_like_expression_needs_parens(
    current: UnaryLike<'_, '_>,
    parent_kind: AstKind<'_>,
) -> bool {
    match parent_kind {
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
fn is_first_in_statement(
    parent_kind: AstKind<'_>,
    f: &Formatter<'_, '_>,
    mode: FirstInStatementMode,
) -> bool {
    // TODO: incomplete
    // https://github.com/biomejs/biome/blob/4a5ef84930344ae54f3877da36888a954711f4a6/crates/biome_js_syntax/src/parentheses/expression.rs#L979-L1105

    if let AstKind::ExpressionStatement(parent) = parent_kind {
        let parent_parent = f.parent_kind_of(Address::from_ptr(parent));
        if let Some(body) = parent_parent.as_function_body() {
            let parent_parent_parent = f.parent_kind_of(Address::from_ptr(body));
            return !matches!(parent_parent_parent, AstKind::ArrowFunctionExpression(arrow) if arrow.expression);
        }
        return true;
    }
    matches!(parent_kind, AstKind::ExportDefaultDeclaration(_))
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

fn ts_as_or_satisfies_needs_parens(parent_kind: AstKind<'_>) -> bool {
    matches!(parent_kind, AstKind::SimpleAssignmentTarget(_))
}

fn is_class_extends(parent: AstKind, span: Span) -> bool {
    if let AstKind::Class(c) = parent {
        return c.super_class.as_ref().is_some_and(|c| c.without_parentheses().span() == span);
    }
    false
}
