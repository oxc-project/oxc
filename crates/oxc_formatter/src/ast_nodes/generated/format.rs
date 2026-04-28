// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/formatter/format.rs`.

#![expect(clippy::match_same_arms)]
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode,
    formatter::{
        Format, Formatter,
        trivia::{format_leading_comments, format_trailing_comments},
    },
    parentheses::NeedsParentheses,
    print::{FormatFunctionOptions, FormatJsArrowFunctionExpressionOptions, FormatWrite},
    utils::{suppressed::FormatSuppressedNode, typecast::format_type_cast_comment_node},
};

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, Program<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        self.write(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, Expression<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        if f.comments().has_trailing_suppression_comment(self.span().end) {
            format_leading_comments(self.span()).fmt(f);
            FormatSuppressedNode(self.span()).fmt(f);
            format_trailing_comments(
                self.parent.span(),
                self.inner.span(),
                self.following_span_start,
            )
            .fmt(f);
            return;
        }
        let parent = self.parent;
        match self.inner {
            Expression::BooleanLiteral(inner) => {
                AstNode::<'_, '_, BooleanLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::NullLiteral(inner) => {
                AstNode::<'_, '_, NullLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::NumericLiteral(inner) => {
                AstNode::<'_, '_, NumericLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::BigIntLiteral(inner) => {
                AstNode::<'_, '_, BigIntLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::RegExpLiteral(inner) => {
                AstNode::<'_, '_, RegExpLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::StringLiteral(inner) => {
                AstNode::<'_, '_, StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TemplateLiteral(inner) => {
                AstNode::<'_, '_, TemplateLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::Identifier(inner) => {
                AstNode::<'_, '_, IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::MetaProperty(inner) => {
                AstNode::<'_, '_, MetaProperty> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::Super(inner) => {
                AstNode::<'_, '_, Super> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ArrayExpression(inner) => {
                AstNode::<'_, '_, ArrayExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ArrowFunctionExpression(inner) => {
                AstNode::<'_, '_, ArrowFunctionExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::AssignmentExpression(inner) => {
                AstNode::<'_, '_, AssignmentExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::AwaitExpression(inner) => {
                AstNode::<'_, '_, AwaitExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::BinaryExpression(inner) => {
                AstNode::<'_, '_, BinaryExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::CallExpression(inner) => {
                AstNode::<'_, '_, CallExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ChainExpression(inner) => {
                AstNode::<'_, '_, ChainExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ClassExpression(inner) => {
                AstNode::<'_, '_, Class> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ConditionalExpression(inner) => {
                AstNode::<'_, '_, ConditionalExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::FunctionExpression(inner) => {
                AstNode::<'_, '_, Function> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ImportExpression(inner) => {
                AstNode::<'_, '_, ImportExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::LogicalExpression(inner) => {
                AstNode::<'_, '_, LogicalExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::NewExpression(inner) => {
                AstNode::<'_, '_, NewExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ObjectExpression(inner) => {
                AstNode::<'_, '_, ObjectExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ParenthesizedExpression(inner) => {
                AstNode::<'_, '_, ParenthesizedExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::SequenceExpression(inner) => {
                AstNode::<'_, '_, SequenceExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TaggedTemplateExpression(inner) => {
                AstNode::<'_, '_, TaggedTemplateExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ThisExpression(inner) => {
                AstNode::<'_, '_, ThisExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::UnaryExpression(inner) => {
                AstNode::<'_, '_, UnaryExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::UpdateExpression(inner) => {
                AstNode::<'_, '_, UpdateExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::YieldExpression(inner) => {
                AstNode::<'_, '_, YieldExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::PrivateInExpression(inner) => {
                AstNode::<'_, '_, PrivateInExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::JSXElement(inner) => {
                AstNode::<'_, '_, JSXElement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::JSXFragment(inner) => {
                AstNode::<'_, '_, JSXFragment> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSAsExpression(inner) => {
                AstNode::<'_, '_, TSAsExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSSatisfiesExpression(inner) => {
                AstNode::<'_, '_, TSSatisfiesExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSTypeAssertion(inner) => {
                AstNode::<'_, '_, TSTypeAssertion> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSNonNullExpression(inner) => {
                AstNode::<'_, '_, TSNonNullExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSInstantiationExpression(inner) => {
                AstNode::<'_, '_, TSInstantiationExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::V8IntrinsicExpression(inner) => {
                AstNode::<'_, '_, V8IntrinsicExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_member_expression!(Expression) => {
                let inner = it.to_member_expression();
                AstNode::<'_, '_, MemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, IdentifierName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, IdentifierReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, BindingIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, LabelIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ThisExpression> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ArrayExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, true, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ArrayExpressionElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ArrayExpressionElement::SpreadElement(inner) => {
                AstNode::<'_, '_, SpreadElement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ArrayExpressionElement::Elision(inner) => {
                AstNode::<'_, '_, Elision> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(ArrayExpressionElement) => {
                let inner = it.to_expression();
                AstNode::<'_, '_, Expression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, Elision> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ObjectExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, true, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ObjectPropertyKind<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ObjectPropertyKind::ObjectProperty(inner) => {
                AstNode::<'_, '_, ObjectProperty> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ObjectPropertyKind::SpreadProperty(inner) => {
                AstNode::<'_, '_, SpreadElement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ObjectProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, PropertyKey<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            PropertyKey::StaticIdentifier(inner) => {
                AstNode::<'_, '_, IdentifierName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            PropertyKey::PrivateIdentifier(inner) => {
                AstNode::<'_, '_, PrivateIdentifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(PropertyKey) => {
                let inner = it.to_expression();
                AstNode::<'_, '_, Expression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TemplateLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TaggedTemplateExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TemplateElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if is_suppressed {
            self.format_leading_comments(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, MemberExpression<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            MemberExpression::ComputedMemberExpression(inner) => {
                AstNode::<'_, '_, ComputedMemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            MemberExpression::StaticMemberExpression(inner) => {
                AstNode::<'_, '_, StaticMemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            MemberExpression::PrivateFieldExpression(inner) => {
                AstNode::<'_, '_, PrivateFieldExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ComputedMemberExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, StaticMemberExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, PrivateFieldExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, CallExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, NewExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, MetaProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, SpreadElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, Argument<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            Argument::SpreadElement(inner) => {
                AstNode::<'_, '_, SpreadElement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(Argument) => {
                let inner = it.to_expression();
                AstNode::<'_, '_, Expression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, UpdateExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, UnaryExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, BinaryExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, PrivateInExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, LogicalExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ConditionalExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, AssignmentExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, AssignmentTarget<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                let inner = it.to_simple_assignment_target();
                AstNode::<'_, '_, SimpleAssignmentTarget> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                let inner = it.to_assignment_target_pattern();
                AstNode::<'_, '_, AssignmentTargetPattern> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, SimpleAssignmentTarget<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(inner) => {
                AstNode::<'_, '_, IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            SimpleAssignmentTarget::TSAsExpression(inner) => {
                AstNode::<'_, '_, TSAsExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(inner) => {
                AstNode::<'_, '_, TSSatisfiesExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            SimpleAssignmentTarget::TSNonNullExpression(inner) => {
                AstNode::<'_, '_, TSNonNullExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            SimpleAssignmentTarget::TSTypeAssertion(inner) => {
                AstNode::<'_, '_, TSTypeAssertion> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                let inner = it.to_member_expression();
                AstNode::<'_, '_, MemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, AssignmentTargetPattern<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(inner) => {
                AstNode::<'_, '_, ArrayAssignmentTarget> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(inner) => {
                AstNode::<'_, '_, ObjectAssignmentTarget> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ArrayAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ObjectAssignmentTarget<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, AssignmentTargetRest<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, AssignmentTargetMaybeDefault<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(inner) => {
                AstNode::<'_, '_, AssignmentTargetWithDefault> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                let inner = it.to_assignment_target();
                AstNode::<'_, '_, AssignmentTarget> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, AssignmentTargetWithDefault<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, AssignmentTargetProperty<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(inner) => {
                AstNode::<'_, '_, AssignmentTargetPropertyIdentifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(inner) => {
                AstNode::<'_, '_, AssignmentTargetPropertyProperty> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, AssignmentTargetPropertyIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, AssignmentTargetPropertyProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, SequenceExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, Super> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, AwaitExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ChainExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ChainElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ChainElement::CallExpression(inner) => {
                AstNode::<'_, '_, CallExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ChainElement::TSNonNullExpression(inner) => {
                AstNode::<'_, '_, TSNonNullExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_member_expression!(ChainElement) => {
                let inner = it.to_member_expression();
                AstNode::<'_, '_, MemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ParenthesizedExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, Statement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        if !matches!(self.inner, Statement::ExpressionStatement(_))
            && f.comments().has_trailing_suppression_comment(self.span().end)
        {
            format_leading_comments(self.span()).fmt(f);
            FormatSuppressedNode(self.span()).fmt(f);
            format_trailing_comments(
                self.parent.span(),
                self.inner.span(),
                self.following_span_start,
            )
            .fmt(f);
            return;
        }
        let parent = self.parent;
        match self.inner {
            Statement::BlockStatement(inner) => {
                AstNode::<'_, '_, BlockStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::BreakStatement(inner) => {
                AstNode::<'_, '_, BreakStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ContinueStatement(inner) => {
                AstNode::<'_, '_, ContinueStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::DebuggerStatement(inner) => {
                AstNode::<'_, '_, DebuggerStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::DoWhileStatement(inner) => {
                AstNode::<'_, '_, DoWhileStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::EmptyStatement(inner) => {
                AstNode::<'_, '_, EmptyStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ExpressionStatement(inner) => {
                AstNode::<'_, '_, ExpressionStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ForInStatement(inner) => {
                AstNode::<'_, '_, ForInStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ForOfStatement(inner) => {
                AstNode::<'_, '_, ForOfStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ForStatement(inner) => {
                AstNode::<'_, '_, ForStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::IfStatement(inner) => {
                AstNode::<'_, '_, IfStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::LabeledStatement(inner) => {
                AstNode::<'_, '_, LabeledStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ReturnStatement(inner) => {
                AstNode::<'_, '_, ReturnStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::SwitchStatement(inner) => {
                AstNode::<'_, '_, SwitchStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ThrowStatement(inner) => {
                AstNode::<'_, '_, ThrowStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::TryStatement(inner) => {
                AstNode::<'_, '_, TryStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::WhileStatement(inner) => {
                AstNode::<'_, '_, WhileStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::WithStatement(inner) => {
                AstNode::<'_, '_, WithStatement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_declaration!(Statement) => {
                let inner = it.to_declaration();
                AstNode::<'_, '_, Declaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_module_declaration!(Statement) => {
                let inner = it.to_module_declaration();
                AstNode::<'_, '_, ModuleDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, Directive<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, Hashbang<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, BlockStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, Declaration<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            Declaration::VariableDeclaration(inner) => {
                AstNode::<'_, '_, VariableDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::FunctionDeclaration(inner) => {
                AstNode::<'_, '_, Function> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::ClassDeclaration(inner) => {
                AstNode::<'_, '_, Class> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSTypeAliasDeclaration(inner) => {
                AstNode::<'_, '_, TSTypeAliasDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSInterfaceDeclaration(inner) => {
                AstNode::<'_, '_, TSInterfaceDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSEnumDeclaration(inner) => {
                AstNode::<'_, '_, TSEnumDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSModuleDeclaration(inner) => {
                AstNode::<'_, '_, TSModuleDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSGlobalDeclaration(inner) => {
                AstNode::<'_, '_, TSGlobalDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSImportEqualsDeclaration(inner) => {
                AstNode::<'_, '_, TSImportEqualsDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, VariableDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, VariableDeclarator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, EmptyStatement> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ExpressionStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, IfStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, DoWhileStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, WhileStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ForStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ForStatementInit<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ForStatementInit::VariableDeclaration(inner) => {
                AstNode::<'_, '_, VariableDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(ForStatementInit) => {
                let inner = it.to_expression();
                AstNode::<'_, '_, Expression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ForInStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ForStatementLeft<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ForStatementLeft::VariableDeclaration(inner) => {
                AstNode::<'_, '_, VariableDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_assignment_target!(ForStatementLeft) => {
                let inner = it.to_assignment_target();
                AstNode::<'_, '_, AssignmentTarget> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ForOfStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ContinueStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, BreakStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ReturnStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, WithStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, SwitchStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, SwitchCase<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, LabeledStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ThrowStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TryStatement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, CatchClause<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if is_suppressed {
            self.format_leading_comments(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, CatchParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if is_suppressed {
            self.format_leading_comments(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, DebuggerStatement> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, BindingPattern<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            BindingPattern::BindingIdentifier(inner) => {
                AstNode::<'_, '_, BindingIdentifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            BindingPattern::ObjectPattern(inner) => {
                AstNode::<'_, '_, ObjectPattern> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            BindingPattern::ArrayPattern(inner) => {
                AstNode::<'_, '_, ArrayPattern> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            BindingPattern::AssignmentPattern(inner) => {
                AstNode::<'_, '_, AssignmentPattern> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, AssignmentPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ObjectPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, BindingProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ArrayPattern<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, BindingRestElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a, FormatFunctionOptions> for AstNode<'me, 'a, Function<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }

    fn fmt_with_options(&self, options: FormatFunctionOptions, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write_with_options(options, f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, FormalParameters<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if is_suppressed {
            self.format_leading_comments(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, FormalParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, FormalParameterRest<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, FunctionBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if is_suppressed {
            self.format_leading_comments(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'me, 'a> Format<'a, FormatJsArrowFunctionExpressionOptions>
    for AstNode<'me, 'a, ArrowFunctionExpression<'a>>
{
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }

    fn fmt_with_options(
        &self,
        options: FormatJsArrowFunctionExpressionOptions,
        f: &mut Formatter<'_, 'a>,
    ) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write_with_options(options, f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, YieldExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, Class<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ClassBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if is_suppressed {
            self.format_leading_comments(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ClassElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ClassElement::StaticBlock(inner) => {
                AstNode::<'_, '_, StaticBlock> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ClassElement::MethodDefinition(inner) => {
                AstNode::<'_, '_, MethodDefinition> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ClassElement::PropertyDefinition(inner) => {
                AstNode::<'_, '_, PropertyDefinition> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ClassElement::AccessorProperty(inner) => {
                AstNode::<'_, '_, AccessorProperty> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ClassElement::TSIndexSignature(inner) => {
                AstNode::<'_, '_, TSIndexSignature> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, MethodDefinition<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, PropertyDefinition<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, PrivateIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, StaticBlock<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ModuleDeclaration<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ModuleDeclaration::ImportDeclaration(inner) => {
                AstNode::<'_, '_, ImportDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::ExportAllDeclaration(inner) => {
                AstNode::<'_, '_, ExportAllDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::ExportDefaultDeclaration(inner) => {
                AstNode::<'_, '_, ExportDefaultDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::ExportNamedDeclaration(inner) => {
                AstNode::<'_, '_, ExportNamedDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::TSExportAssignment(inner) => {
                AstNode::<'_, '_, TSExportAssignment> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(inner) => {
                AstNode::<'_, '_, TSNamespaceExportDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, AccessorProperty<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ImportExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ImportDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ImportDeclarationSpecifier<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ImportDeclarationSpecifier::ImportSpecifier(inner) => {
                AstNode::<'_, '_, ImportSpecifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(inner) => {
                AstNode::<'_, '_, ImportDefaultSpecifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(inner) => {
                AstNode::<'_, '_, ImportNamespaceSpecifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ImportSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ImportDefaultSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ImportNamespaceSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, WithClause<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ImportAttribute<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ImportAttributeKey<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ImportAttributeKey::Identifier(inner) => {
                AstNode::<'_, '_, IdentifierName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ImportAttributeKey::StringLiteral(inner) => {
                AstNode::<'_, '_, StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ExportNamedDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if is_suppressed {
            self.format_leading_comments(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ExportDefaultDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if is_suppressed {
            self.format_leading_comments(f);
            FormatSuppressedNode(self.span()).fmt(f);
            self.format_trailing_comments(f);
        } else {
            self.write(f);
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ExportAllDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ExportSpecifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ExportDefaultDeclarationKind<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ExportDefaultDeclarationKind::FunctionDeclaration(inner) => {
                AstNode::<'_, '_, Function> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(inner) => {
                AstNode::<'_, '_, Class> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(inner) => {
                AstNode::<'_, '_, TSInterfaceDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => {
                let inner = it.to_expression();
                AstNode::<'_, '_, Expression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, ModuleExportName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ModuleExportName::IdentifierName(inner) => {
                AstNode::<'_, '_, IdentifierName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleExportName::IdentifierReference(inner) => {
                AstNode::<'_, '_, IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleExportName::StringLiteral(inner) => {
                AstNode::<'_, '_, StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, V8IntrinsicExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, BooleanLiteral> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, NullLiteral> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, NumericLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, StringLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, BigIntLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, RegExpLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        if format_type_cast_comment_node(self, false, f) {
            return;
        }
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        self.write(f);
        if needs_parentheses {
            ")".fmt(f);
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXOpeningElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXClosingElement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXFragment<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        if format_type_cast_comment_node(self, false, f) {
            return;
        }
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        self.write(f);
        if needs_parentheses {
            ")".fmt(f);
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXOpeningFragment> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXClosingFragment> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXElementName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXElementName::Identifier(inner) => {
                AstNode::<'_, '_, JSXIdentifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXElementName::IdentifierReference(inner) => {
                AstNode::<'_, '_, IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXElementName::NamespacedName(inner) => {
                AstNode::<'_, '_, JSXNamespacedName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXElementName::MemberExpression(inner) => {
                AstNode::<'_, '_, JSXMemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXElementName::ThisExpression(inner) => {
                AstNode::<'_, '_, ThisExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXNamespacedName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXMemberExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXMemberExpressionObject<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXMemberExpressionObject::IdentifierReference(inner) => {
                AstNode::<'_, '_, IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXMemberExpressionObject::MemberExpression(inner) => {
                AstNode::<'_, '_, JSXMemberExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXMemberExpressionObject::ThisExpression(inner) => {
                AstNode::<'_, '_, ThisExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXExpressionContainer<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXExpression<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXExpression::EmptyExpression(inner) => {
                AstNode::<'_, '_, JSXEmptyExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(JSXExpression) => {
                let inner = it.to_expression();
                AstNode::<'_, '_, Expression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXEmptyExpression> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXAttributeItem<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXAttributeItem::Attribute(inner) => {
                AstNode::<'_, '_, JSXAttribute> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeItem::SpreadAttribute(inner) => {
                AstNode::<'_, '_, JSXSpreadAttribute> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXAttribute<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXSpreadAttribute<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXAttributeName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXAttributeName::Identifier(inner) => {
                AstNode::<'_, '_, JSXIdentifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeName::NamespacedName(inner) => {
                AstNode::<'_, '_, JSXNamespacedName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXAttributeValue<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXAttributeValue::StringLiteral(inner) => {
                AstNode::<'_, '_, StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeValue::ExpressionContainer(inner) => {
                AstNode::<'_, '_, JSXExpressionContainer> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeValue::Element(inner) => {
                AstNode::<'_, '_, JSXElement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeValue::Fragment(inner) => {
                AstNode::<'_, '_, JSXFragment> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXIdentifier<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXChild<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXChild::Text(inner) => {
                AstNode::<'_, '_, JSXText> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXChild::Element(inner) => {
                AstNode::<'_, '_, JSXElement> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXChild::Fragment(inner) => {
                AstNode::<'_, '_, JSXFragment> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXChild::ExpressionContainer(inner) => {
                AstNode::<'_, '_, JSXExpressionContainer> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXChild::Spread(inner) => {
                AstNode::<'_, '_, JSXSpreadChild> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXSpreadChild<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSXText<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSThisParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSEnumDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSEnumBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSEnumMember<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSEnumMemberName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSEnumMemberName::Identifier(inner) => {
                AstNode::<'_, '_, IdentifierName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSEnumMemberName::String(inner) => {
                AstNode::<'_, '_, StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSEnumMemberName::ComputedString(inner) => {
                AstNode::<'_, '_, StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSEnumMemberName::ComputedTemplateString(inner) => {
                AstNode::<'_, '_, TemplateLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypeAnnotation<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSLiteralType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSLiteral<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSLiteral::BooleanLiteral(inner) => {
                AstNode::<'_, '_, BooleanLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::NumericLiteral(inner) => {
                AstNode::<'_, '_, NumericLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::BigIntLiteral(inner) => {
                AstNode::<'_, '_, BigIntLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::StringLiteral(inner) => {
                AstNode::<'_, '_, StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::TemplateLiteral(inner) => {
                AstNode::<'_, '_, TemplateLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::UnaryExpression(inner) => {
                AstNode::<'_, '_, UnaryExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSType<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSType::TSAnyKeyword(inner) => {
                AstNode::<'_, '_, TSAnyKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSBigIntKeyword(inner) => {
                AstNode::<'_, '_, TSBigIntKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSBooleanKeyword(inner) => {
                AstNode::<'_, '_, TSBooleanKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSIntrinsicKeyword(inner) => {
                AstNode::<'_, '_, TSIntrinsicKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSNeverKeyword(inner) => {
                AstNode::<'_, '_, TSNeverKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSNullKeyword(inner) => {
                AstNode::<'_, '_, TSNullKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSNumberKeyword(inner) => {
                AstNode::<'_, '_, TSNumberKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSObjectKeyword(inner) => {
                AstNode::<'_, '_, TSObjectKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSStringKeyword(inner) => {
                AstNode::<'_, '_, TSStringKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSSymbolKeyword(inner) => {
                AstNode::<'_, '_, TSSymbolKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSUndefinedKeyword(inner) => {
                AstNode::<'_, '_, TSUndefinedKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSUnknownKeyword(inner) => {
                AstNode::<'_, '_, TSUnknownKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSVoidKeyword(inner) => {
                AstNode::<'_, '_, TSVoidKeyword> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSArrayType(inner) => {
                AstNode::<'_, '_, TSArrayType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSConditionalType(inner) => {
                AstNode::<'_, '_, TSConditionalType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSConstructorType(inner) => {
                AstNode::<'_, '_, TSConstructorType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSFunctionType(inner) => {
                AstNode::<'_, '_, TSFunctionType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSImportType(inner) => {
                AstNode::<'_, '_, TSImportType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSIndexedAccessType(inner) => {
                AstNode::<'_, '_, TSIndexedAccessType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSInferType(inner) => {
                AstNode::<'_, '_, TSInferType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSIntersectionType(inner) => {
                AstNode::<'_, '_, TSIntersectionType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSLiteralType(inner) => {
                AstNode::<'_, '_, TSLiteralType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSMappedType(inner) => {
                AstNode::<'_, '_, TSMappedType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSNamedTupleMember(inner) => {
                AstNode::<'_, '_, TSNamedTupleMember> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTemplateLiteralType(inner) => {
                AstNode::<'_, '_, TSTemplateLiteralType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSThisType(inner) => {
                AstNode::<'_, '_, TSThisType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTupleType(inner) => {
                AstNode::<'_, '_, TSTupleType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypeLiteral(inner) => {
                AstNode::<'_, '_, TSTypeLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypeOperatorType(inner) => {
                AstNode::<'_, '_, TSTypeOperator> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypePredicate(inner) => {
                AstNode::<'_, '_, TSTypePredicate> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypeQuery(inner) => {
                AstNode::<'_, '_, TSTypeQuery> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypeReference(inner) => {
                AstNode::<'_, '_, TSTypeReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSUnionType(inner) => {
                AstNode::<'_, '_, TSUnionType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSParenthesizedType(inner) => {
                AstNode::<'_, '_, TSParenthesizedType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::JSDocNullableType(inner) => {
                AstNode::<'_, '_, JSDocNullableType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::JSDocNonNullableType(inner) => {
                AstNode::<'_, '_, JSDocNonNullableType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::JSDocUnknownType(inner) => {
                AstNode::<'_, '_, JSDocUnknownType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSConditionalType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSUnionType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSIntersectionType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSParenthesizedType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypeOperator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSArrayType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSIndexedAccessType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTupleType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSNamedTupleMember<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSOptionalType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSRestType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTupleElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSTupleElement::TSOptionalType(inner) => {
                AstNode::<'_, '_, TSOptionalType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSTupleElement::TSRestType(inner) => {
                AstNode::<'_, '_, TSRestType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_ts_type!(TSTupleElement) => {
                let inner = it.to_ts_type();
                AstNode::<'_, '_, TSType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSAnyKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSStringKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSBooleanKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSNumberKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSNeverKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSIntrinsicKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSUnknownKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSNullKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSUndefinedKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSVoidKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSSymbolKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSThisType> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSObjectKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSBigIntKeyword> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypeReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypeName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSTypeName::IdentifierReference(inner) => {
                AstNode::<'_, '_, IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSTypeName::QualifiedName(inner) => {
                AstNode::<'_, '_, TSQualifiedName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSTypeName::ThisExpression(inner) => {
                AstNode::<'_, '_, ThisExpression> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSQualifiedName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypeParameterInstantiation<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypeParameter<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypeParameterDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypeAliasDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSClassImplements<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSInterfaceDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSInterfaceBody<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSPropertySignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSSignature<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSSignature::TSIndexSignature(inner) => {
                AstNode::<'_, '_, TSIndexSignature> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSSignature::TSPropertySignature(inner) => {
                AstNode::<'_, '_, TSPropertySignature> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSSignature::TSCallSignatureDeclaration(inner) => {
                AstNode::<'_, '_, TSCallSignatureDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSSignature::TSConstructSignatureDeclaration(inner) => {
                AstNode::<'_, '_, TSConstructSignatureDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSSignature::TSMethodSignature(inner) => {
                AstNode::<'_, '_, TSMethodSignature> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSIndexSignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSCallSignatureDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSMethodSignature<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSConstructSignatureDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSIndexSignatureName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSInterfaceHeritage<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypePredicate<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypePredicateName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSTypePredicateName::Identifier(inner) => {
                AstNode::<'_, '_, IdentifierName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSTypePredicateName::This(inner) => {
                AstNode::<'_, '_, TSThisType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSModuleDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSModuleDeclarationName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSModuleDeclarationName::Identifier(inner) => {
                AstNode::<'_, '_, BindingIdentifier> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSModuleDeclarationName::StringLiteral(inner) => {
                AstNode::<'_, '_, StringLiteral> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSModuleDeclarationBody<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSModuleDeclarationBody::TSModuleDeclaration(inner) => {
                AstNode::<'_, '_, TSModuleDeclaration> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSModuleDeclarationBody::TSModuleBlock(inner) => {
                AstNode::<'_, '_, TSModuleBlock> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSGlobalDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSModuleBlock<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypeLiteral<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSInferType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypeQuery<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypeQueryExprName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSTypeQueryExprName::TSImportType(inner) => {
                AstNode::<'_, '_, TSImportType> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => {
                let inner = it.to_ts_type_name();
                AstNode::<'_, '_, TSTypeName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSImportType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSImportTypeQualifier<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSImportTypeQualifier::Identifier(inner) => {
                AstNode::<'_, '_, IdentifierName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSImportTypeQualifier::QualifiedName(inner) => {
                AstNode::<'_, '_, TSImportTypeQualifiedName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSImportTypeQualifiedName<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSFunctionType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSConstructorType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSMappedType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTemplateLiteralType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSAsExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSSatisfiesExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSTypeAssertion<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSImportEqualsDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSModuleReference<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSModuleReference::ExternalModuleReference(inner) => {
                AstNode::<'_, '_, TSExternalModuleReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSModuleReference::IdentifierReference(inner) => {
                AstNode::<'_, '_, IdentifierReference> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSModuleReference::QualifiedName(inner) => {
                AstNode::<'_, '_, TSQualifiedName> {
                    inner,
                    parent,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSExternalModuleReference<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSNonNullExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, Decorator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSExportAssignment<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSNamespaceExportDeclaration<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, TSInstantiationExpression<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        if !is_suppressed && format_type_cast_comment_node(self, false, f) {
            return;
        }
        self.format_leading_comments(f);
        let needs_parentheses = self.needs_parentheses(f);
        if needs_parentheses {
            "(".fmt(f);
        }
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        if needs_parentheses {
            ")".fmt(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSDocNullableType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSDocNonNullableType<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}

impl<'me, 'a> Format<'a> for AstNode<'me, 'a, JSDocUnknownType> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);
        self.format_leading_comments(f);
        if is_suppressed {
            FormatSuppressedNode(self.span()).fmt(f);
        } else {
            self.write(f);
        }
        self.format_trailing_comments(f);
    }
}
