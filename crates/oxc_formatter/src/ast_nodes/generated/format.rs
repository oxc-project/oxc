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

impl<'a> Format<'a> for AstNode<'a, '_, Program<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        self.write(f);
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, Expression<'a>> {
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
                AstNode::<BooleanLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::NullLiteral(inner) => {
                AstNode::<NullLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::NumericLiteral(inner) => {
                AstNode::<NumericLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::BigIntLiteral(inner) => {
                AstNode::<BigIntLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::RegExpLiteral(inner) => {
                AstNode::<RegExpLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::StringLiteral(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TemplateLiteral(inner) => {
                AstNode::<TemplateLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::Identifier(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::MetaProperty(inner) => {
                AstNode::<MetaProperty> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::Super(inner) => {
                AstNode::<Super> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ArrayExpression(inner) => {
                AstNode::<ArrayExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ArrowFunctionExpression(inner) => {
                AstNode::<ArrowFunctionExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::AssignmentExpression(inner) => {
                AstNode::<AssignmentExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::AwaitExpression(inner) => {
                AstNode::<AwaitExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::BinaryExpression(inner) => {
                AstNode::<BinaryExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::CallExpression(inner) => {
                AstNode::<CallExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ChainExpression(inner) => {
                AstNode::<ChainExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ClassExpression(inner) => {
                AstNode::<Class> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ConditionalExpression(inner) => {
                AstNode::<ConditionalExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::FunctionExpression(inner) => {
                AstNode::<Function> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ImportExpression(inner) => {
                AstNode::<ImportExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::LogicalExpression(inner) => {
                AstNode::<LogicalExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::NewExpression(inner) => {
                AstNode::<NewExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ObjectExpression(inner) => {
                AstNode::<ObjectExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ParenthesizedExpression(inner) => {
                AstNode::<ParenthesizedExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::SequenceExpression(inner) => {
                AstNode::<SequenceExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TaggedTemplateExpression(inner) => {
                AstNode::<TaggedTemplateExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::ThisExpression(inner) => {
                AstNode::<ThisExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::UnaryExpression(inner) => {
                AstNode::<UnaryExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::UpdateExpression(inner) => {
                AstNode::<UpdateExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::YieldExpression(inner) => {
                AstNode::<YieldExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::PrivateInExpression(inner) => {
                AstNode::<PrivateInExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::JSXElement(inner) => {
                AstNode::<JSXElement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::JSXFragment(inner) => {
                AstNode::<JSXFragment> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSAsExpression(inner) => {
                AstNode::<TSAsExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSSatisfiesExpression(inner) => {
                AstNode::<TSSatisfiesExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSTypeAssertion(inner) => {
                AstNode::<TSTypeAssertion> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSNonNullExpression(inner) => {
                AstNode::<TSNonNullExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::TSInstantiationExpression(inner) => {
                AstNode::<TSInstantiationExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Expression::V8IntrinsicExpression(inner) => {
                AstNode::<V8IntrinsicExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_member_expression!(Expression) => {
                let inner = it.to_member_expression();
                AstNode::<'a, '_, MemberExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, IdentifierName<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, IdentifierReference<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, BindingIdentifier<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, LabelIdentifier<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ThisExpression> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ArrayExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ArrayExpressionElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ArrayExpressionElement::SpreadElement(inner) => {
                AstNode::<SpreadElement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ArrayExpressionElement::Elision(inner) => {
                AstNode::<Elision> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(ArrayExpressionElement) => {
                let inner = it.to_expression();
                AstNode::<'a, '_, Expression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, Elision> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ObjectExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ObjectPropertyKind<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ObjectPropertyKind::ObjectProperty(inner) => {
                AstNode::<ObjectProperty> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ObjectPropertyKind::SpreadProperty(inner) => {
                AstNode::<SpreadElement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ObjectProperty<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, PropertyKey<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            PropertyKey::StaticIdentifier(inner) => {
                AstNode::<IdentifierName> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            PropertyKey::PrivateIdentifier(inner) => {
                AstNode::<PrivateIdentifier> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(PropertyKey) => {
                let inner = it.to_expression();
                AstNode::<'a, '_, Expression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TemplateLiteral<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TaggedTemplateExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TemplateElement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, MemberExpression<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            MemberExpression::ComputedMemberExpression(inner) => {
                AstNode::<ComputedMemberExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            MemberExpression::StaticMemberExpression(inner) => {
                AstNode::<StaticMemberExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            MemberExpression::PrivateFieldExpression(inner) => {
                AstNode::<PrivateFieldExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ComputedMemberExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, StaticMemberExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, PrivateFieldExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, CallExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, NewExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, MetaProperty<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, SpreadElement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, Argument<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            Argument::SpreadElement(inner) => {
                AstNode::<SpreadElement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(Argument) => {
                let inner = it.to_expression();
                AstNode::<'a, '_, Expression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, UpdateExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, UnaryExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, BinaryExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, PrivateInExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, LogicalExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ConditionalExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTarget<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            it @ match_simple_assignment_target!(AssignmentTarget) => {
                let inner = it.to_simple_assignment_target();
                AstNode::<'a, '_, SimpleAssignmentTarget> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_assignment_target_pattern!(AssignmentTarget) => {
                let inner = it.to_assignment_target_pattern();
                AstNode::<'a, '_, AssignmentTargetPattern> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, SimpleAssignmentTarget<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            SimpleAssignmentTarget::TSAsExpression(inner) => {
                AstNode::<TSAsExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(inner) => {
                AstNode::<TSSatisfiesExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            SimpleAssignmentTarget::TSNonNullExpression(inner) => {
                AstNode::<TSNonNullExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            SimpleAssignmentTarget::TSTypeAssertion(inner) => {
                AstNode::<TSTypeAssertion> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_member_expression!(SimpleAssignmentTarget) => {
                let inner = it.to_member_expression();
                AstNode::<'a, '_, MemberExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetPattern<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            AssignmentTargetPattern::ArrayAssignmentTarget(inner) => {
                AstNode::<ArrayAssignmentTarget> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(inner) => {
                AstNode::<ObjectAssignmentTarget> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ArrayAssignmentTarget<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ObjectAssignmentTarget<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetRest<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetMaybeDefault<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(inner) => {
                AstNode::<AssignmentTargetWithDefault> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                let inner = it.to_assignment_target();
                AstNode::<'a, '_, AssignmentTarget> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetWithDefault<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetProperty<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(inner) => {
                AstNode::<AssignmentTargetPropertyIdentifier> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(inner) => {
                AstNode::<AssignmentTargetPropertyProperty> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetPropertyIdentifier<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentTargetPropertyProperty<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, SequenceExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, Super> {
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

impl<'a> Format<'a> for AstNode<'a, '_, AwaitExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ChainExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ChainElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ChainElement::CallExpression(inner) => {
                AstNode::<CallExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ChainElement::TSNonNullExpression(inner) => {
                AstNode::<TSNonNullExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_member_expression!(ChainElement) => {
                let inner = it.to_member_expression();
                AstNode::<'a, '_, MemberExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ParenthesizedExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, Statement<'a>> {
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
                AstNode::<BlockStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::BreakStatement(inner) => {
                AstNode::<BreakStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ContinueStatement(inner) => {
                AstNode::<ContinueStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::DebuggerStatement(inner) => {
                AstNode::<DebuggerStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::DoWhileStatement(inner) => {
                AstNode::<DoWhileStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::EmptyStatement(inner) => {
                AstNode::<EmptyStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ExpressionStatement(inner) => {
                AstNode::<ExpressionStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ForInStatement(inner) => {
                AstNode::<ForInStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ForOfStatement(inner) => {
                AstNode::<ForOfStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ForStatement(inner) => {
                AstNode::<ForStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::IfStatement(inner) => {
                AstNode::<IfStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::LabeledStatement(inner) => {
                AstNode::<LabeledStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ReturnStatement(inner) => {
                AstNode::<ReturnStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::SwitchStatement(inner) => {
                AstNode::<SwitchStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::ThrowStatement(inner) => {
                AstNode::<ThrowStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::TryStatement(inner) => {
                AstNode::<TryStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::WhileStatement(inner) => {
                AstNode::<WhileStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Statement::WithStatement(inner) => {
                AstNode::<WithStatement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_declaration!(Statement) => {
                let inner = it.to_declaration();
                AstNode::<'a, '_, Declaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_module_declaration!(Statement) => {
                let inner = it.to_module_declaration();
                AstNode::<'a, '_, ModuleDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, Directive<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, Hashbang<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, BlockStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, Declaration<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            Declaration::VariableDeclaration(inner) => {
                AstNode::<VariableDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::FunctionDeclaration(inner) => {
                AstNode::<Function> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::ClassDeclaration(inner) => {
                AstNode::<Class> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSTypeAliasDeclaration(inner) => {
                AstNode::<TSTypeAliasDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSInterfaceDeclaration(inner) => {
                AstNode::<TSInterfaceDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSEnumDeclaration(inner) => {
                AstNode::<TSEnumDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSModuleDeclaration(inner) => {
                AstNode::<TSModuleDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSGlobalDeclaration(inner) => {
                AstNode::<TSGlobalDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            Declaration::TSImportEqualsDeclaration(inner) => {
                AstNode::<TSImportEqualsDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, VariableDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, VariableDeclarator<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, EmptyStatement> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ExpressionStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, IfStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, DoWhileStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, WhileStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ForStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ForStatementInit<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ForStatementInit::VariableDeclaration(inner) => {
                AstNode::<VariableDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(ForStatementInit) => {
                let inner = it.to_expression();
                AstNode::<'a, '_, Expression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ForInStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ForStatementLeft<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ForStatementLeft::VariableDeclaration(inner) => {
                AstNode::<VariableDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_assignment_target!(ForStatementLeft) => {
                let inner = it.to_assignment_target();
                AstNode::<'a, '_, AssignmentTarget> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ForOfStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ContinueStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, BreakStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ReturnStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, WithStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, SwitchStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, SwitchCase<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, LabeledStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ThrowStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TryStatement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, CatchClause<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, CatchParameter<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, DebuggerStatement> {
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

impl<'a> Format<'a> for AstNode<'a, '_, BindingPattern<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            BindingPattern::BindingIdentifier(inner) => {
                AstNode::<BindingIdentifier> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            BindingPattern::ObjectPattern(inner) => {
                AstNode::<ObjectPattern> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            BindingPattern::ArrayPattern(inner) => {
                AstNode::<ArrayPattern> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            BindingPattern::AssignmentPattern(inner) => {
                AstNode::<AssignmentPattern> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AssignmentPattern<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ObjectPattern<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, BindingProperty<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ArrayPattern<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, BindingRestElement<'a>> {
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

impl<'a> Format<'a, FormatFunctionOptions> for AstNode<'a, '_, Function<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, FormalParameters<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, FormalParameter<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, FormalParameterRest<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, FunctionBody<'a>> {
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

impl<'a> Format<'a, FormatJsArrowFunctionExpressionOptions>
    for AstNode<'a, '_, ArrowFunctionExpression<'a>>
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

impl<'a> Format<'a> for AstNode<'a, '_, YieldExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, Class<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ClassBody<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ClassElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ClassElement::StaticBlock(inner) => {
                AstNode::<StaticBlock> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ClassElement::MethodDefinition(inner) => {
                AstNode::<MethodDefinition> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ClassElement::PropertyDefinition(inner) => {
                AstNode::<PropertyDefinition> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ClassElement::AccessorProperty(inner) => {
                AstNode::<AccessorProperty> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ClassElement::TSIndexSignature(inner) => {
                AstNode::<TSIndexSignature> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, MethodDefinition<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, PropertyDefinition<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, PrivateIdentifier<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, StaticBlock<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ModuleDeclaration<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ModuleDeclaration::ImportDeclaration(inner) => {
                AstNode::<ImportDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::ExportAllDeclaration(inner) => {
                AstNode::<ExportAllDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::ExportDefaultDeclaration(inner) => {
                AstNode::<ExportDefaultDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::ExportNamedDeclaration(inner) => {
                AstNode::<ExportNamedDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::TSExportAssignment(inner) => {
                AstNode::<TSExportAssignment> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(inner) => {
                AstNode::<TSNamespaceExportDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, AccessorProperty<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ImportExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ImportDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ImportDeclarationSpecifier<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ImportDeclarationSpecifier::ImportSpecifier(inner) => {
                AstNode::<ImportSpecifier> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(inner) => {
                AstNode::<ImportDefaultSpecifier> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(inner) => {
                AstNode::<ImportNamespaceSpecifier> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ImportSpecifier<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ImportDefaultSpecifier<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ImportNamespaceSpecifier<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, WithClause<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ImportAttribute<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ImportAttributeKey<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ImportAttributeKey::Identifier(inner) => {
                AstNode::<IdentifierName> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ImportAttributeKey::StringLiteral(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ExportNamedDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ExportDefaultDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ExportAllDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ExportSpecifier<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, ExportDefaultDeclarationKind<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ExportDefaultDeclarationKind::FunctionDeclaration(inner) => {
                AstNode::<Function> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(inner) => {
                AstNode::<Class> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(inner) => {
                AstNode::<TSInterfaceDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(ExportDefaultDeclarationKind) => {
                let inner = it.to_expression();
                AstNode::<'a, '_, Expression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, ModuleExportName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            ModuleExportName::IdentifierName(inner) => {
                AstNode::<IdentifierName> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleExportName::IdentifierReference(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            ModuleExportName::StringLiteral(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, V8IntrinsicExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, BooleanLiteral> {
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

impl<'a> Format<'a> for AstNode<'a, '_, NullLiteral> {
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

impl<'a> Format<'a> for AstNode<'a, '_, NumericLiteral<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, StringLiteral<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, BigIntLiteral<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, RegExpLiteral<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXElement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXOpeningElement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXClosingElement<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXFragment<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXOpeningFragment> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXClosingFragment> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXElementName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXElementName::Identifier(inner) => {
                AstNode::<JSXIdentifier> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXElementName::IdentifierReference(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXElementName::NamespacedName(inner) => {
                AstNode::<JSXNamespacedName> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXElementName::MemberExpression(inner) => {
                AstNode::<JSXMemberExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXElementName::ThisExpression(inner) => {
                AstNode::<ThisExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXNamespacedName<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXMemberExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXMemberExpressionObject<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXMemberExpressionObject::IdentifierReference(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXMemberExpressionObject::MemberExpression(inner) => {
                AstNode::<JSXMemberExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXMemberExpressionObject::ThisExpression(inner) => {
                AstNode::<ThisExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXExpressionContainer<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXExpression<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXExpression::EmptyExpression(inner) => {
                AstNode::<JSXEmptyExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_expression!(JSXExpression) => {
                let inner = it.to_expression();
                AstNode::<'a, '_, Expression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXEmptyExpression> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXAttributeItem<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXAttributeItem::Attribute(inner) => {
                AstNode::<JSXAttribute> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeItem::SpreadAttribute(inner) => {
                AstNode::<JSXSpreadAttribute> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXAttribute<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXSpreadAttribute<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXAttributeName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXAttributeName::Identifier(inner) => {
                AstNode::<JSXIdentifier> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeName::NamespacedName(inner) => {
                AstNode::<JSXNamespacedName> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXAttributeValue<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXAttributeValue::StringLiteral(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeValue::ExpressionContainer(inner) => {
                AstNode::<JSXExpressionContainer> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeValue::Element(inner) => {
                AstNode::<JSXElement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXAttributeValue::Fragment(inner) => {
                AstNode::<JSXFragment> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXIdentifier<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXChild<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            JSXChild::Text(inner) => {
                AstNode::<JSXText> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXChild::Element(inner) => {
                AstNode::<JSXElement> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXChild::Fragment(inner) => {
                AstNode::<JSXFragment> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXChild::ExpressionContainer(inner) => {
                AstNode::<JSXExpressionContainer> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            JSXChild::Spread(inner) => {
                AstNode::<JSXSpreadChild> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, JSXSpreadChild<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSXText<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSThisParameter<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSEnumDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSEnumBody<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSEnumMember<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSEnumMemberName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSEnumMemberName::Identifier(inner) => {
                AstNode::<IdentifierName> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSEnumMemberName::String(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSEnumMemberName::ComputedString(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSEnumMemberName::ComputedTemplateString(inner) => {
                AstNode::<TemplateLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeAnnotation<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSLiteralType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSLiteral<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSLiteral::BooleanLiteral(inner) => {
                AstNode::<BooleanLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::NumericLiteral(inner) => {
                AstNode::<NumericLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::BigIntLiteral(inner) => {
                AstNode::<BigIntLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::StringLiteral(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::TemplateLiteral(inner) => {
                AstNode::<TemplateLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSLiteral::UnaryExpression(inner) => {
                AstNode::<UnaryExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSType<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSType::TSAnyKeyword(inner) => {
                AstNode::<TSAnyKeyword> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSBigIntKeyword(inner) => {
                AstNode::<TSBigIntKeyword> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSBooleanKeyword(inner) => {
                AstNode::<TSBooleanKeyword> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSIntrinsicKeyword(inner) => {
                AstNode::<TSIntrinsicKeyword> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSNeverKeyword(inner) => {
                AstNode::<TSNeverKeyword> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSNullKeyword(inner) => {
                AstNode::<TSNullKeyword> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSNumberKeyword(inner) => {
                AstNode::<TSNumberKeyword> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSObjectKeyword(inner) => {
                AstNode::<TSObjectKeyword> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSStringKeyword(inner) => {
                AstNode::<TSStringKeyword> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSSymbolKeyword(inner) => {
                AstNode::<TSSymbolKeyword> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSUndefinedKeyword(inner) => {
                AstNode::<TSUndefinedKeyword> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSUnknownKeyword(inner) => {
                AstNode::<TSUnknownKeyword> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSVoidKeyword(inner) => {
                AstNode::<TSVoidKeyword> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSArrayType(inner) => {
                AstNode::<TSArrayType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSConditionalType(inner) => {
                AstNode::<TSConditionalType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSConstructorType(inner) => {
                AstNode::<TSConstructorType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSFunctionType(inner) => {
                AstNode::<TSFunctionType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSImportType(inner) => {
                AstNode::<TSImportType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSIndexedAccessType(inner) => {
                AstNode::<TSIndexedAccessType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSInferType(inner) => {
                AstNode::<TSInferType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSIntersectionType(inner) => {
                AstNode::<TSIntersectionType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSLiteralType(inner) => {
                AstNode::<TSLiteralType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSMappedType(inner) => {
                AstNode::<TSMappedType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSNamedTupleMember(inner) => {
                AstNode::<TSNamedTupleMember> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTemplateLiteralType(inner) => {
                AstNode::<TSTemplateLiteralType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSThisType(inner) => {
                AstNode::<TSThisType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTupleType(inner) => {
                AstNode::<TSTupleType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypeLiteral(inner) => {
                AstNode::<TSTypeLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypeOperatorType(inner) => {
                AstNode::<TSTypeOperator> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypePredicate(inner) => {
                AstNode::<TSTypePredicate> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypeQuery(inner) => {
                AstNode::<TSTypeQuery> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSTypeReference(inner) => {
                AstNode::<TSTypeReference> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSUnionType(inner) => {
                AstNode::<TSUnionType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::TSParenthesizedType(inner) => {
                AstNode::<TSParenthesizedType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::JSDocNullableType(inner) => {
                AstNode::<JSDocNullableType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::JSDocNonNullableType(inner) => {
                AstNode::<JSDocNonNullableType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSType::JSDocUnknownType(inner) => {
                AstNode::<JSDocUnknownType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSConditionalType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSUnionType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSIntersectionType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSParenthesizedType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeOperator<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSArrayType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSIndexedAccessType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTupleType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSNamedTupleMember<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSOptionalType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSRestType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTupleElement<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSTupleElement::TSOptionalType(inner) => {
                AstNode::<TSOptionalType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSTupleElement::TSRestType(inner) => {
                AstNode::<TSRestType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_ts_type!(TSTupleElement) => {
                let inner = it.to_ts_type();
                AstNode::<'a, '_, TSType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSAnyKeyword> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSStringKeyword> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSBooleanKeyword> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSNumberKeyword> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSNeverKeyword> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSIntrinsicKeyword> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSUnknownKeyword> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSNullKeyword> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSUndefinedKeyword> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSVoidKeyword> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSSymbolKeyword> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSThisType> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSObjectKeyword> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSBigIntKeyword> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeReference<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSTypeName::IdentifierReference(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSTypeName::QualifiedName(inner) => {
                AstNode::<TSQualifiedName> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSTypeName::ThisExpression(inner) => {
                AstNode::<ThisExpression> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSQualifiedName<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeParameterInstantiation<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeParameter<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeParameterDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeAliasDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSClassImplements<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSInterfaceDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSInterfaceBody<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSPropertySignature<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSSignature<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSSignature::TSIndexSignature(inner) => {
                AstNode::<TSIndexSignature> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSSignature::TSPropertySignature(inner) => {
                AstNode::<TSPropertySignature> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSSignature::TSCallSignatureDeclaration(inner) => {
                AstNode::<TSCallSignatureDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSSignature::TSConstructSignatureDeclaration(inner) => {
                AstNode::<TSConstructSignatureDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSSignature::TSMethodSignature(inner) => {
                AstNode::<TSMethodSignature> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSIndexSignature<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSCallSignatureDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSMethodSignature<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSConstructSignatureDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSIndexSignatureName<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSInterfaceHeritage<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypePredicate<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypePredicateName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSTypePredicateName::Identifier(inner) => {
                AstNode::<IdentifierName> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSTypePredicateName::This(inner) => {
                AstNode::<TSThisType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSModuleDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSModuleDeclarationName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSModuleDeclarationName::Identifier(inner) => {
                AstNode::<BindingIdentifier> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSModuleDeclarationName::StringLiteral(inner) => {
                AstNode::<StringLiteral> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSModuleDeclarationBody<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSModuleDeclarationBody::TSModuleDeclaration(inner) => {
                AstNode::<TSModuleDeclaration> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSModuleDeclarationBody::TSModuleBlock(inner) => {
                AstNode::<TSModuleBlock> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSGlobalDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSModuleBlock<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeLiteral<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSInferType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeQuery<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeQueryExprName<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSTypeQueryExprName::TSImportType(inner) => {
                AstNode::<TSImportType> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            it @ match_ts_type_name!(TSTypeQueryExprName) => {
                let inner = it.to_ts_type_name();
                AstNode::<'a, '_, TSTypeName> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSImportType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSImportTypeQualifier<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSImportTypeQualifier::Identifier(inner) => {
                AstNode::<IdentifierName> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSImportTypeQualifier::QualifiedName(inner) => {
                AstNode::<TSImportTypeQualifiedName> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSImportTypeQualifiedName<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSFunctionType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSConstructorType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSMappedType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTemplateLiteralType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSAsExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSSatisfiesExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSTypeAssertion<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSImportEqualsDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSModuleReference<'a>> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let parent = self.parent;
        match self.inner {
            TSModuleReference::ExternalModuleReference(inner) => {
                AstNode::<TSExternalModuleReference> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSModuleReference::IdentifierReference(inner) => {
                AstNode::<IdentifierReference> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
            TSModuleReference::QualifiedName(inner) => {
                AstNode::<TSQualifiedName> {
                    inner,
                    parent,
                    allocator: self.allocator,
                    following_span_start: self.following_span_start,
                }
                .fmt(f);
            }
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, '_, TSExternalModuleReference<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSNonNullExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, Decorator<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSExportAssignment<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSNamespaceExportDeclaration<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, TSInstantiationExpression<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSDocNullableType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSDocNonNullableType<'a>> {
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

impl<'a> Format<'a> for AstNode<'a, '_, JSDocUnknownType> {
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
