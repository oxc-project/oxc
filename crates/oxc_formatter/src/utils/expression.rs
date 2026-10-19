use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::ast_nodes::{AstNode, AstNodeKind, AstNodes};

#[derive(Debug, Clone, Copy)]
pub enum ExpressionLeftSide<'a, 'b> {
    Expression(AstNode<'b, 'a, Expression<'a>>),
    AssignmentTarget(AstNode<'b, 'a, AssignmentTarget<'a>>),
    SimpleAssignmentTarget(AstNode<'b, 'a, SimpleAssignmentTarget<'a>>),
}

impl<'a, 'b> From<AstNode<'b, 'a, Expression<'a>>> for ExpressionLeftSide<'a, 'b> {
    fn from(value: AstNode<'b, 'a, Expression<'a>>) -> Self {
        Self::Expression(value)
    }
}

impl<'a, 'b> From<AstNode<'b, 'a, AssignmentTarget<'a>>> for ExpressionLeftSide<'a, 'b> {
    fn from(value: AstNode<'b, 'a, AssignmentTarget<'a>>) -> Self {
        Self::AssignmentTarget(value)
    }
}

impl<'a, 'b> From<AstNode<'b, 'a, SimpleAssignmentTarget<'a>>> for ExpressionLeftSide<'a, 'b> {
    fn from(value: AstNode<'b, 'a, SimpleAssignmentTarget<'a>>) -> Self {
        Self::SimpleAssignmentTarget(value)
    }
}

