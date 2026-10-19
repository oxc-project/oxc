use std::iter;

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::JsFormatter,
};

#[derive(Debug, Clone, Copy)]
pub enum ExpressionLeftSide<'a> {
    Expression(AstNode<'a, Expression<'a>>),
    AssignmentTarget(AstNode<'a, AssignmentTarget<'a>>),
    SimpleAssignmentTarget(AstNode<'a, SimpleAssignmentTarget<'a>>),
}

impl<'a> From<AstNode<'a, Expression<'a>>> for ExpressionLeftSide<'a> {
    fn from(value: AstNode<'a, Expression<'a>>) -> Self {
        Self::Expression(value)
    }
}

impl<'a> From<AstNode<'a, AssignmentTarget<'a>>> for ExpressionLeftSide<'a> {
    fn from(value: AstNode<'a, AssignmentTarget<'a>>) -> Self {
        Self::AssignmentTarget(value)
    }
}

impl<'a> From<AstNode<'a, SimpleAssignmentTarget<'a>>> for ExpressionLeftSide<'a> {
    fn from(value: AstNode<'a, SimpleAssignmentTarget<'a>>) -> Self {
        Self::SimpleAssignmentTarget(value)
    }
}

impl<'a> ExpressionLeftSide<'a> {
    pub fn leftmost(expression: AstNode<'a, Expression<'a>>) -> AstNode<'a, Expression<'a>> {
        let current: Self = expression.into();

        current.iter_expression().last().unwrap()
    }

    /// See [Self::left_of].
    pub fn left(&self) -> Option<Self> {
        match self {
            Self::Expression(expression) => Self::left_of(&expression.as_ast_nodes()),
            Self::AssignmentTarget(target) => {
                Self::get_left_side_of_assignment(&target.as_ast_nodes())
            }
            Self::SimpleAssignmentTarget(target) => {
                Self::get_left_side_of_assignment(&target.as_ast_nodes())
            }
        }
    }

    /// Returns the child that starts `node` (e.g. `a` of `a.b`, `a + b`, `a ? b : c`),
    /// or [None] if `node` starts with its own token.
    fn left_of(node: &AstNodes<'a>) -> Option<Self> {
        let left = match node {
            AstNodes::SequenceExpression(expr) => expr.expressions().first().map(Into::into),
            AstNodes::StaticMemberExpression(expr) => Some(expr.object().into()),
            AstNodes::ComputedMemberExpression(expr) => Some(expr.object().into()),
            AstNodes::PrivateFieldExpression(expr) => Some(expr.object().into()),
            AstNodes::TaggedTemplateExpression(expr) => Some(expr.tag().into()),
            AstNodes::CallExpression(expr) => Some(expr.callee().into()),
            AstNodes::ConditionalExpression(expr) => Some(expr.test().into()),
            AstNodes::TSAsExpression(expr) => Some(expr.expression().into()),
            AstNodes::TSSatisfiesExpression(expr) => Some(expr.expression().into()),
            AstNodes::TSNonNullExpression(expr) => Some(expr.expression().into()),
            AstNodes::TSInstantiationExpression(expr) => Some(expr.expression().into()),
            AstNodes::AssignmentExpression(expr) => Some(Self::AssignmentTarget(expr.left())),
            AstNodes::UpdateExpression(expr) => {
                if expr.prefix {
                    None
                } else {
                    Some(Self::SimpleAssignmentTarget(expr.argument()))
                }
            }
            AstNodes::BinaryExpression(binary) => Some(binary.left().into()),
            AstNodes::LogicalExpression(logical) => Some(logical.left().into()),
            AstNodes::ChainExpression(chain) => Self::left_of(&chain.expression().as_ast_nodes()),
            _ => None,
        };
        // Keep the two tables in sync
        debug_assert_eq!(left.map(|left| left.span()), Self::left_span_of(node));
        left
    }