impl<'a> ExpressionLeftSide<'a, '_> {
    /// Returns the leftmost expression that starts `expression` (`expression` itself if none).
    ///
    /// Same walk as [Self::any] over the AST, without creating [AstNode]s.
    pub fn leftmost<'b>(expression: &'b Expression<'a>) -> &'b Expression<'a> {
        let mut leftmost = expression;
        while let Some(left) = Self::left_expression_of(leftmost) {
            leftmost = left;
        }
        leftmost
    }

    /// The next expression on the left of `expression` that [Self::any] visits,
    /// through an assignment target (`a` of `a.b = c`), see [Self::left_of] and [Self::get_left_side_of_assignment].
    fn left_expression_of<'b>(expression: &'b Expression<'a>) -> Option<&'b Expression<'a>> {
        let expression = match expression {
            Expression::ChainExpression(chain) => match &chain.expression {
                ChainElement::CallExpression(call) => &call.callee,
                ChainElement::TSNonNullExpression(expr) => &expr.expression,
                element => element.as_member_expression()?.object(),
            },
            Expression::SequenceExpression(expr) => expr.expressions.first()?,
            Expression::StaticMemberExpression(expr) => &expr.object,
            Expression::ComputedMemberExpression(expr) => &expr.object,
            Expression::PrivateFieldExpression(expr) => &expr.object,
            Expression::TaggedTemplateExpression(expr) => &expr.tag,
            Expression::CallExpression(expr) => &expr.callee,
            Expression::ConditionalExpression(expr) => &expr.test,
            Expression::TSAsExpression(expr) => &expr.expression,
            Expression::TSSatisfiesExpression(expr) => &expr.expression,
            Expression::TSNonNullExpression(expr) => &expr.expression,
            Expression::TSInstantiationExpression(expr) => &expr.expression,
            Expression::AssignmentExpression(expr) => {
                Self::left_expression_of_assignment(expr.left.as_simple_assignment_target()?)?
            }
            Expression::UpdateExpression(expr) => {
                if expr.prefix {
                    return None;
                }
                Self::left_expression_of_assignment(&expr.argument)?
            }
            Expression::BinaryExpression(binary) => &binary.left,
            Expression::LogicalExpression(logical) => &logical.left,
            _ => return None,
        };
        Some(expression)
    }

    /// Same as [Self::get_left_side_of_assignment], over the AST.
    fn left_expression_of_assignment<'b>(
        target: &'b SimpleAssignmentTarget<'a>,
    ) -> Option<&'b Expression<'a>> {
        match target {
            SimpleAssignmentTarget::TSAsExpression(expr) => Some(&expr.expression),
            SimpleAssignmentTarget::TSSatisfiesExpression(expr) => Some(&expr.expression),
            SimpleAssignmentTarget::TSNonNullExpression(expr) => Some(&expr.expression),
            SimpleAssignmentTarget::TSTypeAssertion(expr) => Some(&expr.expression),
            target => target.as_member_expression().map(MemberExpression::object),
        }
    }

    /// Returns `true` if `predicate` holds for `self` or any of the left sides that start it, see [Self::left_of].
    ///
    /// Walks down recursively, so each left side can refer to its parent.
    /// `predicate` is called dynamically, so that the frames of the walk stay small.
    pub fn any(self, predicate: &mut dyn FnMut(ExpressionLeftSide<'a, '_>) -> bool) -> bool {
        if predicate(self) {
            return true;
        }
        match self {
            Self::Expression(expression) => {
                let kind = expression.as_kind();
                let element;
                let left = if let AstNodeKind::ChainExpression(chain) = &kind {
                    element = chain.expression().as_kind();
                    Self::left_of(&element)
                } else {
                    Self::left_of(&kind)
                };
                // Keep the two tables in sync
                debug_assert_eq!(
                    left.map(|left| left.span()),
                    Self::left_span_of(&kind.as_ast_nodes())
                );
                left.is_some_and(|left| left.any(predicate))
            }
            Self::AssignmentTarget(target) => Self::get_left_side_of_assignment(&target.as_kind())
                .is_some_and(|left| left.any(predicate)),
            Self::SimpleAssignmentTarget(target) => {
                Self::get_left_side_of_assignment(&target.as_kind())
                    .is_some_and(|left| left.any(predicate))
            }
        }
    }

    /// Returns the child that starts `node` (e.g. `a` of `a.b`, `a + b`, `a ? b : c`),
    /// or [None] if `node` starts with its own token.
    fn left_of<'k>(node: &'k AstNodeKind<'_, 'a>) -> Option<ExpressionLeftSide<'a, 'k>> {
        match node {
            AstNodeKind::SequenceExpression(expr) => expr.expressions().first().map(Into::into),
            AstNodeKind::StaticMemberExpression(expr) => Some(expr.object().into()),
            AstNodeKind::ComputedMemberExpression(expr) => Some(expr.object().into()),
            AstNodeKind::PrivateFieldExpression(expr) => Some(expr.object().into()),
            AstNodeKind::TaggedTemplateExpression(expr) => Some(expr.tag().into()),
            AstNodeKind::CallExpression(expr) => Some(expr.callee().into()),
            AstNodeKind::ConditionalExpression(expr) => Some(expr.test().into()),
            AstNodeKind::TSAsExpression(expr) => Some(expr.expression().into()),
            AstNodeKind::TSSatisfiesExpression(expr) => Some(expr.expression().into()),
            AstNodeKind::TSNonNullExpression(expr) => Some(expr.expression().into()),
            AstNodeKind::TSInstantiationExpression(expr) => Some(expr.expression().into()),
            AstNodeKind::AssignmentExpression(expr) => Some(expr.left().into()),
            AstNodeKind::UpdateExpression(expr) => {
                if expr.prefix {
                    None
                } else {
                    Some(expr.argument().into())
                }
            }
            AstNodeKind::BinaryExpression(binary) => Some(binary.left().into()),
            AstNodeKind::LogicalExpression(logical) => Some(logical.left().into()),
            _ => None,
        }
    }

    /// Same as [Self::left_of], but reads the span from the field directly,
    /// so no [AstNode] is created.
    pub fn left_span_of(node: &AstNodes<'_, '_>) -> Option<Span> {
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

    pub fn span(&self) -> Span {
        match self {
            ExpressionLeftSide::Expression(expression) => expression.span(),
            ExpressionLeftSide::AssignmentTarget(target) => target.span(),
            ExpressionLeftSide::SimpleAssignmentTarget(target) => target.span(),
        }
    }

    fn get_left_side_of_assignment<'k>(
        node: &'k AstNodeKind<'_, 'a>,
    ) -> Option<ExpressionLeftSide<'a, 'k>> {
        match node {
            AstNodeKind::TSAsExpression(expr) => Some(expr.expression().into()),
            AstNodeKind::TSSatisfiesExpression(expr) => Some(expr.expression().into()),
            AstNodeKind::TSNonNullExpression(expr) => Some(expr.expression().into()),
            AstNodeKind::TSTypeAssertion(expr) => Some(expr.expression().into()),
            AstNodeKind::ComputedMemberExpression(expr) => Some(expr.object().into()),
            AstNodeKind::StaticMemberExpression(expr) => Some(expr.object().into()),
            AstNodeKind::PrivateFieldExpression(expr) => Some(expr.object().into()),
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