    /// Same as [Self::left_of], but reads the span from the field directly,
    /// so no [AstNode] is created.
    pub fn left_span_of(node: &AstNodes<'_>) -> Option<Span> {
        match node {
            AstNodes::SequenceExpression(expr) => expr.expressions.first().map(GetSpan::span),
            AstNodes::StaticMemberExpression(expr) => Some(expr.object.span()),
            AstNodes::ComputedMemberExpression(expr) => Some(expr.object.span()),
            AstNodes::PrivateFieldExpression(expr) => Some(expr.object.span()),
            AstNodes::TaggedTemplateExpression(expr) => Some(expr.tag.span()),
            AstNodes::CallExpression(expr) => Some(expr.callee.span()),
            AstNodes::ConditionalExpression(expr) => Some(expr.test.span()),
            AstNodes::TSAsExpression(expr) => Some(expr.expression.span()),
            AstNodes::TSSatisfiesExpression(expr) => Some(expr.expression.span()),
            AstNodes::TSNonNullExpression(expr) => Some(expr.expression.span()),
            AstNodes::TSInstantiationExpression(expr) => Some(expr.expression.span()),
            AstNodes::AssignmentExpression(expr) => Some(expr.left.span()),
            AstNodes::UpdateExpression(expr) => (!expr.prefix).then(|| expr.argument.span()),
            AstNodes::BinaryExpression(binary) => Some(binary.left.span()),
            AstNodes::LogicalExpression(logical) => Some(logical.left.span()),
            AstNodes::ChainExpression(chain) => match &chain.expression {
                ChainElement::CallExpression(call) => Some(call.callee.span()),
                ChainElement::TSNonNullExpression(expr) => Some(expr.expression.span()),
                element => element.as_member_expression().map(|member| member.object().span()),
            },
            _ => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = ExpressionLeftSide<'a>> {
        iter::successors(Some(*self), Self::left)
    }

    /// Whether `predicate` holds for a side of [`Self::iter`],
    /// asked with the side's ancestors below `self` (a child of the current node) on the ancestor stack.
    pub fn any_with_ancestors(
        self,
        f: &mut JsFormatter<'_, 'a>,
        mut predicate: impl FnMut(Self, &mut JsFormatter<'_, 'a>) -> bool,
    ) -> bool {
        let mut pushed = 0;
        let mut side = self;
        let found = loop {
            if predicate(side, f) {
                break true;
            }
            let Some(left) = side.left() else { break false };
            for node in side.ancestors_of_left() {
                f.context_mut().ancestors_mut().push(node);
                pushed += 1;
            }
            side = left;
        };
        for _ in 0..pushed {
            f.context_mut().ancestors_mut().pop();
        }
        found
    }

    /// The node of this side, and for a `ChainExpression`, its element too ([`Self::left_of`] skips it):
    /// the ancestors of [`Self::left`] below `self`'s parent.
    fn ancestors_of_left(&self) -> impl Iterator<Item = AstNodes<'a>> {
        let node = match self {
            Self::Expression(expression) => expression.as_ast_nodes(),
            Self::AssignmentTarget(target) => target.as_ast_nodes(),
            Self::SimpleAssignmentTarget(target) => target.as_ast_nodes(),
        };
        let element = match node {
            AstNodes::ChainExpression(chain) => Some(chain.expression().as_ast_nodes()),
            _ => None,
        };
        iter::once(node).chain(element)
    }

    pub fn iter_expression(&self) -> impl Iterator<Item = AstNode<'a, Expression<'a>>> {
        self.iter().filter_map(|left| match left {
            ExpressionLeftSide::Expression(expression) => Some(expression),
            _ => None,
        })
    }

    pub fn span(&self) -> Span {
        match self {
            ExpressionLeftSide::Expression(expression) => expression.span(),
            ExpressionLeftSide::AssignmentTarget(target) => target.span(),
            ExpressionLeftSide::SimpleAssignmentTarget(target) => target.span(),
        }
    }

    fn get_left_side_of_assignment(node: &AstNodes<'a>) -> Option<ExpressionLeftSide<'a>> {
        match node {
            AstNodes::TSAsExpression(expr) => Some(expr.expression().into()),
            AstNodes::TSSatisfiesExpression(expr) => Some(expr.expression().into()),
            AstNodes::TSNonNullExpression(expr) => Some(expr.expression().into()),
            AstNodes::TSTypeAssertion(expr) => Some(expr.expression().into()),
            AstNodes::ComputedMemberExpression(expr) => Some(expr.object().into()),
            AstNodes::StaticMemberExpression(expr) => Some(expr.object().into()),
            AstNodes::PrivateFieldExpression(expr) => Some(expr.object().into()),
            _ => None,
        }
    }
}

/// Strip `ChainExpression` / `TSNonNullExpression` wrappers
/// and return the inner [`CallExpression`], if that is what they wrap.
///
/// Equivalent to Prettier's `isCallExpression(stripChainElementWrappers(node))`.
/// A general "strip" cannot return `&Expression` because [`ChainElement`] is a separate enum,
/// hence the call-expression-specialized shape;
/// see also the member-expression sibling [`is_member_expression_without_chain_wrappers`].
pub fn as_call_expression_without_chain_wrappers<'a, 'b>(
    expression: &'b Expression<'a>,
) -> Option<&'b CallExpression<'a>> {
    match expression {
        Expression::CallExpression(call) => Some(call),
        Expression::TSNonNullExpression(non_null) => {
            as_call_expression_without_chain_wrappers(&non_null.expression)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => Some(call),
            ChainElement::TSNonNullExpression(non_null) => {
                as_call_expression_without_chain_wrappers(&non_null.expression)
            }
            _ => None,
        },
        _ => None,
    }
}

/// Strip `ChainExpression` / `TSNonNullExpression` wrappers
/// and check whether they wrap a [`MemberExpression`].
///
/// Equivalent to Prettier's `isMemberExpression(stripChainElementWrappers(node))`;
/// the member-expression sibling of [`as_call_expression_without_chain_wrappers`].
pub fn is_member_expression_without_chain_wrappers(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::TSNonNullExpression(non_null) => {
            is_member_expression_without_chain_wrappers(&non_null.expression)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::TSNonNullExpression(non_null) => {
                is_member_expression_without_chain_wrappers(&non_null.expression)
            }
            element => element.as_member_expression().is_some(),
        },
        _ => expression.as_member_expression().is_some(),
    }
}
